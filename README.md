# Orizon Agents — Smart Contracts (Stellar / Soroban)

Four Rust contracts that put the Orizon Agents stack on-chain:

| crate | purpose |
| --- | --- |
| `agent-registry` | ERC-8004-style identity, skills, price catalog |
| `reputation-ledger` | rating aggregates (rolling mean) per agent |
| `payment-escrow` | x402-style per-call USDC authorize / charge / receipt |
| `attestation-registry` | write-once workflow receipts (job_id → proof record) |

Target: **Stellar testnet**, Protocol 22+. Payments settle in **USDC** via the Stellar Asset Contract (SEP-41).

## One-time setup

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt

stellar keys generate --global admin --network testnet --fund
stellar keys address admin          # your deployer G-address
```

## Common commands

```bash
make check         # cargo check --all
make test          # cargo test --all
make build         # stellar contract build → target/wasm32-unknown-unknown/release/*.wasm
make deploy-test   # deploys all four to testnet; writes addresses.json
```

## Job lifecycle (on-chain)

```
authorize(payer, agent_id, max, expires)  → auth_id      ← PaymentEscrow
charge(caller, auth_id, amount, job_id)   → receipt_id   ← PaymentEscrow (× per step)
seal(caller, job_id, agents, receipts,    → ()           ← AttestationRegistry
     total_spent, orchestrator, intent_hash)
submit(caller, agent_id, rating, job_id)  → ()           ← ReputationLedger
```

The backend (FastAPI + Agno) orchestrates the intent, calls these contracts in order, and streams the SSE trace to the frontend.

## Layout

```
crates/
  shared/                 # #[contracttype] structs shared across contracts
  agent-registry/
  reputation-ledger/
  payment-escrow/
  attestation-registry/
scripts/
  deploy_testnet.sh       # deploys everything, outputs addresses.json
  fund_accounts.sh        # friendbot for local test accounts
```

MVP contracts are **not upgradable**. Re-deploy on logic changes.
