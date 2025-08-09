#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as AddressTestUtils},
    Address, Env, String
};

fn init_order_contract<'a>(env: &'a Env, admin: &Address) -> OrderContractClient<'a> {
    let contract_id = env.register(OrderContract, ());
    let client = OrderContractClient::new(env, &contract_id);
    client.initialize(admin);
    client
}

#[test]
fn test_order_contract_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let order_client = init_order_contract(&env, &admin);

    assert_eq!(order_client.get_order_count(), 0);
    assert_eq!(order_client.get_total_escrowed(), 0);
}

#[test]
fn test_contract_setup() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let user_profile_contract = Address::generate(&env);
    let depin_registry_contract = Address::generate(&env);
    
    let order_client = init_order_contract(&env, &admin);

    // Set contract addresses
    order_client.set_user_profile_contract(&admin, &user_profile_contract);
    order_client.set_depin_registry_contract(&admin, &depin_registry_contract);
    order_client.set_treasury_wallet(&admin, &treasury);

    assert_eq!(order_client.get_treasury_wallet(), Some(treasury));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_create_order_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    let order_client = init_order_contract(&env, &admin);

    // Try to create order with zero duration
    order_client.create_order(
        &user,
        &BytesN::from_array(&env, &[1u8; 32]),
        &String::from_str(&env, "compute"),
        &0, // Invalid duration
        &10,
        &String::from_str(&env, "ethereum"),
        &String::from_str(&env, "{}")
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_create_order_invalid_price() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    let order_client = init_order_contract(&env, &admin);

    // Try to create order with zero price
    order_client.create_order(
        &user,
        &BytesN::from_array(&env, &[1u8; 32]),
        &String::from_str(&env, "compute"),
        &24,
        &0, // Invalid price
        &String::from_str(&env, "ethereum"),
        &String::from_str(&env, "{}")
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_create_order_no_registry_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    let order_client = init_order_contract(&env, &admin);

    // Try to create order without setting registry contract
    order_client.create_order(
        &user,
        &BytesN::from_array(&env, &[1u8; 32]),
        &String::from_str(&env, "compute"),
        &24,
        &10,
        &String::from_str(&env, "ethereum"),
        &String::from_str(&env, "{}")
    );
}

#[test]
fn test_order_status_updates() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    
    let order_client = init_order_contract(&env, &admin);

    // Create a fake order ID for testing status updates
    let order_id = BytesN::from_array(&env, &[1u8; 32]);
    
    // Create order manually for testing using contract context
    let order = Order {
        order_id: order_id.clone(),
        user: Address::generate(&env),
        depin_id: BytesN::from_array(&env, &[2u8; 32]),
        service_type: String::from_str(&env, "compute"),
        duration_hours: 24,
        price_per_hour: 10,
        total_amount: 240,
        status: OrderStatus::Pending,
        created_at: env.ledger().timestamp(),
        external_tx_id: None,
        deployment_chain: String::from_str(&env, "ethereum"),
        service_params: String::from_str(&env, "{}"),
        escrowed_amount: 240,
    };

    // Store order directly using contract context
    env.as_contract(&order_client.address, || {
        env.storage().persistent().set(&DataKey::Order(order_id.clone()), &order);
    });

    // Update to Active
    order_client.update_order_status(
        &admin,
        &order_id,
        &OrderStatus::Active,
        &None
    );
    
    let updated_order = order_client.get_order(&order_id);
    assert_eq!(updated_order.status, OrderStatus::Active);

    // Update to Deployed with tx ID
    order_client.update_order_status(
        &admin,
        &order_id,
        &OrderStatus::Deployed,
        &Some(String::from_str(&env, "0x123abc"))
    );
    
    let updated_order = order_client.get_order(&order_id);
    assert_eq!(updated_order.status, OrderStatus::Deployed);
    assert_eq!(updated_order.external_tx_id, Some(String::from_str(&env, "0x123abc")));
}

#[test]
fn test_complete_order() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    
    let order_client = init_order_contract(&env, &admin);

    // Create a fake order in deployed status
    let order_id = BytesN::from_array(&env, &[1u8; 32]);
    let order = Order {
        order_id: order_id.clone(),
        user: Address::generate(&env),
        depin_id: BytesN::from_array(&env, &[2u8; 32]),
        service_type: String::from_str(&env, "compute"),
        duration_hours: 24,
        price_per_hour: 10,
        total_amount: 240,
        status: OrderStatus::Deployed,
        created_at: env.ledger().timestamp(),
        external_tx_id: Some(String::from_str(&env, "0x123abc")),
        deployment_chain: String::from_str(&env, "ethereum"),
        service_params: String::from_str(&env, "{}"),
        escrowed_amount: 240,
    };

    // Store order and escrow amount using contract context
    env.as_contract(&order_client.address, || {
        env.storage().persistent().set(&DataKey::Order(order_id.clone()), &order);
        env.storage().persistent().set(&DataKey::TotalEscrowed, &240i128);
    });

    // Complete order
    order_client.complete_order(&admin, &order_id);

    let completed_order = order_client.get_order(&order_id);
    assert_eq!(completed_order.status, OrderStatus::Completed);
    assert_eq!(completed_order.escrowed_amount, 0);
    assert_eq!(order_client.get_total_escrowed(), 0);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_complete_order_invalid_status() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    
    let order_client = init_order_contract(&env, &admin);

    // Create a fake order in pending status (not deployed)
    let order_id = BytesN::from_array(&env, &[1u8; 32]);
    let order = Order {
        order_id: order_id.clone(),
        user: Address::generate(&env),
        depin_id: BytesN::from_array(&env, &[2u8; 32]),
        service_type: String::from_str(&env, "compute"),
        duration_hours: 24,
        price_per_hour: 10,
        total_amount: 240,
        status: OrderStatus::Pending, // Invalid for completion
        created_at: env.ledger().timestamp(),
        external_tx_id: None,
        deployment_chain: String::from_str(&env, "ethereum"),
        service_params: String::from_str(&env, "{}"),
        escrowed_amount: 240,
    };

    // Store order using contract context
    env.as_contract(&order_client.address, || {
        env.storage().persistent().set(&DataKey::Order(order_id.clone()), &order);
    });

    // Try to complete order with invalid status
    order_client.complete_order(&admin, &order_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_get_order_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let order_client = init_order_contract(&env, &admin);

    let fake_order_id = BytesN::from_array(&env, &[1u8; 32]);
    order_client.get_order(&fake_order_id);
}

#[test]
fn test_list_orders_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let order_client = init_order_contract(&env, &admin);

    let user_orders = order_client.list_user_orders(&user);
    assert_eq!(user_orders.len(), 0);

    let depin_id = BytesN::from_array(&env, &[1u8; 32]);
    let depin_orders = order_client.list_depin_orders(&depin_id);
    assert_eq!(depin_orders.len(), 0);
}
