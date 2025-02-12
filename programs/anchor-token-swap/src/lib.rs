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
    use crate::curves::{SwapCurve, TradeDirection, CurveType};

    use super::*;

    #[access_control(
        validate_swap_constraints(
            &curve_type,
            &fees,
            ctx.accounts.fee_account.owner,
            None
        )
        validate_mint_uncloseable(&ctx)
    )]
    pub fn initialize(ctx: Context<Initialize>, curve_type: CurveType, fees: Fees) -> Result<()> {
        let swap_v1 = &mut ctx.accounts.swap_v1;
        let swap_key = swap_v1.key();
        let seeds = &[swap_key.as_ref()];
        let (swap_authority, _) = Pubkey::find_program_address(seeds, ctx.program_id);
        require_keys_eq!(
            swap_authority,
            ctx.accounts.authority.key(),
            SwapError::InvalidProgramAddress
        );
        let swap_curve = SwapCurve::new(curve_type);
        let calculator = swap_curve.calculator;
        fees.validate()?;
        calculator.validate()?;
        calculator.validate_supply(ctx.accounts.token_a.amount, ctx.accounts.token_b.amount)?;
        **swap_v1 = SwapV1 {
            is_initialized: true,
            bump_seed: ctx.bumps.authority,
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
    pub fn swap(ctx: Context<TokenSwap>, amount_in: u64, _minimum_amount_out: u64) -> Result<()> {

        let actual_amount_in = if let Ok(transfer_fee_config) = get_mint_extension_data::<TransferFeeConfig>(&ctx.accounts.pool_mint.to_account_info()) {
            amount_in.saturating_sub(transfer_fee_config.calculate_epoch_fee(Clock::get()?.epoch, amount_in).ok_or(SwapError::FeeCalculationFailure)?)
        } else {
            amount_in
        };
        let trade_direction = if ctx.accounts.swap_source.key() == ctx.accounts.token_swap.token_a.key() {
            TradeDirection::AtoB
        } else {
            TradeDirection::BtoA
        };
       let _result = ctx.accounts.token_swap.swap_curve().swap(
            u128::from(actual_amount_in),
                u128::from(ctx.accounts.swap_source.amount),
                u128::from(ctx.accounts.swap_destination.amount),
                trade_direction,
                &ctx.accounts.token_swap.fees(),
        ).ok_or(SwapError::ZeroTradingTokens);
       
        
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
    #[account(
        constraint = token_program.key() == TOKEN_2022_PROGRAM_ID @ SwapError::IncorrectTokenProgramId
    )]
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
        constraint = pool_fee_account.key() != token_swap.pool_fee_account.key() @ SwapError::InvalidFeeAccount
    )]
    pub pool_fee_account: InterfaceAccount<'info, TokenAccount>,
    pub source_token_mint: InterfaceAccount<'info, Mint>,
    pub destination_token_mint: InterfaceAccount<'info, Mint>,
    #[account(
        constraint =  token_program.key()!=TOKEN_2022_PROGRAM_ID @ SwapError::IncorrectTokenProgramId
    )]
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct PeekCurve<'info> {
    pub swap_v1: Account<'info, SwapV1>,
}
