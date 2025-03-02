use {crate::SwapError, anchor_lang::prelude::*};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Fees {
    /// Trade fees are extra token amounts that are held inside the token
    /// accounts during a trade, making the value of liquidity tokens rise.
    /// Trade fee numerator
    pub trade_fee_numerator: u64,
    /// Trade fee denominator
    pub trade_fee_denominator: u64,

    /// Owner trading fees are extra token amounts that are held inside the
    /// token accounts during a trade, with the equivalent in pool tokens
    /// minted to the owner of the program.
    /// Owner trade fee numerator
    pub owner_trade_fee_numerator: u64,
    /// Owner trade fee denominator
    pub owner_trade_fee_denominator: u64,

    /// Owner withdraw fees are extra liquidity pool token amounts that are
    /// sent to the owner on every withdrawal.
    /// Owner withdraw fee numerator
    pub owner_withdraw_fee_numerator: u64,
    /// Owner withdraw fee denominator
    pub owner_withdraw_fee_denominator: u64,

    /// Host fees are a proportion of the owner trading fees, sent to an
    /// extra account provided during the trade.
    /// Host trading fee numerator
    pub host_fee_numerator: u64,
    /// Host trading fee denominator
    pub host_fee_denominator: u64,
}
#[inline]
fn validate_fraction(numerator: u64, denominator: u64) -> Result<()> {
    if denominator == 0 && numerator == 0 {
        Ok(())
    } else if numerator >= denominator {
        err!(SwapError::InvalidFee)
    } else {
        Ok(())
    }
}

#[inline]
pub fn calculate_fee(amount: u128, fee_numerator: u128, fee_denominator: u128) -> Option<u128> {
    if fee_numerator == 0 || fee_denominator == 0 {
        Some(0)
    } else {
        let fee = amount
            .checked_mul(fee_numerator)?
            .checked_div(fee_denominator)?;
        if fee == 0 {
            Some(1)
        } else {
            Some(fee)
        }
    }
}
#[inline]
fn pre_fee_amount(
    post_fee_amount: u128,
    fee_numerator: u128,
    fee_denominator: u128,
) -> Option<u128> {
    if fee_numerator == 0 || fee_denominator == 0 {
        Some(post_fee_amount)
    } else if fee_numerator == fee_denominator || post_fee_amount == 0 {
        Some(0)
    } else {
        let numerator = post_fee_amount.checked_mul(fee_denominator)?;
        let denominator = fee_denominator.checked_sub(fee_numerator)?;
        ceil_div(numerator, denominator)
    }
}
#[inline]
fn ceil_div(dividend: u128, divisor: u128) -> Option<u128> {
    dividend
        .checked_add(divisor)?
        .checked_sub(1)?
        .checked_div(divisor)
}
impl Fees {
    #[inline]
    pub fn validate(&self) -> Result<()> {
        validate_fraction(self.trade_fee_numerator, self.trade_fee_denominator)?;
        validate_fraction(
            self.owner_trade_fee_numerator,
            self.owner_trade_fee_denominator,
        )?;
        validate_fraction(
            self.owner_withdraw_fee_numerator,
            self.owner_withdraw_fee_denominator,
        )?;
        validate_fraction(self.host_fee_numerator, self.host_fee_denominator)?;
        Ok(())
    }
    /// Calculate the host fee based on the owner fee, only used in production
    /// situations where a program is hosted by multiple frontends
    #[inline]
    pub fn host_fee(&self, owner_fee: u128) -> Option<u128> {
        calculate_fee(
            owner_fee,
            u128::from(self.host_fee_numerator),
            u128::from(self.host_fee_denominator),
        )
    }
    /// Calculate the trading fee in trading tokens
    #[inline]
    pub fn trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        calculate_fee(
            trading_tokens,
            u128::from(self.trade_fee_numerator),
            u128::from(self.trade_fee_denominator),
        )
    }

    /// Calculate the inverse trading amount, how much input is needed to give
    /// the provided output
    #[inline]
    pub fn pre_trading_fee_amount(&self, post_fee_amount: u128) -> Option<u128> {
        if self.trade_fee_numerator == 0 || self.trade_fee_denominator == 0 {
            pre_fee_amount(
                post_fee_amount,
                self.owner_trade_fee_numerator as u128,
                self.owner_trade_fee_denominator as u128,
            )
        } else if self.owner_trade_fee_numerator == 0 || self.owner_trade_fee_denominator == 0 {
            pre_fee_amount(
                post_fee_amount,
                self.trade_fee_numerator as u128,
                self.trade_fee_denominator as u128,
            )
        } else {
            pre_fee_amount(
                post_fee_amount,
                (self.trade_fee_numerator as u128)
                    .checked_mul(self.owner_trade_fee_denominator as u128)?
                    .checked_add(
                        (self.owner_trade_fee_numerator as u128)
                            .checked_mul(self.trade_fee_denominator as u128)?,
                    )?,
                (self.trade_fee_denominator as u128)
                    .checked_mul(self.owner_trade_fee_denominator as u128)?,
            )
        }
    }

    /// Calculate the owner trading fee in trading tokens
    #[inline]
    pub fn owner_trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        calculate_fee(
            trading_tokens,
            u128::from(self.owner_trade_fee_numerator),
            u128::from(self.owner_trade_fee_denominator),
        )
    }
    /// Calculate the withdraw fee in pool tokens
    #[inline]
    pub fn owner_withdraw_fee(&self, pool_tokens: u128) -> Option<u128> {
        calculate_fee(
            pool_tokens,
            u128::from(self.owner_withdraw_fee_numerator),
            u128::from(self.owner_withdraw_fee_denominator),
        )
    }
}
