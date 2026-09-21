    pub fn initialize_lending_pool(
        ctx: Context<InitializeLendingPool>,
        args: InitializeLendingPoolArgs,
    ) -> Result<()> {
        assert_protocol_authority(&ctx.accounts.protocol_config, &ctx.accounts.authority)?;
        require!(!ctx.accounts.protocol_config.paused, MiladyError::ProtocolPaused);
        require!(ctx.accounts.usdg_mint.decimals == 6, MiladyError::InvalidUsdgDecimals);
        require!(
            args.reserve_factor_bps <= MAX_RESERVE_FACTOR_BPS,
            MiladyError::InvalidReserveFactor
        );

        let now = Clock::get()?.unix_timestamp;
        let pool = &mut ctx.accounts.lending_pool;
        pool.protocol = ctx.accounts.protocol_config.key();
        pool.usdg_mint = ctx.accounts.usdg_mint.key();
        pool.liquidity_vault = ctx.accounts.liquidity_vault.key();
        pool.total_supplied_usdg = 0;
        pool.total_borrowed_usdg = 0;
        pool.borrow_cap_usdg = args.borrow_cap_usdg;
        pool.borrow_index_e18 = INDEX_SCALE_E18;
        pool.supply_index_e18 = INDEX_SCALE_E18;
        pool.last_accrual_ts = now;
        pool.reserve_factor_bps = args.reserve_factor_bps;
        pool.borrow_enabled = false;
        pool.bump = ctx.bumps.lending_pool;

        emit!(LendingPoolInitialized {
            lending_pool: pool.key(),
            usdg_mint: pool.usdg_mint,
            liquidity_vault: pool.liquidity_vault,
            borrow_cap_usdg: pool.borrow_cap_usdg,
            reserve_factor_bps: pool.reserve_factor_bps,
        });

        Ok(())
    }

    pub fn update_lending_pool(
        ctx: Context<UpdateLendingPool>,
        args: UpdateLendingPoolArgs,
    ) -> Result<()> {
        assert_protocol_authority(&ctx.accounts.protocol_config, &ctx.accounts.authority)?;
        require!(
            args.reserve_factor_bps <= MAX_RESERVE_FACTOR_BPS,
            MiladyError::InvalidReserveFactor
        );
        if args.borrow_cap_usdg > 0 {
            require!(
                args.borrow_cap_usdg >= ctx.accounts.lending_pool.total_borrowed_usdg,
                MiladyError::BorrowCapBelowOutstandingDebt
            );
        }

        let pool = &mut ctx.accounts.lending_pool;
        pool.borrow_cap_usdg = args.borrow_cap_usdg;
        pool.reserve_factor_bps = args.reserve_factor_bps;
        pool.borrow_enabled = args.borrow_enabled;

        emit!(LendingPoolUpdated {
            lending_pool: pool.key(),
            borrow_cap_usdg: pool.borrow_cap_usdg,
            reserve_factor_bps: pool.reserve_factor_bps,
            borrow_enabled: pool.borrow_enabled,
        });

        Ok(())
    }

    pub fn supply_usdg(ctx: Context<SupplyUsdg>, amount: u64) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, MiladyError::ProtocolPaused);
        require!(amount > 0, MiladyError::InvalidAmount);
        require!(ctx.accounts.usdg_mint.decimals == 6, MiladyError::InvalidUsdgDecimals);

        let now = Clock::get()?.unix_timestamp;
        let position = &mut ctx.accounts.supplier_position;
        if position.owner == Pubkey::default() {
            position.owner = ctx.accounts.supplier.key();
            position.lending_pool = ctx.accounts.lending_pool.key();
            position.principal_usdg = 0;
            position.supply_index_snapshot_e18 = ctx.accounts.lending_pool.supply_index_e18;
            position.created_at = now;
            position.last_updated_ts = now;
            position.bump = ctx.bumps.supplier_position;
        }

        require_keys_eq!(position.owner, ctx.accounts.supplier.key(), MiladyError::Unauthorized);
        require_keys_eq!(
            position.lending_pool,
            ctx.accounts.lending_pool.key(),
            MiladyError::InvalidLendingPool
        );

        let new_position_principal = position
            .principal_usdg
            .checked_add(amount)
            .ok_or(MiladyError::MathOverflow)?;
        let new_total_supplied = ctx
            .accounts
            .lending_pool
            .total_supplied_usdg
            .checked_add(amount)
            .ok_or(MiladyError::MathOverflow)?;

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.supplier_usdg_account.to_account_info(),
            mint: ctx.accounts.usdg_mint.to_account_info(),
            to: ctx.accounts.liquidity_vault.to_account_info(),
            authority: ctx.accounts.supplier.to_account_info(),
        };
        token_interface::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount,
            ctx.accounts.usdg_mint.decimals,
        )?;

        position.principal_usdg = new_position_principal;
        position.supply_index_snapshot_e18 = ctx.accounts.lending_pool.supply_index_e18;
        position.last_updated_ts = now;
        ctx.accounts.lending_pool.total_supplied_usdg = new_total_supplied;

        emit!(UsdgSupplied {
            supplier: ctx.accounts.supplier.key(),
            lending_pool: ctx.accounts.lending_pool.key(),
            amount,
            supplier_principal_usdg: position.principal_usdg,
            total_supplied_usdg: new_total_supplied,
        });

        Ok(())
    }

    pub fn withdraw_supplied_usdg(
        ctx: Context<WithdrawSuppliedUsdg>,
        amount: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, MiladyError::ProtocolPaused);
        require!(amount > 0, MiladyError::InvalidAmount);
        require!(ctx.accounts.usdg_mint.decimals == 6, MiladyError::InvalidUsdgDecimals);
        require!(
            ctx.accounts.supplier_position.principal_usdg >= amount,
            MiladyError::InsufficientSupply
        );

        let available = pool_available_liquidity(&ctx.accounts.lending_pool)?;
        require!(available >= amount, MiladyError::InsufficientLiquidity);
        require!(
            ctx.accounts.liquidity_vault.amount >= amount,
            MiladyError::InsufficientLiquidity
        );

        let new_position_principal = ctx
            .accounts
            .supplier_position
            .principal_usdg
            .checked_sub(amount)
            .ok_or(MiladyError::MathOverflow)?;
        let new_total_supplied = ctx
            .accounts
            .lending_pool
            .total_supplied_usdg
            .checked_sub(amount)
            .ok_or(MiladyError::MathOverflow)?;

        let mint_key = ctx.accounts.usdg_mint.key();
        let bump_seed = [ctx.accounts.lending_pool.bump];
        let signer_seeds: &[&[u8]] = &[LENDING_POOL_SEED, mint_key.as_ref(), &bump_seed];
        let signer = &[signer_seeds];

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.liquidity_vault.to_account_info(),
            mint: ctx.accounts.usdg_mint.to_account_info(),
            to: ctx.accounts.supplier_usdg_account.to_account_info(),
            authority: ctx.accounts.lending_pool.to_account_info(),
        };
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            ),
            amount,
            ctx.accounts.usdg_mint.decimals,
        )?;

        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.supplier_position.principal_usdg = new_position_principal;
        ctx.accounts.supplier_position.last_updated_ts = now;
        ctx.accounts.lending_pool.total_supplied_usdg = new_total_supplied;

        emit!(UsdgSupplyWithdrawn {
            supplier: ctx.accounts.supplier.key(),
            lending_pool: ctx.accounts.lending_pool.key(),
            amount,
            supplier_principal_usdg: new_position_principal,
            total_supplied_usdg: new_total_supplied,
        });

        Ok(())
    }

    pub fn borrow_usdg(ctx: Context<BorrowUsdg>, amount: u64) -> Result<()> {
        require!(!ctx.accounts.protocol_config.paused, MiladyError::ProtocolPaused);
        require!(amount > 0, MiladyError::InvalidAmount);
        require!(ctx.accounts.usdg_mint.decimals == 6, MiladyError::InvalidUsdgDecimals);
        require!(ctx.accounts.lending_pool.borrow_enabled, MiladyError::BorrowingDisabled);

        let available = pool_available_liquidity(&ctx.accounts.lending_pool)?;
        require!(available >= amount, MiladyError::InsufficientLiquidity);
        require!(
            ctx.accounts.liquidity_vault.amount >= amount,
            MiladyError::InsufficientLiquidity
        );

        let new_total_borrowed = ctx
            .accounts
            .lending_pool
            .total_borrowed_usdg
            .checked_add(amount)
            .ok_or(MiladyError::MathOverflow)?;
        if ctx.accounts.lending_pool.borrow_cap_usdg > 0 {
            require!(
                new_total_borrowed <= ctx.accounts.lending_pool.borrow_cap_usdg,
                MiladyError::BorrowCapExceeded
            );
        }

        let snapshot = compute_portfolio_risk(
            &ctx.accounts.credit_account,
            ctx.remaining_accounts,
        )?;
        let new_debt = ctx
            .accounts
            .credit_account
            .debt_usdg
            .checked_add(amount)
            .ok_or(MiladyError::MathOverflow)?;
        require!(
            new_debt <= snapshot.borrow_limit_usd_micro,
            MiladyError::BorrowLimitExceeded
        );

        let new_health = health_factor_bps_for_debt(
            snapshot.liquidation_capacity_usd_micro,
            new_debt,
        )?;
        require!(new_health > BPS_DENOMINATOR, MiladyError::BorrowWouldBeLiquidatable);

        let mint_key = ctx.accounts.usdg_mint.key();
        let bump_seed = [ctx.accounts.lending_pool.bump];
        let signer_seeds: &[&[u8]] = &[LENDING_POOL_SEED, mint_key.as_ref(), &bump_seed];
        let signer = &[signer_seeds];

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.liquidity_vault.to_account_info(),
            mint: ctx.accounts.usdg_mint.to_account_info(),
            to: ctx.accounts.borrower_usdg_account.to_account_info(),
            authority: ctx.accounts.lending_pool.to_account_info(),
        };
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            ),
            amount,
            ctx.accounts.usdg_mint.decimals,
        )?;

        let now = Clock::get()?.unix_timestamp;
        let account = &mut ctx.accounts.credit_account;
        account.debt_usdg = new_debt;
        account.borrow_index_snapshot_e18 = ctx.accounts.lending_pool.borrow_index_e18;
        account.last_borrow_ts = now;
        account.last_collateral_value_usd_micro = snapshot.collateral_value_usd_micro;
        account.last_borrow_limit_usd_micro = snapshot.borrow_limit_usd_micro;
        account.last_liquidation_capacity_usd_micro = snapshot.liquidation_capacity_usd_micro;
        account.last_health_factor_bps = new_health;
        account.last_valuation_ts = now;
        ctx.accounts.lending_pool.total_borrowed_usdg = new_total_borrowed;

        emit!(UsdgBorrowed {
            borrower: ctx.accounts.borrower.key(),
            lending_pool: ctx.accounts.lending_pool.key(),
            amount,
            new_debt_usdg: new_debt,
            total_borrowed_usdg: new_total_borrowed,
            borrow_limit_usd_micro: snapshot.borrow_limit_usd_micro,
            health_factor_bps: new_health,
        });

        Ok(())
    }
