#!/usr/bin/env bash
# Deploy all four Orizon contracts to Stellar testnet.
#
# Requires: stellar-cli ≥ 22, a funded `admin` identity:
#   stellar keys generate --global admin --network testnet --fund
#
# Writes contract IDs to addresses.json.

set -euo pipefail
cd "$(dirname "$0")/.."

NETWORK="${NETWORK:-testnet}"
SOURCE="${SOURCE:-admin}"
ADMIN_ADDR="$(stellar keys address "$SOURCE")"

# Testnet USDC is the Circle-issued USDC asset. Its SAC is derived from the asset.
# Known issuer on testnet (Circle): GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5
USDC_ASSET="USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5"

echo "→ admin: $ADMIN_ADDR"
echo "→ network: $NETWORK"

echo "→ building wasm artifacts"
stellar contract build

# Path to each wasm (release profile)
WASM_DIR="target/wasm32-unknown-unknown/release"
REG_WASM="$WASM_DIR/orizon_agent_registry.wasm"
REP_WASM="$WASM_DIR/orizon_reputation_ledger.wasm"
ESC_WASM="$WASM_DIR/orizon_payment_escrow.wasm"
ATT_WASM="$WASM_DIR/orizon_attestation_registry.wasm"

for f in "$REG_WASM" "$REP_WASM" "$ESC_WASM" "$ATT_WASM"; do
  [ -f "$f" ] || { echo "missing $f — run 'stellar contract build'"; exit 1; }
done

# Resolve the USDC SAC (deploy if not yet wrapped)
echo "→ resolving USDC SAC on testnet"
USDC_ID="$(stellar contract asset deploy \
  --source "$SOURCE" \
  --network "$NETWORK" \
  --asset "$USDC_ASSET" 2>/dev/null || \
  stellar contract asset id \
  --source "$SOURCE" \
  --network "$NETWORK" \
  --asset "$USDC_ASSET")"
echo "  USDC SAC: $USDC_ID"

echo "→ deploying AgentRegistry"
REG_ID=$(stellar contract deploy \
  --source "$SOURCE" \
  --network "$NETWORK" \
  --wasm "$REG_WASM" \
  -- __constructor --admin "$ADMIN_ADDR")
echo "  AgentRegistry: $REG_ID"

echo "→ deploying ReputationLedger"
REP_ID=$(stellar contract deploy \
  --source "$SOURCE" \
  --network "$NETWORK" \
  --wasm "$REP_WASM" \
  -- __constructor --admin "$ADMIN_ADDR" --scorer "$ADMIN_ADDR")
echo "  ReputationLedger: $REP_ID"

echo "→ deploying PaymentEscrow"
ESC_ID=$(stellar contract deploy \
  --source "$SOURCE" \
  --network "$NETWORK" \
  --wasm "$ESC_WASM" \
  -- __constructor \
    --admin "$ADMIN_ADDR" \
    --usdc "$USDC_ID" \
    --registry "$REG_ID" \
    --settler "$ADMIN_ADDR")
echo "  PaymentEscrow: $ESC_ID"

echo "→ deploying AttestationRegistry"
ATT_ID=$(stellar contract deploy \
  --source "$SOURCE" \
  --network "$NETWORK" \
  --wasm "$ATT_WASM" \
  -- __constructor --admin "$ADMIN_ADDR" --sealer "$ADMIN_ADDR")
echo "  AttestationRegistry: $ATT_ID"

cat > addresses.json <<EOF
{
  "network": "$NETWORK",
  "admin": "$ADMIN_ADDR",
  "usdc_sac": "$USDC_ID",
  "agent_registry": "$REG_ID",
  "reputation_ledger": "$REP_ID",
  "payment_escrow": "$ESC_ID",
  "attestation_registry": "$ATT_ID"
}
EOF

echo
echo "✓ deployed — addresses.json written:"
cat addresses.json
