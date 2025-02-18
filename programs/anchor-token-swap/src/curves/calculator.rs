use {
    super::INITIAL_SWAP_POOL_AMOUNT, crate::SwapError, anchor_lang::prelude::*,
    spl_math::precise_number::PreciseNumber,
};

#[cfg(feature = "debug")]
pub trait CurveCalculator: CurveCalculatorTrait + Debug {}
#[cfg(not(feature = "debug"))]
pub trait CurveCalculator: CurveCalculatorTrait {}
pub trait CurveCalculatorTrait {
    /// Calculate how much destination token will be provided given an amount
    /// of source token.
    fn swap_without_fees(
        &self,
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        trade_direction: TradeDirection,
    ) -> Option<SwapWithoutFeesResult>;

    /// Get the supply for a new pool
    /// The default implementation is a Balancer-style fixed initial supply
    #[inline]
    fn new_pool_supply(&self) -> u128 {
        INITIAL_SWAP_POOL_AMOUNT
    }

    /// Get the amount of trading tokens for the given amount of pool tokens,
    /// provided the total trading tokens and supply of pool tokens.
    fn pool_tokens_to_trading_tokens(
        &self,
        pool_tokens: u128,
        pool_token_supply: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        round_direction: RoundDirection,
    ) -> Option<TradingTokenResult>;
    /// Validate that the given curve has no invalid parameters
    fn validate(&self) -> Result<()>;

    /// Validate the given supply on initialization. This is useful for curves
    /// that allow zero supply on one or both sides, since the standard constant
    /// product curve must have a non-zero supply on both sides.
    #[inline]
    fn validate_supply(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        if token_a_amount == 0 {
            return err!(SwapError::EmptySupply);
        }
        if token_b_amount == 0 {
            return err!(SwapError::EmptySupply);
        }
        Ok(())
    }

    /// Get the amount of pool tokens for the deposited amount of token A or B.
    ///
    /// This is used for single-sided deposits.  It essentially performs a swap
    /// followed by a deposit.  Because a swap is implicitly performed, this
    /// will change the spot price of the pool.
    ///
    /// See more background for the calculation at:
    ///
    /// <https://balancer.finance/whitepaper/#single-asset-deposit-withdrawal>
    fn deposit_single_token_type(
        &self,
        source_amount: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        pool_supply: u128,
        trade_direction: TradeDirection,
    ) -> Option<u128>;

    /// Get the amount of pool tokens for the withdrawn amount of token A or B.
    ///
    /// This is used for single-sided withdrawals and owner trade fee
    /// calculation. It essentially performs a withdrawal followed by a swap.
    /// Because a swap is implicitly performed, this will change the spot price
    /// of the pool.
    ///
    /// See more background for the calculation at:
    ///
    /// <https://balancer.finance/whitepaper/#single-asset-deposit-withdrawal>
    fn withdraw_single_token_type_exact_out(
        &self,
        source_amount: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        pool_supply: u128,
        trade_direction: TradeDirection,
        round_direction: RoundDirection,
    ) -> Option<u128>;

    /// Some curves function best and prevent attacks if we prevent deposits
    /// after initialization.  For example, the offset curve in `offset.rs`,
    /// which fakes supply on one side of the swap, allows the swap creator
    /// to steal value from all other depositors.
    fn allows_deposits(&self) -> bool {
        true
    }
    /// Calculates the total normalized value of the curve given the liquidity
    /// parameters.
    ///
    /// This value must have the dimension of `tokens ^ 1` For example, the
    /// standard Uniswap invariant has dimension `tokens ^ 2` since we are
    /// multiplying two token values together.  In order to normalize it, we
    /// also need to take the square root.
    ///
    /// This is useful for testing the curves, to make sure that value is not
    /// lost on any trade.  It can also be used to find out the relative value
    /// of pool tokens or liquidity tokens.
    fn normalized_value(
        &self,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
    ) -> Option<PreciseNumber>;
}

#[derive(Clone, PartialEq, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum TradeDirection {
    /// Input token A, output token B
    AtoB,
    /// Input token B, output token A
    BtoA,
}

impl TradeDirection {
    #[inline]
    pub fn opposite(&self) -> TradeDirection {
        match self {
            TradeDirection::AtoB => TradeDirection::BtoA,
            TradeDirection::BtoA => TradeDirection::AtoB,
        }
    }
}

pub enum RoundDirection {
    /// Floor the value, ie. 1.9 => 1, 1.1 => 1, 1.5 => 1
    Floor,
    /// Ceiling the value, ie. 1.9 => 2, 1.1 => 2, 1.5 => 2
    Ceiling,
}
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TradingTokenResult {
    /// Amount of token A
    pub token_a_amount: u128,
    /// Amount of token B
    pub token_b_amount: u128,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SwapWithoutFeesResult {
    /// Amount of source token swapped
    pub source_amount_swapped: u128,
    /// Amount of destination token swapped
    pub destination_amount_swapped: u128,
}

/// Helper function for mapping to SwapError::CalculationFailure
#[inline]
pub fn map_zero_to_none(x: u128) -> Option<u128> {
    if x == 0 {
        None
    } else {
        Some(x)
    }
}
