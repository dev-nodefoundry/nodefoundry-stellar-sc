#![no_std]
use soroban_sdk::{contracttype, contract, contractimpl, Env, String, Vec, Address, Map};

#[contracttype]
pub enum DataKey {
    Admin,
    UserProfiles,
    UserBalances,
    PlatformStats,
    WhitelistedTokens,
    ReferralSystem,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub user_address: Address,
    pub username: String,
    pub email: String,
    pub created_at: u64,
    pub is_active: bool,
    pub is_verified: bool,
    pub referral_code: String,
    pub referred_by: Option<Address>,
    pub total_spent: i128,
    pub loyalty_points: u32,
    pub subscription_tier: u32, // 0: Basic, 1: Premium, 2: Enterprise
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformStats {
    pub total_users: u32,
    pub total_deposits: i128,
    pub total_withdrawals: i128,
    pub active_subscriptions: u32,
}

#[contract]
pub struct UserProfileContract;

impl UserProfileContract {
    fn assert_admin(env: &Env, invoker: &Address) {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if invoker != &admin {
            panic!("Only admin can perform this action");
        }
    }

    fn assert_user_exists(env: &Env, user_address: &Address) {
        let user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        assert!(user_profiles.contains_key(user_address.clone()), "User profile not found");
    }

    fn generate_referral_code(env: &Env) -> String {
        // Simple referral code generation - in production, this would be more sophisticated
        String::from_str(env, "NF123456")
    }

    fn calculate_loyalty_points(amount: i128) -> u32 {
        // 1 point per 1 USDC spent (assuming 6 decimal places)
        (amount / 1_000_000) as u32
    }
}

#[contractimpl]
impl UserProfileContract {
    // Initialize contract
    pub fn initialize(env: Env, admin: Address, usdc_token: Address) {
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::UserProfiles, &Map::<Address, UserProfile>::new(&env));
        env.storage().persistent().set(&DataKey::UserBalances, &Map::<(Address, Address), i128>::new(&env));
        env.storage().persistent().set(&DataKey::WhitelistedTokens, &Map::<Address, bool>::new(&env));
        env.storage().persistent().set(&DataKey::ReferralSystem, &Map::<String, Address>::new(&env));
        
        // Initialize platform stats
        let stats = PlatformStats {
            total_users: 0,
            total_deposits: 0,
            total_withdrawals: 0,
            active_subscriptions: 0,
        };
        env.storage().persistent().set(&DataKey::PlatformStats, &stats);

        // Whitelist USDC by default
        let mut whitelisted_tokens: Map<Address, bool> = env.storage().persistent().get(&DataKey::WhitelistedTokens).unwrap();
        whitelisted_tokens.set(usdc_token, true);
        env.storage().persistent().set(&DataKey::WhitelistedTokens, &whitelisted_tokens);
    }

    // User Management
    pub fn create_user_profile(
        env: Env, 
        user_address: Address, 
        username: String, 
        email: String,
        referral_code: Option<String>
    ) -> String {
        let mut user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        
        // Check if user already exists
        assert!(!user_profiles.contains_key(user_address.clone()), "User profile already exists");
        
        // Validate inputs
        assert!(!username.is_empty(), "Username cannot be empty");
        assert!(!email.is_empty(), "Email cannot be empty");

        let current_time = env.ledger().timestamp();
        let user_referral_code = Self::generate_referral_code(&env);
        
        let mut referred_by = None;
        if let Some(ref_code) = referral_code {
            let referral_map: Map<String, Address> = env.storage().persistent().get(&DataKey::ReferralSystem).unwrap();
            referred_by = referral_map.get(ref_code);
        }

        let profile = UserProfile {
            user_address: user_address.clone(),
            username,
            email,
            created_at: current_time,
            is_active: true,
            is_verified: false,
            referral_code: user_referral_code.clone(),
            referred_by,
            total_spent: 0,
            loyalty_points: 0,
            subscription_tier: 0,
        };

        user_profiles.set(user_address.clone(), profile);
        env.storage().persistent().set(&DataKey::UserProfiles, &user_profiles);

        // Store referral mapping
        let mut referral_map: Map<String, Address> = env.storage().persistent().get(&DataKey::ReferralSystem).unwrap();
        referral_map.set(user_referral_code.clone(), user_address);
        env.storage().persistent().set(&DataKey::ReferralSystem, &referral_map);

        // Update platform stats
        let mut stats: PlatformStats = env.storage().persistent().get(&DataKey::PlatformStats).unwrap();
        stats.total_users += 1;
        env.storage().persistent().set(&DataKey::PlatformStats, &stats);

        user_referral_code
    }

