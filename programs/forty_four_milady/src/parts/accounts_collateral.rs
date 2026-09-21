#[derive(Accounts)]
pub struct InitializeCreditAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        init,
        payer = owner,
        space = 8 + CreditAccount::INIT_SPACE,
        seeds = [CREDIT_SEED, owner.key().as_ref()],
        bump
    )]
    pub credit_account: Account<'info, CreditAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [CREDIT_SEED, owner.key().as_ref()],
        bump = credit_account.bump,
        constraint = credit_account.owner == owner.key() @ MiladyError::Unauthorized
    )]
    pub credit_account: Account<'info, CreditAccount>,
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
        token::mint = collateral_mint,
        token::authority = owner,
        token::token_program = token_program
    )]
    pub owner_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = owner,
        seeds = [VAULT_SEED, credit_account.key().as_ref(), market_config.key().as_ref()],
        bump,
        token::mint = collateral_mint,
        token::authority = credit_account,
        token::token_program = token_program
    )]
    pub collateral_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [CREDIT_SEED, owner.key().as_ref()],
        bump = credit_account.bump,
        constraint = credit_account.owner == owner.key() @ MiladyError::Unauthorized
    )]
    pub credit_account: Account<'info, CreditAccount>,
    #[account(
        mut,
        seeds = [LENDING_POOL_SEED, lending_pool.usdg_mint.as_ref()],
        bump = lending_pool.bump,
        constraint = lending_pool.protocol == protocol_config.key() @ MiladyError::InvalidProtocol
    )]
    pub lending_pool: Account<'info, LendingPool>,
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
        token::mint = collateral_mint,
        token::authority = owner,
        token::token_program = token_program
    )]
    pub owner_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [VAULT_SEED, credit_account.key().as_ref(), market_config.key().as_ref()],
        bump,
        token::mint = collateral_mint,
        token::authority = credit_account,
        token::token_program = token_program
    )]
    pub collateral_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct RefreshHealth<'info> {
    pub caller: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol_config.bump)]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut, seeds = [CREDIT_SEED, credit_account.owner.as_ref()], bump = credit_account.bump)]
    pub credit_account: Account<'info, CreditAccount>,
    #[account(
        mut,
        seeds = [LENDING_POOL_SEED, lending_pool.usdg_mint.as_ref()],
        bump = lending_pool.bump,
        constraint = lending_pool.protocol == protocol_config.key() @ MiladyError::InvalidProtocol
    )]
    pub lending_pool: Account<'info, LendingPool>,
}
