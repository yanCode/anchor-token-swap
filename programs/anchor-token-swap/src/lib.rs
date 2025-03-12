pub mod curves;
mod errors;
pub mod helper;
pub mod instructions;

mod state;
mod swap_constraints;
use {crate::curves::CurveType, anchor_lang::prelude::*};
pub use {errors::*, instructions::*, state::*, swap_constraints::*};
declare_id!("HUYZTGf7FbXt6A28HhvhMiN5SnFGBKe5F9t8Lqw7KSeG");

#[program]
pub mod anchor_token_swap {

    #[cfg(feature = "upgradable-test")]
    use crate::program::AnchorTokenSwap;
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
        ctx: Context<DepositSingleTokenType>,
        source_token_amount: u64,
        slippage_amount: u64,
    ) -> Result<()> {
        instructions::deposit_single_token_type_exact_amount_in_handler(
            ctx,
            source_token_amount,
            slippage_amount,
        )
    }
    pub fn withdraw_single_token_type_exact_amount_out(
        ctx: Context<WithdrawSingleTokenTypeExactAmountOut>,
        destination_token_amount: u64,
        maximum_pool_token_amount: u64,
    ) -> Result<()> {
        instructions::withdraw_single_token_type_exact_amount_out_handler(
            ctx,
            destination_token_amount,
            maximum_pool_token_amount,
        )
    }
    #[cfg(feature = "upgradable-test")]
    pub fn upgrade_verifier(_ctx: Context<UpgradableVerifier>) -> Result<()> {
        Ok(())
    }
    #[cfg(feature = "upgradable-test")]
    #[derive(Accounts)]
    pub struct UpgradableVerifier<'info> {
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(
            constraint = program.programdata_address()? == Some(program_data.key())
        )]
        pub program: Program<'info, AnchorTokenSwap>,
        #[account(
            constraint = program_data.upgrade_authority_address == Some(payer.key())
        )]
        pub program_data: Account<'info, ProgramData>,
        pub system_program: Program<'info, System>,
    }
}
