#!/usr/bin/env bash
set -euo pipefail

RPC_URL="${SOLANA_RPC_URL:-https://api.devnet.solana.com}"
: "${ANCHOR_WALLET:?Set ANCHOR_WALLET to the funded Devnet deployer keypair JSON}"

solana config set --url "$RPC_URL" >/dev/null
solana config set --keypair "$ANCHOR_WALLET" >/dev/null

if ! command -v cargo-build-sbf >/dev/null 2>&1 || ! cargo-build-sbf --version 2>/dev/null | grep -q "4.1.0"; then
  cargo install cargo-build-sbf --version 4.1.0 --locked
fi
PATH="$HOME/.cargo/bin:$PATH" cargo-build-sbf --tools-version v1.57 --manifest-path programs/forty_four_milady/Cargo.toml
./scripts/devnet/prepare-program.sh
REQUIRE_NON_PLACEHOLDER_ID=1 npm run release:check
PATH="$HOME/.cargo/bin:$PATH" anchor build
anchor deploy

PROGRAM_ID="$(solana-keygen pubkey target/deploy/forty_four_milady-keypair.json)"
solana program show "$PROGRAM_ID" --url "$RPC_URL"

echo "44 Milady deployed to Devnet: $PROGRAM_ID"
