use {crate::SwapError, anchor_lang::prelude::*};

pub fn to_u64(amount: u128) -> Result<u64> {
    amount
        .try_into()
        .map_err(|_| SwapError::ConversionFailure.into())
}
