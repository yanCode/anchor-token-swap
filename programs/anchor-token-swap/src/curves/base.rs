use anchor_lang::prelude::*;

use super::{
    constant_product::ConstantProductCurve, ConstantPriceCurve, CurveCalculator, OffsetCurve,
};

/// Initial amount of pool tokens for swap contract, hard-coded to something
/// "sensible" given a maximum of u128.
/// Note that on Ethereum, Uniswap uses the geometric mean of all provided
/// input amounts, and Balancer uses 100 * 10 ^ 18.

pub const INITIAL_SWAP_POOL_AMOUNT: u128 = 1_000_000_000;
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapCurve {
    pub curve_type: CurveType,
}
impl SwapCurve {
    pub fn calculator(&self) -> Box<dyn CurveCalculator> {
        match self.curve_type {
            CurveType::ConstantProduct => Box::new(ConstantProductCurve {}),
            CurveType::ConstantPrice { token_b_price } => {
                Box::new(ConstantPriceCurve { token_b_price })
            }
            CurveType::Offset { token_b_offset } => Box::new(OffsetCurve { token_b_offset }),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum CurveType {
    /// Uniswap-style constant product curve, invariant = token_a_amount *
    /// token_b_amount
    ConstantProduct,
    /// Flat line, always providing 1:1 from one token to another
    ConstantPrice { token_b_price: u64 },
    /// Offset curve, like Uniswap, but the token B side has a faked offset
    Offset { token_b_offset: u64 },
}
