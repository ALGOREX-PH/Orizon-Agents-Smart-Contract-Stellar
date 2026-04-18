#!/usr/bin/env bash
# Deploy all four Orizon contracts to Stellar testnet.
#
# Requires: stellar-cli ≥ 26, a funded `admin` identity:
#   stellar keys generate admin --network testnet --fund
#
# Writes contract IDs to addresses.json.

set -euo pipefail
cd "$(dirname "$0")/.."

NETWORK="${NETWORK:-testnet}"
SOURCE="${SOURCE:-admin}"
# Default to native XLM SAC for the MVP. Override with USDC by setting
#   ASSET="USDC:G..."   before running.
ASSET="${ASSET:-native}"

ADMIN_ADDR="$(stellar keys address "$SOURCE")"
echo "→ admin: $ADMIN_ADDR"
echo "→ network: $NETWORK"
echo "→ payment asset: $ASSET"

echo "→ building wasm artifacts"
stellar contract build

# stellar-cli v26+ uses wasm32v1-none. Older toolchains land in wasm32-unknown-unknown.
WASM_DIR="target/wasm32v1-none/release"
[ -d "$WASM_DIR" ] || WASM_DIR="target/wasm32-unknown-unknown/release"
REG_WASM="$WASM_DIR/orizon_agent_registry.wasm"
REP_WASM="$WASM_DIR/orizon_reputation_ledger.wasm"
ESC_WASM="$WASM_DIR/orizon_payment_escrow.wasm"
ATT_WASM="$WASM_DIR/orizon_attestation_registry.wasm"

for f in "$REG_WASM" "$REP_WASM" "$ESC_WASM" "$ATT_WASM"; do
  [ -f "$f" ] || { echo "missing $f — run 'stellar contract build'"; exit 1; }
done

# Resolve (or deploy) the asset's Soroban Asset Contract id.
# `asset id` is a pure lookup — no signing. If the SAC isn't wrapped yet on
# this network, `asset deploy` wraps + returns the id (idempotent in practice).
echo "→ resolving asset SAC ($ASSET)"
ASSET_SAC="$(stellar contract id asset --asset "$ASSET" --network "$NETWORK" 2>/dev/null || true)"
if [ -z "$ASSET_SAC" ]; then
  ASSET_SAC="$(stellar contract asset deploy \
    --source "$SOURCE" \
    --network "$NETWORK" \
    --asset "$ASSET")"
fi
echo "  asset SAC: $ASSET_SAC"

deploy_contract() {
  local label="$1" wasm="$2"; shift 2
  echo "→ deploying $label" >&2
  local id
  id="$(stellar contract deploy \
    --source "$SOURCE" \
    --network "$NETWORK" \
    --wasm "$wasm" \
    -- "$@" 2>&1 | tail -n1)"
  echo "  $label: $id" >&2
  printf "%s" "$id"
}

REG_ID=$(deploy_contract "AgentRegistry" "$REG_WASM" \
  --admin "$ADMIN_ADDR")

REP_ID=$(deploy_contract "ReputationLedger" "$REP_WASM" \
  --admin "$ADMIN_ADDR" --scorer "$ADMIN_ADDR")

ESC_ID=$(deploy_contract "PaymentEscrow" "$ESC_WASM" \
  --admin "$ADMIN_ADDR" \
  --usdc "$ASSET_SAC" \
  --registry "$REG_ID" \
  --settler "$ADMIN_ADDR")

ATT_ID=$(deploy_contract "AttestationRegistry" "$ATT_WASM" \
  --admin "$ADMIN_ADDR" --sealer "$ADMIN_ADDR")

cat > addresses.json <<EOF
{
  "network": "$NETWORK",
  "admin": "$ADMIN_ADDR",
  "asset": "$ASSET",
  "asset_sac": "$ASSET_SAC",
  "agent_registry": "$REG_ID",
  "reputation_ledger": "$REP_ID",
  "payment_escrow": "$ESC_ID",
  "attestation_registry": "$ATT_ID"
}
EOF

echo
echo "✓ deployed — addresses.json:"
cat addresses.json
