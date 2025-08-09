#![no_std]
use soroban_sdk::{contracttype, contract, contractimpl, Env, String, Vec, Address, Map};

#[contracttype]
pub enum DataKey {
    Admin,
    DepinMap,
    Counter,  // Add counter for DePIN IDs
}

#[contract]
pub struct Contract;

// DePIN as tuple for storage compatibility
type DePIN = (soroban_sdk::BytesN<32>, String, String, bool, i32, i32, i32);

impl Contract {
    fn assert_admin(env: &Env, invoker: &Address) {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if invoker != &admin {
            panic!("Only admin can perform this action");
        }
    }
}

#[contractimpl]
impl Contract {
    // Initialize contract and set admin
    pub fn initialize(env: Env, admin: Address) {
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::DepinMap, &Map::<soroban_sdk::BytesN<32>, DePIN>::new(&env));
        env.storage().persistent().set(&DataKey::Counter, &0u32); // Initialize counter
    }

    // Add a new DePIN (admin only)
    pub fn add_depin(env: Env, invoker: Address, name: String, description: String, uptime: i32, reliability: i32, cost: i32) -> soroban_sdk::BytesN<32> {
        Self::assert_admin(&env, &invoker);
        
        // Get and increment counter
        let mut counter: u32 = env.storage().persistent().get(&DataKey::Counter).unwrap();
        counter += 1;
        env.storage().persistent().set(&DataKey::Counter, &counter);
        
        // Create BytesN from counter
        let mut bytes = [0u8; 32];
        bytes[..4].copy_from_slice(&counter.to_be_bytes());
        // Validate input parameters
        assert!(!name.is_empty(), "Name cannot be empty");
        assert!(!description.is_empty(), "Description cannot be empty");
        assert!(uptime >= 0 && uptime <= 100, "Uptime must be between 0 and 100");
        assert!(reliability >= 0 && reliability <= 100, "Reliability must be between 0 and 100");
        assert!(cost >= 0, "Cost must be non-negative");

        let depin_id = soroban_sdk::BytesN::from_array(&env, &bytes);
        let depin: DePIN = (depin_id.clone(), name, description, true, uptime, reliability, cost);
        let mut depin_map: Map<soroban_sdk::BytesN<32>, DePIN> = env.storage().persistent().get(&DataKey::DepinMap).unwrap();
        depin_map.set(depin_id.clone(), depin.clone());
        env.storage().persistent().set(&DataKey::DepinMap, &depin_map);
        depin_id
    }

    // Update DePIN details (admin only)
    pub fn update_depin(env: Env, invoker: Address, depin_id: soroban_sdk::BytesN<32>, name: String, description: String, uptime: i32, reliability: i32, cost: i32) {
        Self::assert_admin(&env, &invoker);

        // Validate input parameters
        assert!(!name.is_empty(), "Name cannot be empty");
        assert!(!description.is_empty(), "Description cannot be empty");
        assert!(uptime >= 0 && uptime <= 100, "Uptime must be between 0 and 100");
        assert!(reliability >= 0 && reliability <= 100, "Reliability must be between 0 and 100");
        assert!(cost >= 0, "Cost must be non-negative");

        let mut depin_map: Map<soroban_sdk::BytesN<32>, DePIN> = env.storage().persistent().get(&DataKey::DepinMap).unwrap();
        if let Some(mut depin) = depin_map.get(depin_id.clone()) {
            depin.1 = name;
            depin.2 = description;
            depin.4 = uptime;
            depin.5 = reliability;
            depin.6 = cost;
            depin_map.set(depin_id, depin);
            env.storage().persistent().set(&DataKey::DepinMap, &depin_map);
        }
    }

    // Remove DePIN (admin only)
    pub fn remove_depin(env: Env, invoker: Address, depin_id: soroban_sdk::BytesN<32>) {
        Self::assert_admin(&env, &invoker);
        let mut depin_map: Map<soroban_sdk::BytesN<32>, DePIN> = env.storage().persistent().get(&DataKey::DepinMap).unwrap();
        // Ensure the DePIN exists before removing
        assert!(depin_map.contains_key(depin_id.clone()), "DePIN not found");
        depin_map.remove(depin_id);
        env.storage().persistent().set(&DataKey::DepinMap, &depin_map);
    }

    // Change DePIN status (admin only)
    pub fn set_depin_status(env: Env, invoker: Address, depin_id: soroban_sdk::BytesN<32>, status: bool) {
        Self::assert_admin(&env, &invoker);
        let mut depin_map: Map<soroban_sdk::BytesN<32>, DePIN> = env.storage().persistent().get(&DataKey::DepinMap).unwrap();
        if let Some(mut depin) = depin_map.get(depin_id.clone()) {
            depin.3 = status;
            depin_map.set(depin_id, depin);
            env.storage().persistent().set(&DataKey::DepinMap, &depin_map);
        }
    }

    // Get DePIN details
    pub fn get_depin(env: Env, depin_id: soroban_sdk::BytesN<32>) -> Option<DePIN> {
        let depin_map: Map<soroban_sdk::BytesN<32>, DePIN> = env.storage().persistent().get(&DataKey::DepinMap).unwrap();
        depin_map.get(depin_id)
    }

    // List all DePINs (returns vector of DePIN IDs)
    pub fn list_depins(env: Env) -> Vec<soroban_sdk::BytesN<32>> {
        let depin_map: Map<soroban_sdk::BytesN<32>, DePIN> = env.storage().persistent().get(&DataKey::DepinMap).unwrap();
        let mut depin_ids = Vec::new(&env);
        
        for i in 0..depin_map.len() {
            if let Some(key) = depin_map.keys().get(i) {
                depin_ids.push_back(key);
            }
        }
        depin_ids
    }

    // Get total count of DePINs
    pub fn get_depin_count(env: Env) -> u32 {
        let depin_map: Map<soroban_sdk::BytesN<32>, DePIN> = env.storage().persistent().get(&DataKey::DepinMap).unwrap();
        depin_map.len()
    }

    // Check if a DePIN exists
    pub fn depin_exists(env: Env, depin_id: soroban_sdk::BytesN<32>) -> bool {
        let depin_map: Map<soroban_sdk::BytesN<32>, DePIN> = env.storage().persistent().get(&DataKey::DepinMap).unwrap();
        depin_map.contains_key(depin_id)
    }
}

#[cfg(test)]
mod test;
