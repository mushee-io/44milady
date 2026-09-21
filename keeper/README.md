# 44 Milady liquidation keeper

The keeper polls 44 Milady credit accounts, ranks cached unhealthy positions, supplies the complete market/oracle account set, and submits the on-chain `liquidate` instruction.

It is intentionally **dry-run by default**.

## Required environment

- `MILADY_IDL_PATH` — generated Anchor IDL after a program build
- `LIQUIDATOR_KEYPAIR` — Devnet liquidator keypair JSON
- `USDG_MINT` — deployed mock USDG mint
- `PYTH_PRICE_ACCOUNT_MAP` — JSON mapping 32-byte feed id hex → fresh Pyth PriceUpdateV2 account pubkey

Optional:

- `SOLANA_RPC_URL` — defaults to Solana Devnet
- `TOKEN_PROGRAM_ID` — defaults to the classic SPL Token program
- `KEEPER_INTERVAL_MS` — defaults to 15000
- `DRY_RUN=false` — enables transaction submission

The liquidator must already have USDG and token accounts for collateral it may receive.

The keeper's cached health ranking is only a discovery optimization. **The Solana program recomputes fresh Pyth health and all liquidation caps before any collateral can move.**
