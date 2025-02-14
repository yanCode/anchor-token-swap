use {
    crate::{SwapError, SwapV1},
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{Token2022, ID as TOKEN_2022_PROGRAM_ID},
        token_interface::{Mint, TokenAccount},
    },
};

#[derive(Accounts)]
pub struct TokenSwap<'info> {
    #[account(
        constraint = token_swap.is_initialized @ SwapError::IncorrectSwapAccount,
        constraint = token_swap.token_program_id == TOKEN_2022_PROGRAM_ID @ SwapError::IncorrectTokenProgramId
    )]
    pub token_swap: Account<'info, SwapV1>,
    #[account(
        seeds = [token_swap.key().as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,
    pub user_transfer_authority: UncheckedAccount<'info>,
    pub source: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::token_program = token_swap.token_program_id,
        constraint = (swap_source.key() != token_swap.token_a.key()) && (swap_source.key() != token_swap.token_b.key())
        @ SwapError::IncorrectSwapAccount,

        constraint = swap_source.key() != source.key() @ SwapError::InvalidInput,
        constraint = swap_source.key() != swap_destination.key() @ SwapError::InvalidInput
    )]
    pub swap_source: InterfaceAccount<'info, TokenAccount>,
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::token_program = token_swap.token_program_id,
        constraint = (swap_destination.key() != token_swap.token_a.key()) && (swap_destination.key() != token_swap.token_b.key())
        @ SwapError::IncorrectSwapAccount,

        constraint = swap_destination.key() != destination.key() @ SwapError::InvalidOutput

    )]
    pub swap_destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mint::token_program = token_swap.token_program_id,
        constraint = pool_mint.key() != token_swap.pool_mint.key() @ SwapError::IncorrectPoolMint
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,
    #[account(
        token::token_program = token_swap.token_program_id,
        token::mint = pool_mint.key(),
        constraint = host_fee_account.owner != authority.key() @ SwapError::InvalidOwner
    )]
    pub host_fee_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        token::token_program = token_swap.token_program_id,
        constraint = pool_fee_account.key() != token_swap.pool_fee_account.key() @ SwapError::InvalidFeeAccount
    )]
    pub pool_fee_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mint::token_program = token_swap.token_program_id,
        mint::authority = authority.key(),
    )]
    pub source_token_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mint::token_program = token_swap.token_program_id,
        mint::authority = authority.key(),
        constraint = destination_token_mint.key() != source_token_mint.key() @ SwapError::RepeatedMint
    )]
    pub destination_token_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
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

#[derive(Accounts)]
pub struct DepositAllTokenTypes<'info> {
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
      token::mint = token_a.mint,
      constraint = source_a.key() != token_b.key() @ SwapError::InvalidInput
    )]
    pub source_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
      token::mint = token_b.mint,
      constraint = source_b.key() != token_a.key() @ SwapError::InvalidInput
    )]
    pub source_b: InterfaceAccount<'info, TokenAccount>,
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
      constraint = token_a.key() == swap_v1.token_a @ SwapError::InvalidInput,
    )]
    pub token_a_mint: InterfaceAccount<'info, Mint>,
    #[account(
      constraint = token_b.key() == swap_v1.token_b @ SwapError::InvalidInput,
    )]
    pub token_b_mint: InterfaceAccount<'info, Mint>,
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
     token::mint = pool_mint.key(),
     constraint = pool_fee_account.key() != swap_v1.pool_fee_account @ SwapError::InvalidInput,
    )]
    pub pool_fee_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      token::token_program = swap_v1.token_program_id
    )]
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
