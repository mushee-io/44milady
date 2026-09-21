#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Stored as a destination address only. No account data is read.
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: Stored as an emergency signer address only. No account data is read.
    pub emergency_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + ProtocolConfig::INIT_SPACE,
        seeds = [PROTOCOL_SEED],
        bump
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetPause<'info> {
    pub emergency_authority: Signer<'info>,
    #[account(mut, seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
}

#[derive(Accounts)]
pub struct RegisterMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    pub collateral_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = authority,
        space = 8 + MarketConfig::INIT_SPACE,
        seeds = [MARKET_SEED, collateral_mint.key().as_ref()],
        bump
    )]
    pub market_config: Account<'info, MarketConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateMarket<'info> {
    pub authority: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        constraint = market_config.protocol == protocol_config.key() @ MiladyError::InvalidProtocol
    )]
    pub market_config: Account<'info, MarketConfig>,
}

#[derive(Accounts)]
pub struct RegisterFaucetAsset<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = authority,
        space = 8 + FaucetConfig::INIT_SPACE,
        seeds = [FAUCET_SEED, mint.key().as_ref()],
        bump
    )]
    pub faucet_config: Account<'info, FaucetConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetFaucetStatus<'info> {
    pub authority: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        constraint = faucet_config.protocol == protocol_config.key() @ MiladyError::InvalidProtocol
    )]
    pub faucet_config: Account<'info, FaucetConfig>,
}

#[derive(Accounts)]
pub struct ClaimFaucet<'info> {
    #[account(mut)]
    pub claimant: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        seeds = [FAUCET_SEED, mint.key().as_ref()],
        bump = faucet_config.bump,
        constraint = faucet_config.protocol == protocol_config.key() @ MiladyError::InvalidProtocol,
        constraint = faucet_config.mint == mint.key() @ MiladyError::InvalidMint
    )]
    pub faucet_config: Account<'info, FaucetConfig>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = claimant,
        token::token_program = token_program
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = claimant,
        space = 8 + FaucetClaim::INIT_SPACE,
        seeds = [CLAIM_SEED, claimant.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub faucet_claim: Account<'info, FaucetClaim>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

