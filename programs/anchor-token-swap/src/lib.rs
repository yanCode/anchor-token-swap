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
            ctx.accounts.fee_account.owner,
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
}
