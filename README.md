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
- withdrawal health guard wired for future debt
- persisted risk snapshots + events

See docs/M2-M5.md for architecture and account ordering.

## Security assumptions at M2–M5

- Devnet mock assets have **no monetary value**.
- Mock mint authority must be the 44 Milady protocol PDA for faucet minting.
- Pyth data is never trusted from the frontend; the program validates account owner, feed id, age, price and confidence.
- Borrowing is deliberately **not enabled yet**. Milestone 6 adds the USDG liquidity pool and debt mutations.
- The current declare_id / Anchor.toml program address is a placeholder and must be replaced by the generated program keypair before deployment.

## Local model checks

```bash
npm run test:structure
npm run test:model
```

Full Anchor compilation requires Rust, Solana CLI and Anchor CLI.

## Next

6. USDG pool + borrowing
7. interest indexes / APY / APR
8. repayment + complete withdrawal lifecycle
9. liquidations + keeper
10. hardening + public Devnet release
