use crate::*;

uint::construct_uint!(
    pub struct U256(4);
);

pub const MAX_RATIO: u32 = 10000;

pub(crate) fn u128_ratio(a: u128, num: u128, denom: u128) -> Balance {
    (U256::from(a) * U256::from(num) / U256::from(denom)).as_u128()
}

pub fn ratio(balance: Balance, r: u32) -> Balance {
    assert!(r <= MAX_RATIO);
    u128_ratio(balance, u128::from(r), u128::from(MAX_RATIO))
}

pub fn nano_to_sec(nano: u64) -> u32 {
    (nano / 10u64.pow(9)) as u32
}