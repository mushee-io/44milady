use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, CloseAccount, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked};

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
pub const MAX_BORROW_APR_BPS: u32 = 100_000;
pub const SECONDS_PER_YEAR: u64 = 31_536_000;
pub const DEFAULT_LIQUIDATION_CLOSE_FACTOR_BPS: u16 = 5_000;

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
mod handlers_liquidation {
    use super::*;
    include!("parts/handlers_liquidation.rs");
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
    pub fn propose_authority(ctx: Context<ProposeAuthority>, new_authority: Pubkey) -> Result<()> {
        handlers_admin::propose_authority(ctx, new_authority)
    }
    pub fn accept_authority(ctx: Context<AcceptAuthority>) -> Result<()> {
        handlers_admin::accept_authority(ctx)
    }
    pub fn cancel_authority_transfer(ctx: Context<ProposeAuthority>) -> Result<()> {
        handlers_admin::cancel_authority_transfer(ctx)
    }
    pub fn propose_emergency_authority(
        ctx: Context<ProposeEmergencyAuthority>,
        new_emergency_authority: Pubkey,
    ) -> Result<()> {
        handlers_admin::propose_emergency_authority(ctx, new_emergency_authority)
    }
    pub fn accept_emergency_authority(ctx: Context<AcceptEmergencyAuthority>) -> Result<()> {
        handlers_admin::accept_emergency_authority(ctx)
    }
    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        handlers_admin::set_treasury(ctx, treasury)
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

    // Milestone 7 — utilization-based interest and index synchronization.
    pub fn accrue_interest(ctx: Context<AccrueInterest>) -> Result<()> {
        handlers_lending::accrue_interest(ctx)
    }
    pub fn sync_borrower_interest(ctx: Context<SyncBorrowerInterest>) -> Result<()> {
        handlers_lending::sync_borrower_interest(ctx)
    }
    pub fn sync_supplier_interest(ctx: Context<SyncSupplierInterest>) -> Result<()> {
        handlers_lending::sync_supplier_interest(ctx)
    }

    // Milestone 8 — repayment and complete position lifecycle.
    pub fn repay_usdg(ctx: Context<RepayUsdg>, amount: u64) -> Result<()> {
        handlers_lending::repay_usdg(ctx, amount)
    }
    pub fn repay_usdg_max(ctx: Context<RepayUsdg>) -> Result<()> {
        handlers_lending::repay_usdg_max(ctx)
    }
    pub fn repay_usdg_on_behalf(ctx: Context<RepayUsdgOnBehalf>, amount: u64) -> Result<()> {
        handlers_lending::repay_usdg_on_behalf(ctx, amount)
    }
    pub fn repay_usdg_on_behalf_max(ctx: Context<RepayUsdgOnBehalf>) -> Result<()> {
        handlers_lending::repay_usdg_on_behalf_max(ctx)
    }
    pub fn close_credit_account(ctx: Context<CloseCreditAccount>) -> Result<()> {
        handlers_lending::close_credit_account(ctx)
    }
    pub fn close_supplier_position(ctx: Context<CloseSupplierPosition>) -> Result<()> {
        handlers_lending::close_supplier_position(ctx)
    }
    pub fn close_collateral_vault(ctx: Context<CloseCollateralVault>) -> Result<()> {
        handlers_collateral::close_collateral_vault(ctx)
    }

    // Milestone 9 — liquidation, insurance reserve and bad-debt handling.
    pub fn update_liquidation_config(
        ctx: Context<UpdateLiquidationConfig>,
        close_factor_bps: u16,
        enabled: bool,
    ) -> Result<()> {
        handlers_liquidation::update_liquidation_config(ctx, close_factor_bps, enabled)
    }
    pub fn liquidate(ctx: Context<Liquidate>, max_repay_usdg: u64) -> Result<()> {
        handlers_liquidation::liquidate(ctx, max_repay_usdg)
    }
    pub fn fund_insurance_reserve(
        ctx: Context<FundInsuranceReserve>,
        amount: u64,
    ) -> Result<()> {
        handlers_liquidation::fund_insurance_reserve(ctx, amount)
    }
    pub fn absorb_bad_debt(ctx: Context<AbsorbBadDebt>) -> Result<()> {
        handlers_liquidation::absorb_bad_debt(ctx)
    }
    pub fn recapitalize_bad_debt(
        ctx: Context<RecapitalizeBadDebt>,
        amount: u64,
    ) -> Result<()> {
        handlers_liquidation::recapitalize_bad_debt(ctx, amount)
    }
}

include!("parts/accounts_admin.rs");
include!("parts/accounts_collateral.rs");
include!("parts/accounts_lending.rs");
include!("parts/accounts_liquidation.rs");
include!("parts/state.rs");
include!("parts/interest.rs");
include!("parts/risk.rs");
include!("parts/events.rs");
include!("parts/errors.rs");
include!("parts/tests.rs");
