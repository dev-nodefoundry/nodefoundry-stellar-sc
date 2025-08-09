#![no_std]
use soroban_sdk::{contracttype, contract, contractimpl, Env, String, Vec, Address, Map};

#[contracttype]
pub enum DataKey {
    Admin,
    Ratings,
    DepinRegistry, // Store the address of the DePIN registry contract
}

#[contract]
pub struct ReputationContract;

impl ReputationContract {
    fn assert_admin(env: &Env, invoker: &Address) {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if invoker != &admin {
            panic!("Only admin can perform this action");
        }
    }

    fn assert_depin_exists(_env: &Env, _depin_id: soroban_sdk::BytesN<32>) {
        // In a real implementation, you would call the DePIN registry contract
        // to verify the DePIN exists. For now, we'll assume it's validated externally.
        // This is a placeholder for cross-contract call validation
    }
}

#[contractimpl]
impl ReputationContract {
    // Initialize contract and set admin
    pub fn initialize(env: Env, admin: Address, depin_registry_address: Address) {
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Ratings, &Map::<soroban_sdk::BytesN<32>, Vec<(Address, i32, String)>>::new(&env));
        env.storage().persistent().set(&DataKey::DepinRegistry, &depin_registry_address);
    }

    // Update the DePIN registry address (admin only)
    pub fn set_depin_registry(env: Env, invoker: Address, depin_registry_address: Address) {
        Self::assert_admin(&env, &invoker);
        env.storage().persistent().set(&DataKey::DepinRegistry, &depin_registry_address);
    }

    // User: Rate and review a DePIN
    pub fn rate_and_review_depin(env: Env, invoker: Address, depin_id: soroban_sdk::BytesN<32>, rating: i32, review: String) {
        // Validate inputs
        assert!(rating >= 1 && rating <= 5, "Rating must be 1-5");
        assert!(!review.is_empty(), "Review cannot be empty");
        
        // Verify that the DePIN exists (placeholder for cross-contract call)
        Self::assert_depin_exists(&env, depin_id.clone());

        let mut ratings_map: Map<soroban_sdk::BytesN<32>, Vec<(Address, i32, String)>> = env.storage().persistent().get(&DataKey::Ratings).unwrap();
        let reviews = ratings_map.get(depin_id.clone()).unwrap_or(Vec::new(&env));
        let mut filtered = Vec::new(&env);
        
        // Remove any existing review from this user
        for i in 0..reviews.len() {
            let (addr, r, rev) = reviews.get_unchecked(i);
            if addr != invoker {
                filtered.push_back((addr, r, rev));
            }
        }
        
        // Add the new review
        filtered.push_back((invoker, rating, review));
        ratings_map.set(depin_id, filtered);
        env.storage().persistent().set(&DataKey::Ratings, &ratings_map);
    }

    // Get all reviews for a DePIN
    pub fn get_reviews(env: Env, depin_id: soroban_sdk::BytesN<32>) -> Vec<(Address, i32, String)> {
        // Verify that the DePIN exists (placeholder for cross-contract call)
        Self::assert_depin_exists(&env, depin_id.clone());

        let ratings_map: Map<soroban_sdk::BytesN<32>, Vec<(Address, i32, String)>> = env.storage().persistent().get(&DataKey::Ratings).unwrap();
        ratings_map.get(depin_id).unwrap_or(Vec::new(&env))
    }

    // Get average rating for a DePIN
    pub fn get_average_rating(env: Env, depin_id: soroban_sdk::BytesN<32>) -> Option<i32> {
        let reviews = Self::get_reviews(env, depin_id);
        if reviews.is_empty() {
            return None;
        }
        
        let mut total = 0;
        for (_, rating, _) in reviews.iter() {
            total += rating;
        }
        Some(total / reviews.len() as i32)
    }

    // Get the number of reviews for a DePIN
    pub fn get_review_count(env: Env, depin_id: soroban_sdk::BytesN<32>) -> u32 {
        let reviews = Self::get_reviews(env, depin_id);
        reviews.len()
    }

    // Get rating statistics for a DePIN
    pub fn get_rating_stats(env: Env, depin_id: soroban_sdk::BytesN<32>) -> (Option<i32>, u32, i32, i32) {
        let reviews = Self::get_reviews(env, depin_id);
        if reviews.is_empty() {
            return (None, 0, 0, 0);
        }
        
        let mut total = 0;
        let mut min_rating = 5;
        let mut max_rating = 1;
        
        for (_, rating, _) in reviews.iter() {
            total += rating;
            if rating < min_rating {
                min_rating = rating;
            }
            if rating > max_rating {
                max_rating = rating;
            }
        }
        
        let avg_rating = total / reviews.len() as i32;
        (Some(avg_rating), reviews.len(), min_rating, max_rating)
    }

    // Remove all reviews for a DePIN (admin only, for cleanup)
    pub fn remove_depin_reviews(env: Env, invoker: Address, depin_id: soroban_sdk::BytesN<32>) {
        Self::assert_admin(&env, &invoker);
        let mut ratings_map: Map<soroban_sdk::BytesN<32>, Vec<(Address, i32, String)>> = env.storage().persistent().get(&DataKey::Ratings).unwrap();
        ratings_map.remove(depin_id);
        env.storage().persistent().set(&DataKey::Ratings, &ratings_map);
    }
}

#[cfg(test)]
mod test;
