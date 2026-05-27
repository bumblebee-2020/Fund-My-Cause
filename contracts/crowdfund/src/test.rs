#![cfg(test)]
#![allow(deprecated)]

use super::*;
use crate::types::Category;
use crate::{CrowdfundContract, CrowdfundContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

fn setup_contract(
    env: &Env,
    deadline: u64,
    goal: i128,
    min_contribution: i128,
) -> (
    Address,
    Address,
    CrowdfundContractClient<'_>,
    token::StellarAssetClient<'_>,
) {
    env.mock_all_auths();

    let creator = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(env, &contract_id);

    client.initialize(
        &creator,
        &token_id,
        &goal,
        &deadline,
        &min_contribution,
        &0i128,
        &String::from_str(env, "My Title"),
        &String::from_str(env, "My Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );

    (creator, token_id, client, token_admin_client)
}

#[test]
fn initialize_and_contribute_updates_state() {
    let env = Env::default();
    let deadline = 1_000u64;
    let goal = 10_000i128;
    let min_contribution = 100i128;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, goal, min_contribution);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);

    assert_eq!(client.total_raised(), 500);
    assert_eq!(client.contribution(&contributor), 500);
    assert!(client.is_contributor(&contributor));

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 500);
    assert_eq!(stats.goal, goal);
    assert_eq!(stats.contributor_count, 1);
    assert_eq!(stats.average_contribution, 500);
    assert_eq!(stats.largest_contribution, 500);
}

#[test]
fn cancel_allows_refund_before_deadline() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 10_000, 100);

    let contributor = Address::generate(&env);
    let token_client = token::Client::new(&env, &token_id);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    client.cancel_campaign();

    env.ledger().set_timestamp(deadline - 10);
    client.refund_single(&contributor);

    assert_eq!(client.contribution(&contributor), 0);
    assert_eq!(token_client.balance(&contributor), 500);
}

#[test]
fn invalid_platform_fee_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "My Title"),
        &String::from_str(&env, "My Description"),
        &None,
        &Some(PlatformConfig {
            address: Address::generate(&env),
            fee_bps: 10_001,
        }),
        &None,
        &Category::Other,
        &None,
        &None,
    );

    assert_eq!(result.err(), Some(Ok(ContractError::InvalidFee)));
}

// ── Boundary tests (#107) ─────────────────────────────────────────────────────

#[test]
fn accepted_token_whitelist_is_enforced() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let allowed_token = env.register_stellar_asset_contract(token_admin.clone());
    let other_token = env.register_stellar_asset_contract(token_admin);
    let allowed_token_admin = token::StellarAssetClient::new(&env, &allowed_token);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let mut accepted_tokens = Vec::new(&env);
    accepted_tokens.push_back(allowed_token.clone());

    client.initialize(
        &creator,
        &allowed_token,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "My Title"),
        &String::from_str(&env, "My Description"),
        &None,
        &None,
        &Some(accepted_tokens),
        &Category::Other,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    allowed_token_admin.mint(&contributor, &100);

    let result = client.try_contribute(&contributor, &100, &other_token, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::TokenNotAccepted)));
}

// ── refund_batch tests (#278) ─────────────────────────────────────────────────

#[test]
fn refund_batch_refunds_multiple_contributors() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let token_client = token::Client::new(&env, &token_id);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);
    let c3 = Address::generate(&env);

    token_admin_client.mint(&c1, &500);
    token_admin_client.mint(&c2, &300);
    token_admin_client.mint(&c3, &200);

    client.contribute(&c1, &500, &token_id, &None);
    client.contribute(&c2, &300, &token_id, &None);
    client.contribute(&c3, &200, &token_id, &None);

    client.cancel_campaign();

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());
    batch.push_back(c2.clone());
    batch.push_back(c3.clone());

    let refunded = client.refund_batch(&batch);
    assert_eq!(refunded, 3);

    assert_eq!(token_client.balance(&c1), 500);
    assert_eq!(token_client.balance(&c2), 300);
    assert_eq!(token_client.balance(&c3), 200);
    assert_eq!(client.contribution(&c1), 0);
    assert_eq!(client.contribution(&c2), 0);
    assert_eq!(client.contribution(&c3), 0);
}

