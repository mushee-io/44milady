#!/usr/bin/env bash
set -euo pipefail

KEYPAIR="target/deploy/forty_four_milady-keypair.json"
mkdir -p target/deploy

if [[ ! -f "$KEYPAIR" ]]; then
  solana-keygen new --no-bip39-passphrase --force --outfile "$KEYPAIR"
fi

anchor keys sync
anchor build

PROGRAM_ID="$(solana-keygen pubkey "$KEYPAIR")"
echo "44 Milady canonical program id: $PROGRAM_ID"
echo "Program keypair stays under target/ (gitignored). Back it up securely before deployment."
