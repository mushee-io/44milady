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

// Pyth Receiver program. Mainnet and Devnet currently use the same program id.
// We parse PriceUpdateV2 locally to avoid transitive Anchor/Borsh version drift
// in the receiver SDK while preserving its wire format and verification checks.
pub const PYTH_RECEIVER_ID: Pubkey = pubkey!("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ");
const PYTH_PRICE_UPDATE_V2_DISCRIMINATOR: [u8; 8] = [34, 241, 35, 99, 157, 126, 244, 205];

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum PythVerificationLevel {
    Partial { num_signatures: u8 },
    Full,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
struct PythPriceFeedMessage {
    feed_id: [u8; 32],
    price: i64,
    conf: u64,
    exponent: i32,
    publish_time: i64,
    prev_publish_time: i64,
    ema_price: i64,
    ema_conf: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
struct PythPriceUpdateV2 {
    write_authority: Pubkey,
    verification_level: PythVerificationLevel,
    price_message: PythPriceFeedMessage,
    posted_slot: u64,
}

#[derive(Clone, Copy, Debug)]
struct OraclePrice {
    price: i64,
    exponent: i32,
}

fn read_pyth_price(
    price_info: &AccountInfo,
    clock: &Clock,
    market: &MarketConfig,
) -> Result<OraclePrice> {
    require_keys_eq!(*price_info.owner, PYTH_RECEIVER_ID, MiladyError::InvalidOracleOwner);

    let data = price_info.try_borrow_data()?;
    require!(data.len() >= 8, MiladyError::OracleReadFailed);
    require!(
        &data[..8] == PYTH_PRICE_UPDATE_V2_DISCRIMINATOR.as_ref(),
        MiladyError::OracleReadFailed
    );

    let mut payload: &[u8] = &data[8..];
    let update = PythPriceUpdateV2::deserialize(&mut payload)
        .map_err(|_| error!(MiladyError::OracleReadFailed))?;

    require!(
        matches!(update.verification_level, PythVerificationLevel::Full),
        MiladyError::OracleReadFailed
    );
    require!(
        update.price_message.feed_id == market.feed_id,
        MiladyError::OracleReadFailed
    );

    let max_age = i64::from(market.max_price_age_secs);
    require!(
        update
            .price_message
            .publish_time
            .saturating_add(max_age)
            >= clock.unix_timestamp,
        MiladyError::OracleReadFailed
    );

    require!(update.price_message.price > 0, MiladyError::InvalidOraclePrice);
    validate_oracle_confidence(
        update.price_message.price,
        update.price_message.conf,
        market.max_confidence_bps,
    )?;

    Ok(OraclePrice {
        price: update.price_message.price,
        exponent: update.price_message.exponent,
    })
}

fn compute_portfolio_risk<'info>(
    credit_account: &CreditAccount,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<RiskSnapshot> {
    compute_portfolio_risk_internal(credit_account, remaining_accounts, true)
}

fn compute_portfolio_risk_for_liquidation<'info>(
    credit_account: &CreditAccount,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<RiskSnapshot> {
    compute_portfolio_risk_internal(credit_account, remaining_accounts, false)
}

fn compute_portfolio_risk_internal<'info>(
    credit_account: &CreditAccount,
    remaining_accounts: &[AccountInfo<'info>],
    require_enabled: bool,
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
        if require_enabled {
            require!(market.enabled, MiladyError::MarketDisabled);
        }

        let price = read_pyth_price(price_info, &clock, &market)?;

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

fn usd_micro_to_token_amount(
    usd_micro: u64,
    decimals: u8,
    price: i64,
    exponent: i32,
) -> Result<u64> {
    require!(price > 0, MiladyError::InvalidOraclePrice);
    let price_u128 = u128::try_from(price).map_err(|_| error!(MiladyError::InvalidOraclePrice))?;
    let token_scale = pow10(u32::from(decimals))?;

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

    require!(micro_price > 0, MiladyError::InvalidOraclePrice);
    let raw = u128::from(usd_micro)
        .checked_mul(token_scale)
        .ok_or(MiladyError::MathOverflow)?
        / micro_price;

    u64::try_from(raw).map_err(|_| error!(MiladyError::MathOverflow))
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
