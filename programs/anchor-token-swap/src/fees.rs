use anchor_lang::prelude::*;

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