#[test]
fn refund_batch_skips_already_refunded() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let c1 = Address::generate(&env);
    token_admin_client.mint(&c1, &500);
    client.contribute(&c1, &500, &token_id, &None);
    client.cancel_campaign();

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());
    let r1 = client.refund_batch(&batch);
    assert_eq!(r1, 1);

    let r2 = client.refund_batch(&batch);
    assert_eq!(r2, 0);
}

#[test]
fn refund_batch_fails_when_campaign_still_active() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let c1 = Address::generate(&env);
    token_admin_client.mint(&c1, &500);
    client.contribute(&c1, &500, &token_id, &None);

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());

    let result = client.try_refund_batch(&batch);
    assert_eq!(result.err(), Some(Ok(ContractError::CampaignStillActive)));
}

// ── pause/unpause tests (#279) ────────────────────────────────────────────────

#[test]
fn pause_blocks_contributions_and_unpause_resumes() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);

    client.pause();
    assert_eq!(client.status(), Status::Paused);

    let result = client.try_contribute(&contributor, &500, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::CampaignPaused)));

    client.unpause();
    assert_eq!(client.status(), Status::Active);

    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.total_raised(), 500);
}

#[test]
fn pause_allows_refunds_when_cancelled() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let contributor = Address::generate(&env);
    let token_client = token::Client::new(&env, &token_id);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    client.cancel_campaign();

    client.refund_single(&contributor);
    assert_eq!(token_client.balance(&contributor), 500);
}

#[test]
fn unpause_fails_when_not_paused() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, _token_id, client, _) = setup_contract(&env, deadline, 100_000, 100);

    let result = client.try_unpause();
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

#[test]
fn pause_fails_when_not_active() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, _token_id, client, _) = setup_contract(&env, deadline, 100_000, 100);

    client.cancel_campaign();

    let result = client.try_pause();
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

// ── max_contribution tests ────────────────────────────────────────────────────

fn setup_contract_with_max(
    env: &Env,
    deadline: u64,
    goal: i128,
    min_contribution: i128,
    max_contribution: i128,
) -> (
    Address,
    Address,
    CrowdfundContractClient<'_>,
    token::StellarAssetClient<'_>,
) {
    env.mock_all_auths();

    let creator = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(env, &contract_id);

    client.initialize(
        &creator,
        &token_id,
        &goal,
        &deadline,
        &min_contribution,
        &max_contribution,
        &String::from_str(env, "My Title"),
        &String::from_str(env, "My Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );

    (creator, token_id, client, token_admin_client)
}

#[test]
fn contribute_within_max_succeeds() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 500);
}

#[test]
fn contribute_exceeding_max_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &600);

    let result = client.try_contribute(&contributor, &600, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

#[test]
fn cumulative_contribution_exceeding_max_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &600);

    client.contribute(&contributor, &300, &token_id, &None);
    let result = client.try_contribute(&contributor, &300, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

#[test]
fn no_max_limit_allows_large_contribution() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) = setup_contract(&env, 1_000, 10_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &9_000);

    client.contribute(&contributor, &9_000, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 9_000);
}

#[test]
fn max_contribution_view_returns_stored_value() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract_with_max(&env, 1_000, 10_000, 100, 750);

    assert_eq!(client.max_contribution(), 750);
}

#[test]
fn initialize_with_max_below_min_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &10_000,
        &1_000,
        &200,
        &100, // max < min — invalid
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Desc"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

// ── Input validation tests ────────────────────────────────────────────────────

#[test]
fn initialize_with_empty_title_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, ""), // empty title
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::StringEmpty)));
}

#[test]
fn initialize_with_title_too_long_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    // 65-character title (max is 64)
    let long_title = String::from_str(
        &env,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );
    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &long_title,
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::StringTooLong)));
}

