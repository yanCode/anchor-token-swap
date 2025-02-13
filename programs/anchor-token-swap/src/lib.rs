mod account_infos;
pub mod curves;
mod errors;
pub mod instructions;

mod state;
mod swap_constraints;
use anchor_lang::prelude::*;

use crate::curves::CurveType;
pub use account_infos::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;
pub use swap_constraints::*;
declare_id!("Bspu3p7dUX27mCSG5jaQkqoVwA6V2fMB9zZNpfu2dY9J");

#[program]
pub mod anchor_token_swap {

    use crate::curves::CurveType;

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
        instructions::initialize_handler(ctx, curve_type, fees)
    }

    pub fn swap(ctx: Context<TokenSwap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
        instructions::swap_handler(ctx, amount_in, minimum_amount_out)
    }
}
