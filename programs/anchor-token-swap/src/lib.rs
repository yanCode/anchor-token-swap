pub mod curves;
mod errors;

mod state;
mod swap_constraints;
use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{Token2022, ID as TOKEN_2022_PROGRAM_ID},
    token_interface::{Mint, TokenAccount},
};

pub use errors::*;
use crate::curves::CurveType;
pub use state::*;
pub use swap_constraints::*;
declare_id!("Bspu3p7dUX27mCSG5jaQkqoVwA6V2fMB9zZNpfu2dY9J");

#[program]
pub mod anchor_token_swap {

    use anchor_spl::{token_2022::spl_token_2022::extension::transfer_fee::TransferFeeConfig, token_interface::get_mint_extension_data};
    use crate::curves::{CurveType, RoundDirection, SwapCurve, TradeDirection};

    use super::*;

    #[access_control(
        validate_swap_constraints(
            &curve_type,
            &fees,
            ctx.accounts.fee_account.owner,
            None
        )
        validate_mint_uncloseable(&ctx.accounts.pool_mint)
    )]
    pub fn initialize(ctx: Context<Initialize>, curve_type: CurveType, fees: Fees) -> Result<()> {
        let swap_v1 = &mut ctx.accounts.swap_v1;
        let swap_curve = SwapCurve::new(curve_type);
        let calculator = &swap_curve.calculator;
        fees.validate()?;
        calculator.validate()?;
        calculator.validate_supply(ctx.accounts.token_a.amount, ctx.accounts.token_b.amount)?;
        let initial_amount = swap_curve.calculator.new_pool_supply();
        
        anchor_spl::token_interface::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::MintTo {
                    mint: ctx.accounts.pool_mint.to_account_info(),
                    to: ctx.accounts.fee_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                &[&[&swap_v1.key().to_bytes(), &[ctx.bumps.authority]]],
            ),
            initial_amount as u64,
        )?;

        **swap_v1 = SwapV1 {
            is_initialized: true,
            token_program_id: TOKEN_2022_PROGRAM_ID,
            token_a: *ctx.accounts.token_a.to_account_info().key,
            token_b: *ctx.accounts.token_b.to_account_info().key,
            pool_mint: *ctx.accounts.pool_mint.to_account_info().key,
            token_a_mint: ctx.accounts.token_a.mint,
            token_b_mint: ctx.accounts.token_b.mint,
            pool_fee_account: *ctx.accounts.fee_account.to_account_info().key,
            fees,
            curve_type,
        };

        Ok(())
    }
    pub fn swap(ctx: Context<TokenSwap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
        let swap_curve = &ctx.accounts.token_swap.swap_curve();
        let source_mint_transfer_fee_config = get_mint_extension_data::<TransferFeeConfig>(&
        ctx.accounts.source_token_mint.to_account_info());
        
        let actual_amount_in = 
        if let Ok(ref config) = source_mint_transfer_fee_config{
            amount_in.saturating_sub(config.calculate_epoch_fee(Clock::get()?.epoch, amount_in).ok_or(SwapError::FeeCalculationFailure)?)
        } else {
            amount_in
        };

        let trade_direction = if ctx.accounts.swap_source.key() == ctx.accounts.token_swap.token_a.key() {
            TradeDirection::AtoB
        } else {
            TradeDirection::BtoA
        };

       let result = ctx.accounts.token_swap.swap_curve().swap(
            u128::from(actual_amount_in),
                u128::from(ctx.accounts.swap_source.amount),
                u128::from(ctx.accounts.swap_destination.amount),
                trade_direction,
                &ctx.accounts.token_swap.fees(),
        ).ok_or(SwapError::ZeroTradingTokens)?;
        
        let (source_transfer_amount, source_mint_decimals)=
        {
            let source_amount_swapped = result.source_amount_swapped as u64;
            let amount = if let Ok(ref config) = source_mint_transfer_fee_config{
                source_amount_swapped.saturating_sub(config.calculate_epoch_fee(Clock::get()?.epoch, source_amount_swapped).ok_or(SwapError::FeeCalculationFailure)?)
            } else {
                source_amount_swapped
            };
            (amount, ctx.accounts.source_token_mint.decimals)
        };
        
        let destination_mint_transfer_fee_config = get_mint_extension_data::<TransferFeeConfig>(&ctx.accounts.destination_token_mint.to_account_info());
        let (destination_transfer_amount, destination_mint_decimals)=
        {
            let amount_out = result.destination_amount_swapped as u64;
            let amount_received = if let Ok(ref config) = destination_mint_transfer_fee_config{
                amount_out.saturating_sub(config.calculate_epoch_fee(Clock::get()?.epoch, amount_out).ok_or(SwapError::FeeCalculationFailure)?)
            } else {
                amount_out
            };
            if amount_received < minimum_amount_out {
                return Err(SwapError::ExceededSlippage.into());
            }
            (amount_received, ctx.accounts.destination_token_mint.decimals)
        };
        let (swap_token_a_amount, swap_token_b_amount) = match trade_direction {
            TradeDirection::AtoB => (
                result.new_swap_source_amount,
                result.new_swap_destination_amount,
            ),
            TradeDirection::BtoA => (
                result.new_swap_destination_amount,
                result.new_swap_source_amount,
            ),
        };
        anchor_spl::token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: ctx.accounts.swap_source.to_account_info(),   
                    to: ctx.accounts.source.to_account_info(),
                    authority: ctx.accounts.user_transfer_authority.to_account_info(),
                    mint: ctx.accounts.source_token_mint.to_account_info(),
                },
            ),
            source_transfer_amount,
            source_mint_decimals,
        )?;

       
        if result.owner_fee > 0 {
            let pool_token_amount = swap_curve.calculator.withdraw_single_token_type_exact_out(
                result.owner_fee,
                swap_token_a_amount,
                swap_token_b_amount,
                ctx.accounts.pool_mint.supply as u128,
                trade_direction,
                RoundDirection::Floor,
            ).ok_or(SwapError::FeeCalculationFailure)?;
            if let Some(host_fee_account) = &ctx.accounts.host_fee_account {
                let host_fee = ctx.accounts.token_swap.fees().host_fee(pool_token_amount).ok_or(SwapError::FeeCalculationFailure)?;
                if host_fee > 0 {
                    pool_token_amount.checked_sub(host_fee).ok_or(SwapError::FeeCalculationFailure)?;
                    anchor_spl::token_interface::mint_to(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            anchor_spl::token_interface::MintTo {
                                mint: ctx.accounts.pool_mint.to_account_info(),
                                to: host_fee_account.to_account_info(),
                                authority: ctx.accounts.authority.to_account_info(),
                            },
                            &[&[&ctx.accounts.token_swap.key().to_bytes(), &[ctx.bumps.authority]]],
                        ),
                        host_fee as u64,
                    )?;
                }
                anchor_spl::token_interface::mint_to(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        anchor_spl::token_interface::MintTo {
                            mint: ctx.accounts.pool_mint.to_account_info(),
                            to: ctx.accounts.pool_fee_account.to_account_info(),
                            authority: ctx.accounts.authority.to_account_info(),
                        },
                        &[&[&ctx.accounts.token_swap.key().to_bytes(), &[ctx.bumps.authority]]],
                    ),
                    pool_token_amount as u64,
                )?;

            }
            // let host_fee = ctx.accounts.token_swap.fees().host_fee(pool_token_amount);

        }

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: ctx.accounts.swap_destination.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.user_transfer_authority.to_account_info(),
                    mint: ctx.accounts.destination_token_mint.to_account_info(),
                },
            ),
            destination_transfer_amount,
            destination_mint_decimals,
        )?;


        Ok(())
    }

    pub fn peek_curve(_ctx: Context<PeekCurve>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = SwapV1::INIT_SPACE + 8
    )]
    pub swap_v1: Account<'info, SwapV1>,
    #[account(
        seeds = [swap_v1.key().as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,
    #[account(
        token::token_program = token_program.key(),
        // token::delegate = None todo validate the delegate & close_authority
        constraint = token_a.delegate.is_none() @ SwapError::InvalidDelegate,
        constraint = token_a.mint != token_b.mint @ SwapError::RepeatedMint,
        constraint = token_a.owner != authority.key() @ SwapError::InvalidOwner,
    )]
    pub token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::token_program = token_program.key(),
        constraint = token_b.owner != authority.key() @ SwapError::InvalidOwner,
        constraint = token_b.delegate.is_none() @ SwapError::InvalidDelegate,

    )]
    pub token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mint::authority = authority.key(),
        mint::token_program = token_program.key(),
        constraint = pool_mint.supply > 0 @ SwapError::InvalidSupply,
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
        token::mint = pool_mint.key(),
        token::token_program = token_program.key(),
        constraint = destination.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::mint = pool_mint.key(),
        token::token_program = token_program.key(),
        constraint = fee_account.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub fee_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account()]
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct TokenSwap<'info> {
    #[account(
        constraint = token_swap.is_initialized @ SwapError::IncorrectSwapAccount,
        constraint = token_swap.token_program_id == TOKEN_2022_PROGRAM_ID @ SwapError::IncorrectTokenProgramId
    )]
    pub token_swap: Account<'info, SwapV1>,
    #[account(
        seeds = [token_swap.key().as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,
    pub user_transfer_authority: UncheckedAccount<'info>,
    pub source: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::token_program = token_swap.token_program_id,
        constraint = (swap_source.key() != token_swap.token_a.key()) 
        && (swap_source.key() != token_swap.token_b.key())
        @ SwapError::IncorrectSwapAccount,

        constraint = swap_source.key() != source.key() @ SwapError::InvalidInput,
        constraint = swap_source.key() != swap_destination.key() @ SwapError::InvalidInput
    )]
    pub swap_source: InterfaceAccount<'info, TokenAccount>,
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::token_program = token_swap.token_program_id,
        constraint = (swap_destination.key() != token_swap.token_a.key()) 
        && (swap_destination.key() != token_swap.token_b.key())
        @ SwapError::IncorrectSwapAccount,

        constraint = swap_destination.key() != destination.key() @ SwapError::InvalidOutput

    )]
    pub swap_destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mint::token_program = token_swap.token_program_id,
        constraint = pool_mint.key() != token_swap.pool_mint.key() @ SwapError::IncorrectPoolMint
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
        token::token_program = token_swap.token_program_id,
        token::mint = pool_mint.key(),
        constraint = host_fee_account.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub host_fee_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        token::token_program = token_swap.token_program_id,
        constraint = pool_fee_account.key() != token_swap.pool_fee_account.key() @ SwapError::InvalidFeeAccount
    )]
    pub pool_fee_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mint::token_program = token_swap.token_program_id,
        mint::authority = authority.key(),
    )]
    pub source_token_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mint::token_program = token_swap.token_program_id,
        mint::authority = authority.key(),
        constraint = destination_token_mint.key() != source_token_mint.key() @ SwapError::RepeatedMint
    )]
    pub destination_token_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct PeekCurve<'info> {
    pub swap_v1: Account<'info, SwapV1>,
}
