use {
    super::Fees,
    crate::curves::{CurveType, SwapCurve},
    anchor_lang::prelude::*,
};

pub trait SwapState {
    // Address of token A liquidity account
    fn token_a_account(&self) -> &Pubkey;
    // Address of token B liquidity account
    fn token_b_account(&self) -> &Pubkey;
    // Address of pool token mint
    fn pool_mint(&self) -> &Pubkey;

    // Address of token A mint
    fn token_a_mint(&self) -> &Pubkey;
    // Address of token B mint
    fn token_b_mint(&self) -> &Pubkey;

    // Address of pool fee account
    fn pool_fee_account(&self) -> &Pubkey;

    // Fees associated with swap
    fn fees(&self) -> &Fees;
    // Curve associated with swap
    fn swap_curve(&self) -> SwapCurve;
}

/// All versions of SwapState
// #[enum_dispatch(SwapState)]
pub enum SwapVersion {
    /// Latest version, used for all new swaps
    SwapV1,
}

#[derive(InitSpace)]
#[non_exhaustive]
#[account]
pub struct SwapV1 {
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
    #[inline]
    fn token_a_account(&self) -> &Pubkey {
        &self.token_a
    }

    #[inline]
    fn token_b_account(&self) -> &Pubkey {
        &self.token_b
    }

    #[inline]
    fn pool_mint(&self) -> &Pubkey {
        &self.pool_mint
    }

    #[inline]
    fn token_a_mint(&self) -> &Pubkey {
        &self.token_a_mint
    }

    #[inline]
    fn token_b_mint(&self) -> &Pubkey {
        &self.token_b_mint
    }

    #[inline]
    fn pool_fee_account(&self) -> &Pubkey {
        &self.pool_fee_account
    }

    #[inline]
    fn fees(&self) -> &Fees {
        &self.fees
    }

    #[inline]
    fn swap_curve(&self) -> SwapCurve {
        SwapCurve::new(self.curve_type)
    }
}
