#[derive(Clone, Copy, Debug, Default)]
pub struct InterestAccrual {
    pub elapsed_seconds: u64,
    pub utilization_bps: u16,
    pub borrow_apr_bps: u32,
    pub supply_apr_bps: u32,
    pub gross_interest_usdg: u64,
    pub supplier_interest_usdg: u64,
    pub reserve_interest_usdg: u64,
}

fn validate_interest_model(
    reserve_factor_bps: u16,
    base_rate_bps: u32,
    slope1_bps: u32,
    slope2_bps: u32,
    kink_utilization_bps: u16,
) -> Result<()> {
    require!(
        reserve_factor_bps <= MAX_RESERVE_FACTOR_BPS,
        MiladyError::InvalidReserveFactor
    );
    require!(
        kink_utilization_bps > 0 && kink_utilization_bps < BPS_DENOMINATOR as u16,
        MiladyError::InvalidInterestKink
    );
    let max_rate = base_rate_bps
        .checked_add(slope1_bps)
        .and_then(|v| v.checked_add(slope2_bps))
        .ok_or(MiladyError::MathOverflow)?;
    require!(max_rate <= MAX_BORROW_APR_BPS, MiladyError::InterestRateTooHigh);
    Ok(())
}

fn pool_total_assets_usdg(pool: &LendingPool) -> Result<u64> {
    let gross_claims = pool
        .total_supplied_usdg
        .checked_add(pool.protocol_reserves_usdg)
        .and_then(|v| v.checked_add(pool.insurance_reserve_usdg))
        .ok_or(MiladyError::MathOverflow)?;

    gross_claims
        .checked_sub(pool.bad_debt_usdg)
        .ok_or_else(|| error!(MiladyError::PoolInsolvent))
}

fn pool_available_liquidity(pool: &LendingPool) -> Result<u64> {
    pool_total_assets_usdg(pool)?
        .checked_sub(pool.total_borrowed_usdg)
        .ok_or_else(|| error!(MiladyError::PoolAccountingInvariant))
}

fn utilization_bps(pool: &LendingPool) -> Result<u16> {
    if pool.total_borrowed_usdg == 0 {
        return Ok(0);
    }
    let assets = pool_total_assets_usdg(pool)?;
    require!(assets > 0, MiladyError::PoolAccountingInvariant);

    let utilization = u128::from(pool.total_borrowed_usdg)
        .checked_mul(u128::from(BPS_DENOMINATOR))
        .ok_or(MiladyError::MathOverflow)?
        / u128::from(assets);

    Ok(u16::try_from(utilization.min(u128::from(BPS_DENOMINATOR)))
        .map_err(|_| error!(MiladyError::MathOverflow))?)
}

fn borrow_apr_bps(pool: &LendingPool) -> Result<u32> {
    let utilization = u32::from(utilization_bps(pool)?);
    let kink = u32::from(pool.kink_utilization_bps);
    let bps = BPS_DENOMINATOR as u32;

    let rate = if utilization <= kink {
        let variable = u128::from(pool.slope1_bps)
            .checked_mul(u128::from(utilization))
            .ok_or(MiladyError::MathOverflow)?
            / u128::from(kink);
        u128::from(pool.base_rate_bps)
            .checked_add(variable)
            .ok_or(MiladyError::MathOverflow)?
    } else {
        let excess = utilization
            .checked_sub(kink)
            .ok_or(MiladyError::MathOverflow)?;
        let denominator = bps
            .checked_sub(kink)
            .ok_or(MiladyError::MathOverflow)?;
        let jump = u128::from(pool.slope2_bps)
            .checked_mul(u128::from(excess))
            .ok_or(MiladyError::MathOverflow)?
            / u128::from(denominator);

        u128::from(pool.base_rate_bps)
            .checked_add(u128::from(pool.slope1_bps))
            .and_then(|v| v.checked_add(jump))
            .ok_or(MiladyError::MathOverflow)?
    };

    let rate_u32 = u32::try_from(rate).map_err(|_| error!(MiladyError::MathOverflow))?;
    require!(rate_u32 <= MAX_BORROW_APR_BPS, MiladyError::InterestRateTooHigh);
    Ok(rate_u32)
}

