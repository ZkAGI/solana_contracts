pub mod general {
    pub const BPS: u16 = 10_000;
    pub const MAX_ALLOWED_FEE_IN_BPS: u64 = 500;
    pub const DECIMALS: u8 = 9;
    pub const DISCRIMINATOR_SIZE: usize = 8;
}

pub mod seeds {
    pub const PLATFORM_SEED: &[u8] = b"platform";
    pub const MINT_SEED: &[u8] = b"mint";
    pub const TOKEN_SEED: &[u8] = b"token";
    pub const TOKEN_ACCOUNT_SEED: &[u8] = b"token_account";
}