#[test]
fn initialize_with_self_fee_address_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Description"),
        &None,
        &Some(PlatformConfig {
            address: creator.clone(), // same as creator — invalid
            fee_bps: 100,
        }),
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::SelfFeeAddress)));
}

#[test]
fn contribute_with_zero_amount_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    let contributor = Address::generate(&env);
    let result = client.try_contribute(&contributor, &0, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::AmountNotPositive)));
}

#[test]
fn contribute_with_negative_amount_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    let contributor = Address::generate(&env);
    let result = client.try_contribute(&contributor, &-1, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::AmountNotPositive)));
}

#[test]
fn update_metadata_with_empty_title_is_rejected() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let result = client.try_update_metadata(&Some(String::from_str(&env, "")), &None, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::StringEmpty)));
}

#[test]
fn update_metadata_with_valid_title_succeeds() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    client.update_metadata(&Some(String::from_str(&env, "New Title")), &None, &None);
    assert_eq!(client.title(), String::from_str(&env, "New Title"));
}

// ── Issue #416: Campaign cancellation with multiple contributors ──────────────

#[test]
fn cancel_with_multiple_contributors_allows_all_refunds() {
    let env = Env::default();
    let deadline = 2_000u64;
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 50_000, 100);

    let token_client = token::Client::new(&env, &token_id);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);
    let c3 = Address::generate(&env);
    let c4 = Address::generate(&env);

    token_admin_client.mint(&c1, &1_000);
    token_admin_client.mint(&c2, &2_000);
    token_admin_client.mint(&c3, &3_000);
    token_admin_client.mint(&c4, &4_000);

    client.contribute(&c1, &1_000, &token_id, &None);
    client.contribute(&c2, &2_000, &token_id, &None);
    client.contribute(&c3, &3_000, &token_id, &None);
    client.contribute(&c4, &4_000, &token_id, &None);

    assert_eq!(client.total_raised(), 10_000);
    assert_eq!(client.get_stats().contributor_count, 4);

    // Creator cancels — still before the deadline
    client.cancel_campaign();
    assert_eq!(client.status(), Status::Cancelled);

    // All four contributors can claim refunds before deadline
    env.ledger().set_timestamp(deadline - 500);
    client.refund_single(&c1);
    client.refund_single(&c2);
    client.refund_single(&c3);
    client.refund_single(&c4);

    assert_eq!(token_client.balance(&c1), 1_000);
    assert_eq!(token_client.balance(&c2), 2_000);
    assert_eq!(token_client.balance(&c3), 3_000);
    assert_eq!(token_client.balance(&c4), 4_000);

    assert_eq!(client.contribution(&c1), 0);
    assert_eq!(client.contribution(&c2), 0);
    assert_eq!(client.contribution(&c3), 0);
    assert_eq!(client.contribution(&c4), 0);
}

#[test]
fn cancel_from_paused_state_succeeds() {
    // A paused campaign can also be cancelled (creator should not be locked out).
    let env = Env::default();
    let deadline = 1_000u64;
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 10_000, 100);

    let contributor = Address::generate(&env);
    let token_client = token::Client::new(&env, &token_id);
    token_admin_client.mint(&contributor, &500);
    client.contribute(&contributor, &500, &token_id, &None);

    client.pause();
    assert_eq!(client.status(), Status::Paused);

    // Cancelling from Paused should succeed
    client.cancel_campaign();
    assert_eq!(client.status(), Status::Cancelled);

    client.refund_single(&contributor);
    assert_eq!(token_client.balance(&contributor), 500);
}

#[test]
fn cancel_already_cancelled_is_rejected() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    client.cancel_campaign();
    let result = client.try_cancel_campaign();
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

// ── Issue #417: Pause / resume integration tests ──────────────────────────────

