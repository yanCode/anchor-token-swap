use {
    crate::{curves::TradeDirection, to_u64, SwapError, SwapState, SwapV1},
    anchor_lang::prelude::*,
    anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface},
};

pub fn deposit_single_token_type_exact_amount_in_handler(
    ctx: Context<DepositSingleTokenType>,
    source_token_amount: u64,
    min_slippage_amount: u64,
) -> Result<()> {
    let swap_v1 = &ctx.accounts.swap_v1;
    let swap_curve = ctx.accounts.swap_v1.swap_curve();
    let calculator = swap_curve.calculator.as_ref();
    require!(
        calculator.allows_deposits(),
        SwapError::UnsupportedCurveOperation
    );
    let trade_direction = if ctx.accounts.source.mint == ctx.accounts.swap_token_a.mint {
        require_keys_neq!(
            ctx.accounts.source.key(),
            ctx.accounts.swap_token_a.key(),
            SwapError::SameAccountTransfer
        );
        TradeDirection::AtoB
    } else if ctx.accounts.source.mint == ctx.accounts.swap_token_b.mint {
        require_keys_neq!(
            ctx.accounts.source.key(),
            ctx.accounts.swap_token_b.key(),
            SwapError::SameAccountTransfer
        );
        TradeDirection::BtoA
    } else {
        return err!(SwapError::IncorrectSwapAccount);
    };

    let pool_mint_supply = ctx.accounts.pool_mint.supply;
    let pool_token_amount = if pool_mint_supply > 0 {
        swap_curve
            .deposit_single_token_type(
                source_token_amount as u128,
                ctx.accounts.swap_token_a.amount as u128,
                ctx.accounts.swap_token_b.amount as u128,
                pool_mint_supply as u128,
                trade_direction,
                swap_v1.fees(),
            )
            .ok_or(SwapError::ZeroTradingTokens)?
    } else {
        calculator.new_pool_supply()
    };
    let pool_token_amount = to_u64(pool_token_amount)?;
    if pool_token_amount == 0 {
        return err!(SwapError::ZeroTradingTokens);
    }
    if pool_token_amount < min_slippage_amount {
        return err!(SwapError::ExceededSlippage);
    }
    let to_swap_account_info = match trade_direction {
        TradeDirection::AtoB => ctx.accounts.swap_token_b.to_account_info(),
        TradeDirection::BtoA => ctx.accounts.swap_token_a.to_account_info(),
    };

    anchor_spl::token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.source_token_program.to_account_info(),
            anchor_spl::token_interface::TransferChecked {
                from: ctx.accounts.source.to_account_info(),
                to: to_swap_account_info,
                authority: ctx.accounts.user_transfer_authority.to_account_info(),
                mint: ctx.accounts.source_token_mint.to_account_info(),
            },
        ),
        source_token_amount,
        ctx.accounts.source_token_mint.decimals,
    )?;

    anchor_spl::token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_pool_program.to_account_info(),
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
        pool_token_amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct DepositSingleTokenType<'info> {
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
        token::mint = source_token_mint.key()
    )]
    pub source: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = swap_token_a.mint,
        constraint = source.key() != swap_token_a.key() @ SwapError::InvalidInput
    )]
    pub swap_token_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = swap_token_b.mint,
        constraint = source.key() != swap_token_b.key() @ SwapError::InvalidInput
    )]
    pub swap_token_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        mint::token_program = token_pool_program.key(),
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
      mut,
      token::mint = pool_mint.key()
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mint::token_program = source_token_program.key(),
    )]
    pub source_token_mint: InterfaceAccount<'info, Mint>,
    pub source_token_program: Interface<'info, TokenInterface>,
    pub token_pool_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
