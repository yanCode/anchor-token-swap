pub mod curves;
mod errors;
pub mod instructions;

mod state;
mod swap_constraints;
use {crate::curves::CurveType, anchor_lang::prelude::*};
pub use {errors::*, instructions::*, state::*, swap_constraints::*};
declare_id!("Bspu3p7dUX27mCSG5jaQkqoVwA6V2fMB9zZNpfu2dY9J");

#[program]
pub mod anchor_token_swap {

    use {super::*, crate::curves::CurveType};

    #[access_control(
        validate_swap_constraints(
            &curve_type,
            &fees,
            ctx.accounts.pool_fee_account.owner,
            None
        )
        validate_mint_uncloseable(&ctx.accounts.pool_mint)
    )]
    pub fn initialize(ctx: Context<Initialize>, curve_type: CurveType, fees: Fees) -> Result<()> {
        instructions::initialize_handler(ctx, curve_type, fees)
    }

    pub fn swap(ctx: Context<TokenSwap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
        instructions::swap_handler(ctx, amount_in, minimum_amount_out)
    }

    pub fn deposit_all_token_types(
        ctx: Context<DepositAllTokenTypes>,
        pool_token_amount: u64,
        token_a_slippage_limit: u64,
        token_b_slippage_limit: u64,
    ) -> Result<()> {
        instructions::deposit_all_token_types_handler(
            ctx,
            pool_token_amount,
            token_a_slippage_limit,
            token_b_slippage_limit,
        )
    }
    pub fn withdraw_all_token_types(
        ctx: Context<WithdrawAllTokenTypes>,
        pool_token_amount: u64,
        slippage_a_amount: u64,
        slippage_b_amount: u64,
    ) -> Result<()> {
        instructions::withdraw_all_token_types_handler(
            ctx,
            pool_token_amount,
            slippage_a_amount,
            slippage_b_amount,
        )
    }
    pub fn deposit_single_token_type_exact_amount_in(
        _ctx: Context<DepositSingleTokenTypeExactAmountIn>,
    ) -> Result<()> {
        todo!()
    }
    pub fn withdraw_single_token_type_exact_amount_out(
        _ctx: Context<WithdrawSingleTokenTypeExactAmountOut>,
    ) -> Result<()> {
        todo!()
    }
}

#[derive(Accounts)]
pub struct DepositSingleTokenTypeExactAmountIn<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawSingleTokenTypeExactAmountOut<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}
