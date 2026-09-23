#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROGRAM_NAME="forty_four_milady"
KEYPAIR="target/deploy/${PROGRAM_NAME}-keypair.json"
SO="target/deploy/${PROGRAM_NAME}.so"

mkdir -p target/deploy

echo "==> Selecting the installed Solana 3.x toolchain"
CURRENT_SOLANA="$(command -v solana || true)"
echo "Current PATH solana: ${CURRENT_SOLANA:-missing}"
[ -n "$CURRENT_SOLANA" ] && "$CURRENT_SOLANA" --version || true

SOLANA3=""
declare -a CANDIDATES=()
while IFS= read -r p; do
  [ -n "$p" ] && CANDIDATES+=("$p")
done < <(type -a -p solana 2>/dev/null | awk '!seen[$0]++')

for root in   "$HOME/.local/share/solana/install/releases"   "$HOME/.local/share/agave/install/releases"   "$HOME/.local/bin"   "$HOME/.cargo/bin"
do
  [ -e "$root" ] || continue
  while IFS= read -r p; do
    [ -n "$p" ] && CANDIDATES+=("$p")
  done < <(find "$root" -name solana -perm -u+x 2>/dev/null || true)
done

for p in "${CANDIDATES[@]}"; do
  [ -x "$p" ] || continue
  version="$("$p" --version 2>/dev/null || true)"
  if [[ "$version" == solana-cli\ 3.* ]]; then
    bindir="$(dirname "$p")"
    if [ -x "$bindir/cargo-build-sbf" ]; then
      SOLANA3="$p"
      break
    fi
  fi
done

if [ -z "$SOLANA3" ]; then
  echo "ERROR: Solana 3.x with cargo-build-sbf is installed on this machine but is not currently discoverable."
  echo "Run: type -a solana; find ~/.local/share -path '*/bin/solana' -print 2>/dev/null"
  exit 2
fi

SOLANA_BIN_DIR="$(dirname "$SOLANA3")"
CARGO_BUILD_SBF="$SOLANA_BIN_DIR/cargo-build-sbf"
export PATH="$SOLANA_BIN_DIR:$PATH"
hash -r

echo "Using: $SOLANA3"
"$SOLANA3" --version
echo "SBF builder: $CARGO_BUILD_SBF"

echo "==> Current Devnet wallet"
"$SOLANA3" address
"$SOLANA3" balance --url devnet

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
"$CARGO_BUILD_SBF" --manifest-path programs/forty_four_milady/Cargo.toml

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
