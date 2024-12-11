use anchor_lang::prelude::*;

#[event]
pub struct PlatformInitialized {
    pub platform: Pubkey,
    pub owner: Pubkey,
}

#[event]
pub struct OwnerChanged {
    pub new_owner: Pubkey,
}

#[event]
pub struct FeesChanged {
    pub new_fees: u64,
}

#[event]
pub struct TotalSupplyChanged {
    pub new_total_supply: u64,
}

#[event]
pub struct VirtualSolChanged {
    pub new_virtual_sol_amount: u64,
}

#[event]
pub struct TargetPoolBalanceChanged {
    pub new_target_pool_balance: u64,
}

#[event]
pub struct FeesWithdrawn {
    pub amount: u64,
}

#[event]
pub struct TokenCreated {
    pub token: Pubkey,
}

#[event]
pub struct TokensSold {
    pub token: Pubkey,
    pub by: Pubkey,
    pub amount: u64,
}

#[event]
pub struct LiquidityAdded {
    pub token: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
}
