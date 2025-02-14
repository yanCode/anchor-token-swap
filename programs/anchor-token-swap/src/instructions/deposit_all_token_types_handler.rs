use {
    crate::{
        curves::{RoundDirection, SwapCurve},
        DepositAllTokenTypes, SwapError,
    },
    anchor_lang::prelude::*,
};

pub fn deposit_all_token_types_handler(
    ctx: Context<DepositAllTokenTypes>,
    pool_token_amount: u64,
    token_a_slippage_limit: u64,
    token_b_slippage_limit: u64,
) -> Result<()> {
    let token_swap = &ctx.accounts.swap_v1;
    let swap_curve = SwapCurve::new(token_swap.curve_type);
    let calculator = swap_curve.calculator;
    if !calculator.allows_deposits() {
        return err!(SwapError::UnsupportedCurveOperation);
    }

    let current_pool_supply = ctx.accounts.pool_mint.supply as u128;

    let (pool_token_amount, pool_mint_supply) = if current_pool_supply > 0 {
        (pool_token_amount as u128, current_pool_supply)
    } else {
        (calculator.new_pool_supply(), calculator.new_pool_supply())
    };
    let results = calculator
        .pool_tokens_to_trading_tokens(
            pool_token_amount,
            pool_mint_supply,
            ctx.accounts.token_a.amount as u128,
            ctx.accounts.token_b.amount as u128,
            RoundDirection::Ceiling,
        )
        .ok_or(SwapError::ZeroTradingTokens)?;
    let token_a_amount = results.token_a_amount as u64;
    let token_b_amount = results.token_b_amount as u64;

    require!(
        token_a_amount > token_a_slippage_limit,
        SwapError::ExceededSlippage
    );
    require!(
        token_b_amount > token_b_slippage_limit,
        SwapError::ExceededSlippage
    );

    require_neq!(token_a_amount, 0, SwapError::ZeroTradingTokens);
    require_neq!(token_b_amount, 0, SwapError::ZeroTradingTokens);

    anchor_spl::token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_interface::TransferChecked {
                from: ctx.accounts.source_a.to_account_info(),
                to: ctx.accounts.token_a.to_account_info(),
                authority: ctx.accounts.user_transfer_authority.to_account_info(),
                mint: ctx.accounts.token_a_mint.to_account_info(),
            },
        ),
        token_a_amount,
        ctx.accounts.token_a_mint.decimals,
    )?;

    anchor_spl::token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_interface::TransferChecked {
                from: ctx.accounts.source_b.to_account_info(),
                to: ctx.accounts.token_b.to_account_info(),
                authority: ctx.accounts.user_transfer_authority.to_account_info(),
                mint: ctx.accounts.token_b_mint.to_account_info(),
            },
        ),
        token_b_amount,
        ctx.accounts.token_b_mint.decimals,
    )?;

    anchor_spl::token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_interface::MintTo {
                mint: ctx.accounts.pool_mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
            &[&[
                &ctx.accounts.swap_v1.key().to_bytes(),
                &[ctx.bumps.authority],
            ]],
        ),
        pool_token_amount as u64,
    )?;
    Ok(())
}
