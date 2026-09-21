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

### ✅ Source implementation: Milestone 9 — liquidation + solvency
- fresh-Pyth whole-account liquidation checks
- configurable liquidation close factor
- independent liquidation kill switch
- market-specific liquidation bonus
- partial collateral seizure
- liquidator USDG repayment into the pool
- keeper service with dry-run default
- worst-health-first keeper prioritization
- protocol reserve loss waterfall
- dedicated USDG insurance reserve
- uncovered bad-debt accounting
- bad-debt recapitalization with real USDG
- liquidation remains available during general pause unless explicitly disabled

See `docs/M2-M5.md`, `docs/M6.md`, `docs/M7.md`, `docs/M8.md`, `docs/M9.md`, and `docs/M10.md`.

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
- The current `declare_id!` / `Anchor.toml` address remains a placeholder until the stable program keypair is generated; the release workflow blocks deployment while they do not match.
- M9 enforces unhealthy debt with fresh-Pyth liquidations, then absorbs collateral-exhausted losses through protocol reserves, dedicated insurance, and explicit bad-debt accounting.

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

## Liquidation + loss waterfall

```
health <= 1.0
→ liquidator repays USDG
→ receives collateral + market bonus
→ repeat if needed
→ no collateral + residual debt
→ protocol reserves
→ insurance reserve
→ bad debt
→ USDG recapitalization
```

The keeper under `keeper/` is dry-run by default; the Solana program remains the final authority for fresh price, health, close-factor and collateral checks.

### 🚧 Milestone 10 — release hardening + public Devnet

Built in the release branch:

- two-step protocol-authority transfer
- two-step emergency-authority transfer
- multisig-compatible authority model
- explicit treasury rotation
- deterministic adversarial state-machine tests
- release secret/keypair leakage checks
- canonical program-ID preflight
- Devnet program-key generation/sync script
- Devnet deploy script
- mock asset + faucet bootstrap
- market + Pyth configuration bootstrap
- lending-pool initialization
- public Devnet verification script
- live borrow/repay E2E script
- pool/bad-debt monitoring process
- manual GitHub Actions Devnet deployment workflow
- SBF build gate in CI

The only part that cannot be completed from source code alone is the chain transaction itself: a funded Devnet deployer keypair and the stable program keypair must be provided as secrets. No private key is committed to this repository.

See `docs/M10.md` for the release runbook.

## Release commands

```bash
cp config/devnet.example.json config/devnet.local.json
# fill verified Pyth feed IDs + Devnet price feed accounts
export ANCHOR_WALLET=/secure/path/devnet-deployer.json

./scripts/devnet/prepare-program.sh
# commit the program ID produced by anchor keys sync, then securely back up the keypair
./scripts/devnet/deploy.sh
npm run devnet:bootstrap
npm run devnet:verify
npm run devnet:e2e
npm run devnet:monitor
```

## Status

**Milestones 1–10 source/release engineering: built.**

**Public Devnet chain deployment: credential-gated** until the canonical program keypair and funded Devnet deployer are supplied.
