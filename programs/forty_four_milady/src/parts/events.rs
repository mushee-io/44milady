#[event]
pub struct ProtocolInitialized {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub emergency_authority: Pubkey,
    pub version: u16,
}

#[event]
pub struct ProtocolPauseChanged {
    pub paused: bool,
}

#[event]
pub struct MarketRegistered {
    pub market: Pubkey,
    pub mint: Pubkey,
    pub symbol: [u8; MAX_SYMBOL_BYTES],
    pub ltv_bps: u16,
    pub liquidation_threshold_bps: u16,
}

#[event]
pub struct MarketUpdated {
    pub market: Pubkey,
    pub enabled: bool,
    pub ltv_bps: u16,
    pub liquidation_threshold_bps: u16,
}

#[event]
pub struct FaucetAssetRegistered {
    pub mint: Pubkey,
    pub amount_per_claim: u64,
    pub cooldown_seconds: u32,
}

#[event]
pub struct FaucetClaimed {
    pub claimant: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub total_claimed: u64,
}

#[event]
pub struct CreditAccountInitialized {
    pub owner: Pubkey,
    pub credit_account: Pubkey,
}

#[event]
pub struct CollateralDeposited {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CollateralWithdrawn {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct HealthRefreshed {
    pub credit_account: Pubkey,
    pub collateral_value_usd_micro: u64,
    pub borrow_limit_usd_micro: u64,
    pub liquidation_capacity_usd_micro: u64,
    pub debt_usdg: u64,
    pub health_factor_bps: u64,
}

#[event]
pub struct LendingPoolInitialized {
    pub lending_pool: Pubkey,
    pub usdg_mint: Pubkey,
    pub liquidity_vault: Pubkey,
    pub borrow_cap_usdg: u64,
    pub reserve_factor_bps: u16,
}

#[event]
pub struct LendingPoolUpdated {
    pub lending_pool: Pubkey,
    pub borrow_cap_usdg: u64,
    pub reserve_factor_bps: u16,
    pub borrow_enabled: bool,
}

#[event]
pub struct UsdgSupplied {
    pub supplier: Pubkey,
    pub lending_pool: Pubkey,
    pub amount: u64,
    pub supplier_principal_usdg: u64,
    pub total_supplied_usdg: u64,
}

#[event]
pub struct UsdgSupplyWithdrawn {
    pub supplier: Pubkey,
    pub lending_pool: Pubkey,
    pub amount: u64,
    pub supplier_principal_usdg: u64,
    pub total_supplied_usdg: u64,
}

#[event]
pub struct UsdgBorrowed {
    pub borrower: Pubkey,
    pub lending_pool: Pubkey,
    pub amount: u64,
    pub new_debt_usdg: u64,
    pub total_borrowed_usdg: u64,
    pub borrow_limit_usd_micro: u64,
    pub health_factor_bps: u64,
}
