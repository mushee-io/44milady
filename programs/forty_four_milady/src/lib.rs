use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked};

declare_id!("7ahY74GVSGRf9sDXFPtX6EnynoxWz2myNijQd7MPH5vF");

pub const PROTOCOL_SEED: &[u8] = b"protocol";
pub const MARKET_SEED: &[u8] = b"market";
pub const FAUCET_SEED: &[u8] = b"faucet";
pub const CLAIM_SEED: &[u8] = b"claim";
pub const CREDIT_SEED: &[u8] = b"credit";
pub const VAULT_SEED: &[u8] = b"vault";
pub const LENDING_POOL_SEED: &[u8] = b"lending_pool";
pub const LIQUIDITY_VAULT_SEED: &[u8] = b"liquidity_vault";
pub const SUPPLIER_SEED: &[u8] = b"supplier";
pub const BPS_DENOMINATOR: u64 = 10_000;
pub const USD_MICRO: u64 = 1_000_000;
pub const INDEX_SCALE_E18: u128 = 1_000_000_000_000_000_000;
pub const MAX_COLLATERAL_ASSETS: usize = 8;
pub const MAX_SYMBOL_BYTES: usize = 8;
pub const MAX_LIQUIDATION_BONUS_BPS: u16 = 2_500;
pub const MAX_CONFIDENCE_BPS: u16 = 2_000;
pub const MAX_ORACLE_AGE_SECS: u32 = 3_600;
pub const MAX_RESERVE_FACTOR_BPS: u16 = 5_000;

mod handlers_admin {
    use super::*;
    include!("parts/handlers_admin.rs");
}
mod handlers_collateral {
    use super::*;
    include!("parts/handlers_collateral.rs");
}
mod handlers_lending {
    use super::*;
    include!("parts/handlers_lending.rs");
}

#[program]
pub mod forty_four_milady {
    use super::*;

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        handlers_admin::initialize_protocol(ctx)
    }
    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        handlers_admin::set_pause(ctx, paused)
    }
    pub fn register_market(ctx: Context<RegisterMarket>, args: RegisterMarketArgs) -> Result<()> {
        handlers_admin::register_market(ctx, args)
    }
    pub fn update_market(ctx: Context<UpdateMarket>, args: UpdateMarketArgs) -> Result<()> {
        handlers_admin::update_market(ctx, args)
    }
    pub fn register_faucet_asset(ctx: Context<RegisterFaucetAsset>, amount_per_claim: u64, cooldown_seconds: u32) -> Result<()> {
        handlers_admin::register_faucet_asset(ctx, amount_per_claim, cooldown_seconds)
    }
    pub fn set_faucet_status(ctx: Context<SetFaucetStatus>, enabled: bool) -> Result<()> {
        handlers_admin::set_faucet_status(ctx, enabled)
    }
    pub fn claim_faucet(ctx: Context<ClaimFaucet>) -> Result<()> {
        handlers_admin::claim_faucet(ctx)
    }
    pub fn initialize_credit_account(ctx: Context<InitializeCreditAccount>) -> Result<()> {
        handlers_collateral::initialize_credit_account(ctx)
    }
    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        handlers_collateral::deposit_collateral(ctx, amount)
    }
    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
        handlers_collateral::withdraw_collateral(ctx, amount)
    }
    pub fn refresh_health(ctx: Context<RefreshHealth>) -> Result<()> {
        handlers_collateral::refresh_health(ctx)
    }

    // Milestone 6 — USDG liquidity + collateral-backed borrowing.
    pub fn initialize_lending_pool(
        ctx: Context<InitializeLendingPool>,
        args: InitializeLendingPoolArgs,
    ) -> Result<()> {
        handlers_lending::initialize_lending_pool(ctx, args)
    }
    pub fn update_lending_pool(
        ctx: Context<UpdateLendingPool>,
        args: UpdateLendingPoolArgs,
    ) -> Result<()> {
        handlers_lending::update_lending_pool(ctx, args)
    }
    pub fn supply_usdg(ctx: Context<SupplyUsdg>, amount: u64) -> Result<()> {
        handlers_lending::supply_usdg(ctx, amount)
    }
    pub fn withdraw_supplied_usdg(ctx: Context<WithdrawSuppliedUsdg>, amount: u64) -> Result<()> {
        handlers_lending::withdraw_supplied_usdg(ctx, amount)
    }
    pub fn borrow_usdg(ctx: Context<BorrowUsdg>, amount: u64) -> Result<()> {
        handlers_lending::borrow_usdg(ctx, amount)
    }
}

include!("parts/accounts_admin.rs");
include!("parts/accounts_collateral.rs");
include!("parts/accounts_lending.rs");
include!("parts/state.rs");
include!("parts/risk.rs");
include!("parts/events.rs");
include!("parts/errors.rs");
include!("parts/tests.rs");
