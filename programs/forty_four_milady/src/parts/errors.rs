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
    #[msg("Token mint does not match the configured market or pool.")]
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
    #[msg("Withdrawal would leave the account above its safe borrowing limit.")]
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
    #[msg("USDG must use exactly 6 decimals.")]
    InvalidUsdgDecimals,
    #[msg("Reserve factor is outside the supported range.")]
    InvalidReserveFactor,
    #[msg("Interest-rate kink must be between 0% and 100% utilization.")]
    InvalidInterestKink,
    #[msg("Configured maximum borrow APR exceeds the protocol safety ceiling.")]
    InterestRateTooHigh,
    #[msg("Clock moved backwards relative to the pool's last accrual timestamp.")]
    ClockWentBackwards,
    #[msg("Interest index is zero, decreasing, or otherwise invalid.")]
    InvalidInterestIndex,
    #[msg("Borrowing is disabled for this lending pool.")]
    BorrowingDisabled,
    #[msg("Borrow would exceed the pool borrow cap.")]
    BorrowCapExceeded,
    #[msg("Borrow cap cannot be set below outstanding debt.")]
    BorrowCapBelowOutstandingDebt,
    #[msg("The pool does not have enough available USDG.")]
    InsufficientLiquidity,
    #[msg("The requested borrow would exceed this account's collateral borrowing limit.")]
    BorrowLimitExceeded,
    #[msg("The requested borrow would make this account immediately liquidatable.")]
    BorrowWouldBeLiquidatable,
    #[msg("Supplier position does not have enough supplied principal.")]
    InsufficientSupply,
    #[msg("Liquidity vault does not match the lending pool.")]
    InvalidLiquidityVault,
    #[msg("Supplier position does not belong to this lending pool.")]
    InvalidLendingPool,
    #[msg("Lending pool supplied/borrowed accounting invariant is broken.")]
    PoolAccountingInvariant,
    #[msg("This credit account has no debt to repay.")]
    NoDebtToRepay,
    #[msg("Repayment amount exceeds the synchronized debt balance.")]
    RepayAmountExceedsDebt,
    #[msg("The repayment wallet does not hold enough USDG.")]
    InsufficientRepaymentFunds,
    #[msg("Credit account still has outstanding debt.")]
    CreditAccountHasDebt,
    #[msg("Credit account still has deposited collateral.")]
    CreditAccountHasCollateral,
    #[msg("Supplier position still has a non-zero USDG claim.")]
    SupplierPositionNotEmpty,
    #[msg("Collateral vault still contains tokens.")]
    CollateralVaultNotEmpty,
    #[msg("Collateral market is still recorded on the credit account.")]
    CollateralStillRecorded,
    #[msg("Liquidation close factor must be between 1 and 10000 basis points.")]
    InvalidCloseFactor,
    #[msg("Liquidations are disabled for this lending pool.")]
    LiquidationDisabled,
    #[msg("The borrower is not below the liquidation health threshold.")]
    PositionNotLiquidatable,
    #[msg("The liquidation amount is too small after close-factor/collateral caps.")]
    LiquidationTooSmall,
    #[msg("Collateral vault balance is below the protocol's recorded collateral.")]
    CollateralVaultAccountingMismatch,
    #[msg("Collateral remains and must be liquidated before bad debt can be written off.")]
    CollateralRemainsForLiquidation,
    #[msg("Bad-debt recapitalization exceeds the recorded deficit.")]
    BadDebtCoverageExceedsDeficit,
    #[msg("Pool claims are below recorded bad debt; the pool is insolvent.")]
    PoolInsolvent,
    #[msg("Admin address must be non-zero and different where required.")]
    InvalidAdminAddress,
    #[msg("There is no pending protocol-authority transfer.")]
    NoPendingAuthorityTransfer,
    #[msg("There is no pending emergency-authority transfer.")]
    NoPendingEmergencyAuthorityTransfer,
    #[msg("Arithmetic overflow or underflow.")]
    MathOverflow,
}
