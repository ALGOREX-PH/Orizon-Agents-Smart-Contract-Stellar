#![no_std]

//! # ReputationLedger v2
//!
//! Decayed, value-weighted, dispute-aware rating evidence store.
//!
//! Design (per Jøsang beta-reputation with a forgetting factor, and ERC-8004
//! conventions of raw evidence on-chain / complex aggregation off-chain):
//!
//! - **Value weighting** — each rating carries the job's USDC value (stroops)
//!   as its weight, so a 100-USDC job moves reputation more than a 0.01-USDC
//!   one. A per-rating weight cap stops any single job from buying dominance.
//! - **Exponential decay** — evidence loses influence at λ = 0.925 per weekly
//!   epoch (≈ 9-week half-life), applied lazily on write and on read, so an
//!   agent's score reflects recent behavior rather than ancient history.
//! - **Persistent replay guard** — v1 kept the `(agent, job)` seen-marker in
//!   TEMPORARY storage, which expires after hours and re-opened the replay
//!   window; v2 stores it in PERSISTENT storage.
//! - **Lifetime counters** — `count` and `disputed` never decay; they are raw
//!   evidence for off-chain consumers (dispute rate, volume checks).

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

/// Per-agent reputation accumulator.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepState {
    /// Σ (rating_bps × weight), decayed. rating_bps = rating_0_to_100 × 100,
    /// i.e. 0..10_000, so `sum_w / weight` is already a basis-point mean.
    pub sum_w: i128,
    /// Σ weight, decayed. Weight is the job's USDC value in stroops
    /// (7 decimals, Stellar convention).
    pub weight: i128,
    /// Lifetime rating count — never decayed.
    pub count: u32,
    /// Lifetime dispute count — never decayed.
    pub disputed: u32,
    /// Epoch of the last write (epoch = ledger timestamp / EPOCH_SECONDS).
    pub last_epoch: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Scorer,
    /// agent_id → RepState (persistent).
    Rep(Symbol),
    /// (agent_id, job_id) replay marker — PERSISTENT storage so the guard
    /// never lapses (the v1 bug kept it in temporary storage).
    Rated(Symbol, BytesN<16>),
    /// (agent_id, payer) → cumulative i128 weight (persistent, never decayed).
    /// Raw per-payer stake for off-chain Sybil / self-dealing analysis.
    PayerW(Symbol, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 1,
    NotFound = 2,
    Replay = 7,
    OutOfRange = 100,
}

#[contract]
pub struct ReputationLedger;

#[allow(deprecated)]
#[contractimpl]
impl ReputationLedger {
    pub fn __constructor(env: Env, admin: Address, scorer: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Scorer, &scorer);
    }

    /// Admin-only: swap the scorer address.
    pub fn set_scorer(env: Env, new_scorer: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotFound)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Scorer, &new_scorer);
        Ok(())
    }
}

#[cfg(test)]
mod test;
