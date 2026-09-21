    pub fn update_liquidation_config(
        ctx: Context<UpdateLiquidationConfig>,
        close_factor_bps: u16,
        enabled: bool,
    ) -> Result<()> {
        assert_protocol_authority(&ctx.accounts.protocol_config, &ctx.accounts.authority)?;
        require!(
            close_factor_bps > 0 && u64::from(close_factor_bps) <= BPS_DENOMINATOR,
            MiladyError::InvalidCloseFactor
        );

        let pool = &mut ctx.accounts.lending_pool;
        pool.liquidation_close_factor_bps = close_factor_bps;
        pool.liquidation_enabled = enabled;

        emit!(LiquidationConfigUpdated {
            lending_pool: pool.key(),
            close_factor_bps,
            enabled,
        });

        Ok(())
    }

    pub fn liquidate(ctx: Context<Liquidate>, max_repay_usdg: u64) -> Result<()> {
        require!(max_repay_usdg > 0, MiladyError::InvalidAmount);
        require!(
            ctx.accounts.lending_pool.liquidation_enabled,
            MiladyError::LiquidationDisabled
        );
        require!(ctx.accounts.usdg_mint.decimals == 6, MiladyError::InvalidUsdgDecimals);

        let now = Clock::get()?.unix_timestamp;
        let pool_key = ctx.accounts.lending_pool.key();
        accrue_pool_interest(&mut ctx.accounts.lending_pool, pool_key, now)?;
        sync_borrower_debt(
            &mut ctx.accounts.credit_account,
            &ctx.accounts.lending_pool,
            now,
        )?;

        require!(ctx.accounts.credit_account.debt_usdg > 0, MiladyError::NoDebtToRepay);

        let pre_risk = compute_portfolio_risk_for_liquidation(
            &ctx.accounts.credit_account,
            ctx.remaining_accounts,
        )?;
        require!(
            pre_risk.health_factor_bps <= BPS_DENOMINATOR,
            MiladyError::PositionNotLiquidatable
        );

        let market_key = ctx.accounts.market_config.key();
        let recorded_collateral = ctx
            .accounts
            .credit_account
            .collaterals
            .iter()
            .find(|entry| entry.market == market_key)
            .map(|entry| entry.amount)
            .ok_or(MiladyError::CollateralNotFound)?;
        require!(recorded_collateral > 0, MiladyError::CollateralNotFound);
        require!(
            ctx.accounts.collateral_vault.amount >= recorded_collateral,
            MiladyError::CollateralVaultAccountingMismatch
        );

        let clock = Clock::get()?;
        let price_info = ctx.accounts.price_update.to_account_info();
        let oracle = read_pyth_price(&price_info, &clock, &ctx.accounts.market_config)?;
        let collateral_value_usd_micro = token_value_usd_micro(
            recorded_collateral,
            ctx.accounts.market_config.decimals,
            oracle.price,
            oracle.exponent,
        )?;

        let debt = ctx.accounts.credit_account.debt_usdg;
        let close_limit_u128 = u128::from(debt)
            .checked_mul(u128::from(ctx.accounts.lending_pool.liquidation_close_factor_bps))
            .ok_or(MiladyError::MathOverflow)?
            / u128::from(BPS_DENOMINATOR);
        let close_limit = u64::try_from(close_limit_u128)
            .map_err(|_| error!(MiladyError::MathOverflow))?
            .max(1)
            .min(debt);

        let bonus_denominator = BPS_DENOMINATOR
            .checked_add(u64::from(ctx.accounts.market_config.liquidation_bonus_bps))
            .ok_or(MiladyError::MathOverflow)?;
        let max_repay_by_collateral = u64::try_from(
            u128::from(collateral_value_usd_micro)
                .checked_mul(u128::from(BPS_DENOMINATOR))
                .ok_or(MiladyError::MathOverflow)?
                / u128::from(bonus_denominator),
        )
        .map_err(|_| error!(MiladyError::MathOverflow))?;

        let repay_amount = max_repay_usdg
            .min(debt)
            .min(close_limit)
            .min(max_repay_by_collateral);
        require!(repay_amount > 0, MiladyError::LiquidationTooSmall);
        require!(
            ctx.accounts.liquidator_usdg_account.amount >= repay_amount,
            MiladyError::InsufficientRepaymentFunds
        );

        let seize_value_usd_micro = u64::try_from(
            u128::from(repay_amount)
                .checked_mul(u128::from(bonus_denominator))
                .ok_or(MiladyError::MathOverflow)?
                / u128::from(BPS_DENOMINATOR),
        )
        .map_err(|_| error!(MiladyError::MathOverflow))?;

        let collateral_to_seize = usd_micro_to_token_amount(
            seize_value_usd_micro,
            ctx.accounts.market_config.decimals,
            oracle.price,
            oracle.exponent,
        )?
        .min(recorded_collateral);
        require!(collateral_to_seize > 0, MiladyError::LiquidationTooSmall);

        // The liquidator repays USDG into the pool.
        let repay_cpi = TransferChecked {
            from: ctx.accounts.liquidator_usdg_account.to_account_info(),
            mint: ctx.accounts.usdg_mint.to_account_info(),
            to: ctx.accounts.liquidity_vault.to_account_info(),
            authority: ctx.accounts.liquidator.to_account_info(),
        };
        token_interface::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), repay_cpi),
            repay_amount,
            ctx.accounts.usdg_mint.decimals,
        )?;

        // The credit-account PDA releases discounted collateral to the liquidator.
        let borrower_key = ctx.accounts.borrower.key();
        let bump_seed = [ctx.accounts.credit_account.bump];
        let signer_seeds: &[&[u8]] = &[CREDIT_SEED, borrower_key.as_ref(), &bump_seed];
        let signer = &[signer_seeds];

        let seize_cpi = TransferChecked {
            from: ctx.accounts.collateral_vault.to_account_info(),
            mint: ctx.accounts.collateral_mint.to_account_info(),
            to: ctx.accounts.liquidator_collateral_account.to_account_info(),
            authority: ctx.accounts.credit_account.to_account_info(),
        };
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                seize_cpi,
                signer,
            ),
            collateral_to_seize,
            ctx.accounts.collateral_mint.decimals,
        )?;

        reduce_collateral(
            &mut ctx.accounts.credit_account,
            market_key,
            collateral_to_seize,
        )?;
        ctx.accounts.market_config.total_deposited = ctx
            .accounts
            .market_config
            .total_deposited
            .checked_sub(collateral_to_seize)
            .ok_or(MiladyError::MathOverflow)?;

        ctx.accounts.credit_account.debt_usdg = ctx
            .accounts
            .credit_account
            .debt_usdg
            .checked_sub(repay_amount)
            .ok_or(MiladyError::MathOverflow)?;
        ctx.accounts.lending_pool.total_borrowed_usdg = ctx
            .accounts
            .lending_pool
            .total_borrowed_usdg
            .checked_sub(repay_amount)
            .ok_or(MiladyError::PoolAccountingInvariant)?;

        let seized_value = token_value_usd_micro(
            collateral_to_seize,
            ctx.accounts.market_config.decimals,
            oracle.price,
            oracle.exponent,
        )?;
        let seized_borrow_limit = u64::try_from(weighted_value(
            seized_value,
            ctx.accounts.market_config.ltv_bps,
        )?)
        .map_err(|_| error!(MiladyError::MathOverflow))?;
        let seized_liquidation_capacity = u64::try_from(weighted_value(
            seized_value,
            ctx.accounts.market_config.liquidation_threshold_bps,
        )?)
        .map_err(|_| error!(MiladyError::MathOverflow))?;

        let account = &mut ctx.accounts.credit_account;
        account.borrow_index_snapshot_e18 = ctx.accounts.lending_pool.borrow_index_e18;
        account.last_borrow_ts = now;
        account.last_collateral_value_usd_micro = pre_risk
            .collateral_value_usd_micro
            .saturating_sub(seized_value);
        account.last_borrow_limit_usd_micro = pre_risk
            .borrow_limit_usd_micro
            .saturating_sub(seized_borrow_limit);
        account.last_liquidation_capacity_usd_micro = pre_risk
            .liquidation_capacity_usd_micro
            .saturating_sub(seized_liquidation_capacity);
        account.last_health_factor_bps = health_factor_bps_for_debt(
            account.last_liquidation_capacity_usd_micro,
            account.debt_usdg,
        )?;
        account.last_valuation_ts = now;

        refresh_rate_cache(&mut ctx.accounts.lending_pool)?;
        let available_liquidity_usdg = pool_available_liquidity(&ctx.accounts.lending_pool)?;

        emit!(PositionLiquidated {
            liquidator: ctx.accounts.liquidator.key(),
            borrower: borrower_key,
            lending_pool: ctx.accounts.lending_pool.key(),
            market: market_key,
            repay_usdg: repay_amount,
            collateral_seized: collateral_to_seize,
            liquidation_bonus_bps: ctx.accounts.market_config.liquidation_bonus_bps,
            debt_remaining_usdg: account.debt_usdg,
            health_before_bps: pre_risk.health_factor_bps,
            health_after_bps: account.last_health_factor_bps,
            available_liquidity_usdg,
        });

        Ok(())
    }

    pub fn fund_insurance_reserve(
        ctx: Context<FundInsuranceReserve>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, MiladyError::InvalidAmount);
        require!(ctx.accounts.usdg_mint.decimals == 6, MiladyError::InvalidUsdgDecimals);
        require!(
            ctx.accounts.funder_usdg_account.amount >= amount,
            MiladyError::InsufficientRepaymentFunds
        );

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.funder_usdg_account.to_account_info(),
            mint: ctx.accounts.usdg_mint.to_account_info(),
            to: ctx.accounts.liquidity_vault.to_account_info(),
            authority: ctx.accounts.funder.to_account_info(),
        };
        token_interface::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount,
            ctx.accounts.usdg_mint.decimals,
        )?;

        ctx.accounts.lending_pool.insurance_reserve_usdg = ctx
            .accounts
            .lending_pool
            .insurance_reserve_usdg
            .checked_add(amount)
            .ok_or(MiladyError::MathOverflow)?;
        refresh_rate_cache(&mut ctx.accounts.lending_pool)?;

        emit!(InsuranceReserveFunded {
            funder: ctx.accounts.funder.key(),
            lending_pool: ctx.accounts.lending_pool.key(),
            amount,
            insurance_reserve_usdg: ctx.accounts.lending_pool.insurance_reserve_usdg,
        });

        Ok(())
    }

    pub fn absorb_bad_debt(ctx: Context<AbsorbBadDebt>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let pool_key = ctx.accounts.lending_pool.key();
        accrue_pool_interest(&mut ctx.accounts.lending_pool, pool_key, now)?;
        sync_borrower_debt(
            &mut ctx.accounts.credit_account,
            &ctx.accounts.lending_pool,
            now,
        )?;

        require!(
            ctx.accounts.credit_account.collaterals.is_empty(),
            MiladyError::CollateralRemainsForLiquidation
        );
        let written_off = ctx.accounts.credit_account.debt_usdg;
        require!(written_off > 0, MiladyError::NoDebtToRepay);
        require!(
            ctx.accounts.lending_pool.total_borrowed_usdg >= written_off,
            MiladyError::PoolAccountingInvariant
        );

        let reserve_cover = written_off.min(ctx.accounts.lending_pool.protocol_reserves_usdg);
        let after_reserve = written_off
            .checked_sub(reserve_cover)
            .ok_or(MiladyError::MathOverflow)?;
        let insurance_cover = after_reserve.min(ctx.accounts.lending_pool.insurance_reserve_usdg);
        let uncovered = after_reserve
            .checked_sub(insurance_cover)
            .ok_or(MiladyError::MathOverflow)?;

        ctx.accounts.lending_pool.total_borrowed_usdg = ctx
            .accounts
            .lending_pool
            .total_borrowed_usdg
            .checked_sub(written_off)
            .ok_or(MiladyError::MathOverflow)?;
        ctx.accounts.lending_pool.protocol_reserves_usdg = ctx
            .accounts
            .lending_pool
            .protocol_reserves_usdg
            .checked_sub(reserve_cover)
            .ok_or(MiladyError::MathOverflow)?;
        ctx.accounts.lending_pool.insurance_reserve_usdg = ctx
            .accounts
            .lending_pool
            .insurance_reserve_usdg
            .checked_sub(insurance_cover)
            .ok_or(MiladyError::MathOverflow)?;
        ctx.accounts.lending_pool.bad_debt_usdg = ctx
            .accounts
            .lending_pool
            .bad_debt_usdg
            .checked_add(uncovered)
            .ok_or(MiladyError::MathOverflow)?;

        let account = &mut ctx.accounts.credit_account;
        account.debt_usdg = 0;
        account.borrow_index_snapshot_e18 = ctx.accounts.lending_pool.borrow_index_e18;
        account.last_borrow_ts = now;
        account.last_health_factor_bps = u64::MAX;

        refresh_rate_cache(&mut ctx.accounts.lending_pool)?;

        emit!(BadDebtAbsorbed {
            borrower: ctx.accounts.borrower.key(),
            lending_pool: ctx.accounts.lending_pool.key(),
            written_off_usdg: written_off,
            protocol_reserve_cover_usdg: reserve_cover,
            insurance_cover_usdg: insurance_cover,
            uncovered_bad_debt_usdg: uncovered,
            total_bad_debt_usdg: ctx.accounts.lending_pool.bad_debt_usdg,
        });

        Ok(())
    }

    pub fn recapitalize_bad_debt(
        ctx: Context<RecapitalizeBadDebt>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, MiladyError::InvalidAmount);
        require!(ctx.accounts.usdg_mint.decimals == 6, MiladyError::InvalidUsdgDecimals);
        require!(
            amount <= ctx.accounts.lending_pool.bad_debt_usdg,
            MiladyError::BadDebtCoverageExceedsDeficit
        );
        require!(
            ctx.accounts.funder_usdg_account.amount >= amount,
            MiladyError::InsufficientRepaymentFunds
        );

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.funder_usdg_account.to_account_info(),
            mint: ctx.accounts.usdg_mint.to_account_info(),
            to: ctx.accounts.liquidity_vault.to_account_info(),
            authority: ctx.accounts.funder.to_account_info(),
        };
        token_interface::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount,
            ctx.accounts.usdg_mint.decimals,
        )?;

        ctx.accounts.lending_pool.bad_debt_usdg = ctx
            .accounts
            .lending_pool
            .bad_debt_usdg
            .checked_sub(amount)
            .ok_or(MiladyError::MathOverflow)?;
        refresh_rate_cache(&mut ctx.accounts.lending_pool)?;

        emit!(BadDebtRecapitalized {
            funder: ctx.accounts.funder.key(),
            lending_pool: ctx.accounts.lending_pool.key(),
            amount,
            remaining_bad_debt_usdg: ctx.accounts.lending_pool.bad_debt_usdg,
        });

        Ok(())
    }
