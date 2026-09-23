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

echo "Using $("$SOLANA3" --version)"
PROGRAM_ID="$(solana-keygen pubkey "$KEYPAIR")"
echo "Program ID: $PROGRAM_ID"
echo "Wallet: $("$SOLANA3" address)"
echo "Balance before funding: $("$SOLANA3" balance --url devnet)"

# Large upgradeable programs need several Devnet SOL for rent.
# Try to top up automatically; rate limits are non-fatal and deployment will report any remaining shortage.
for i in 1 2 3; do
  BAL="$("$SOLANA3" balance --url devnet 2>/dev/null | awk '{print $1}')"
  if awk "BEGIN {exit !($BAL >= 6.5)}"; then break; fi
  echo "Airdrop attempt $i..."
  "$SOLANA3" airdrop 2 --url devnet || true
  sleep 6
done

echo "Balance before deploy: $("$SOLANA3" balance --url devnet)"
echo "Deploying 44 Milady..."
"$SOLANA3" program deploy "$SO" --program-id "$KEYPAIR" --url devnet

echo
echo "VERIFYING..."
"$SOLANA3" program show "$PROGRAM_ID" --url devnet
echo
echo "44 MILADY DEVNET DEPLOYMENT: PASS"
