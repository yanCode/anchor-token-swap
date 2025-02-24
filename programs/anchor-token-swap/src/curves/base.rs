use {
    super::{
        ConstantPriceCurve, ConstantProductCurve, CurveCalculator, OffsetCurve, RoundDirection,
        SwapWithoutFeesResult, TradeDirection,
    },
    crate::Fees,
    anchor_lang::prelude::*,
};

/// Initial amount of pool tokens for swap contract, hard-coded to something
/// "sensible" given a maximum of u128.
/// Note that on Ethereum, Uniswap uses the geometric mean of all provided
/// input amounts, and Balancer uses 100 * 10 ^ 18.

pub const INITIAL_SWAP_POOL_AMOUNT: u128 = 1_000_000_000;

pub struct SwapCurve {
    pub curve_type: CurveType,
    pub calculator: Box<dyn CurveCalculator>,
}
impl SwapCurve {
    pub fn new(curve_type: CurveType) -> Self {
        let calculator: Box<dyn CurveCalculator> = match curve_type {
            CurveType::ConstantProduct => Box::new(ConstantProductCurve {}),
            CurveType::ConstantPrice { token_b_price } => {
                Box::new(ConstantPriceCurve { token_b_price })
            }
            CurveType::Offset { token_b_offset } => Box::new(OffsetCurve { token_b_offset }),
        };
        SwapCurve {
            curve_type,
            calculator,
        }
    }
    /// Subtract fees and calculate how much destination token will be provided
    /// given an amount of source token.
    pub fn swap(
        &self,
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        trade_direction: TradeDirection,
        fees: &Fees,
    ) -> Option<SwapResult> {
        // debit the fee to calculate the amount swapped
        let trade_fee = fees.trading_fee(source_amount)?;
        let owner_fee = fees.owner_trading_fee(source_amount)?;

        let total_fees = trade_fee.checked_add(owner_fee)?;
        let source_amount_less_fees = source_amount.checked_sub(total_fees)?;

        let SwapWithoutFeesResult {
            source_amount_swapped,
            destination_amount_swapped,
        } = self.calculator.swap_without_fees(
            source_amount_less_fees,
            swap_source_amount,
            swap_destination_amount,
            trade_direction,
        )?;

        let source_amount_swapped = source_amount_swapped.checked_add(total_fees)?;
        Some(SwapResult {
            new_swap_source_amount: swap_source_amount.checked_add(source_amount_swapped)?,
            new_swap_destination_amount: swap_destination_amount
                .checked_sub(destination_amount_swapped)?,
            source_amount_swapped,
            destination_amount_swapped,
            trade_fee,
            owner_fee,
        })
    }
    /// Get the amount of pool tokens for the deposited amount of token A or B
    pub fn deposit_single_token_type(
        &self,
        source_amount: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        pool_supply: u128,
        trade_direction: TradeDirection,
        fees: &Fees,
    ) -> Option<u128> {
        if source_amount == 0 {
            return Some(0);
        }
        // Get the trading fee incurred if *half* the source amount is swapped
        // for the other side. Reference at:
        // https://github.com/balancer-labs/balancer-core/blob/f4ed5d65362a8d6cec21662fb6eae233b0babc1f/contracts/BMath.sol#L117
        let half_source_amount = std::cmp::max(1, source_amount.checked_div(2)?);
        let trade_fee = fees.trading_fee(half_source_amount)?;
        let owner_fee = fees.owner_trading_fee(half_source_amount)?;
        let total_fees = trade_fee.checked_add(owner_fee)?;
        let source_amount = source_amount.checked_sub(total_fees)?;
        self.calculator.deposit_single_token_type(
            source_amount,
            swap_token_a_amount,
            swap_token_b_amount,
            pool_supply,
            trade_direction,
        )
    }
    /// Get the amount of pool tokens for the withdrawn amount of token A or B
    pub fn withdraw_single_token_type_exact_out(
        &self,
        source_amount: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        pool_supply: u128,
        trade_direction: TradeDirection,
        fees: &Fees,
    ) -> Option<u128> {
        if source_amount == 0 {
            return Some(0);
        }
        // Since we want to get the amount required to get the exact amount out,
        // we need the inverse trading fee incurred if *half* the source amount
        // is swapped for the other side. Reference at:
        // https://github.com/balancer-labs/balancer-core/blob/f4ed5d65362a8d6cec21662fb6eae233b0babc1f/contracts/BMath.sol#L117
        let half_source_amount = source_amount.checked_add(1)?.checked_div(2)?; // round up
        let pre_fee_source_amount = fees.pre_trading_fee_amount(half_source_amount)?;
        let source_amount = source_amount
            .checked_sub(half_source_amount)?
            .checked_add(pre_fee_source_amount)?;
        self.calculator.withdraw_single_token_type_exact_out(
            source_amount,
            swap_token_a_amount,
            swap_token_b_amount,
            pool_supply,
            trade_direction,
            RoundDirection::Ceiling,
        )
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace, Copy)]
pub enum CurveType {
    /// Uniswap-style constant product curve, invariant = token_a_amount *
    /// token_b_amount
    ConstantProduct,
    /// Flat line, always providing 1:1 from one token to another
    ConstantPrice { token_b_price: u64 },
    /// Offset curve, like Uniswap, but the token B side has a faked offset
    Offset { token_b_offset: u64 },
}

pub struct SwapResult {
    /// New amount of source token
    pub new_swap_source_amount: u128,
    /// New amount of destination token
    pub new_swap_destination_amount: u128,
    /// Amount of source token swapped (includes fees)
    pub source_amount_swapped: u128,
    /// Amount of destination token swapped
    pub destination_amount_swapped: u128,
    /// Amount of source tokens going to pool holders
    pub trade_fee: u128,
    /// Amount of source tokens going to owner
    pub owner_fee: u128,
}
