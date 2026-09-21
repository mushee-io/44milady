#[derive(Clone, Copy, Debug)]
pub struct RiskSnapshot {
    pub collateral_value_usd_micro: u64,
    pub borrow_limit_usd_micro: u64,
    pub liquidation_capacity_usd_micro: u64,
    pub health_factor_bps: u64,
}

// -----------------------------------------------------------------------------
// Risk/oracle helpers
// -----------------------------------------------------------------------------

fn compute_portfolio_risk<'info>(
    credit_account: &CreditAccount,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<RiskSnapshot> {
    let expected = credit_account
        .collaterals
        .len()
        .checked_mul(2)
        .ok_or(MiladyError::MathOverflow)?;
    require!(remaining_accounts.len() == expected, MiladyError::InvalidRiskAccounts);

    let clock = Clock::get()?;
    let mut total_value: u128 = 0;
    let mut borrow_limit: u128 = 0;
    let mut liquidation_capacity: u128 = 0;

    for (index, collateral) in credit_account.collaterals.iter().enumerate() {
        let market_info = &remaining_accounts[index * 2];
        let price_info = &remaining_accounts[index * 2 + 1];

        require_keys_eq!(market_info.key(), collateral.market, MiladyError::MarketAccountMismatch);
        require_keys_eq!(*market_info.owner, crate::ID, MiladyError::InvalidMarketOwner);

        let market = {
            let data = market_info.try_borrow_data()?;
            let mut slice: &[u8] = &data;
            MarketConfig::try_deserialize(&mut slice)?
        };
        require!(market.enabled, MiladyError::MarketDisabled);

        require_keys_eq!(
            *price_info.owner,
            pyth_solana_receiver_sdk::ID,
            MiladyError::InvalidOracleOwner
        );
        let price_update = {
            let data = price_info.try_borrow_data()?;
            let mut slice: &[u8] = &data;
            PriceUpdateV2::try_deserialize(&mut slice)
                .map_err(|_| error!(MiladyError::OracleReadFailed))?
        };

        let price = price_update
            .get_price_no_older_than(
                &clock,
                u64::from(market.max_price_age_secs),
                &market.feed_id,
            )
            .map_err(|_| error!(MiladyError::OracleReadFailed))?;

        require!(price.price > 0, MiladyError::InvalidOraclePrice);
        validate_oracle_confidence(price.price, price.conf, market.max_confidence_bps)?;

        let value = token_value_usd_micro(
            collateral.amount,
            market.decimals,
            price.price,
            price.exponent,
        )?;
        let value_u128 = u128::from(value);
        total_value = total_value
            .checked_add(value_u128)
            .ok_or(MiladyError::MathOverflow)?;
        borrow_limit = borrow_limit
            .checked_add(weighted_value(value, market.ltv_bps)?)
            .ok_or(MiladyError::MathOverflow)?;
        liquidation_capacity = liquidation_capacity
            .checked_add(weighted_value(value, market.liquidation_threshold_bps)?)
            .ok_or(MiladyError::MathOverflow)?;
    }

    let total_value_u64 = u64::try_from(total_value).map_err(|_| error!(MiladyError::MathOverflow))?;
    let borrow_limit_u64 = u64::try_from(borrow_limit).map_err(|_| error!(MiladyError::MathOverflow))?;
    let liquidation_capacity_u64 =
        u64::try_from(liquidation_capacity).map_err(|_| error!(MiladyError::MathOverflow))?;

    Ok(RiskSnapshot {
        collateral_value_usd_micro: total_value_u64,
        borrow_limit_usd_micro: borrow_limit_u64,
        liquidation_capacity_usd_micro: liquidation_capacity_u64,
        health_factor_bps: health_factor_bps_for_debt(
            liquidation_capacity_u64,
            credit_account.debt_usdg,
        )?,
    })
}

fn health_factor_bps_for_debt(liquidation_capacity_usd_micro: u64, debt_usdg: u64) -> Result<u64> {
    if debt_usdg == 0 {
        return Ok(u64::MAX);
    }
    let health = u128::from(liquidation_capacity_usd_micro)
        .checked_mul(u128::from(BPS_DENOMINATOR))
        .ok_or(MiladyError::MathOverflow)?
        / u128::from(debt_usdg);
    Ok(u64::try_from(health).unwrap_or(u64::MAX))
}

fn pool_available_liquidity(pool: &LendingPool) -> Result<u64> {
    pool.total_supplied_usdg
        .checked_sub(pool.total_borrowed_usdg)
        .ok_or_else(|| error!(MiladyError::PoolAccountingInvariant))
}

