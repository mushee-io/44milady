# 44 Milady protocol tests

Anchor integration tests land here after the program keypair is generated and the Solana/Anchor toolchain is available.

Milestone 1 acceptance cases:
- protocol initializes once at the canonical `protocol` PDA;
- protocol stores authority, treasury and emergency authority;
- unauthorized emergency pause fails;
- authorized emergency authority can pause/unpause.

Milestones 2–5 acceptance cases:
- only protocol authority can register/update collateral markets;
- faucet claims enforce mint, claimant and cooldown constraints;
- collateral deposits update both vault custody and market accounting;
- withdrawals cannot exceed the user's recorded collateral;
- Pyth updates must match the configured feed and freshness/confidence limits;
- multi-collateral valuation applies each market's own LTV and liquidation threshold;
- risk snapshots calculate collateral value, borrow limit, liquidation capacity and health factor;
- unsafe collateral withdrawal is rejected once debt is enabled.
