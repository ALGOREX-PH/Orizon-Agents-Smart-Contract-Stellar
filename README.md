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
| `ReputationLedger`     | [`CDCSOBEV…22ZT`](https://stellar.expert/explorer/testnet/contract/CDCSOBEVZUPQZV5GV4D6KYHZCLNGW2KXY74RUHSZ3EZUXF34DPW422ZT) |
| Asset SAC (XLM)        | [`CDLZFC3S…CYSC`](https://stellar.expert/explorer/testnet/contract/CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC) |

Admin: `GA7AI5TAJEZA27I666DSJC4MUJYBEWUYNNZWPU7R2ONA7IZQVO6R5OQV`

### Current mainnet deployment

| contract | id |
| --- | --- |
| `AgentRegistry`        | [`CBTJ3BXT…LTD4`](https://stellar.expert/explorer/public/contract/CBTJ3BXTMTA2PQLRTSAZHEWQRTBMNHYCOKY5WOIYAH36LT4HTN63LTD4) |
| `PaymentEscrow` (x402) | [`CBJCQBA4…5CNF`](https://stellar.expert/explorer/public/contract/CBJCQBA47Q3EQ7HC46GAWJPVM7KMD5KAEI5KG4FPYJFKR3NYB4QR5CNF) |
| `AttestationRegistry`  | [`CBLV6QGF…AAK4`](https://stellar.expert/explorer/public/contract/CBLV6QGFCMXBXHT62JZ7YH22NXW7MVBGV6TGOGX3OHY46GQGPYCTAAK4) |
| `ReputationLedger`     | [`CDFWQJY7…AXSX`](https://stellar.expert/explorer/public/contract/CDFWQJY72GPH7PEQVFGBDZESZNVRF6LQLVWU42CFMWPGRME5RWN5AXSX) |
| Asset SAC (XLM)        | [`CAS3J7GY…OWMA`](https://stellar.expert/explorer/public/contract/CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA) |

Admin: `GA7AI5TAJEZA27I666DSJC4MUJYBEWUYNNZWPU7R2ONA7IZQVO6R5OQV`

---


| crate | purpose |
| --- | --- |
| `agent-registry` | ERC-8004-style identity, skills, price catalog |
| `reputation-ledger` | decayed, value-weighted rating evidence per agent (v2) |
| `payment-escrow` | x402-style per-call USDC authorize / charge / receipt |
| `attestation-registry` | write-once workflow receipts (job_id → proof record) |

Target: **Stellar testnet**, Protocol 22+. Payments settle in **USDC** via the Stellar Asset Contract (SEP-41).

### ReputationLedger v2

Decayed, value-weighted, dispute-aware evidence store (Jøsang beta-reputation with a forgetting factor; ERC-8004 convention of raw evidence on-chain, complex aggregation off-chain):

- `submit(caller, agent_id, job_id, rating_0_to_100, weight, payer, kind)` — scorer-only. `weight` is the job's USDC value in stroops, capped at 100 USDC per rating; the `(agent, job)` replay guard lives in **persistent** storage (v1 kept it in temporary storage, which expires). `kind = "dispute"` also bumps the lifetime dispute counter.
- Evidence decays by λ = 0.925 per weekly epoch (≈ 9-week half-life), applied lazily; after 96 idle epochs it is fully forgotten. Lifetime `count` / `disputed` never decay.
- Views (all decay-to-now, read-only): `rep_state`, `avg_bps` (weighted mean, basis points), `rep_bps(prior_bps, prior_weight)` (Bayesian-smoothed toward a caller-supplied prior), `dispute_rate_bps`, `payer_weight` (cumulative per-payer stake for off-chain Sybil analysis).

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
make deploy-main   # deploys all four to mainnet (CONFIRM_MAINNET=yes guard); writes addresses.mainnet.json
```

Per-network address books (`addresses.json` for testnet, `addresses.mainnet.json` for mainnet) are gitignored.

## Job lifecycle (on-chain)

```
authorize(payer, agent_id, max, expires)  → auth_id      ← PaymentEscrow
charge(caller, auth_id, amount, job_id)   → receipt_id   ← PaymentEscrow (× per step)
seal(caller, job_id, orchestrator,        → ()           ← AttestationRegistry
     intent_hash, agents, receipts, total_spent)
submit(caller, agent_id, job_id, rating,  → ()           ← ReputationLedger
       weight, payer, kind)
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
