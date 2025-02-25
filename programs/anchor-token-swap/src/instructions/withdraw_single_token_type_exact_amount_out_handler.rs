use {
    crate::{curves::TradeDirection, to_u64, SwapError, SwapState, SwapV1},
    anchor_lang::prelude::*,
    anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface},
};

pub fn withdraw_single_token_type_exact_amount_out_handler(
    ctx: Context<WithdrawSingleTokenTypeExactAmountOut>,
    destination_token_amount: u64,
    maximum_pool_token_amount: u64,
) -> Result<()> {
    let swap_v1 = &ctx.accounts.swap_v1;
    let swap_curve = &swap_v1.swap_curve();

    let trade_direction =
        if ctx.accounts.user_token_destination.mint == ctx.accounts.swap_token_a.mint {
            require_keys_neq!(
                ctx.accounts.user_token_destination.key(),
                ctx.accounts.swap_token_a.key(),
                SwapError::SameAccountTransfer
            );
            TradeDirection::AtoB
        } else if ctx.accounts.user_token_destination.mint == ctx.accounts.swap_token_b.mint {
            require_keys_neq!(
                ctx.accounts.user_token_destination.key(),
                ctx.accounts.swap_token_b.key(),
                SwapError::SameAccountTransfer
            );
            TradeDirection::BtoA
        } else {
            return err!(SwapError::IncorrectSwapAccount);
        };

    let pool_mint_supply = u128::from(ctx.accounts.pool_mint.supply);
    let swap_token_a_amount = u128::from(ctx.accounts.swap_token_a.amount);
    let swap_token_b_amount = u128::from(ctx.accounts.swap_token_b.amount);

    let burn_pool_token_amount = swap_curve
        .withdraw_single_token_type_exact_out(
            u128::from(destination_token_amount),
            swap_token_a_amount,
            swap_token_b_amount,
            pool_mint_supply,
            trade_direction,
            swap_v1.fees(),
        )
        .ok_or(SwapError::ZeroTradingTokens)?;

    let withdraw_fee = match &ctx.accounts.pool_fee_account {
        Some(ref pool_fee_account) => {
            if pool_fee_account.key() == ctx.accounts.pool_token_source.key() {
                0
            } else {
                swap_v1
                    .fees()
                    .owner_withdraw_fee(burn_pool_token_amount)
                    .ok_or(SwapError::FeeCalculationFailure)?
            }
        }
        None => 0,
    };
    let pool_token_amount = burn_pool_token_amount
        .checked_add(withdraw_fee)
        .ok_or(SwapError::FeeCalculationFailure)?;
    if to_u64(pool_token_amount)? > maximum_pool_token_amount {
        return err!(SwapError::ExceededSlippage);
    }
    if pool_token_amount == 0 {
        return err!(SwapError::ZeroTradingTokens);
    }
    if withdraw_fee > 0 {
        if let Some(pool_fee_account) = &ctx.accounts.pool_fee_account {
            anchor_spl::token_interface::transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.destination_token_program.to_account_info(),
                    anchor_spl::token_interface::TransferChecked {
                        from: ctx.accounts.user_token_destination.to_account_info(),
                        to: pool_fee_account.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                        mint: ctx.accounts.destination_token_mint.to_account_info(),
                    },
                    &[&[
                        &ctx.accounts.swap_v1.key().to_bytes(),
                        &[ctx.bumps.authority],
                    ]],
                ),
                to_u64(withdraw_fee)?,
                ctx.accounts.destination_token_mint.decimals,
            )?;
        }
    }
    anchor_spl::token_interface::burn_checked(
        CpiContext::new(
            ctx.accounts.token_pool_program.to_account_info(),
            anchor_spl::token_interface::BurnChecked {
                mint: ctx.accounts.pool_mint.to_account_info(),
                from: ctx.accounts.pool_token_source.to_account_info(),
                authority: ctx.accounts.user_transfer_authority.to_account_info(),
            },
        ),
        to_u64(pool_token_amount)?,
        ctx.accounts.pool_mint.decimals,
    )?;
    let from_token_account = match trade_direction {
        TradeDirection::AtoB => ctx.accounts.swap_token_a.to_account_info(),
        TradeDirection::BtoA => ctx.accounts.swap_token_b.to_account_info(),
    };
    anchor_spl::token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.destination_token_program.to_account_info(),
            anchor_spl::token_interface::TransferChecked {
                from: from_token_account,
                to: ctx.accounts.user_token_destination.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                mint: ctx.accounts.destination_token_mint.to_account_info(),
            },
            &[&[
                &ctx.accounts.swap_v1.key().to_bytes(),
                &[ctx.bumps.authority],
            ]],
        ),
        destination_token_amount,
        ctx.accounts.destination_token_mint.decimals,
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawSingleTokenTypeExactAmountOut<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
    constraint = !swap_v1.to_account_info().data_is_empty() @ SwapError::IncorrectSwapAccount,
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
        token::mint = pool_mint.key()
    )]
    pub pool_token_source: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = swap_token_a.mint,
        constraint = swap_v1.token_a == swap_token_a.key() @ SwapError::IncorrectSwapAccount
    )]
    pub swap_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = swap_token_b.mint,
        constraint = swap_v1.token_b == swap_token_b.key() @ SwapError::IncorrectSwapAccount
    )]
    pub swap_token_b: InterfaceAccount<'info, TokenAccount>,

    pub token_a_mint: InterfaceAccount<'info, Mint>,
    pub token_b_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        mint::token_program = token_pool_program.key(),
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = pool_mint.key()
    )]
    pub pool_fee_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = destination_token_mint.key()
    )]
    pub user_token_destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        mint::token_program = destination_token_program.key(),
    )]
    pub destination_token_mint: InterfaceAccount<'info, Mint>,
    pub destination_token_program: Interface<'info, TokenInterface>,
    pub token_pool_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
