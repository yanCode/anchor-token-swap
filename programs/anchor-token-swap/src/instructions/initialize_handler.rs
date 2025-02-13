use {
    crate::{
        curves::{CurveType, SwapCurve},
        Fees, Initialize, SwapV1,
    },
    anchor_lang::prelude::*,
    anchor_spl::token_2022::ID as TOKEN_2022_PROGRAM_ID,
};

pub fn initialize_handler(
    ctx: Context<Initialize>,
    curve_type: CurveType,
    fees: Fees,
) -> Result<()> {
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
            &[&[
                &ctx.accounts.swap_v1.key().to_bytes(),
                &[ctx.bumps.authority],
            ]],
        ),
        initial_amount as u64,
    )?;

    *ctx.accounts.swap_v1 = SwapV1 {
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
