#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_reputation_contract() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depin_registry = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin, &depin_registry);

    // Create a mock DePIN ID
    let depin_id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);

    // Test rating and review
    client.rate_and_review_depin(&user, &depin_id, &4, &String::from_str(&env, "Great service!"));

    // Test getting reviews
    let reviews = client.get_reviews(&depin_id);
    assert_eq!(reviews.len(), 1);

    // Test getting average rating
    let avg_rating = client.get_average_rating(&depin_id);
    assert_eq!(avg_rating, Some(4));

    // Test getting rating stats
    let (avg, count, min, max) = client.get_rating_stats(&depin_id);
    assert_eq!(avg, Some(4));
    assert_eq!(count, 1);
    assert_eq!(min, 4);
    assert_eq!(max, 4);

    // Add another rating
    let user2 = Address::generate(&env);
    client.rate_and_review_depin(&user2, &depin_id, &5, &String::from_str(&env, "Excellent!"));

    // Test updated stats
    let (avg, count, min, max) = client.get_rating_stats(&depin_id);
    assert_eq!(avg, Some(4)); // (4 + 5) / 2 = 4 (integer division)
    assert_eq!(count, 2);
    assert_eq!(min, 4);
    assert_eq!(max, 5);
}

#[test]
fn test_user_can_update_review() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depin_registry = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin, &depin_registry);

    let depin_id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);

    // Add initial review
    client.rate_and_review_depin(&user, &depin_id, &3, &String::from_str(&env, "Average service"));

    // Verify initial review
    let reviews = client.get_reviews(&depin_id);
    assert_eq!(reviews.len(), 1);
    let (addr, rating, review) = reviews.get(0).unwrap();
    assert_eq!(addr, user);
    assert_eq!(rating, 3);
    assert_eq!(review, String::from_str(&env, "Average service"));

    // Update the same user's review
    client.rate_and_review_depin(&user, &depin_id, &5, &String::from_str(&env, "Much improved!"));

    // Verify updated review (should still be only 1 review from this user)
    let updated_reviews = client.get_reviews(&depin_id);
    assert_eq!(updated_reviews.len(), 1);
    let (addr, rating, review) = updated_reviews.get(0).unwrap();
    assert_eq!(addr, user);
    assert_eq!(rating, 5);
    assert_eq!(review, String::from_str(&env, "Much improved!"));
}

#[test]
fn test_empty_reviews() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depin_registry = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin, &depin_registry);

    let depin_id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);

    // Test empty reviews
    let reviews = client.get_reviews(&depin_id);
    assert_eq!(reviews.len(), 0);

    // Test no average rating
    let avg_rating = client.get_average_rating(&depin_id);
    assert_eq!(avg_rating, None);

    // Test empty rating stats
    let (avg, count, min, max) = client.get_rating_stats(&depin_id);
    assert_eq!(avg, None);
    assert_eq!(count, 0);
    assert_eq!(min, 0);
    assert_eq!(max, 0);

    // Test review count
    let count = client.get_review_count(&depin_id);
    assert_eq!(count, 0);
}

#[test]
fn test_admin_can_remove_reviews() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depin_registry = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin, &depin_registry);

    let depin_id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);

    // Add a review
    client.rate_and_review_depin(&user, &depin_id, &4, &String::from_str(&env, "Good service"));

    // Verify review exists
    let reviews = client.get_reviews(&depin_id);
    assert_eq!(reviews.len(), 1);

    // Admin removes all reviews for this DePIN
    client.remove_depin_reviews(&admin, &depin_id);

    // Verify reviews are removed
    let reviews_after = client.get_reviews(&depin_id);
    assert_eq!(reviews_after.len(), 0);
}

#[test]
fn test_admin_can_update_depin_registry() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depin_registry = Address::generate(&env);
    let new_depin_registry = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin, &depin_registry);

    // Admin updates the DePIN registry address
    client.set_depin_registry(&admin, &new_depin_registry);

    // Note: We can't directly test the storage change without reading it,
    // but we can verify the function doesn't panic
}

#[test]
#[should_panic(expected = "Rating must be 1-5")]
fn test_invalid_rating_too_high() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depin_registry = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin, &depin_registry);

    let depin_id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);

    // Try to add invalid rating (too high)
    client.rate_and_review_depin(&user, &depin_id, &6, &String::from_str(&env, "Great service!"));
}

#[test]
#[should_panic(expected = "Rating must be 1-5")]
fn test_invalid_rating_too_low() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depin_registry = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin, &depin_registry);

    let depin_id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);

    // Try to add invalid rating (too low)
    client.rate_and_review_depin(&user, &depin_id, &0, &String::from_str(&env, "Bad service!"));
}

#[test]
#[should_panic(expected = "Review cannot be empty")]
fn test_empty_review_text() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(ReputationContract, ());
    let client = ReputationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let depin_registry = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin, &depin_registry);

    let depin_id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);

    // Try to add review with empty text
    client.rate_and_review_depin(&user, &depin_id, &4, &String::from_str(&env, ""));
}
