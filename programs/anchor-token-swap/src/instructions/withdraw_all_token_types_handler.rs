use {
    crate::{
        curves::{RoundDirection, SwapCurve},
        SwapError, SwapState, SwapV1,
    },
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::Token2022,
        token_interface::{Mint, TokenAccount},
    },
    std::cmp::min,
};

pub fn withdraw_all_token_types_handler(
    ctx: Context<WithdrawAllTokenTypes>,
    pool_token_amount: u64,
    slippage_a_amount: u64,
    slippage_b_amount: u64,
) -> Result<()> {
    let swap_curve = SwapCurve::new(ctx.accounts.swap_v1.curve_type);
    let calculator = swap_curve.calculator;
    let withdraw_fee = match &ctx.accounts.pool_fee_account {
        Some(_) => {
            if ctx.accounts.swap_v1.pool_fee_account.key() == ctx.accounts.source.key() {
                0
            } else {
                ctx.accounts
                    .swap_v1
                    .fees()
                    .owner_withdraw_fee(pool_token_amount as u128)
                    .ok_or(SwapError::FeeCalculationFailure)?
            }
        }
        None => 0,
    };
    let pool_token_amount = u128::from(pool_token_amount)
        .checked_sub(withdraw_fee)
        .ok_or(SwapError::CalculationFailure)?;
    let results = calculator
        .pool_tokens_to_trading_tokens(
            pool_token_amount,
            u128::from(ctx.accounts.pool_mint.supply),
            u128::from(ctx.accounts.token_a.amount),
            u128::from(ctx.accounts.token_b.amount),
            RoundDirection::Floor,
        )
        .ok_or(SwapError::ZeroTradingTokens)?;
    let mut token_a_amount = results.token_a_amount as u64;
    token_a_amount = min(token_a_amount, ctx.accounts.token_a.amount);
    require_gt!(
        token_a_amount,
        slippage_a_amount,
        SwapError::ExceededSlippage
    );
    require!(
        token_a_amount != 0 || ctx.accounts.token_a.amount == 0,
        SwapError::ZeroTradingTokens
    );
    let mut token_b_amount = results.token_b_amount as u64;
    token_b_amount = min(token_b_amount, ctx.accounts.token_b.amount);
    require_gt!(
        token_b_amount,
        slippage_b_amount,
        SwapError::ExceededSlippage
    );
    require!(
        token_b_amount != 0 || ctx.accounts.token_b.amount == 0,
        SwapError::ZeroTradingTokens
    );
    if withdraw_fee > 0 && ctx.accounts.pool_fee_account.is_some() {
        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: ctx.accounts.source.to_account_info(),
                    to: ctx
                        .accounts
                        .pool_fee_account
                        .as_ref()
                        .unwrap()
                        .to_account_info(),
                    authority: ctx.accounts.user_transfer_authority.to_account_info(),
                    mint: ctx.accounts.pool_mint.to_account_info(),
                },
                &[&[ctx.accounts.swap_v1.key().as_ref(), &[ctx.bumps.authority]]],
            ),
            withdraw_fee as u64,
            ctx.accounts.pool_mint.decimals,
        )?;
    }
    anchor_spl::token_interface::burn_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_interface::BurnChecked {
                from: ctx.accounts.source.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                mint: ctx.accounts.pool_mint.to_account_info(),
            },
            &[&[ctx.accounts.swap_v1.key().as_ref(), &[ctx.bumps.authority]]],
        ),
        token_a_amount,
        ctx.accounts.token_a_mint.decimals,
    )?;

    if token_a_amount > 0 {
        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: ctx.accounts.token_a.to_account_info(),
                    to: ctx.accounts.destination_a.to_account_info(),
                    authority: ctx.accounts.user_transfer_authority.to_account_info(),
                    mint: ctx.accounts.token_a_mint.to_account_info(),
                },
                &[&[ctx.accounts.swap_v1.key().as_ref(), &[ctx.bumps.authority]]],
            ),
            token_a_amount,
            ctx.accounts.token_a_mint.decimals,
        )?;
    }
    if token_b_amount > 0 {
        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: ctx.accounts.token_b.to_account_info(),
                    to: ctx.accounts.destination_b.to_account_info(),
                    authority: ctx.accounts.user_transfer_authority.to_account_info(),
                    mint: ctx.accounts.token_b_mint.to_account_info(),
                },
                &[&[ctx.accounts.swap_v1.key().as_ref(), &[ctx.bumps.authority]]],
            ),
            token_b_amount,
            ctx.accounts.token_b_mint.decimals,
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawAllTokenTypes<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub swap_v1: Account<'info, SwapV1>,
    #[account(
      seeds = [swap_v1.key().as_ref()],
      bump,
  )]
    pub authority: AccountInfo<'info>,

    pub user_transfer_authority: SystemAccount<'info>,
    #[account(
      constraint = token_a.key() == swap_v1.token_a @ SwapError::InvalidInput,
    )]
    pub token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
      constraint = token_b.key() == swap_v1.token_b @ SwapError::InvalidInput,
    )]
    pub token_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
      constraint = pool_mint.key() == swap_v1.pool_mint @ SwapError::IncorrectPoolMint,
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
      token::mint = token_a.mint,
      constraint = destination_a.key() != token_a.key() @ SwapError::InvalidInput
    )]
    pub destination_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
      token::mint = token_b.mint,
      constraint = destination_b.key() != token_a.key() @ SwapError::InvalidInput
    )]
    pub destination_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
      constraint = token_a_mint.key() == swap_v1.token_a_mint @ SwapError::InvalidInput,
    )]
    pub token_a_mint: InterfaceAccount<'info, Mint>,
    #[account(
      constraint = token_b_mint.key() == swap_v1.token_b_mint @ SwapError::InvalidInput,
    )]
    pub token_b_mint: InterfaceAccount<'info, Mint>,
    #[account(
      token::mint = pool_mint.key(),
      constraint = pool_fee_account.key() == swap_v1.pool_fee_account @ SwapError::InvalidInput,
    )]
    pub pool_fee_account: Option<InterfaceAccount<'info, TokenAccount>>,
    pub source: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    #[account(
      token::token_program = swap_v1.token_program_id
    )]
    pub token_program: Program<'info, Token2022>,
}