    pub fn update_user_profile(
        env: Env,
        user_address: Address,
        username: Option<String>,
        email: Option<String>
    ) {
        Self::assert_user_exists(&env, &user_address);
        
        let mut user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        let mut profile = user_profiles.get(user_address.clone()).unwrap();

        if let Some(new_username) = username {
            assert!(!new_username.is_empty(), "Username cannot be empty");
            profile.username = new_username;
        }

        if let Some(new_email) = email {
            assert!(!new_email.is_empty(), "Email cannot be empty");
            profile.email = new_email;
        }

        user_profiles.set(user_address, profile);
        env.storage().persistent().set(&DataKey::UserProfiles, &user_profiles);
    }

    pub fn verify_user(env: Env, invoker: Address, user_address: Address) {
        Self::assert_admin(&env, &invoker);
        Self::assert_user_exists(&env, &user_address);

        let mut user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        let mut profile = user_profiles.get(user_address.clone()).unwrap();
        profile.is_verified = true;

        user_profiles.set(user_address, profile);
        env.storage().persistent().set(&DataKey::UserProfiles, &user_profiles);
    }

    // Wallet Management
    pub fn deposit_funds(
        env: Env,
        user_address: Address,
        token_address: Address,
        amount: i128
    ) {
        Self::assert_user_exists(&env, &user_address);
        
        // Check if token is whitelisted
        let whitelisted_tokens: Map<Address, bool> = env.storage().persistent().get(&DataKey::WhitelistedTokens).unwrap();
        assert!(whitelisted_tokens.get(token_address.clone()).unwrap_or(false), "Token not whitelisted");
        
        assert!(amount > 0, "Deposit amount must be positive");

        let mut user_balances: Map<(Address, Address), i128> = env.storage().persistent().get(&DataKey::UserBalances).unwrap();
        let balance_key = (user_address.clone(), token_address.clone());
        let current_balance = user_balances.get(balance_key.clone()).unwrap_or(0);
        
        user_balances.set(balance_key, current_balance + amount);
        env.storage().persistent().set(&DataKey::UserBalances, &user_balances);

        // Update platform stats
        let mut stats: PlatformStats = env.storage().persistent().get(&DataKey::PlatformStats).unwrap();
        stats.total_deposits += amount;
        env.storage().persistent().set(&DataKey::PlatformStats, &stats);
    }

    pub fn withdraw_funds(
        env: Env,
        user_address: Address,
        token_address: Address,
        amount: i128
    ) {
        Self::assert_user_exists(&env, &user_address);
        
        let mut user_balances: Map<(Address, Address), i128> = env.storage().persistent().get(&DataKey::UserBalances).unwrap();
        let balance_key = (user_address.clone(), token_address.clone());
        let current_balance = user_balances.get(balance_key.clone()).unwrap_or(0);
        
        assert!(current_balance >= amount, "Insufficient balance");
        assert!(amount > 0, "Withdrawal amount must be positive");

        user_balances.set(balance_key, current_balance - amount);
        env.storage().persistent().set(&DataKey::UserBalances, &user_balances);

        // Update platform stats
        let mut stats: PlatformStats = env.storage().persistent().get(&DataKey::PlatformStats).unwrap();
        stats.total_withdrawals += amount;
        env.storage().persistent().set(&DataKey::PlatformStats, &stats);
    }

    pub fn get_user_balance(env: Env, user_address: Address, token_address: Address) -> i128 {
        let user_balances: Map<(Address, Address), i128> = env.storage().persistent().get(&DataKey::UserBalances).unwrap();
        let balance_key = (user_address, token_address);
        user_balances.get(balance_key).unwrap_or(0)
    }

    // Get user profile
    pub fn get_user_profile(env: Env, user_address: Address) -> Option<UserProfile> {
        let user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        user_profiles.get(user_address)
    }

    // Admin functions
    pub fn whitelist_token(env: Env, invoker: Address, token_address: Address) {
        Self::assert_admin(&env, &invoker);
        
        let mut whitelisted_tokens: Map<Address, bool> = env.storage().persistent().get(&DataKey::WhitelistedTokens).unwrap();
        whitelisted_tokens.set(token_address, true);
        env.storage().persistent().set(&DataKey::WhitelistedTokens, &whitelisted_tokens);
    }

    pub fn remove_token_whitelist(env: Env, invoker: Address, token_address: Address) {
        Self::assert_admin(&env, &invoker);
        
        let mut whitelisted_tokens: Map<Address, bool> = env.storage().persistent().get(&DataKey::WhitelistedTokens).unwrap();
        whitelisted_tokens.set(token_address, false);
        env.storage().persistent().set(&DataKey::WhitelistedTokens, &whitelisted_tokens);
    }

    pub fn get_platform_stats(env: Env) -> PlatformStats {
        env.storage().persistent().get(&DataKey::PlatformStats).unwrap()
    }