#[test]
fn resume_restores_active_state() {
    let env = Env::default();
    let deadline = 1_000u64;
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 10_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);

    client.pause();
    assert_eq!(client.status(), Status::Paused);

    // Contributions blocked while paused
    let result = client.try_contribute(&contributor, &500, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::CampaignPaused)));

    // Resume using the new `resume()` function
    client.resume();
    assert_eq!(client.status(), Status::Active);

    // Contributions accepted again
    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.total_raised(), 500);
}

#[test]
fn resume_fails_when_not_paused() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let result = client.try_resume();
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

#[test]
fn pause_and_resume_multiple_times() {
    let env = Env::default();
    let deadline = 1_000u64;
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 10_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &3_000);

    // Contribute 1_000
    client.contribute(&contributor, &1_000, &token_id, &None);

    // Pause → Resume → Contribute again
    client.pause();
    assert_eq!(client.status(), Status::Paused);
    client.resume();
    assert_eq!(client.status(), Status::Active);
    client.contribute(&contributor, &1_000, &token_id, &None);

    // Pause again
    client.pause();
    assert_eq!(client.status(), Status::Paused);
    client.unpause(); // legacy alias
    assert_eq!(client.status(), Status::Active);
    client.contribute(&contributor, &1_000, &token_id, &None);

    assert_eq!(client.total_raised(), 3_000);
    assert_eq!(client.contribution(&contributor), 3_000);
}

// ── Issue #418: Tiered contribution rewards ───────────────────────────────────

#[test]
fn set_reward_tiers_and_get_tier_for_amount() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 100_000, 100);

    let mut tiers = Vec::new(&env);
    tiers.push_back(crate::types::RewardTier {
        min_amount: 100,
        name: String::from_str(&env, "Bronze"),
        description: String::from_str(&env, "Entry level"),
    });
    tiers.push_back(crate::types::RewardTier {
        min_amount: 1_000,
        name: String::from_str(&env, "Silver"),
        description: String::from_str(&env, "Mid level"),
    });
    tiers.push_back(crate::types::RewardTier {
        min_amount: 10_000,
        name: String::from_str(&env, "Gold"),
        description: String::from_str(&env, "Top level"),
    });

    client.set_reward_tiers(&tiers);

    // Below all tiers
    assert!(client.get_tier_for_amount(&50).is_none());

    // Bronze range
    let bronze = client.get_tier_for_amount(&100).unwrap();
    assert_eq!(bronze.name, String::from_str(&env, "Bronze"));

    // Silver range
    let silver = client.get_tier_for_amount(&1_000).unwrap();
    assert_eq!(silver.name, String::from_str(&env, "Silver"));

    // Gold range
    let gold = client.get_tier_for_amount(&10_000).unwrap();
    assert_eq!(gold.name, String::from_str(&env, "Gold"));

    // Above gold threshold still returns gold
    let also_gold = client.get_tier_for_amount(&99_999).unwrap();
    assert_eq!(also_gold.name, String::from_str(&env, "Gold"));
}

#[test]
fn contribute_assigns_tier_to_contributor() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 1_000, 100_000, 100);

    let mut tiers = Vec::new(&env);
    tiers.push_back(crate::types::RewardTier {
        min_amount: 500,
        name: String::from_str(&env, "Bronze"),
        description: String::from_str(&env, "Basic supporter"),
    });
    tiers.push_back(crate::types::RewardTier {
        min_amount: 2_000,
        name: String::from_str(&env, "Gold"),
        description: String::from_str(&env, "Major supporter"),
    });
    client.set_reward_tiers(&tiers);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &3_000);

    // Contribute 600 → qualifies for Bronze
    client.contribute(&contributor, &600, &token_id, &None);
    let tier = client.get_contributor_tier(&contributor).unwrap();
    assert_eq!(tier.name, String::from_str(&env, "Bronze"));

    // Contribute another 1_500 (total 2_100) → upgrades to Gold
    client.contribute(&contributor, &1_400, &token_id, &None);
    let tier = client.get_contributor_tier(&contributor).unwrap();
    assert_eq!(tier.name, String::from_str(&env, "Gold"));
}

