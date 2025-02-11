use crate::{CurveType, Fees, SwapCurve, SwapError};
use anchor_lang::prelude::*;

/// Encodes fee constraints, used in multihost environments where the program
/// may be used by multiple frontends, to ensure that proper fees are being
/// assessed.
/// Since this struct needs to be created at compile-time, we only have access
/// to const functions and constructors. Since SwapCurve contains a Arc, it
/// cannot be used, so we have to split the curves based on their types.
pub struct SwapConstraints<'a> {
    /// Owner of the program
    pub owner_key: Option<Pubkey>,
    /// Valid curve types
    pub valid_curve_types: &'a [CurveType],
    /// Valid fees
    pub fees: &'a Fees,
}

impl<'a> SwapConstraints<'a> {
    pub fn validate_curve(&self, swap_curve: &SwapCurve) -> Result<()> {
        if !self.valid_curve_types.contains(&swap_curve.curve_type) {
            return err!(SwapError::UnsupportedCurveType);
        }
        Ok(())
    }

    /// Checks that the provided curve is valid for the given constraints
    pub fn validate_fees(&self, fees: &Fees) -> Result<()> {
        if fees.trade_fee_numerator >= self.fees.trade_fee_numerator
            && fees.trade_fee_denominator == self.fees.trade_fee_denominator
            && fees.owner_trade_fee_numerator >= self.fees.owner_trade_fee_numerator
            && fees.owner_trade_fee_denominator == self.fees.owner_trade_fee_denominator
            && fees.owner_withdraw_fee_numerator >= self.fees.owner_withdraw_fee_numerator
            && fees.owner_withdraw_fee_denominator == self.fees.owner_withdraw_fee_denominator
            && fees.host_fee_numerator == self.fees.host_fee_numerator
            && fees.host_fee_denominator == self.fees.host_fee_denominator
        {
            Ok(())
        } else {
            Err(SwapError::InvalidFee.into())
        }
    }
}
/// Fee structure defined by program creator in order to enforce certain
/// fees when others use the program.  Adds checks on pool creation and
/// swapping to ensure the correct fees and account owners are passed.
/// Fees provided during production build currently are considered min
/// fees that creator of the pool can specify. Host fee is a fixed
/// percentage that host receives as a portion of owner fees
pub const SWAP_CONSTRAINTS: Option<SwapConstraints> = {
    // #[cfg(feature = "production")]
    // {
    //     Some(SwapConstraints {
    //         owner_key: OWNER_KEY,
    //         valid_curve_types: VALID_CURVE_TYPES,
    //         fees: FEES,
    //     })
    // }
    // #[cfg(not(feature = "production"))]
    // {
    //     None
    // }
    None
};

pub fn validate_swap_constraints(
    swap_curve: &SwapCurve,
    fees: &Fees,
    fee_account_owner: Pubkey,
    constraints: Option<SwapConstraints>,
) -> Result<()> {
    if let Some(constraints) = constraints {
        if let Some(owner_key) = constraints.owner_key {
            require_keys_eq!(owner_key, fee_account_owner, SwapError::InvalidOwner);
        }
        constraints.validate_curve(swap_curve)?;
        constraints.validate_fees(fees)?;
    }
    Ok(())
}
