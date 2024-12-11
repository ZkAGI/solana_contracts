use crate::constants::general;

pub fn get_amount_out(amount_in: &u128, reserve_in: &u128, reserve_out: &u128) -> u64 {
    (amount_in * reserve_out / (amount_in + reserve_in)) as u64
}

pub fn get_amount_using_spot_price(amount_in: &u128, reserve_in: &u128, reserve_out: &u128) -> u64 {
    ((amount_in * reserve_out) / reserve_in) as u64
}

pub fn calculate_sell_fee(amount: &u128, fee_in_bps: &u128) -> u64 {
    ((amount * fee_in_bps) / general::BPS as u128) as u64
}

pub fn calculate_buy_fee(amount: &u128, fee_in_bps: &u128) -> u64 {
    (amount * fee_in_bps / (general::BPS as u128 + fee_in_bps)) as u64
}
