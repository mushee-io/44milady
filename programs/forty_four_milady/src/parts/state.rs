#[account]
#[derive(InitSpace)]
pub struct ProtocolConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub emergency_authority: Pubkey,
    pub version: u16,
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct MarketConfig {
    pub protocol: Pubkey,
    pub mint: Pubkey,
    pub symbol: [u8; MAX_SYMBOL_BYTES],
    pub feed_id: [u8; 32],
    pub decimals: u8,
    pub ltv_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub liquidation_bonus_bps: u16,
    pub max_confidence_bps: u16,
    pub max_price_age_secs: u32,
    pub supply_cap: u64,
    pub debt_ceiling_usdg: u64,
    pub total_deposited: u64,
    pub enabled: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct FaucetConfig {
    pub protocol: Pubkey,
    pub mint: Pubkey,
    pub amount_per_claim: u64,
    pub cooldown_seconds: u32,
    pub enabled: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct FaucetClaim {
    pub claimant: Pubkey,
    pub mint: Pubkey,
    pub last_claim_ts: i64,
    pub total_claimed: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct CollateralBalance {
    pub market: Pubkey,
    pub amount: u64,
}

#[account]
#[derive(InitSpace)]
pub struct CreditAccount {
    pub owner: Pubkey,
    /// USDG has 6 decimals, so raw debt units equal USD micro-units.
    pub debt_usdg: u64,
    /// Reserved now so Milestone 7 can add indexed interest without a schema break.
    pub borrow_index_snapshot_e18: u128,
    pub last_borrow_ts: i64,
    #[max_len(8)]
    pub collaterals: Vec<CollateralBalance>,
    pub last_collateral_value_usd_micro: u64,
    pub last_borrow_limit_usd_micro: u64,
    pub last_liquidation_capacity_usd_micro: u64,
    pub last_health_factor_bps: u64,
    pub last_valuation_ts: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct LendingPool {
    pub protocol: Pubkey,
    pub usdg_mint: Pubkey,
    pub liquidity_vault: Pubkey,
    pub total_supplied_usdg: u64,
    pub total_borrowed_usdg: u64,
    /// Reserve claim created from the protocol's share of borrower interest.
    pub protocol_reserves_usdg: u64,
    /// Carries sub-micro-unit borrower interest across frequent accrual calls.
    pub borrow_interest_remainder: u128,
    /// Zero means uncapped on Devnet.
    pub borrow_cap_usdg: u64,
    /// Global indexes, scaled by 1e18.
    pub borrow_index_e18: u128,
    pub supply_index_e18: u128,
    pub last_accrual_ts: i64,
    pub reserve_factor_bps: u16,
    /// Piecewise utilization curve: base + slope1 up to kink, then slope2.
    pub base_rate_bps: u32,
    pub slope1_bps: u32,
    pub slope2_bps: u32,
    pub kink_utilization_bps: u16,
    pub last_borrow_apr_bps: u32,
    pub last_supply_apr_bps: u32,
    pub borrow_enabled: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct SupplierPosition {
    pub owner: Pubkey,
    pub lending_pool: Pubkey,
    /// Principal-only balance until Milestone 7 activates supply interest.
    pub principal_usdg: u64,
    pub supply_index_snapshot_e18: u128,
    pub created_at: i64,
    pub last_updated_ts: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegisterMarketArgs {
    pub symbol: [u8; MAX_SYMBOL_BYTES],
    pub feed_id: [u8; 32],
    pub ltv_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub liquidation_bonus_bps: u16,
    pub max_confidence_bps: u16,
    pub max_price_age_secs: u32,
    /// Raw token units. Zero means uncapped on Devnet.
    pub supply_cap: u64,
    /// Reserved for per-market debt attribution in a later risk milestone.
    pub debt_ceiling_usdg: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UpdateMarketArgs {
    pub feed_id: [u8; 32],
    pub ltv_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub liquidation_bonus_bps: u16,
    pub max_confidence_bps: u16,
    pub max_price_age_secs: u32,
    pub supply_cap: u64,
    pub debt_ceiling_usdg: u64,
    pub enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitializeLendingPoolArgs {
    /// Zero means uncapped on Devnet.
    pub borrow_cap_usdg: u64,
    pub reserve_factor_bps: u16,
    pub base_rate_bps: u32,
    pub slope1_bps: u32,
    pub slope2_bps: u32,
    pub kink_utilization_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UpdateLendingPoolArgs {
    pub borrow_cap_usdg: u64,
    pub reserve_factor_bps: u16,
    pub base_rate_bps: u32,
    pub slope1_bps: u32,
    pub slope2_bps: u32,
    pub kink_utilization_bps: u16,
    pub borrow_enabled: bool,
}