    // Utility functions for order contract integration
    pub fn deduct_balance(
        env: Env,
        user_address: Address,
        token_address: Address,
        amount: i128
    ) -> bool {
        Self::assert_user_exists(&env, &user_address);
        
        let mut user_balances: Map<(Address, Address), i128> = env.storage().persistent().get(&DataKey::UserBalances).unwrap();
        let balance_key = (user_address.clone(), token_address.clone());
        let current_balance = user_balances.get(balance_key.clone()).unwrap_or(0);
        
        if current_balance >= amount {
            user_balances.set(balance_key, current_balance - amount);
            env.storage().persistent().set(&DataKey::UserBalances, &user_balances);
            
            // Update user profile spending and loyalty points
            let mut user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
            let mut profile = user_profiles.get(user_address.clone()).unwrap();
            profile.total_spent += amount;
            profile.loyalty_points += Self::calculate_loyalty_points(amount);
            user_profiles.set(user_address.clone(), profile);
            env.storage().persistent().set(&DataKey::UserProfiles, &user_profiles);
            
            true
        } else {
            false
        }
    }

    pub fn refund_balance(
        env: Env,
        user_address: Address,
        token_address: Address,
        amount: i128
    ) {
        Self::assert_user_exists(&env, &user_address);
        
        let mut user_balances: Map<(Address, Address), i128> = env.storage().persistent().get(&DataKey::UserBalances).unwrap();
        let balance_key = (user_address.clone(), token_address.clone());
        let current_balance = user_balances.get(balance_key.clone()).unwrap_or(0);
        
        user_balances.set(balance_key, current_balance + amount);
        env.storage().persistent().set(&DataKey::UserBalances, &user_balances);
    }

    // Subscription Management
    pub fn upgrade_subscription(
        env: Env,
        user_address: Address,
        tier: u32,
        token_address: Address
    ) {
        Self::assert_user_exists(&env, &user_address);
        assert!(tier <= 2, "Invalid subscription tier");
        
        let subscription_costs = [0i128, 10_000_000, 50_000_000]; // Basic: Free, Premium: 10 USDC, Enterprise: 50 USDC
        let cost = subscription_costs[tier as usize];
        
        if cost > 0 {
            let balance = Self::get_user_balance(env.clone(), user_address.clone(), token_address.clone());
            assert!(balance >= cost, "Insufficient balance for subscription upgrade");
            
            // Deduct subscription cost
            let mut user_balances: Map<(Address, Address), i128> = env.storage().persistent().get(&DataKey::UserBalances).unwrap();
            let balance_key = (user_address.clone(), token_address.clone());
            let current_balance = user_balances.get(balance_key.clone()).unwrap_or(0);
            
            user_balances.set(balance_key, current_balance - cost);
            env.storage().persistent().set(&DataKey::UserBalances, &user_balances);
        }

        // Update user profile
        let mut user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        let mut profile = user_profiles.get(user_address.clone()).unwrap();
        let old_tier = profile.subscription_tier;
        profile.subscription_tier = tier;
        
        if cost > 0 {
            profile.total_spent += cost;
            profile.loyalty_points += Self::calculate_loyalty_points(cost);
        }
        
        user_profiles.set(user_address.clone(), profile);
        env.storage().persistent().set(&DataKey::UserProfiles, &user_profiles);

        // Update platform stats
        let mut stats: PlatformStats = env.storage().persistent().get(&DataKey::PlatformStats).unwrap();
        if old_tier == 0 && tier > 0 {
            stats.active_subscriptions += 1;
        } else if old_tier > 0 && tier == 0 {
            stats.active_subscriptions -= 1;
        }
        env.storage().persistent().set(&DataKey::PlatformStats, &stats);
    }

    // Check if user exists (for order contract)
    pub fn user_exists(env: Env, user_address: Address) -> bool {
        let user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        user_profiles.contains_key(user_address)
    }

    // Check if user has sufficient balance (for order contract)
    pub fn has_sufficient_balance(env: Env, user_address: Address, token_address: Address, amount: i128) -> bool {
        let balance = Self::get_user_balance(env, user_address, token_address);
        balance >= amount
    }

    // Get all users (admin only)
    pub fn get_all_users(env: Env, invoker: Address) -> Vec<UserProfile> {
        Self::assert_admin(&env, &invoker);
        
        let user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        let mut users = Vec::new(&env);
        
        for i in 0..user_profiles.len() {
            if let Some(profile) = user_profiles.values().get(i) {
                users.push_back(profile);
            }
        }
        
        users
    }

    // Get user count
    pub fn get_user_count(env: Env) -> u32 {
        let user_profiles: Map<Address, UserProfile> = env.storage().persistent().get(&DataKey::UserProfiles).unwrap();
        user_profiles.len()
    }
}

#[cfg(test)]
mod test;
