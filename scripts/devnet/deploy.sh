#!/usr/bin/env bash
set -euo pipefail

RPC_URL="${SOLANA_RPC_URL:-https://api.devnet.solana.com}"
: "${ANCHOR_WALLET:?Set ANCHOR_WALLET to the funded Devnet deployer keypair JSON}"

solana config set --url "$RPC_URL" >/dev/null
solana config set --keypair "$ANCHOR_WALLET" >/dev/null

cargo build-sbf --tools-version v1.57 --install-only
./scripts/devnet/prepare-program.sh
REQUIRE_NON_PLACEHOLDER_ID=1 npm run release:check
anchor build
anchor deploy

PROGRAM_ID="$(solana-keygen pubkey target/deploy/forty_four_milady-keypair.json)"
solana program show "$PROGRAM_ID" --url "$RPC_URL"

echo "44 Milady deployed to Devnet: $PROGRAM_ID"
