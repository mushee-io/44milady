#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SO="target/deploy/forty_four_milady.so"
KEYPAIR="target/deploy/forty_four_milady-keypair.json"

if [ ! -f "$SO" ] || [ ! -f "$KEYPAIR" ]; then
  echo "ERROR: build artifacts missing. Run: bash scripts/devnet-build.sh"
  exit 1
fi

SOLANA3=""
declare -a CANDIDATES=()
while IFS= read -r p; do [ -n "$p" ] && CANDIDATES+=("$p"); done < <(type -a -p solana 2>/dev/null | awk '!seen[$0]++')
for root in "$HOME/.local/share/solana/install/releases" "$HOME/.local/share/agave/install/releases" "$HOME/.local/bin" "$HOME/.cargo/bin"; do
  [ -e "$root" ] || continue
  while IFS= read -r p; do [ -n "$p" ] && CANDIDATES+=("$p"); done < <(find "$root" -name solana -perm -u+x 2>/dev/null || true)
done
for p in "${CANDIDATES[@]}"; do
  [ -x "$p" ] || continue
  v="$("$p" --version 2>/dev/null || true)"
  if [[ "$v" == solana-cli\ 3.* ]]; then SOLANA3="$p"; break; fi
done
if [ -z "$SOLANA3" ]; then SOLANA3="$(command -v solana)"; fi

PROGRAM_ID="$(solana-keygen pubkey "$KEYPAIR")"
PROGRAM_SIZE="$(stat -c %s "$SO")"
# UpgradeableLoader ProgramData metadata is 45 bytes in addition to the bytecode.
PROGRAMDATA_SIZE=$((PROGRAM_SIZE + 45))

echo "Using: $("$SOLANA3" --version)"
echo "Program ID: $PROGRAM_ID"
echo "Wallet: $("$SOLANA3" address)"
echo "Binary bytes: $PROGRAM_SIZE"
echo "ProgramData bytes: $PROGRAMDATA_SIZE"

BAL_SOL="$("$SOLANA3" balance --url devnet | awk '{print $1}')"
RENT_SOL="$("$SOLANA3" rent "$PROGRAMDATA_SIZE" --url devnet | awk '{print $1}')"
# Reserve a small amount for the program account and deployment transaction fees.
SAFETY_SOL="0.10"
NEEDED_SOL="$(awk -v r="$RENT_SOL" -v s="$SAFETY_SOL" 'BEGIN { printf "%.9f", r+s }')"

echo
echo "SAFE PREFLIGHT"
echo "Current balance : $BAL_SOL SOL"
echo "ProgramData rent: $RENT_SOL SOL"
echo "Fee reserve     : $SAFETY_SOL SOL"
echo "Required target : $NEEDED_SOL SOL"

if ! awk -v b="$BAL_SOL" -v n="$NEEDED_SOL" 'BEGIN { exit !(b >= n) }'; then
  SHORT="$(awk -v b="$BAL_SOL" -v n="$NEEDED_SOL" 'BEGIN { printf "%.9f", n-b }')"
  echo
  echo "STOPPED BEFORE DEPLOYMENT."
  echo "No deployment transaction was sent."
  echo "Short by approximately: $SHORT SOL"
  exit 3
fi

echo
echo "Preflight PASS. Deploying with exact max-len=$PROGRAM_SIZE."
"$SOLANA3" program deploy "$SO"   --program-id "$KEYPAIR"   --max-len "$PROGRAM_SIZE"   --url devnet

echo
echo "VERIFYING..."
"$SOLANA3" program show "$PROGRAM_ID" --url devnet
echo
echo "44 MILADY DEVNET DEPLOYMENT: PASS"