#[test]
fn set_reward_tiers_unsorted_is_rejected() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 100_000, 100);

    let mut tiers = Vec::new(&env);
    tiers.push_back(crate::types::RewardTier {
        min_amount: 1_000,
        name: String::from_str(&env, "Silver"),
        description: String::from_str(&env, "Mid level"),
    });
    tiers.push_back(crate::types::RewardTier {
        min_amount: 100, // lower than previous — invalid
        name: String::from_str(&env, "Bronze"),
        description: String::from_str(&env, "Entry level"),
    });

    let result = client.try_set_reward_tiers(&tiers);
    assert_eq!(result.err(), Some(Ok(ContractError::InvalidGoal)));
}

#[test]
fn no_tier_assigned_when_none_configured() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 1_000, 100_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);
    client.contribute(&contributor, &1_000, &token_id, &None);

    // No tiers configured — contributor should have no tier
    assert!(client.get_contributor_tier(&contributor).is_none());
}

// ── Issue #419: Contribution history tracking ─────────────────────────────────

#[test]
fn contribution_history_tracks_single_contribution() {
    let env = Env::default();
    let deadline = 1_000u64;
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);

    env.ledger().set_timestamp(100);
    client.contribute(&contributor, &500, &token_id, &None);

    let history = client.get_contribution_history(&contributor);
    assert_eq!(history.len(), 1);

    let record = history.get(0).unwrap();
    assert_eq!(record.amount, 500);
    assert_eq!(record.timestamp, 100);
    assert_eq!(record.running_total, 500);
}

#[test]
fn contribution_history_tracks_multiple_contributions() {
    let env = Env::default();
    let deadline = 1_000u64;
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 0);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &10_000);

    env.ledger().set_timestamp(10);
    client.contribute(&contributor, &1_000, &token_id, &None);

    env.ledger().set_timestamp(20);
    client.contribute(&contributor, &2_000, &token_id, &None);

    env.ledger().set_timestamp(30);
    client.contribute(&contributor, &3_000, &token_id, &None);

    let history = client.get_contribution_history(&contributor);
    assert_eq!(history.len(), 3);

    let r0 = history.get(0).unwrap();
    assert_eq!(r0.amount, 1_000);
    assert_eq!(r0.timestamp, 10);
    assert_eq!(r0.running_total, 1_000);

    let r1 = history.get(1).unwrap();
    assert_eq!(r1.amount, 2_000);
    assert_eq!(r1.timestamp, 20);
    assert_eq!(r1.running_total, 3_000);

    let r2 = history.get(2).unwrap();
    assert_eq!(r2.amount, 3_000);
    assert_eq!(r2.timestamp, 30);
    assert_eq!(r2.running_total, 6_000);
}

#[test]
fn contribution_history_is_independent_per_contributor() {
    let env = Env::default();
    let deadline = 1_000u64;
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 0);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);
    token_admin_client.mint(&c1, &5_000);
    token_admin_client.mint(&c2, &5_000);

    env.ledger().set_timestamp(5);
    client.contribute(&c1, &1_000, &token_id, &None);
    client.contribute(&c2, &2_000, &token_id, &None);

    env.ledger().set_timestamp(15);
    client.contribute(&c1, &500, &token_id, &None);

    let h1 = client.get_contribution_history(&c1);
    let h2 = client.get_contribution_history(&c2);

    // c1 has 2 records, c2 has 1
    assert_eq!(h1.len(), 2);
    assert_eq!(h2.len(), 1);

    assert_eq!(h1.get(0).unwrap().running_total, 1_000);
    assert_eq!(h1.get(1).unwrap().running_total, 1_500);
    assert_eq!(h2.get(0).unwrap().running_total, 2_000);
}

#[test]
fn contribution_history_empty_for_non_contributor() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 100_000, 100);

    let stranger = Address::generate(&env);
    let history = client.get_contribution_history(&stranger);
    assert_eq!(history.len(), 0);
}
