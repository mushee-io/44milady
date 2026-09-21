    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        let config = &mut ctx.accounts.protocol_config;
        config.authority = ctx.accounts.authority.key();
        config.treasury = ctx.accounts.treasury.key();
        config.emergency_authority = ctx.accounts.emergency_authority.key();
        config.version = 4;
        config.paused = false;
        config.bump = ctx.bumps.protocol_config;

        emit!(ProtocolInitialized {
            authority: config.authority,
            treasury: config.treasury,
            emergency_authority: config.emergency_authority,
            version: config.version,
        });

        Ok(())
    }

    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.emergency_authority.key(),
            ctx.accounts.protocol_config.emergency_authority,
            MiladyError::Unauthorized
        );
        ctx.accounts.protocol_config.paused = paused;
        emit!(ProtocolPauseChanged { paused });
        Ok(())
    }

    // ----- Milestone 2: market registry + faucet -----

    pub fn register_market(ctx: Context<RegisterMarket>, args: RegisterMarketArgs) -> Result<()> {
        assert_protocol_authority(&ctx.accounts.protocol_config, &ctx.accounts.authority)?;
        validate_market_args(&args)?;
        require!(args.symbol.iter().any(|b| *b != 0), MiladyError::InvalidSymbol);
        require!(args.feed_id.iter().any(|b| *b != 0), MiladyError::InvalidFeedId);

        let market = &mut ctx.accounts.market_config;
        market.protocol = ctx.accounts.protocol_config.key();
        market.mint = ctx.accounts.collateral_mint.key();
        market.symbol = args.symbol;
        market.feed_id = args.feed_id;
        market.decimals = ctx.accounts.collateral_mint.decimals;
        market.ltv_bps = args.ltv_bps;
        market.liquidation_threshold_bps = args.liquidation_threshold_bps;
        market.liquidation_bonus_bps = args.liquidation_bonus_bps;
        market.max_confidence_bps = args.max_confidence_bps;
        market.max_price_age_secs = args.max_price_age_secs;
        market.supply_cap = args.supply_cap;
        market.debt_ceiling_usdg = args.debt_ceiling_usdg;
        market.total_deposited = 0;
        market.enabled = true;
        market.bump = ctx.bumps.market_config;

        emit!(MarketRegistered {
            market: market.key(),
            mint: market.mint,
            symbol: market.symbol,
            ltv_bps: market.ltv_bps,
            liquidation_threshold_bps: market.liquidation_threshold_bps,
        });

        Ok(())
    }

    pub fn update_market(ctx: Context<UpdateMarket>, args: UpdateMarketArgs) -> Result<()> {
        assert_protocol_authority(&ctx.accounts.protocol_config, &ctx.accounts.authority)?;
        validate_update_market_args(&args)?;

        let market = &mut ctx.accounts.market_config;
        market.feed_id = args.feed_id;
        market.ltv_bps = args.ltv_bps;
        market.liquidation_threshold_bps = args.liquidation_threshold_bps;
        market.liquidation_bonus_bps = args.liquidation_bonus_bps;
        market.max_confidence_bps = args.max_confidence_bps;
        market.max_price_age_secs = args.max_price_age_secs;
        market.supply_cap = args.supply_cap;
        market.debt_ceiling_usdg = args.debt_ceiling_usdg;
        market.enabled = args.enabled;

        emit!(MarketUpdated {
            market: market.key(),
            enabled: market.enabled,
            ltv_bps: market.ltv_bps,
            liquidation_threshold_bps: market.liquidation_threshold_bps,
        });

        Ok(())
    }

    pub fn register_faucet_asset(
        ctx: Context<RegisterFaucetAsset>,
        amount_per_claim: u64,
        cooldown_seconds: u32,
    ) -> Result<()> {
        assert_protocol_authority(&ctx.accounts.protocol_config, &ctx.accounts.authority)?;
        require!(amount_per_claim > 0, MiladyError::InvalidAmount);
        require!(cooldown_seconds <= 86_400, MiladyError::InvalidCooldown);

        let faucet = &mut ctx.accounts.faucet_config;
        faucet.protocol = ctx.accounts.protocol_config.key();
        faucet.mint = ctx.accounts.mint.key();
        faucet.amount_per_claim = amount_per_claim;
        faucet.cooldown_seconds = cooldown_seconds;
        faucet.enabled = true;
        faucet.bump = ctx.bumps.faucet_config;

        emit!(FaucetAssetRegistered {
            mint: faucet.mint,
            amount_per_claim,
            cooldown_seconds,
        });

        Ok(())
    }

    pub fn set_faucet_status(ctx: Context<SetFaucetStatus>, enabled: bool) -> Result<()> {
        assert_protocol_authority(&ctx.accounts.protocol_config, &ctx.accounts.authority)?;
        ctx.accounts.faucet_config.enabled = enabled;
        Ok(())
    }

    pub fn claim_faucet(ctx: Context<ClaimFaucet>) -> Result<()> {
        let protocol = &ctx.accounts.protocol_config;
        require!(!protocol.paused, MiladyError::ProtocolPaused);
        require!(ctx.accounts.faucet_config.enabled, MiladyError::FaucetDisabled);

        let now = Clock::get()?.unix_timestamp;
        let claim = &mut ctx.accounts.faucet_claim;

        if claim.claimant == Pubkey::default() {
            claim.claimant = ctx.accounts.claimant.key();
            claim.mint = ctx.accounts.mint.key();
            claim.last_claim_ts = 0;
            claim.total_claimed = 0;
            claim.bump = ctx.bumps.faucet_claim;
        }

        require_keys_eq!(claim.claimant, ctx.accounts.claimant.key(), MiladyError::Unauthorized);
        require_keys_eq!(claim.mint, ctx.accounts.mint.key(), MiladyError::InvalidMint);

        if claim.last_claim_ts > 0 {
            let elapsed = now
                .checked_sub(claim.last_claim_ts)
                .ok_or(MiladyError::MathOverflow)?;
            require!(
                elapsed >= i64::from(ctx.accounts.faucet_config.cooldown_seconds),
                MiladyError::FaucetCooldown
            );
        }

        let amount = ctx.accounts.faucet_config.amount_per_claim;
        let bump_seed = [protocol.bump];
        let signer_seeds: &[&[u8]] = &[PROTOCOL_SEED, &bump_seed];
        let signer = &[signer_seeds];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.protocol_config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token_interface::mint_to(cpi_ctx, amount)?;

        claim.last_claim_ts = now;
        claim.total_claimed = claim
            .total_claimed
            .checked_add(amount)
            .ok_or(MiladyError::MathOverflow)?;

        emit!(FaucetClaimed {
            claimant: claim.claimant,
            mint: claim.mint,
            amount,
            total_claimed: claim.total_claimed,
        });

        Ok(())
    }
