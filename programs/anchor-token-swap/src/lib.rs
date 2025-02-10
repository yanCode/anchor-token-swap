mod curves;
mod errors;
mod fees;
mod state;
use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::ID as TOKEN_2022_PROGRAM_ID,
    token_interface::{Mint, TokenAccount},
};
pub use curves::*;
pub use errors::*;
pub use fees::*;
pub use state::*;

declare_id!("Bspu3p7dUX27mCSG5jaQkqoVwA6V2fMB9zZNpfu2dY9J");

#[program]
pub mod anchor_token_swap {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, swap_curve: SwapCurve, fees: Fees) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        let swap = &mut ctx.accounts.swap;
        let swap_key = swap.key();
        let seeds = &[swap_key.as_ref()];
        let (swap_authority, _) = Pubkey::find_program_address(seeds, ctx.program_id);
        require_keys_eq!(
            swap_authority,
            ctx.accounts.authority.key(),
            SwapError::InvalidProgramAddress
        );
        let calculator = swap_curve.calculator();

        calculator.validate_supply(ctx.accounts.token_a.amount, ctx.accounts.token_b.amount)?;
        let obj = SwapV1 {
            is_initialized: true,
            bump_seed: ctx.bumps.swap,
            token_program_id: TOKEN_2022_PROGRAM_ID,
            token_a: *ctx.accounts.token_a.to_account_info().key,
            token_b: *ctx.accounts.token_b.to_account_info().key,
            pool_mint: *ctx.accounts.pool_mint.to_account_info().key,
            token_a_mint: ctx.accounts.token_a.mint,
            token_b_mint: ctx.accounts.token_b.mint,
            pool_fee_account: *ctx.accounts.fee_account.to_account_info().key,
            fees,
            // swap_curve,
        };
        **swap = obj;

        Ok(())
    }

    pub fn peek_curve(ctx: Context<PeekCurve>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = SwapV1::INIT_SPACE + 8,
        seeds = [b"swap_v1".as_ref()],
        bump,
    )]
    pub swap: Account<'info, SwapV1>,
    pub authority: AccountInfo<'info>,
    #[account(
        token::token_program = TOKEN_2022_PROGRAM_ID,
        // token::delegate = None todo validate the delegate & close_authority
        constraint = token_a.delegate.is_none() @ SwapError::InvalidDelegate,
        constraint = token_a.mint != token_b.mint @ SwapError::RepeatedMint,
        constraint = token_a.owner != authority.key() @ SwapError::InvalidOwner,
    )]
    pub token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::token_program = TOKEN_2022_PROGRAM_ID,
        constraint = token_b.owner != authority.key() @ SwapError::InvalidOwner,
        constraint = token_b.delegate.is_none() @ SwapError::InvalidDelegate,

    )]
    pub token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mint::authority = authority.key(),
        mint::token_program = TOKEN_2022_PROGRAM_ID,
        constraint = pool_mint.supply > 0 @ SwapError::InvalidSupply,
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
        token::mint = pool_mint.key(),
        token::token_program = TOKEN_2022_PROGRAM_ID,
        constraint = destination.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::mint = pool_mint.key(),
        token::token_program = TOKEN_2022_PROGRAM_ID,
        constraint = fee_account.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub fee_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PeekCurve<'info> {
    #[account(
        seeds = [b"swap_v1".as_ref()],
        bump,
    )]
    pub swap: Account<'info, SwapV1>,
}
