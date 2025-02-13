use anchor_lang::prelude::*;
use anchor_spl::{token_2022::{Token2022, ID as TOKEN_2022_PROGRAM_ID}, token_interface::{Mint, TokenAccount}};

use crate::{SwapError, SwapV1};

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
        constraint = (swap_source.key() != token_swap.token_a.key()) 
        && (swap_source.key() != token_swap.token_b.key())
        @ SwapError::IncorrectSwapAccount,

        constraint = swap_source.key() != source.key() @ SwapError::InvalidInput,
        constraint = swap_source.key() != swap_destination.key() @ SwapError::InvalidInput
    )]
    pub swap_source: InterfaceAccount<'info, TokenAccount>,
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        token::token_program = token_swap.token_program_id,
        constraint = (swap_destination.key() != token_swap.token_a.key()) 
        && (swap_destination.key() != token_swap.token_b.key())
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
