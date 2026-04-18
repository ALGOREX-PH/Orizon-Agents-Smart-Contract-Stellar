#![no_std]

use orizon_shared::{codes, Authorization, Receipt};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, BytesN,
    Env, IntoVal, Symbol,
};

/// Minimal import of the agent-registry's `owner_of` view so we can resolve payouts.
mod registry {
    use soroban_sdk::{contractclient, Address, Env, Symbol};
    #[contractclient(name = "RegistryClient")]
    pub trait Registry {
        fn owner_of(env: Env, id: Symbol) -> Address;
    }
}

#[contracttype]
pub enum DataKey {
    Admin,
    Usdc,
    Registry,
    Settler,
    Auth(BytesN<16>),
    Receipt(BytesN<16>),
    /// Used to derive unique auth / receipt ids.
    Nonce,
}

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Error {
    Unauthorized = codes::UNAUTHORIZED,
    NotFound = codes::NOT_FOUND,
    Expired = codes::EXPIRED,
    Insufficient = codes::INSUFFICIENT_AUTH,
    Revoked = codes::REVOKED,
    BadAmount = 101,
}

#[contract]
pub struct PaymentEscrow;

#[contractimpl]
impl PaymentEscrow {
    pub fn __constructor(
        env: Env,
        admin: Address,
        usdc: Address,
        registry: Address,
        settler: Address,
    ) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Usdc, &usdc);
        env.storage().instance().set(&DataKey::Registry, &registry);
        env.storage().instance().set(&DataKey::Settler, &settler);
        env.storage().instance().set(&DataKey::Nonce, &0u64);
    }

    /// STEP 1 — payer pre-authorizes a max spend for a given agent.
    /// On HTTP: this matches the initial 402 challenge response.
    pub fn authorize(
        env: Env,
        payer: Address,
        agent_id: Symbol,
        max_amount: i128,
        expires_at: u64,
    ) -> Result<BytesN<16>, Error> {
        payer.require_auth();
        if max_amount <= 0 {
            return Err(Error::BadAmount);
        }

        let auth_id = next_id(&env);
        let record = Authorization {
            payer: payer.clone(),
            agent_id: agent_id.clone(),
            max_amount,
            spent: 0,
            expires_at,
            revoked: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Auth(auth_id.clone()), &record);

        env.events().publish(
            (symbol_short!("authd"), agent_id),
            (auth_id.clone(), payer, max_amount),
        );
        Ok(auth_id)
    }

    /// STEP 2 — the settler (backend JobManager) draws `amount` from the auth
    /// and triggers a USDC transfer from payer → agent owner.
    pub fn charge(
        env: Env,
        caller: Address,
        auth_id: BytesN<16>,
        amount: i128,
        job_id: BytesN<16>,
    ) -> Result<BytesN<16>, Error> {
        caller.require_auth();
        let settler: Address = env
            .storage()
            .instance()
            .get(&DataKey::Settler)
            .ok_or(Error::NotFound)?;
        if caller != settler {
            return Err(Error::Unauthorized);
        }
        if amount <= 0 {
            return Err(Error::BadAmount);
        }

        let mut auth: Authorization = env
            .storage()
            .persistent()
            .get(&DataKey::Auth(auth_id.clone()))
            .ok_or(Error::NotFound)?;

        if auth.revoked {
            return Err(Error::Revoked);
        }
        if auth.expires_at < env.ledger().timestamp() {
            return Err(Error::Expired);
        }
        if auth.spent + amount > auth.max_amount {
            return Err(Error::Insufficient);
        }

        // Resolve agent owner via AgentRegistry cross-call
        let registry_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Registry)
            .ok_or(Error::NotFound)?;
        let reg = registry::RegistryClient::new(&env, &registry_addr);
        let agent_owner: Address = reg.owner_of(&auth.agent_id);

        // Move USDC payer → agent_owner
        let usdc_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Usdc)
            .ok_or(Error::NotFound)?;
        let usdc = token::TokenClient::new(&env, &usdc_addr);
        usdc.transfer(&auth.payer, &agent_owner, &amount);

        // Persist updated authorization + receipt
        auth.spent += amount;
        env.storage()
            .persistent()
            .set(&DataKey::Auth(auth_id.clone()), &auth);

        let receipt_id = next_id(&env);
        let receipt = Receipt {
            auth_id: auth_id.clone(),
            agent_id: auth.agent_id.clone(),
            amount,
            job_id: job_id.clone(),
            settled_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Receipt(receipt_id.clone()), &receipt);

        env.events().publish(
            (symbol_short!("charged"), auth.agent_id),
            (receipt_id.clone(), auth_id, amount, job_id),
        );
        Ok(receipt_id)
    }

    /// Payer cancels an unused authorization.
    pub fn revoke(env: Env, payer: Address, auth_id: BytesN<16>) -> Result<(), Error> {
        payer.require_auth();
        let mut auth: Authorization = env
            .storage()
            .persistent()
            .get(&DataKey::Auth(auth_id.clone()))
            .ok_or(Error::NotFound)?;
        if auth.payer != payer {
            return Err(Error::Unauthorized);
        }
        auth.revoked = true;
        env.storage()
            .persistent()
            .set(&DataKey::Auth(auth_id.clone()), &auth);
        env.events()
            .publish((symbol_short!("revoked"),), auth_id);
        Ok(())
    }

    // ── Views ─────────────────────────────────────────────────────────
    pub fn authorization(env: Env, auth_id: BytesN<16>) -> Result<Authorization, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Auth(auth_id))
            .ok_or(Error::NotFound)
    }

    pub fn receipt(env: Env, receipt_id: BytesN<16>) -> Result<Receipt, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Receipt(receipt_id))
            .ok_or(Error::NotFound)
    }

    pub fn settler(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Settler)
            .expect("settler must be set")
    }
}

fn next_id(env: &Env) -> BytesN<16> {
    let n: u64 = env
        .storage()
        .instance()
        .get(&DataKey::Nonce)
        .unwrap_or(0u64);
    env.storage().instance().set(&DataKey::Nonce, &(n + 1));

    // Derive a BytesN<16> from (timestamp, ledger_seq, nonce, contract_addr).
    // sha256 output is 32 bytes; we take the high half for a compact id.
    let topic: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![
        env,
        env.ledger().timestamp().into_val(env),
        env.ledger().sequence().into_val(env),
        n.into_val(env),
        env.current_contract_address().into_val(env),
    ];
    let bytes = env.crypto().sha256(&topic.to_xdr(env));
    let full = bytes.to_array();
    let mut half = [0u8; 16];
    half.copy_from_slice(&full[..16]);
    BytesN::from_array(env, &half)
}

#[cfg(test)]
mod test;
