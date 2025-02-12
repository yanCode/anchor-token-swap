use anchor_lang::prelude::*;

use crate::{CurveType, Fees, SwapCurve};


pub trait SwapState {
    /// Is the swap initialized, with data written to it
    fn is_initialized(&self) -> bool;
    /// Bump seed used to generate the program address / authority
    fn bump_seed(&self) -> u8;
    /// Token program ID associated with the swap
    fn token_program_id(&self) -> &Pubkey;
    /// Address of token A liquidity account
    fn token_a_account(&self) -> &Pubkey;
    /// Address of token B liquidity account
    fn token_b_account(&self) -> &Pubkey;
    /// Address of pool token mint
    fn pool_mint(&self) -> &Pubkey;

    /// Address of token A mint
    fn token_a_mint(&self) -> &Pubkey;
    /// Address of token B mint
    fn token_b_mint(&self) -> &Pubkey;

    /// Address of pool fee account
    fn pool_fee_account(&self) -> &Pubkey;
    /// Check if the pool fee info is a valid token program account
    /// capable of receiving tokens from the mint.
    fn check_pool_fee_info(&self, pool_fee_info: &AccountInfo) -> Result<()>;

    // Fees associated with swap
    fn fees(&self) -> &Fees;
    // /// Curve associated with swap
    fn swap_curve(&self) -> SwapCurve;
}

/// All versions of SwapState
// #[enum_dispatch(SwapState)]
pub enum SwapVersion {
    /// Latest version, used for all new swaps
    SwapV1,
}

#[derive(InitSpace)]
#[account]
pub struct SwapV1 {
    /// Initialized state.
    pub is_initialized: bool,
    /// Bump seed used in program address.
    /// The program address is created deterministically with the bump seed,
    /// swap program id, and swap account pubkey.  This program address has
    /// authority over the swap's token A account, token B account, and pool
    /// token mint.
    pub bump_seed: u8,

    /// Program ID of the tokens being exchanged.
    pub token_program_id: Pubkey,

    /// Token A
    pub token_a: Pubkey,
    /// Token B
    pub token_b: Pubkey,

    /// Pool tokens are issued when A or B tokens are deposited.
    /// Pool tokens can be withdrawn back to the original A or B token.
    pub pool_mint: Pubkey,

    /// Mint information for token A
    pub token_a_mint: Pubkey,
    /// Mint information for token B
    pub token_b_mint: Pubkey,

    /// Pool token account to receive trading and / or withdrawal fees
    pub pool_fee_account: Pubkey,
    // All fee information
    pub fees: Fees,
    // curve_type to construct CurveCalculator, which can be used by the SwapCurve, that
    // calculates swaps, deposits, and withdrawals
    pub curve_type: CurveType,
}

impl SwapState for SwapV1 {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn bump_seed(&self) -> u8 {
        self.bump_seed
    }

    fn token_program_id(&self) -> &Pubkey {
        &self.token_program_id
    }

    fn token_a_account(&self) -> &Pubkey {
        &self.token_a
    }

    fn token_b_account(&self) -> &Pubkey {
        &self.token_b
    }

    fn pool_mint(&self) -> &Pubkey {
        &self.pool_mint
    }

    fn token_a_mint(&self) -> &Pubkey {
        &self.token_a_mint
    }

    fn token_b_mint(&self) -> &Pubkey {
        &self.token_b_mint
    }

    fn pool_fee_account(&self) -> &Pubkey {
        &self.pool_fee_account
    }
    fn check_pool_fee_info(&self, _pool_fee_info: &AccountInfo) -> Result<()> {
        // let data = &pool_fee_info.data.borrow();
        // let token_account =
        //     StateWithExtensions::<Account>::unpack(data).map_err(|err| match err {
        //         ProgramError::InvalidAccountData | ProgramError::UninitializedAccount => {
        //             SwapError::InvalidFeeAccount.into()
        //         }
        //         _ => err,
        //     })?;
        // if pool_fee_info.owner != &self.token_program_id
        //     || token_account.base.state != AccountState::Initialized
        //     || token_account.base.mint != self.pool_mint
        // {
        //     msg!("Pool fee account is not owned by token program, is not initialized, or does not match stake pool's mint");
        //     return Err(SwapError::InvalidFeeAccount.into());
        // }
        Ok(())
    }
    fn fees(&self) -> &Fees {
        &self.fees
    }

    fn swap_curve(&self) -> SwapCurve {
        SwapCurve::new(self.curve_type)
    }
}
