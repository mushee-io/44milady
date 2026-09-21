#[derive(Accounts)]
pub struct UpdateLiquidationConfig<'info> {
    pub authority: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [LENDING_POOL_SEED, lending_pool.usdg_mint.as_ref()],
        bump = lending_pool.bump,
        constraint = lending_pool.protocol == protocol_config.key() @ MiladyError::InvalidProtocol
    )]
    pub lending_pool: Account<'info, LendingPool>,
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,
    /// CHECK: Used only as the borrower identity for PDA derivation.
    pub borrower: UncheckedAccount<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [CREDIT_SEED, borrower.key().as_ref()],
        bump = credit_account.bump,
        constraint = credit_account.owner == borrower.key() @ MiladyError::Unauthorized
    )]
    pub credit_account: Account<'info, CreditAccount>,
    #[account(
        mut,
        seeds = [LENDING_POOL_SEED, usdg_mint.key().as_ref()],
        bump = lending_pool.bump,
        constraint = lending_pool.protocol == protocol_config.key() @ MiladyError::InvalidProtocol,
        constraint = lending_pool.usdg_mint == usdg_mint.key() @ MiladyError::InvalidMint,
        constraint = lending_pool.liquidity_vault == liquidity_vault.key() @ MiladyError::InvalidLiquidityVault
    )]
    pub lending_pool: Account<'info, LendingPool>,
    pub usdg_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = liquidator,
        token::token_program = token_program
    )]
    pub liquidator_usdg_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = lending_pool,
        token::token_program = token_program
    )]
    pub liquidity_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [MARKET_SEED, collateral_mint.key().as_ref()],
        bump = market_config.bump,
        constraint = market_config.protocol == protocol_config.key() @ MiladyError::InvalidProtocol,
        constraint = market_config.mint == collateral_mint.key() @ MiladyError::InvalidMint
    )]
    pub market_config: Account<'info, MarketConfig>,
    pub collateral_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [VAULT_SEED, credit_account.key().as_ref(), market_config.key().as_ref()],
        bump,
        token::mint = collateral_mint,
        token::authority = credit_account,
        token::token_program = token_program
    )]
    pub collateral_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = collateral_mint,
        token::authority = liquidator,
        token::token_program = token_program
    )]
    pub liquidator_collateral_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Parsed as a fully verified Pyth PriceUpdateV2 account in the handler.
    pub price_update: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct FundInsuranceReserve<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [LENDING_POOL_SEED, usdg_mint.key().as_ref()],
        bump = lending_pool.bump,
        constraint = lending_pool.protocol == protocol_config.key() @ MiladyError::InvalidProtocol,
        constraint = lending_pool.usdg_mint == usdg_mint.key() @ MiladyError::InvalidMint,
        constraint = lending_pool.liquidity_vault == liquidity_vault.key() @ MiladyError::InvalidLiquidityVault
    )]
    pub lending_pool: Account<'info, LendingPool>,
    pub usdg_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = funder,
        token::token_program = token_program
    )]
    pub funder_usdg_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = lending_pool,
        token::token_program = token_program
    )]
    pub liquidity_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct AbsorbBadDebt<'info> {
    pub caller: Signer<'info>,
    /// CHECK: Used only as the borrower identity for PDA derivation.
    pub borrower: UncheckedAccount<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [CREDIT_SEED, borrower.key().as_ref()],
        bump = credit_account.bump,
        constraint = credit_account.owner == borrower.key() @ MiladyError::Unauthorized
    )]
    pub credit_account: Account<'info, CreditAccount>,
    #[account(
        mut,
        seeds = [LENDING_POOL_SEED, lending_pool.usdg_mint.as_ref()],
        bump = lending_pool.bump,
        constraint = lending_pool.protocol == protocol_config.key() @ MiladyError::InvalidProtocol
    )]
    pub lending_pool: Account<'info, LendingPool>,
}

#[derive(Accounts)]
pub struct RecapitalizeBadDebt<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [LENDING_POOL_SEED, usdg_mint.key().as_ref()],
        bump = lending_pool.bump,
        constraint = lending_pool.protocol == protocol_config.key() @ MiladyError::InvalidProtocol,
        constraint = lending_pool.usdg_mint == usdg_mint.key() @ MiladyError::InvalidMint,
        constraint = lending_pool.liquidity_vault == liquidity_vault.key() @ MiladyError::InvalidLiquidityVault
    )]
    pub lending_pool: Account<'info, LendingPool>,
    pub usdg_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = funder,
        token::token_program = token_program
    )]
    pub funder_usdg_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = lending_pool,
        token::token_program = token_program
    )]
    pub liquidity_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}
