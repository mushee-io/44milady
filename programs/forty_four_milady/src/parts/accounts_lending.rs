#[derive(Accounts)]
#[instruction(args: InitializeLendingPoolArgs)]
pub struct InitializeLendingPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    pub usdg_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = authority,
        space = 8 + LendingPool::INIT_SPACE,
        seeds = [LENDING_POOL_SEED, usdg_mint.key().as_ref()],
        bump
    )]
    pub lending_pool: Account<'info, LendingPool>,
    #[account(
        init,
        payer = authority,
        seeds = [LIQUIDITY_VAULT_SEED, lending_pool.key().as_ref()],
        bump,
        token::mint = usdg_mint,
        token::authority = lending_pool,
        token::token_program = token_program
    )]
    pub liquidity_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateLendingPool<'info> {
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
pub struct SupplyUsdg<'info> {
    #[account(mut)]
    pub supplier: Signer<'info>,
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
        token::authority = supplier,
        token::token_program = token_program
    )]
    pub supplier_usdg_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = lending_pool,
        token::token_program = token_program
    )]
    pub liquidity_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = supplier,
        space = 8 + SupplierPosition::INIT_SPACE,
        seeds = [SUPPLIER_SEED, lending_pool.key().as_ref(), supplier.key().as_ref()],
        bump
    )]
    pub supplier_position: Account<'info, SupplierPosition>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSuppliedUsdg<'info> {
    #[account(mut)]
    pub supplier: Signer<'info>,
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
        token::authority = supplier,
        token::token_program = token_program
    )]
    pub supplier_usdg_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = lending_pool,
        token::token_program = token_program
    )]
    pub liquidity_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [SUPPLIER_SEED, lending_pool.key().as_ref(), supplier.key().as_ref()],
        bump = supplier_position.bump,
        constraint = supplier_position.owner == supplier.key() @ MiladyError::Unauthorized,
        constraint = supplier_position.lending_pool == lending_pool.key() @ MiladyError::InvalidLendingPool
    )]
    pub supplier_position: Account<'info, SupplierPosition>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct BorrowUsdg<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
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
        token::authority = borrower,
        token::token_program = token_program
    )]
    pub borrower_usdg_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = usdg_mint,
        token::authority = lending_pool,
        token::token_program = token_program
    )]
    pub liquidity_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}
