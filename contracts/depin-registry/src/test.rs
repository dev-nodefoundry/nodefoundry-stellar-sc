#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{Env, Address, String, BytesN, testutils::Address as _};

fn u32_from_id(id: &BytesN<32>) -> u32 {
    let mut four = [0u8;4];
    four.copy_from_slice(&id.to_array()[..4]);
    u32::from_be_bytes(four)
}

fn init_registry<'a>(env: &'a Env, admin: &'a Address) -> ContractClient<'a> {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    client.initialize(admin);
    client
}

// Add this function right after init_registry
fn create_depin_registry<'a>(env: &'a Env, admin: &'a Address) -> ContractClient<'a> {
    init_registry(env, admin)
}

#[test]
fn test_depin_registry_happy_path() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let registry = init_registry(&env, &admin);

    // Add first DePIN
    let depin_id1 = registry.add_depin(
        &admin,
        &String::from_str(&env, "NodeX"),
        &String::from_str(&env, "A test node"),
        &99,
        &95,
        &10,
    );
    assert_eq!(u32_from_id(&depin_id1), 1);

    // Update it
    registry.update_depin(
        &admin,
        &depin_id1,
        &String::from_str(&env, "NodeX Updated"),
        &String::from_str(&env, "Updated description"),
        &100,
        &98,
        &12,
    );
    registry.set_depin_status(&admin, &depin_id1, &false);
    let depin1 = registry.get_depin(&depin_id1).unwrap();
    assert_eq!(depin1.1, String::from_str(&env, "NodeX Updated"));
    assert_eq!(depin1.3, false);
    assert_eq!(depin1.4, 100);
    assert_eq!(depin1.5, 98);
    assert_eq!(depin1.6, 12);

    // Remove it
    registry.remove_depin(&admin, &depin_id1);
    assert!(registry.get_depin(&depin_id1).is_none());

    // Add second DePIN
    let depin_id2 = registry.add_depin(
        &admin,
        &String::from_str(&env, "NodeY"),
        &String::from_str(&env, "Another node"),
        &88,
        &90,
        &15,
    );
    assert_eq!(u32_from_id(&depin_id2), 2);

    // Test listing DePINs
    let depin_list = registry.list_depins();
    assert_eq!(depin_list.len(), 1); // Only depin_id2 should exist (depin_id1 was removed)
    assert_eq!(depin_list.get(0).unwrap(), depin_id2);

    // Test depin_exists
    assert!(registry.depin_exists(&depin_id2));
    assert!(!registry.depin_exists(&depin_id1)); // This was removed

    // Test get_depin_count
    assert_eq!(registry.get_depin_count(), 1);
}

#[test]
#[should_panic(expected = "Only admin can perform this action")]
fn test_non_admin_cannot_add_depin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Try to add DePIN as non-admin, should panic
    registry.add_depin(
        &non_admin,
        &String::from_str(&env, "NodeX"),
        &String::from_str(&env, "A test node"),
        &99,
        &95,
        &10,
    );
}

#[test]
#[should_panic(expected = "Uptime must be between 0 and 100")]
fn test_invalid_uptime() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Try to add DePIN with invalid uptime
    registry.add_depin(
        &admin,
        &String::from_str(&env, "NodeX"),
        &String::from_str(&env, "A test node"),
        &101, // Invalid uptime > 100
        &95,
        &10,
    );
}

#[test]
#[should_panic(expected = "DePIN not found")]
fn test_remove_non_existent_depin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Create a random BytesN<32> for non-existent DePIN ID
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    let fake_id = BytesN::from_array(&env, &bytes);

    // Try to remove a non-existent DePIN
    registry.remove_depin(&admin, &fake_id);
}

#[test]
fn test_depin_status_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Add a DePIN
    let depin_id = registry.add_depin(
        &admin,
        &String::from_str(&env, "NodeX"),
        &String::from_str(&env, "A test node"),
        &99,
        &95,
        &10,
    );

    // Check initial status (should be true)
    let depin = registry.get_depin(&depin_id).unwrap();
    assert_eq!(depin.3, true); // status field

    // Deactivate DePIN
    registry.set_depin_status(&admin, &depin_id, &false);
    let depin = registry.get_depin(&depin_id).unwrap();
    assert_eq!(depin.3, false);

    // Reactivate DePIN
    registry.set_depin_status(&admin, &depin_id, &true);
    let depin = registry.get_depin(&depin_id).unwrap();
    assert_eq!(depin.3, true);
}

#[test]
fn test_depin_data_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Add a valid DePIN
    let depin_id = registry.add_depin(
        &admin,
        &String::from_str(&env, "ValidNode"),
        &String::from_str(&env, "A valid test node"),
        &85,
        &92,
        &20,
    );

    let depin = registry.get_depin(&depin_id).unwrap();
    assert_eq!(depin.1, String::from_str(&env, "ValidNode"));
    assert_eq!(depin.2, String::from_str(&env, "A valid test node"));
    assert_eq!(depin.4, 85); // uptime
    assert_eq!(depin.5, 92); // reliability
    assert_eq!(depin.6, 20); // cost
}

#[test]
#[should_panic(expected = "Name cannot be empty")]
fn test_empty_name_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Try to add DePIN with empty name
    registry.add_depin(
        &admin,
        &String::from_str(&env, ""),
        &String::from_str(&env, "A test node"),
        &99,
        &95,
        &10,
    );
}

#[test]
#[should_panic(expected = "Description cannot be empty")]
fn test_empty_description_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Try to add DePIN with empty description
    registry.add_depin(
        &admin,
        &String::from_str(&env, "NodeX"),
        &String::from_str(&env, ""),
        &99,
        &95,
        &10,
    );
}

#[test]
#[should_panic(expected = "Reliability must be between 0 and 100")]
fn test_invalid_reliability() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Try to add DePIN with invalid reliability
    registry.add_depin(
        &admin,
        &String::from_str(&env, "NodeX"),
        &String::from_str(&env, "A test node"),
        &99,
        &105, // Invalid reliability > 100
        &10,
    );
}

#[test]
#[should_panic(expected = "Cost must be non-negative")]
fn test_negative_cost() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registry = create_depin_registry(&env, &admin);

    // Try to add DePIN with negative cost
    registry.add_depin(
        &admin,
        &String::from_str(&env, "NodeX"),
        &String::from_str(&env, "A test node"),
        &99,
        &95,
        &-5, // Invalid negative cost
    );
}
