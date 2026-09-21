#[error_code]
pub enum MiladyError {
    #[msg("The signer is not authorized for this action.")]
    Unauthorized,
    #[msg("The protocol is paused.")]
    ProtocolPaused,
    #[msg("The referenced protocol account is invalid.")]
    InvalidProtocol,
    #[msg("Amount must be greater than zero.")]
    InvalidAmount,
    #[msg("Token mint does not match the configured market.")]
    InvalidMint,
    #[msg("Market symbol must not be empty.")]
    InvalidSymbol,
    #[msg("Oracle feed id must not be empty.")]
    InvalidFeedId,
    #[msg("LTV must be between 0 and 10000 basis points.")]
    InvalidLtv,
    #[msg("Liquidation threshold must be above LTV and no more than 10000 basis points.")]
    InvalidLiquidationThreshold,
    #[msg("Liquidation bonus is outside the supported range.")]
    InvalidLiquidationBonus,
    #[msg("Oracle confidence limit is outside the supported range.")]
    InvalidConfidenceLimit,
    #[msg("Oracle maximum age is outside the supported range.")]
    InvalidOracleAge,
    #[msg("Market is disabled.")]
    MarketDisabled,
    #[msg("Collateral market supply cap exceeded.")]
    SupplyCapExceeded,
    #[msg("Faucet is disabled for this asset.")]
    FaucetDisabled,
    #[msg("Faucet claim is still in cooldown.")]
    FaucetCooldown,
    #[msg("Faucet cooldown exceeds the Devnet safety limit.")]
    InvalidCooldown,
    #[msg("Maximum collateral asset count reached.")]
    TooManyCollateralAssets,
    #[msg("Collateral entry was not found.")]
    CollateralNotFound,
    #[msg("Insufficient deposited collateral.")]
    InsufficientCollateral,
    #[msg("Withdrawal would leave the account unsafe.")]
    UnsafeWithdrawal,
    #[msg("Risk refresh accounts are missing or incorrectly ordered.")]
    InvalidRiskAccounts,
    #[msg("Market account does not match the collateral entry.")]
    MarketAccountMismatch,
    #[msg("Market config account is not owned by 44 Milady.")]
    InvalidMarketOwner,
    #[msg("Oracle account is not owned by the Pyth Receiver program.")]
    InvalidOracleOwner,
    #[msg("Pyth oracle update could not be read, is stale, or is for the wrong feed.")]
    OracleReadFailed,
    #[msg("Oracle returned a non-positive price.")]
    InvalidOraclePrice,
    #[msg("Oracle confidence interval is wider than this market permits.")]
    OracleConfidenceTooWide,
    #[msg("Arithmetic overflow or underflow.")]
    MathOverflow,
}
