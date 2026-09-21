    pub fn initialize_credit_account(ctx: Context<InitializeCreditAccount>) -> Result<()> {
        let account = &mut ctx.accounts.credit_account;
        account.owner = ctx.accounts.owner.key();
        account.debt_usdg = 0;
        account.borrow_index_snapshot_e18 = INDEX_SCALE_E18;
        account.last_borrow_ts = 0;
        account.collaterals = Vec::new();
        account.last_collateral_value_usd_micro = 0;
        account.last_borrow_limit_usd_micro = 0;
        account.last_liquidation_capacity_usd_micro = 0;
        account.last_health_factor_bps = u64::MAX;
        account.last_valuation_ts = 0;
        account.bump = ctx.bumps.credit_account;

        emit!(CreditAccountInitialized {
            owner: account.owner,
            credit_account: account.key(),
        });
        Ok(())
    }

    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, MiladyError::ProtocolPaused);
        require!(ctx.accounts.market_config.enabled, MiladyError::MarketDisabled);
        require!(amount > 0, MiladyError::InvalidAmount);

        let market_key = ctx.accounts.market_config.key();
        let new_total = ctx
            .accounts
            .market_config
            .total_deposited
            .checked_add(amount)
            .ok_or(MiladyError::MathOverflow)?;
        if ctx.accounts.market_config.supply_cap > 0 {
            require!(new_total <= ctx.accounts.market_config.supply_cap, MiladyError::SupplyCapExceeded);
        }

        upsert_collateral(
            &mut ctx.accounts.credit_account,
            market_key,
            amount,
        )?;
        ctx.accounts.market_config.total_deposited = new_total;

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.owner_token_account.to_account_info(),
            mint: ctx.accounts.collateral_mint.to_account_info(),
            to: ctx.accounts.collateral_vault.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token_interface::transfer_checked(cpi_ctx, amount, ctx.accounts.collateral_mint.decimals)?;

        emit!(CollateralDeposited {
            owner: ctx.accounts.owner.key(),
            market: market_key,
            mint: ctx.accounts.collateral_mint.key(),
            amount,
        });

        Ok(())
    }

    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, MiladyError::ProtocolPaused);
        require!(amount > 0, MiladyError::InvalidAmount);

        let market_key = ctx.accounts.market_config.key();
        reduce_collateral(&mut ctx.accounts.credit_account, market_key, amount)?;
        ctx.accounts.market_config.total_deposited = ctx
            .accounts
            .market_config
            .total_deposited
            .checked_sub(amount)
            .ok_or(MiladyError::MathOverflow)?;

        if ctx.accounts.credit_account.debt_usdg > 0 {
            let snapshot = compute_portfolio_risk(&ctx.accounts.credit_account, ctx.remaining_accounts)?;
            // Withdrawals must remain within the initial borrowing limit, not merely
            // above the liquidation threshold.
            require!(
                ctx.accounts.credit_account.debt_usdg <= snapshot.borrow_limit_usd_micro,
                MiladyError::UnsafeWithdrawal
            );
            require!(
                snapshot.health_factor_bps > BPS_DENOMINATOR,
                MiladyError::UnsafeWithdrawal
            );
        }

        let owner_key = ctx.accounts.owner.key();
        let bump = ctx.accounts.credit_account.bump;
        let bump_seed = [bump];
        let signer_seeds: &[&[u8]] = &[CREDIT_SEED, owner_key.as_ref(), &bump_seed];
        let signer = &[signer_seeds];

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.collateral_vault.to_account_info(),
            mint: ctx.accounts.collateral_mint.to_account_info(),
            to: ctx.accounts.owner_token_account.to_account_info(),
            authority: ctx.accounts.credit_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token_interface::transfer_checked(cpi_ctx, amount, ctx.accounts.collateral_mint.decimals)?;

        emit!(CollateralWithdrawn {
            owner: owner_key,
            market: market_key,
            mint: ctx.accounts.collateral_mint.key(),
            amount,
        });

        Ok(())
    }

    // ----- Milestones 4-5: Pyth valuation + risk engine -----

    /// Refreshes the persisted portfolio risk snapshot.
    ///
    /// `remaining_accounts` MUST contain ordered pairs for every collateral
    /// entry in `credit_account.collaterals`:
    /// [MarketConfig, Pyth PriceUpdateV2, MarketConfig, Pyth PriceUpdateV2, ...]
    /// The program validates both account ownership and that every market key
    /// matches the corresponding collateral entry.
    pub fn refresh_health(ctx: Context<RefreshHealth>) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, MiladyError::ProtocolPaused);
        let snapshot = compute_portfolio_risk(&ctx.accounts.credit_account, ctx.remaining_accounts)?;
        let account = &mut ctx.accounts.credit_account;
        account.last_collateral_value_usd_micro = snapshot.collateral_value_usd_micro;
        account.last_borrow_limit_usd_micro = snapshot.borrow_limit_usd_micro;
        account.last_liquidation_capacity_usd_micro = snapshot.liquidation_capacity_usd_micro;
        account.last_health_factor_bps = snapshot.health_factor_bps;
        account.last_valuation_ts = Clock::get()?.unix_timestamp;

        emit!(HealthRefreshed {
            credit_account: account.key(),
            collateral_value_usd_micro: snapshot.collateral_value_usd_micro,
            borrow_limit_usd_micro: snapshot.borrow_limit_usd_micro,
            liquidation_capacity_usd_micro: snapshot.liquidation_capacity_usd_micro,
            debt_usdg: account.debt_usdg,
            health_factor_bps: snapshot.health_factor_bps,
        });

        Ok(())
    }
