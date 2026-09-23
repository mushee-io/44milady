#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROGRAM_NAME="forty_four_milady"
KEYPAIR="target/deploy/${PROGRAM_NAME}-keypair.json"
SO="target/deploy/${PROGRAM_NAME}.so"

mkdir -p target/deploy

echo "==> Solana CLI"
solana --version

echo "==> Current Devnet wallet"
solana address
solana balance --url devnet

if [ ! -f "$KEYPAIR" ]; then
  echo "==> Generating canonical 44 Milady program keypair"
  solana-keygen new --no-bip39-passphrase --silent --force -o "$KEYPAIR"
fi

PROGRAM_ID="$(solana-keygen pubkey "$KEYPAIR")"
echo "==> Canonical 44 Milady Program ID: $PROGRAM_ID"

PROGRAM_ID="$PROGRAM_ID" python3 - <<'PY'
from pathlib import Path
import os, re
pid=os.environ["PROGRAM_ID"]

anchor=Path("Anchor.toml")
txt=anchor.read_text()
txt=re.sub(r'(forty_four_milady\s*=\s*")[^"]+(")', rf'\g<1>{pid}\2', txt)
anchor.write_text(txt)

lib=Path("programs/forty_four_milady/src/lib.rs")
txt=lib.read_text()
txt=re.sub(r'declare_id!\("[^"]+"\);', f'declare_id!("{pid}");', txt, count=1)
lib.write_text(txt)
PY

echo "==> Building with current Solana SBF toolchain (not Anchor's legacy Cargo 1.79)"
rm -f Cargo.lock
cargo build-sbf --manifest-path programs/forty_four_milady/Cargo.toml

if [ ! -f "$SO" ]; then
  echo "ERROR: expected program binary at $SO"
  exit 1
fi

echo
echo "BUILD PASS"
echo "Program ID: $PROGRAM_ID"
echo "Program binary: $SO"
ls -lh "$SO"
echo
echo "Next deploy command:"
echo "solana program deploy \"$SO\" --program-id \"$KEYPAIR\" --url devnet"
