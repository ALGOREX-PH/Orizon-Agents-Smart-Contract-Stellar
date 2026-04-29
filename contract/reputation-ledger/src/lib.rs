#![no_std]

use orizon_shared::Score;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    Symbol,
};

#[contracttype]
pub enum DataKey {
    Admin,
    Scorer,
    Score(Symbol),
    /// (agent_id, job_id) → seen marker; prevents replay of the same rating.
    Rated(Symbol, BytesN<16>),
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

    /// Submit a rating on behalf of a completed job.
    /// `caller` must be the registered `scorer` (the backend JobManager).
    pub fn submit(
        env: Env,
        caller: Address,
        agent_id: Symbol,
        rating_0_to_5: u32,
        job_id: BytesN<16>,
    ) -> Result<(), Error> {
        caller.require_auth();

        let scorer: Address = env
            .storage()
            .instance()
            .get(&DataKey::Scorer)
            .ok_or(Error::NotFound)?;
        if caller != scorer {
            return Err(Error::Unauthorized);
        }
        if rating_0_to_5 > 5 {
            return Err(Error::OutOfRange);
        }
        let seen_key = DataKey::Rated(agent_id.clone(), job_id.clone());
        if env.storage().temporary().has(&seen_key) {
            return Err(Error::Replay);
        }

        let key = DataKey::Score(agent_id.clone());
        let mut score: Score = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Score { sum: 0, count: 0 });
        score.sum += rating_0_to_5 as u64;
        score.count += 1;
        env.storage().persistent().set(&key, &score);
        env.storage().temporary().set(&seen_key, &true);

        env.events()
            .publish((symbol_short!("rated"), agent_id), (rating_0_to_5, job_id));
        Ok(())
    }

    /// View — raw sum + count.
    pub fn score(env: Env, agent_id: Symbol) -> Score {
        env.storage()
            .persistent()
            .get(&DataKey::Score(agent_id))
            .unwrap_or(Score { sum: 0, count: 0 })
    }

    /// View — mean × 10000 (basis points). 0 if count == 0.
    pub fn avg_bps(env: Env, agent_id: Symbol) -> u32 {
        let s = Self::score(env, agent_id);
        if s.count == 0 {
            0
        } else {
            ((s.sum * 10_000) / (s.count as u64)) as u32
        }
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
