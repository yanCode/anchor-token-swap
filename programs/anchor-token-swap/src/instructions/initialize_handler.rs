use {
    crate::{
        curves::{CurveType, SwapCurve},
        Fees, SwapError, SwapV1,
    },
    anchor_lang::prelude::*,
    anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface},
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
    calculator.validate_supply(
        ctx.accounts.swap_token_a.amount,
        ctx.accounts.swap_token_b.amount,
    )?;
    let initial_amount = swap_curve.calculator.new_pool_supply();
    anchor_spl::token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_pool_program.to_account_info(),
            anchor_spl::token_interface::MintTo {
                mint: ctx.accounts.pool_mint.to_account_info(),
                to: ctx.accounts.pool_token_reciever.to_account_info(),
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
        token_a: *ctx.accounts.swap_token_a.to_account_info().key,
        token_b: *ctx.accounts.swap_token_b.to_account_info().key,
        pool_mint: *ctx.accounts.pool_mint.to_account_info().key,
        token_a_mint: ctx.accounts.swap_token_a.mint,
        token_b_mint: ctx.accounts.swap_token_b.mint,
        pool_fee_account: *ctx.accounts.pool_fee_account.to_account_info().key,
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
      constraint = swap_token_a.delegate.is_none() @ SwapError::InvalidDelegate,
        constraint = swap_token_a.mint != swap_token_b.mint @ SwapError::RepeatedMint,
        constraint = swap_token_a.owner == authority.key() @ SwapError::InvalidOwner,
        constraint = swap_token_a.close_authority.is_none() @ SwapError::InvalidCloseAuthority,
    )]
    pub swap_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        constraint = swap_token_b.delegate.is_none() @ SwapError::InvalidDelegate,
        constraint = swap_token_b.owner == authority.key() @ SwapError::InvalidOwner,
        constraint = swap_token_b.close_authority.is_none() @ SwapError::InvalidCloseAuthority,
    )]
    pub swap_token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        mint::authority = authority.key(),
        mint::token_program = token_pool_program.key(),
        constraint = pool_mint.supply == 0 @ SwapError::InvalidSupply,
        constraint = pool_mint.freeze_authority.is_none() @ SwapError::InvalidFreezeAuthority,
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = pool_mint.key(),
        constraint = pool_token_reciever.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub pool_token_reciever: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = pool_mint.key(),
        constraint = pool_fee_account.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub pool_fee_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_pool_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
