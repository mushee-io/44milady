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
    /// USDG uses 6 decimals, so raw debt units are also USD micro-units.
    pub debt_usdg: u64,
    #[max_len(8)]
    pub collaterals: Vec<CollateralBalance>,
    pub last_collateral_value_usd_micro: u64,
    pub last_borrow_limit_usd_micro: u64,
    pub last_liquidation_capacity_usd_micro: u64,
    pub last_health_factor_bps: u64,
    pub last_valuation_ts: i64,
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
    /// USDG raw units (6 decimals). Reserved for Milestone 6 borrowing.
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
