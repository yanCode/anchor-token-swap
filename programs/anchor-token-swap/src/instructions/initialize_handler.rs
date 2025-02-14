use {
    crate::{
        curves::{CurveType, SwapCurve},
        Fees, SwapError, SwapV1,
    },
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{Token2022, ID as TOKEN_2022_PROGRAM_ID},
        token_interface::{Mint, TokenAccount},
    },
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
        mut,
        mint::authority = authority.key(),
        mint::token_program = token_program.key(),
        constraint = pool_mint.supply == 0 @ SwapError::InvalidSupply,
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = pool_mint.key(),
        token::token_program = token_program.key(),
        constraint = destination.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
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
