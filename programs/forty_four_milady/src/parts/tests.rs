#[cfg(test)]
mod tests {
    use super::*;

    fn test_pool() -> LendingPool {
        LendingPool {
            protocol: Pubkey::default(),
            usdg_mint: Pubkey::default(),
            liquidity_vault: Pubkey::default(),
            total_supplied_usdg: 10_000_000_000,
            total_borrowed_usdg: 8_000_000_000,
            protocol_reserves_usdg: 0,
            borrow_interest_remainder: 0,
            borrow_cap_usdg: 0,
            borrow_index_e18: INDEX_SCALE_E18,
            supply_index_e18: INDEX_SCALE_E18,
            last_accrual_ts: 0,
            reserve_factor_bps: 1_000,
            base_rate_bps: 200,
            slope1_bps: 800,
            slope2_bps: 5_000,
            kink_utilization_bps: 8_000,
            last_borrow_apr_bps: 1_000,
            last_supply_apr_bps: 720,
            borrow_enabled: true,
            bump: 255,
        }
    }

    #[test]
    fn converts_pyth_style_price_to_micro_usd() {
        let value = token_value_usd_micro(10_000_000, 6, 19_025_000_000, -8).unwrap();
        assert_eq!(value, 1_902_500_000);
    }

    #[test]
    fn validates_confidence_band() {
        assert!(validate_oracle_confidence(100_000_000, 500_000, 100).is_ok());
        assert!(validate_oracle_confidence(100_000_000, 2_000_000, 100).is_err());
    }

    #[test]
    fn weighted_ltv_math_is_exact() {
        let value = 10_000_000_000u64;
        assert_eq!(weighted_value(value, 6_000).unwrap(), 6_000_000_000u128);
    }

    #[test]
    fn health_factor_uses_liquidation_capacity_over_debt() {
        assert_eq!(health_factor_bps_for_debt(7_500_000_000, 5_000_000_000).unwrap(), 15_000);
        assert_eq!(health_factor_bps_for_debt(7_500_000_000, 0).unwrap(), u64::MAX);
    }

    #[test]
    fn kink_rate_curve_hits_expected_apr() {
        let pool = test_pool();
        assert_eq!(utilization_bps(&pool).unwrap(), 8_000);
        assert_eq!(borrow_apr_bps(&pool).unwrap(), 1_000);
        assert_eq!(supply_apr_bps(&pool, 1_000).unwrap(), 720);
    }

    #[test]
    fn jump_slope_reprices_above_kink() {
        let mut pool = test_pool();
        pool.total_borrowed_usdg = 9_000_000_000;
        assert_eq!(utilization_bps(&pool).unwrap(), 9_000);
        assert_eq!(borrow_apr_bps(&pool).unwrap(), 3_500);
    }

    #[test]
    fn one_year_interest_splits_suppliers_and_reserves() {
        let mut pool = test_pool();
        let accrual = accrue_pool_interest(&mut pool, Pubkey::default(), SECONDS_PER_YEAR as i64).unwrap();
        assert_eq!(accrual.gross_interest_usdg, 800_000_000);
        assert_eq!(accrual.reserve_interest_usdg, 80_000_000);
        assert_eq!(accrual.supplier_interest_usdg, 720_000_000);
        assert_eq!(pool.total_borrowed_usdg, 8_800_000_000);
        assert_eq!(pool.total_supplied_usdg, 10_720_000_000);
        assert_eq!(pool.protocol_reserves_usdg, 80_000_000);
        assert!(pool.borrow_index_e18 > INDEX_SCALE_E18);
        assert!(pool.supply_index_e18 > INDEX_SCALE_E18);
    }

    #[test]
    fn frequent_accrual_keeps_fractional_interest_remainder() {
        let (first, remainder) = annual_interest_with_remainder(1_000_000, 1_000, 1, 0).unwrap();
        assert_eq!(first, 0);
        assert!(remainder > 0);
        let (_, remainder2) = annual_interest_with_remainder(1_000_000, 1_000, 1, remainder).unwrap();
        assert!(remainder2 > remainder);
    }

    #[test]
    fn available_liquidity_includes_reserve_side_of_balance_sheet() {
        let mut pool = test_pool();
        pool.protocol_reserves_usdg = 80_000_000;
        pool.total_supplied_usdg = 10_720_000_000;
        pool.total_borrowed_usdg = 8_800_000_000;
        assert_eq!(pool_available_liquidity(&pool).unwrap(), 2_000_000_000);
    }
}
