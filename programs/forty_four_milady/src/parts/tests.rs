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
}
