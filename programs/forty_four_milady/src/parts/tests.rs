#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_pyth_style_price_to_micro_usd() {
        // $190.25 represented as 19025000000 * 10^-8.
        let value = token_value_usd_micro(10_000_000, 6, 19_025_000_000, -8).unwrap();
        assert_eq!(value, 1_902_500_000); // $1,902.50 in micro USD.
    }

    #[test]
    fn validates_confidence_band() {
        assert!(validate_oracle_confidence(100_000_000, 500_000, 100).is_ok()); // 0.5% <= 1%
        assert!(validate_oracle_confidence(100_000_000, 2_000_000, 100).is_err()); // 2% > 1%
    }

    #[test]
    fn weighted_ltv_math_is_exact() {
        let value = 10_000_000_000u64; // $10k micro USD
        assert_eq!(weighted_value(value, 6_000).unwrap(), 6_000_000_000u128);
    }

    #[test]
    fn health_factor_uses_liquidation_capacity_over_debt() {
        assert_eq!(
            health_factor_bps_for_debt(7_500_000_000, 5_000_000_000).unwrap(),
            15_000
        );
        assert_eq!(
            health_factor_bps_for_debt(7_500_000_000, 0).unwrap(),
            u64::MAX
        );
    }

    #[test]
    fn pool_available_liquidity_is_supplied_minus_borrowed() {
        let pool = LendingPool {
            protocol: Pubkey::default(),
            usdg_mint: Pubkey::default(),
            liquidity_vault: Pubkey::default(),
            total_supplied_usdg: 10_000_000_000,
            total_borrowed_usdg: 3_000_000_000,
            borrow_cap_usdg: 0,
            borrow_index_e18: INDEX_SCALE_E18,
            supply_index_e18: INDEX_SCALE_E18,
            last_accrual_ts: 0,
            reserve_factor_bps: 1_000,
            borrow_enabled: true,
            bump: 255,
        };
        assert_eq!(pool_available_liquidity(&pool).unwrap(), 7_000_000_000);
    }
}
