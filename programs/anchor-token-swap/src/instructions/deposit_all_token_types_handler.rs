use {
    crate::{
        curves::{RoundDirection, SwapCurve},
        SwapError, SwapV1,
    },
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::Token2022,
        token_interface::{Mint, TokenAccount},
    },
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

    require_gte!(
        token_a_amount,
        token_a_slippage_limit,
        SwapError::ExceededSlippage
    );
    require_gte!(
        token_b_amount,
        token_b_slippage_limit,
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

#[derive(Accounts)]
pub struct DepositAllTokenTypes<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
      constraint = !swap_v1.to_account_info().data_is_empty() @ SwapError::IncorrectSwapAccount,
      constraint = swap_v1.token_program_id == token_program.key() @ SwapError::IncorrectTokenProgramId
  )]
    pub swap_v1: Account<'info, SwapV1>,
    #[account(
      seeds = [swap_v1.key().as_ref()],
      bump,
  )]
    pub authority: AccountInfo<'info>,

    pub user_transfer_authority: Signer<'info>,
    #[account(
      mut,
      token::mint = token_a_mint.key(),
      constraint = source_a.key() != token_a.key() @ SwapError::InvalidInput,
    )]
    pub source_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
      mut,
      token::mint = token_b_mint.key(),
      constraint = source_b.key() != token_b.key() @ SwapError::InvalidInput,
    )]
    pub source_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
      mut,
      token::mint = token_a_mint.key(),
      constraint = token_a.key() == swap_v1.token_a @ SwapError::InvalidInput,
    )]
    pub token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
      mut,
      token::mint = token_b_mint.key(),
      constraint = token_b.key() == swap_v1.token_b @ SwapError::InvalidInput,
    )]
    pub token_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
      mut,
      constraint = pool_mint.key() == swap_v1.pool_mint @ SwapError::IncorrectPoolMint,
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
      constraint = token_a_mint.key() == swap_v1.token_a_mint @ SwapError::InvalidInput,
    )]
    pub token_a_mint: InterfaceAccount<'info, Mint>,
    #[account(
      constraint = token_b_mint.key() == swap_v1.token_b_mint @ SwapError::InvalidInput,
    )]
    pub token_b_mint: InterfaceAccount<'info, Mint>,
    #[account(
      mut,
      token::mint = pool_mint.key(),
      constraint = destination.key() != token_a.key() @ SwapError::InvalidInput,
      constraint = destination.key() != token_b.key() @ SwapError::InvalidInput,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
     token::mint = pool_mint.key(),
     constraint = pool_fee_account.key() == swap_v1.pool_fee_account @ SwapError::InvalidInput,
    )]
    pub pool_fee_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      constraint = token_program.key() == swap_v1.token_program_id @ SwapError::InvalidInput,
    )]
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