fn supply_apr_bps(pool: &LendingPool, borrow_rate_bps: u32) -> Result<u32> {
    if pool.total_supplied_usdg == 0 || pool.total_borrowed_usdg == 0 {
        return Ok(0);
    }

    let supplier_share_bps = BPS_DENOMINATOR
        .checked_sub(u64::from(pool.reserve_factor_bps))
        .ok_or(MiladyError::MathOverflow)?;

    let numerator = u128::from(borrow_rate_bps)
        .checked_mul(u128::from(pool.total_borrowed_usdg))
        .and_then(|v| v.checked_mul(u128::from(supplier_share_bps)))
        .ok_or(MiladyError::MathOverflow)?;
    let denominator = u128::from(pool.total_supplied_usdg)
        .checked_mul(u128::from(BPS_DENOMINATOR))
        .ok_or(MiladyError::MathOverflow)?;

    u32::try_from(numerator / denominator).map_err(|_| error!(MiladyError::MathOverflow))
}

fn refresh_rate_cache(pool: &mut LendingPool) -> Result<()> {
    let borrow_rate = borrow_apr_bps(pool)?;
    let supply_rate = supply_apr_bps(pool, borrow_rate)?;
    pool.last_borrow_apr_bps = borrow_rate;
    pool.last_supply_apr_bps = supply_rate;
    Ok(())
}

fn annual_interest_with_remainder(
    principal: u64,
    apr_bps: u32,
    elapsed_seconds: u64,
    remainder: u128,
) -> Result<(u64, u128)> {
    if principal == 0 || apr_bps == 0 || elapsed_seconds == 0 {
        return Ok((0, remainder));
    }

    let denominator = u128::from(SECONDS_PER_YEAR)
        .checked_mul(u128::from(BPS_DENOMINATOR))
        .ok_or(MiladyError::MathOverflow)?;
    let numerator = u128::from(principal)
        .checked_mul(u128::from(apr_bps))
        .and_then(|v| v.checked_mul(u128::from(elapsed_seconds)))
        .and_then(|v| v.checked_add(remainder))
        .ok_or(MiladyError::MathOverflow)?;

    let interest = numerator / denominator;
    let new_remainder = numerator % denominator;

    Ok((
        u64::try_from(interest).map_err(|_| error!(MiladyError::MathOverflow))?,
        new_remainder,
    ))
}

fn apply_index_interest(index_e18: u128, principal: u64, interest: u64) -> Result<u128> {
    if principal == 0 || interest == 0 {
        return Ok(index_e18);
    }

    let increment = index_e18
        .checked_mul(u128::from(interest))
        .ok_or(MiladyError::MathOverflow)?
        / u128::from(principal);

    index_e18
        .checked_add(increment)
        .ok_or_else(|| error!(MiladyError::MathOverflow))
}

