#!/usr/bin/env bash
# Create + fund a handful of testnet identities used by the contract tests
# and the backend end-to-end flow.
set -euo pipefail

NETWORK="${NETWORK:-testnet}"
IDS=(admin payer agent_copy agent_seo agent_research agent_audit)

for id in "${IDS[@]}"; do
  if ! stellar keys address "$id" >/dev/null 2>&1; then
    echo "→ generating $id"
    stellar keys generate "$id" --network "$NETWORK" --fund
  else
    echo "→ re-funding $id"
    stellar keys fund "$id" --network "$NETWORK" || true
  fi
  echo "  $id = $(stellar keys address "$id")"
done
