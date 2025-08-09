#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn create_test_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let user = Address::generate(&env);
    
    (env, admin, usdc_token, user)
}

fn init_contract<'a>(env: &'a Env, admin: &Address, usdc_token: &Address) -> UserProfileContractClient<'a> {
    let contract_id = env.register(UserProfileContract, ());
    let client = UserProfileContractClient::new(env, &contract_id);
    client.initialize(admin, usdc_token);
    client
}

#[test]
fn test_contract_initialization() {
    let (env, admin, usdc_token, _user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    let stats = client.get_platform_stats();
    assert_eq!(stats.total_users, 0);
    assert_eq!(stats.total_deposits, 0);
    assert_eq!(stats.total_withdrawals, 0);
    assert_eq!(stats.active_subscriptions, 0);
}

#[test]
fn test_user_profile_creation() {
    let (env, admin, usdc_token, user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    let referral_code = client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser"),
        &String::from_str(&env, "test@example.com"),
        &None
    );
    
    assert!(!referral_code.is_empty());
    
    let profile = client.get_user_profile(&user).unwrap();
    assert_eq!(profile.username, String::from_str(&env, "testuser"));
    assert_eq!(profile.email, String::from_str(&env, "test@example.com"));
    assert_eq!(profile.subscription_tier, 0);
    assert!(!profile.is_verified);
    assert!(profile.is_active);
    
    let stats = client.get_platform_stats();
    assert_eq!(stats.total_users, 1);
}

#[test]
fn test_wallet_operations() {
    let (env, admin, usdc_token, user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    // Create user profile first
    client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser"),
        &String::from_str(&env, "test@example.com"),
        &None
    );
    
    // Test deposit
    let deposit_amount = 1000_000_000i128; // 1000 USDC
    client.deposit_funds(&user, &usdc_token, &deposit_amount);
    
    let balance = client.get_user_balance(&user, &usdc_token);
    assert_eq!(balance, deposit_amount);
    
    // Test withdrawal
    let withdraw_amount = 100_000_000i128; // 100 USDC
    client.withdraw_funds(&user, &usdc_token, &withdraw_amount);
    
    let new_balance = client.get_user_balance(&user, &usdc_token);
    assert_eq!(new_balance, deposit_amount - withdraw_amount);
}

#[test]
fn test_subscription_upgrade() {
    let (env, admin, usdc_token, user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    // Create user and deposit funds
    client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser"),
        &String::from_str(&env, "test@example.com"),
        &None
    );
    
    client.deposit_funds(&user, &usdc_token, &100_000_000i128); // 100 USDC
    
    // Upgrade to premium (tier 1)
    client.upgrade_subscription(&user, &1, &usdc_token);
    
    let profile = client.get_user_profile(&user).unwrap();
    assert_eq!(profile.subscription_tier, 1);
    assert_eq!(profile.total_spent, 10_000_000i128); // 10 USDC
    assert!(profile.loyalty_points > 0);
    
    let balance = client.get_user_balance(&user, &usdc_token);
    assert_eq!(balance, 90_000_000i128); // 90 USDC remaining
}

#[test]
fn test_referral_system() {
    let (env, admin, usdc_token, _) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    let referrer = Address::generate(&env);
    let referee = Address::generate(&env);
    
    // Create referrer profile
    let referral_code = client.create_user_profile(
        &referrer,
        &String::from_str(&env, "referrer"),
        &String::from_str(&env, "referrer@example.com"),
        &None
    );
    
    // Create referee profile with referral code
    client.create_user_profile(
        &referee,
        &String::from_str(&env, "referee"),
        &String::from_str(&env, "referee@example.com"),
        &Some(referral_code)
    );
    
    let referee_profile = client.get_user_profile(&referee).unwrap();
    assert_eq!(referee_profile.referred_by, Some(referrer.clone()));
}

#[test]
#[should_panic(expected = "User profile already exists")]
fn test_duplicate_user_creation() {
    let (env, admin, usdc_token, user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser"),
        &String::from_str(&env, "test@example.com"),
        &None
    );
    
    // Try to create the same user again
    client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser2"),
        &String::from_str(&env, "test2@example.com"),
        &None
    );
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_insufficient_balance_withdrawal() {
    let (env, admin, usdc_token, user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser"),
        &String::from_str(&env, "test@example.com"),
        &None
    );
    
    // Try to withdraw without depositing
    client.withdraw_funds(&user, &usdc_token, &100_000_000i128);
}

#[test]
fn test_admin_functions() {
    let (env, admin, usdc_token, user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    // Create user profile
    client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser"),
        &String::from_str(&env, "test@example.com"),
        &None
    );
    
    // Test user verification (admin only)
    client.verify_user(&admin, &user);
    
    let profile = client.get_user_profile(&user).unwrap();
    assert!(profile.is_verified);
    
    // Test token whitelisting
    let new_token = Address::generate(&env);
    client.whitelist_token(&admin, &new_token);
    
    // Test depositing with newly whitelisted token
    client.deposit_funds(&user, &new_token, &50_000_000i128);
    let balance = client.get_user_balance(&user, &new_token);
    assert_eq!(balance, 50_000_000i128);
}

#[test]
fn test_balance_utility_functions() {
    let (env, admin, usdc_token, user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    // Create user and deposit funds
    client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser"),
        &String::from_str(&env, "test@example.com"),
        &None
    );
    
    client.deposit_funds(&user, &usdc_token, &100_000_000i128);
    
    // Test user exists
    assert!(client.user_exists(&user));
    
    // Test has sufficient balance
    assert!(client.has_sufficient_balance(&user, &usdc_token, &50_000_000i128));
    assert!(!client.has_sufficient_balance(&user, &usdc_token, &150_000_000i128));
    
    // Test deduct balance
    let success = client.deduct_balance(&user, &usdc_token, &30_000_000i128);
    assert!(success);
    
    let balance = client.get_user_balance(&user, &usdc_token);
    assert_eq!(balance, 70_000_000i128);
    
    // Test refund balance
    client.refund_balance(&user, &usdc_token, &10_000_000i128);
    let new_balance = client.get_user_balance(&user, &usdc_token);
    assert_eq!(new_balance, 80_000_000i128);
    
    // Check loyalty points were added during deduct
    let profile = client.get_user_profile(&user).unwrap();
    assert!(profile.loyalty_points > 0);
    assert_eq!(profile.total_spent, 30_000_000i128);
}

#[test] 
fn test_user_profile_updates() {
    let (env, admin, usdc_token, user) = create_test_env();
    let client = init_contract(&env, &admin, &usdc_token);
    
    // Create user profile
    client.create_user_profile(
        &user,
        &String::from_str(&env, "testuser"),
        &String::from_str(&env, "test@example.com"),
        &None
    );
    
    // Update username only
    client.update_user_profile(
        &user,
        &Some(String::from_str(&env, "newusername")),
        &None
    );
    
    let profile = client.get_user_profile(&user).unwrap();
    assert_eq!(profile.username, String::from_str(&env, "newusername"));
    assert_eq!(profile.email, String::from_str(&env, "test@example.com")); // unchanged
    
    // Update email only
    client.update_user_profile(
        &user,
        &None,
        &Some(String::from_str(&env, "newemail@example.com"))
    );
    
    let updated_profile = client.get_user_profile(&user).unwrap();
    assert_eq!(updated_profile.username, String::from_str(&env, "newusername")); // unchanged
    assert_eq!(updated_profile.email, String::from_str(&env, "newemail@example.com"));
}
