use {
    crate::{
        curves::{RoundDirection, TradeDirection},
        helper::to_u64,
        state::SwapState,
        SwapError, SwapV1,
    },
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::spl_token_2022::extension::transfer_fee::TransferFeeConfig,
        token_interface::{get_mint_extension_data, Mint, TokenAccount, TokenInterface},
    },
};
pub fn swap_handler(
    ctx: Context<TokenSwap>,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<()> {
    let swap_curve = &ctx.accounts.token_swap.swap_curve();
    let source_mint_transfer_fee_config = get_mint_extension_data::<TransferFeeConfig>(
        &ctx.accounts.source_token_mint.to_account_info(),
    );

    let actual_amount_in = if let Ok(ref config) = source_mint_transfer_fee_config {
        amount_in.saturating_sub(
            config
                .calculate_epoch_fee(Clock::get()?.epoch, amount_in)
                .ok_or(SwapError::FeeCalculationFailure)?,
        )
    } else {
        amount_in
    };

    let trade_direction =
        match ctx.accounts.swap_source.key() == ctx.accounts.token_swap.token_a.key() {
            true => TradeDirection::AtoB,
            false => TradeDirection::BtoA,
        };

    let result = ctx
        .accounts
        .token_swap
        .swap_curve()
        .swap(
            u128::from(actual_amount_in),
            u128::from(ctx.accounts.swap_source.amount),
            u128::from(ctx.accounts.swap_destination.amount),
            trade_direction,
            &ctx.accounts.token_swap.fees,
        )
        .ok_or(SwapError::ZeroTradingTokens)?;

    let (source_transfer_amount, source_mint_decimals) = {
        let source_amount_swapped = to_u64(result.source_amount_swapped)?;
        let amount = if let Ok(ref config) = source_mint_transfer_fee_config {
            source_amount_swapped.saturating_sub(
                config
                    .calculate_epoch_fee(Clock::get()?.epoch, source_amount_swapped)
                    .ok_or(SwapError::FeeCalculationFailure)?,
            )
        } else {
            source_amount_swapped
        };
        (amount, ctx.accounts.source_token_mint.decimals)
    };

    let destination_mint_transfer_fee_config = get_mint_extension_data::<TransferFeeConfig>(
        &ctx.accounts.destination_token_mint.to_account_info(),
    );
    let (destination_transfer_amount, destination_mint_decimals) = {
        let amount_out = to_u64(result.destination_amount_swapped)?;
        let amount_received = if let Ok(ref config) = destination_mint_transfer_fee_config {
            amount_out.saturating_sub(
                config
                    .calculate_epoch_fee(Clock::get()?.epoch, amount_out)
                    .ok_or(SwapError::FeeCalculationFailure)?,
            )
        } else {
            amount_out
        };
        require_gte!(
            amount_received,
            minimum_amount_out,
            SwapError::ExceededSlippage
        );
        (
            amount_received,
            ctx.accounts.destination_token_mint.decimals,
        )
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
            ctx.accounts.token_source_program.to_account_info(),
            anchor_spl::token_interface::TransferChecked {
                from: ctx.accounts.user_source.to_account_info(),
                to: ctx.accounts.swap_source.to_account_info(),
                authority: ctx.accounts.user_transfer_authority.to_account_info(),
                mint: ctx.accounts.source_token_mint.to_account_info(),
            },
        ),
        source_transfer_amount,
        source_mint_decimals,
    )?;

    if result.owner_fee > 0 {
        let mut pool_token_amount = swap_curve
            .calculator
            .withdraw_single_token_type_exact_out(
                result.owner_fee,
                swap_token_a_amount,
                swap_token_b_amount,
                ctx.accounts.pool_mint.supply as u128,
                trade_direction,
                RoundDirection::Floor,
            )
            .ok_or(SwapError::FeeCalculationFailure)?;

        if let Some(host_fee_account) = &ctx.accounts.host_fee_account {
            let host_fee = ctx
                .accounts
                .token_swap
                .fees()
                .host_fee(pool_token_amount)
                .ok_or(SwapError::FeeCalculationFailure)?;
            if host_fee > 0 {
                pool_token_amount = pool_token_amount
                    .checked_sub(host_fee)
                    .ok_or(SwapError::FeeCalculationFailure)?;
                anchor_spl::token_interface::mint_to(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_pool_program.to_account_info(),
                        anchor_spl::token_interface::MintTo {
                            mint: ctx.accounts.pool_mint.to_account_info(),
                            to: host_fee_account.to_account_info(),
                            authority: ctx.accounts.authority.to_account_info(),
                        },
                        &[&[
                            &ctx.accounts.token_swap.key().to_bytes(),
                            &[ctx.bumps.authority],
                        ]],
                    ),
                    to_u64(host_fee)?,
                )?;
            }
        }
        if let Some(pool_fee_account) = &ctx.accounts.pool_fee_account {
            anchor_spl::token_interface::mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_pool_program.to_account_info(),
                    anchor_spl::token_interface::MintTo {
                        mint: ctx.accounts.pool_mint.to_account_info(),
                        to: pool_fee_account.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                    &[&[
                        &ctx.accounts.token_swap.key().to_bytes(),
                        &[ctx.bumps.authority],
                    ]],
                ),
                to_u64(pool_token_amount)?,
            )?;
        }
    }

    anchor_spl::token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_destination_program.to_account_info(),
            anchor_spl::token_interface::TransferChecked {
                from: ctx.accounts.swap_destination.to_account_info(),
                to: ctx.accounts.user_destination.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                mint: ctx.accounts.destination_token_mint.to_account_info(),
            },
            &[&[
                &ctx.accounts.token_swap.key().to_bytes(),
                &[ctx.bumps.authority],
            ]],
        ),
        destination_transfer_amount,
        destination_mint_decimals,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct TokenSwap<'info> {
    #[account(
        constraint = !token_swap.to_account_info().data_is_empty() @ SwapError::IncorrectSwapAccount,
    )]
    pub token_swap: Account<'info, SwapV1>,
    #[account(
        seeds = [token_swap.key().as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,
    pub user_transfer_authority: Signer<'info>,
    #[account(
        mut,
        token::mint = swap_source.mint,
       constraint = user_source.key() != swap_source.key() @ SwapError::InvalidInput
    )]
    pub user_source: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = swap_source.mint,
        constraint = (swap_source.key() == token_swap.token_a.key()) || (swap_source.key() == token_swap.token_b.key())
        @ SwapError::IncorrectSwapAccount,
        constraint = swap_source.key() != swap_destination.key() @ SwapError::SameAccountTransfer
    )]
    pub swap_source: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = swap_destination.mint,
        constraint = user_destination.key() != swap_destination.key() @ SwapError::SameAccountTransfer
    )]
    pub user_destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = swap_destination.mint,
        token::token_program = token_destination_program.key(),
        constraint = (swap_destination.key() == token_swap.token_a.key()) || (swap_destination.key() == token_swap.token_b.key())
        @ SwapError::IncorrectSwapAccount,
    )]
    pub swap_destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        mint::token_program = token_pool_program.key(),
        constraint = pool_mint.key() == token_swap.pool_mint.key() @ SwapError::IncorrectPoolMint
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = pool_mint.key(),
        constraint = host_fee_account.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub host_fee_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = pool_mint.key(),
        constraint = pool_fee_account.key() == token_swap.pool_fee_account.key() @ SwapError::InvalidFeeAccount
    )]
    pub pool_fee_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mint::token_program = token_source_program.key(),
    )]
    pub source_token_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mint::token_program = token_destination_program.key(),
        constraint = destination_token_mint.key() != source_token_mint.key() @ SwapError::RepeatedMint
    )]
    pub destination_token_mint: InterfaceAccount<'info, Mint>,

    pub token_source_program: Interface<'info, TokenInterface>,
    pub token_destination_program: Interface<'info, TokenInterface>,
    pub token_pool_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
