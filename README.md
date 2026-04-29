# Orizon Agents — Smart Contracts (Stellar / Soroban)

Four Rust contracts that put the Orizon Agents stack on-chain:

## 🚀 Live deployment

| layer | live URL | source |
| --- | --- | --- |
| 🔗 **Soroban contracts** (this repo, Stellar testnet) | 4 contracts deployed — [see addresses ↓](#current-testnet-deployment) | this repo |
| 🌐 **Frontend** (Vercel) | **https://orizon-agents-fe-stellar.vercel.app** | [Frontend repo](https://github.com/ALGOREX-PH/Orizon-Agents-FE-Stellar) |
| ⚙️ **Backend** (Render) | **https://orizon-agents-be-stellar.onrender.com** | [Backend repo](https://github.com/ALGOREX-PH/Orizon-Agents-BE-Stellar) |

**▸ See the contracts in action:** [open the dApp](https://orizon-agents-fe-stellar.vercel.app/app/orchestrator) → connect [Freighter](https://freighter.app) on **Test Net** → type `code a calculator web app` → **Authorize & Execute**. The trace ends with two real testnet transactions calling `PaymentEscrow.charge` and `AttestationRegistry.seal`, both linked to `stellar.expert`.

### Current testnet deployment

| contract | id |
| --- | --- |
| `AgentRegistry`        | [`CAPHXWU5…J3GQ`](https://stellar.expert/explorer/testnet/contract/CAPHXWU53UZUZJGV7IAE57NNMH3YYB5MTWO6YA53KKMXSFVLOITBJ3GQ) |
| `PaymentEscrow` (x402) | [`CBJPTMAP…525PI`](https://stellar.expert/explorer/testnet/contract/CBJPTMAPMGODGZCZ2IMEQSRUX3WGUXNMKDTNN2KMJ3NFGYZ5OJ5525PI) |
| `AttestationRegistry`  | [`CBYUZKOE…HEGK`](https://stellar.expert/explorer/testnet/contract/CBYUZKOET43UXTBXZUJIBBJW5ODGD2J2AZVVXCR3QONGOCAHOXQQHEGK) |
| `ReputationLedger`     | [`CDHDMVVE…WXKV`](https://stellar.expert/explorer/testnet/contract/CDHDMVVERSNZWFJIVOBM34CYLXE4A7UACHD3A6ROI63EYJY43J63WXKV) |
| Asset SAC (XLM)        | [`CDLZFC3S…CYSC`](https://stellar.expert/explorer/testnet/contract/CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC) |

Admin: `GA7AI5TAJEZA27I666DSJC4MUJYBEWUYNNZWPU7R2ONA7IZQVO6R5OQV`

---


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

# Easiest install: pre-built binary from GitHub releases
mkdir -p ~/.local/bin
curl -L https://github.com/stellar/stellar-cli/releases/download/v26.0.0/stellar-cli-26.0.0-x86_64-unknown-linux-gnu.tar.gz \
  | tar xz -C ~/.local/bin/
chmod +x ~/.local/bin/stellar
stellar --version                       # → stellar 26.x.x

# Identity (v26: no --global flag — identities are global by default)
stellar keys generate admin --network testnet --fund
stellar keys address admin              # your deployer G-address
```

> ⚠️ Don't `apt install seqan-apps`. Ubuntu's seqan-apps package ships
> an unrelated `stellar` binary that will shadow this one. If you've
> installed it, remove with `sudo apt remove --purge seqan-apps`.

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
contract/
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
