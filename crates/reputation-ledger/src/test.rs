#![cfg(test)]

use crate::{Error, ReputationLedger, ReputationLedgerClient};
use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
    Address, BytesN, Env,
};

fn setup(env: &Env) -> (ReputationLedgerClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let scorer = Address::generate(env);
    let id = env.register(ReputationLedger, (admin.clone(), scorer.clone()));
    (ReputationLedgerClient::new(env, &id), admin, scorer)
}

#[test]
fn submit_and_average() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, scorer) = setup(&env);
    let agent = symbol_short!("copy_v3");

    client.submit(&scorer, &agent, &5, &BytesN::from_array(&env, &[1u8; 16]));
    client.submit(&scorer, &agent, &4, &BytesN::from_array(&env, &[2u8; 16]));

    let s = client.score(&agent);
    assert_eq!(s.sum, 9);
    assert_eq!(s.count, 2);

    // mean = 4.5 → 45000 bps
    assert_eq!(client.avg_bps(&agent), 45_000);
}

#[test]
fn replay_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, scorer) = setup(&env);
    let agent = symbol_short!("copy_v3");
    let job = BytesN::from_array(&env, &[9u8; 16]);

    client.submit(&scorer, &agent, &5, &job);
    let err = client.try_submit(&scorer, &agent, &5, &job);
    assert_eq!(err.err().unwrap().unwrap(), Error::Replay);
}

#[test]
fn rejects_non_scorer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _scorer) = setup(&env);
    let stranger = Address::generate(&env);
    let err = client.try_submit(
        &stranger,
        &symbol_short!("copy_v3"),
        &5,
        &BytesN::from_array(&env, &[1u8; 16]),
    );
    assert_eq!(err.err().unwrap().unwrap(), Error::Unauthorized);
}
