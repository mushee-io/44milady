# 44 Milady

44 Milady is a Solana collateralized-credit protocol: deposit market-linked collateral, value it through Pyth, calculate borrowing power and health, then borrow USDG without selling the underlying exposure.

## Build status

### ✅ Milestone 1 — foundation
- Anchor/Solana workspace
- protocol configuration PDA
- authority / treasury / emergency authority
- emergency pause
- Next.js application shell
- shared SDK

### ✅ Source implementation: Milestone 2 — markets + Devnet faucet
- market registry with per-asset risk parameters
- mock-asset faucet registry
- per-wallet/mint faucet cooldown PDA
- protocol-PDA-authorized faucet minting
- mock asset plan: USDG, NVDAx, AAPLx, SPYx, TSLAx

### ✅ Source implementation: Milestone 3 — collateral vault
- one credit account per wallet
- up to 8 collateral assets per credit account
- isolated collateral vault PDA per account/market
- checked SPL/Token-2022 compatible transfers
- deposit / withdraw accounting
- supply caps

### ✅ Source implementation: Milestone 4 — Pyth valuation
- PriceUpdateV2 integration
- feed-id validation
- stale-price rejection
- Pyth receiver owner validation
- confidence-band guard
- fixed-point USD micro-unit valuation

### ✅ Source implementation: Milestone 5 — risk engine
- weighted portfolio LTV
- weighted liquidation capacity
- health factor
- multi-collateral aggregation
- persisted risk snapshots

### ✅ Source implementation: Milestone 6 — USDG liquidity + borrowing
- canonical USDG lending pool PDA
- program-controlled liquidity vault
- lender SupplierPosition accounts
- USDG supply and withdrawal
- idle-liquidity enforcement
- global borrow cap + borrow enable switch
- collateral-backed USDG borrowing
- fresh Pyth/risk recomputation at borrow time
- post-borrow LTV + health enforcement

### ✅ Source implementation: Milestone 7 — interest engine
- utilization-based piecewise borrow-rate curve
- Borrow APR and derived Supply APR
- SDK APY helper for UI display
- 1e18 borrow and supply indexes
- lazy borrower/supplier balance synchronization
- automatic accrual before risk-sensitive actions
- fractional-interest remainder carry
- protocol reserve accrual
- reserve-aware balance-sheet accounting
- interest-model admin controls and safety ceilings

### ✅ Source implementation: Milestone 8 — repayment + lifecycle
- partial USDG repayment
- full / MAX repayment using synchronized debt
- third-party repayment on behalf of a borrower
- third-party MAX repayment
- physical USDG return to the pool vault
- borrower debt + pool debt settlement
- immediate liquidity restoration
- health improvement after repayment
- repayment remains enabled during emergency pause
- empty collateral-vault closure with rent return
- debt-free/collateral-free credit-account closure
- zero-balance supplier-position closure
- overpayment and insufficient-funds protection

See `docs/M2-M5.md`, `docs/M6.md`, `docs/M7.md`, and `docs/M8.md`.

## M6 borrow path

```
Lender USDG
   ↓ supply
44 Milady liquidity vault
   ↓ borrow (only after Pyth + LTV checks)
Borrower USDG wallet

Borrower collateral
   ↓
44 Milady collateral vault
   ↓
Pyth valuation → risk engine → borrow capacity
```

## Security assumptions

- Devnet mock assets have **no monetary value**.
- Mock mint authority must be the 44 Milady protocol PDA for faucet minting.
- Pyth data is never trusted from the frontend.
- USDG is required to use 6 decimals so debt units match the protocol's USD micro-unit risk accounting.
- Pool borrowing starts disabled after initialization and must be explicitly enabled by protocol authority.
- The current `declare_id!` / `Anchor.toml` program address is a placeholder and must be replaced by the generated program keypair before deployment.
- M8 allows partial, MAX and on-behalf repayment; repayment returns physical USDG to the vault and makes supplier/reserve claims liquid as debt is settled.

## Local model checks

```bash
npm run test:structure
npm run test:model
npm run build:web
```

Full Anchor compilation requires Rust, Solana CLI and Anchor CLI.

## Interest model

Default Devnet target configuration is 2% base, 8% slope to an 80% utilization kink, then a 50% jump slope, with a 10% protocol reserve factor. The exact values are configurable by protocol authority within hard safety bounds.

At 80% utilization that model yields about 10% Borrow APR and 7.2% Supply APR before UI APY compounding.

## Healthy lifecycle

```
Supply USDG → Deposit collateral → Borrow → Accrue interest
→ Partial repay / MAX repay → Withdraw collateral
→ Close empty vaults → Close credit account
```

Third parties can also repay another borrower's debt without receiving their collateral.

## Next

9. liquidations + keeper
10. hardening + public Devnet release
