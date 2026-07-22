#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    Env,
};

/// Deploys a fresh Stellar asset (test USDC stand-in) with the given admin,
/// mints starting balance to `holder`, and returns (token client, admin client).
fn setup_token<'a>(
    env: &Env,
    admin: &Address,
    holder: &Address,
    starting_balance: i128,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = token::Client::new(env, &sac.address());
    let asset_client = token::StellarAssetClient::new(env, &sac.address());
    asset_client.mint(holder, &starting_balance);
    (token_client, asset_client)
}

fn setup_contract(env: &Env) -> HarvestEscrowContractClient {
    let contract_id = env.register(HarvestEscrowContract, ());
    HarvestEscrowContractClient::new(env, &contract_id)
}

#[test]
fn test_happy_path_full_escrow_flow() {
    // Test 1 (Happy path): trader deposits, validator confirms, farmer is paid.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let farmer = Address::generate(&env);
    let validator = Address::generate(&env);

    let (token_client, _) = setup_token(&env, &admin, &buyer, 1_000_000);
    let contract = setup_contract(&env);

    contract.set_validator(&validator);
    contract.create_escrow(&1u64, &buyer, &farmer, &token_client.address, &500_000i128);
    contract.confirm_delivery(&1u64);

    assert_eq!(token_client.balance(&farmer), 500_000);
    assert_eq!(token_client.balance(&buyer), 500_000);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_edge_case_zero_amount_rejected() {
    // Test 2 (Edge case): an escrow with a non-positive amount must be rejected
    // so a malformed request can't lock zero-value or negative funds.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let farmer = Address::generate(&env);

    let (token_client, _) = setup_token(&env, &admin, &buyer, 1_000_000);
    let contract = setup_contract(&env);

    contract.create_escrow(&1u64, &buyer, &farmer, &token_client.address, &0i128);
}

#[test]
fn test_state_verification_after_creation_and_release() {
    // Test 3 (State verification): storage correctly reflects escrow status
    // both right after creation and right after release.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let farmer = Address::generate(&env);
    let validator = Address::generate(&env);

    let (token_client, _) = setup_token(&env, &admin, &buyer, 1_000_000);
    let contract = setup_contract(&env);

    contract.set_validator(&validator);
    contract.create_escrow(&7u64, &buyer, &farmer, &token_client.address, &250_000i128);

    let pending = contract.get_escrow(&7u64);
    assert_eq!(pending.delivered, false);
    assert_eq!(pending.released, false);
    assert_eq!(pending.amount, 250_000);

    contract.confirm_delivery(&7u64);

    let settled = contract.get_escrow(&7u64);
    assert_eq!(settled.delivered, true);
    assert_eq!(settled.released, true);
}

#[test]
#[should_panic(expected = "escrow already exists for this id")]
fn test_edge_case_duplicate_escrow_id_rejected() {
    // Test 4 (Edge case): reusing an escrow id for a new harvest lot must fail,
    // preventing a trader from accidentally or maliciously overwriting a
    // pending payment record.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let farmer = Address::generate(&env);

    let (token_client, _) = setup_token(&env, &admin, &buyer, 1_000_000);
    let contract = setup_contract(&env);

    contract.create_escrow(&3u64, &buyer, &farmer, &token_client.address, &100_000i128);
    contract.create_escrow(&3u64, &buyer, &farmer, &token_client.address, &100_000i128);
}

#[test]
#[should_panic(expected = "escrow already released")]
fn test_edge_case_double_release_rejected() {
    // Test 5 (Edge case): confirming delivery twice must not pay the farmer
    // twice out of a single escrow.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let farmer = Address::generate(&env);
    let validator = Address::generate(&env);

    let (token_client, _) = setup_token(&env, &admin, &buyer, 1_000_000);
    let contract = setup_contract(&env);

    contract.set_validator(&validator);
    contract.create_escrow(&9u64, &buyer, &farmer, &token_client.address, &100_000i128);
    contract.confirm_delivery(&9u64);
    contract.confirm_delivery(&9u64);
}
