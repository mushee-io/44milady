# 44 Milady protocol tests

Anchor integration tests land here after the program keypair is generated and the Solana/Anchor toolchain is available.

Milestone 1 acceptance cases:
- protocol initializes once at the canonical `protocol` PDA;
- protocol stores authority, treasury and emergency authority;
- unauthorized emergency pause fails;
- authorized emergency authority can pause/unpause.
