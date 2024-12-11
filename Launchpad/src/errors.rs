use anchor_lang::prelude::*;

#[error_code]
pub enum CustomErrors {
    #[msg("Fee in bips should not exceed 500")]
    ExcessiveFees,
    #[msg("Not owner")]
    NotOwner,
    #[msg("Bonding curve breached")]
    BondingCurveBreached,
    #[msg("Already launched")]
    AlreadyLaunched,
    #[msg("Not launched")]
    NotLaunched,
}