fn accrue_pool_interest(
    pool: &mut LendingPool,
    pool_key: Pubkey,
    now: i64,
) -> Result<InterestAccrual> {
    require!(now >= pool.last_accrual_ts, MiladyError::ClockWentBackwards);

    let elapsed_i64 = now
        .checked_sub(pool.last_accrual_ts)
        .ok_or(MiladyError::MathOverflow)?;
    let elapsed = u64::try_from(elapsed_i64).map_err(|_| error!(MiladyError::MathOverflow))?;
    let utilization = utilization_bps(pool)?;
    let rate = borrow_apr_bps(pool)?;
    let supply_rate = supply_apr_bps(pool, rate)?;

    if elapsed == 0 {
        return Ok(InterestAccrual {
            elapsed_seconds: 0,
            utilization_bps: utilization,
            borrow_apr_bps: rate,
            supply_apr_bps: supply_rate,
            ..InterestAccrual::default()
        });
    }

    let old_borrowed = pool.total_borrowed_usdg;
    let old_supplied = pool.total_supplied_usdg;
    let (gross_interest, new_remainder) = annual_interest_with_remainder(
        old_borrowed,
        rate,
        elapsed,
        pool.borrow_interest_remainder,
    )?;

    let reserve_interest = u64::try_from(
        u128::from(gross_interest)
            .checked_mul(u128::from(pool.reserve_factor_bps))
            .ok_or(MiladyError::MathOverflow)?
            / u128::from(BPS_DENOMINATOR),
    )
    .map_err(|_| error!(MiladyError::MathOverflow))?;
    let supplier_interest = gross_interest
        .checked_sub(reserve_interest)
        .ok_or(MiladyError::MathOverflow)?;

    pool.borrow_index_e18 = apply_index_interest(
        pool.borrow_index_e18,
        old_borrowed,
        gross_interest,
    )?;
    pool.supply_index_e18 = apply_index_interest(
        pool.supply_index_e18,
        old_supplied,
        supplier_interest,
    )?;

    pool.total_borrowed_usdg = pool
        .total_borrowed_usdg
        .checked_add(gross_interest)
        .ok_or(MiladyError::MathOverflow)?;
    pool.total_supplied_usdg = pool
        .total_supplied_usdg
        .checked_add(supplier_interest)
        .ok_or(MiladyError::MathOverflow)?;
    pool.protocol_reserves_usdg = pool
        .protocol_reserves_usdg
        .checked_add(reserve_interest)
        .ok_or(MiladyError::MathOverflow)?;
    pool.borrow_interest_remainder = new_remainder;
    pool.last_accrual_ts = now;

    let current_utilization = utilization_bps(pool)?;
    let current_borrow_rate = borrow_apr_bps(pool)?;
    let current_supply_rate = supply_apr_bps(pool, current_borrow_rate)?;
    pool.last_borrow_apr_bps = current_borrow_rate;
    pool.last_supply_apr_bps = current_supply_rate;

    let accrual = InterestAccrual {
        elapsed_seconds: elapsed,
        utilization_bps: current_utilization,
        borrow_apr_bps: rate,
        supply_apr_bps: supply_rate,
        gross_interest_usdg: gross_interest,
        supplier_interest_usdg: supplier_interest,
        reserve_interest_usdg: reserve_interest,
    };

    emit!(InterestAccrued {
        lending_pool: pool_key,
        elapsed_seconds: elapsed,
        utilization_bps: current_utilization,
        borrow_apr_bps: rate,
        supply_apr_bps: supply_rate,
        gross_interest_usdg: gross_interest,
        supplier_interest_usdg: supplier_interest,
        reserve_interest_usdg: reserve_interest,
        total_borrowed_usdg: pool.total_borrowed_usdg,
        total_supplied_usdg: pool.total_supplied_usdg,
        protocol_reserves_usdg: pool.protocol_reserves_usdg,
        borrow_index_e18: pool.borrow_index_e18,
        supply_index_e18: pool.supply_index_e18,
    });

    Ok(accrual)
}

fn sync_borrower_debt(
    account: &mut CreditAccount,
    pool: &LendingPool,
    now: i64,
) -> Result<u64> {
    require!(account.borrow_index_snapshot_e18 > 0, MiladyError::InvalidInterestIndex);
    require!(
        pool.borrow_index_e18 >= account.borrow_index_snapshot_e18,
        MiladyError::InvalidInterestIndex
    );

    if account.debt_usdg == 0 {
        account.borrow_index_snapshot_e18 = pool.borrow_index_e18;
        account.last_borrow_ts = now;
        return Ok(0);
    }

    let new_debt = u64::try_from(
        u128::from(account.debt_usdg)
            .checked_mul(pool.borrow_index_e18)
            .ok_or(MiladyError::MathOverflow)?
            / account.borrow_index_snapshot_e18,
    )
    .map_err(|_| error!(MiladyError::MathOverflow))?;

    let accrued = new_debt
        .checked_sub(account.debt_usdg)
        .ok_or(MiladyError::MathOverflow)?;
    account.debt_usdg = new_debt;
    account.borrow_index_snapshot_e18 = pool.borrow_index_e18;
    account.last_borrow_ts = now;
    Ok(accrued)
}

fn sync_supplier_balance(
    position: &mut SupplierPosition,
    pool: &LendingPool,
    now: i64,
) -> Result<u64> {
    require!(position.supply_index_snapshot_e18 > 0, MiladyError::InvalidInterestIndex);
    require!(
        pool.supply_index_e18 >= position.supply_index_snapshot_e18,
        MiladyError::InvalidInterestIndex
    );

    if position.principal_usdg == 0 {
        position.supply_index_snapshot_e18 = pool.supply_index_e18;
        position.last_updated_ts = now;
        return Ok(0);
    }

    let new_balance = u64::try_from(
        u128::from(position.principal_usdg)
            .checked_mul(pool.supply_index_e18)
            .ok_or(MiladyError::MathOverflow)?
            / position.supply_index_snapshot_e18,
    )
    .map_err(|_| error!(MiladyError::MathOverflow))?;

    let accrued = new_balance
        .checked_sub(position.principal_usdg)
        .ok_or(MiladyError::MathOverflow)?;
    position.principal_usdg = new_balance;
    position.supply_index_snapshot_e18 = pool.supply_index_e18;
    position.last_updated_ts = now;
    Ok(accrued)
}