fn token_value_usd_micro(amount: u64, decimals: u8, price: i64, exponent: i32) -> Result<u64> {
    require!(price > 0, MiladyError::InvalidOraclePrice);
    let price_u128 = u128::try_from(price).map_err(|_| error!(MiladyError::InvalidOraclePrice))?;

    let micro_price = if exponent >= 0 {
        price_u128
            .checked_mul(pow10(u32::try_from(exponent).map_err(|_| error!(MiladyError::MathOverflow))?)?)
            .and_then(|v| v.checked_mul(u128::from(USD_MICRO)))
            .ok_or(MiladyError::MathOverflow)?
    } else {
        let abs_exp = exponent.checked_abs().ok_or(MiladyError::MathOverflow)?;
        let scale = pow10(u32::try_from(abs_exp).map_err(|_| error!(MiladyError::MathOverflow))?)?;
        price_u128
            .checked_mul(u128::from(USD_MICRO))
            .ok_or(MiladyError::MathOverflow)?
            / scale
    };

    let token_scale = pow10(u32::from(decimals))?;
    let value = u128::from(amount)
        .checked_mul(micro_price)
        .ok_or(MiladyError::MathOverflow)?
        / token_scale;

    u64::try_from(value).map_err(|_| error!(MiladyError::MathOverflow))
}

fn validate_oracle_confidence(price: i64, conf: u64, max_confidence_bps: u16) -> Result<()> {
    require!(price > 0, MiladyError::InvalidOraclePrice);
    let price_abs = u128::try_from(price).map_err(|_| error!(MiladyError::InvalidOraclePrice))?;
    let lhs = u128::from(conf)
        .checked_mul(u128::from(BPS_DENOMINATOR))
        .ok_or(MiladyError::MathOverflow)?;
    let rhs = price_abs
        .checked_mul(u128::from(max_confidence_bps))
        .ok_or(MiladyError::MathOverflow)?;
    require!(lhs <= rhs, MiladyError::OracleConfidenceTooWide);
    Ok(())
}

fn weighted_value(value: u64, bps: u16) -> Result<u128> {
    let weighted = u128::from(value)
        .checked_mul(u128::from(bps))
        .ok_or(MiladyError::MathOverflow)?
        / u128::from(BPS_DENOMINATOR);
    Ok(weighted)
}

fn pow10(exponent: u32) -> Result<u128> {
    10_u128
        .checked_pow(exponent)
        .ok_or_else(|| error!(MiladyError::MathOverflow))
}

fn upsert_collateral(account: &mut CreditAccount, market: Pubkey, amount: u64) -> Result<()> {
    if let Some(balance) = account.collaterals.iter_mut().find(|entry| entry.market == market) {
        balance.amount = balance
            .amount
            .checked_add(amount)
            .ok_or(MiladyError::MathOverflow)?;
        return Ok(());
    }

    require!(
        account.collaterals.len() < MAX_COLLATERAL_ASSETS,
        MiladyError::TooManyCollateralAssets
    );
    account.collaterals.push(CollateralBalance { market, amount });
    Ok(())
}

fn reduce_collateral(account: &mut CreditAccount, market: Pubkey, amount: u64) -> Result<()> {
    let index = account
        .collaterals
        .iter()
        .position(|entry| entry.market == market)
        .ok_or(MiladyError::CollateralNotFound)?;

    let balance = account.collaterals[index]
        .amount
        .checked_sub(amount)
        .ok_or(MiladyError::InsufficientCollateral)?;
    account.collaterals[index].amount = balance;
    if balance == 0 {
        account.collaterals.remove(index);
    }
    Ok(())
}

fn assert_protocol_authority(config: &ProtocolConfig, authority: &Signer) -> Result<()> {
    require_keys_eq!(authority.key(), config.authority, MiladyError::Unauthorized);
    Ok(())
}

fn validate_market_args(args: &RegisterMarketArgs) -> Result<()> {
    validate_risk_parameters(
        args.ltv_bps,
        args.liquidation_threshold_bps,
        args.liquidation_bonus_bps,
        args.max_confidence_bps,
        args.max_price_age_secs,
    )
}

fn validate_update_market_args(args: &UpdateMarketArgs) -> Result<()> {
    require!(args.feed_id.iter().any(|b| *b != 0), MiladyError::InvalidFeedId);
    validate_risk_parameters(
        args.ltv_bps,
        args.liquidation_threshold_bps,
        args.liquidation_bonus_bps,
        args.max_confidence_bps,
        args.max_price_age_secs,
    )
}

fn validate_risk_parameters(
    ltv_bps: u16,
    liquidation_threshold_bps: u16,
    liquidation_bonus_bps: u16,
    max_confidence_bps: u16,
    max_price_age_secs: u32,
) -> Result<()> {
    require!(ltv_bps > 0 && ltv_bps < 10_000, MiladyError::InvalidLtv);
    require!(
        liquidation_threshold_bps > ltv_bps && liquidation_threshold_bps <= 10_000,
        MiladyError::InvalidLiquidationThreshold
    );
    require!(
        liquidation_bonus_bps <= MAX_LIQUIDATION_BONUS_BPS,
        MiladyError::InvalidLiquidationBonus
    );
    require!(
        max_confidence_bps > 0 && max_confidence_bps <= MAX_CONFIDENCE_BPS,
        MiladyError::InvalidConfidenceLimit
    );
    require!(
        max_price_age_secs > 0 && max_price_age_secs <= MAX_ORACLE_AGE_SECS,
        MiladyError::InvalidOracleAge
    );
    Ok(())
}
