#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, BytesN, Env, String, Vec, IntoVal,
    panic_with_error
};

#[contract]
pub struct OrderContract;

#[contracttype]
pub struct Order {
    pub order_id: BytesN<32>,
    pub user: Address,
    pub depin_id: BytesN<32>,
    pub service_type: String,
    pub duration_hours: u64,
    pub price_per_hour: i128,
    pub total_amount: i128,
    pub status: OrderStatus,
    pub created_at: u64,
    pub external_tx_id: Option<String>,
    pub deployment_chain: String,
    pub service_params: String,
    pub escrowed_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrderStatus {
    Pending,        // Created, funds held in escrow
    Active,         // Deployment started on external chain
    Deployed,       // Successfully deployed (has external_tx_id)
    Completed,      // Service completed, payment released to treasury
    Cancelled,      // Cancelled before deployment
    Failed,         // Deployment failed, funds refunded
}

#[contracttype]
pub enum DataKey {
    Order(BytesN<32>),          // order_id -> Order
    OrderCounter,               // u32 counter for generating IDs
    UserProfileContract,        // Address of user profile contract
    DepinRegistryContract,      // Address of DePIN registry contract
    TreasuryWallet,            // NodeFoundry treasury address
    TotalEscrowed,             // Total amount in escrow across all orders
    Admin,                     // Admin address
    UserOrders(Address),       // user -> Vec<BytesN<32>> (order IDs)
    DepinOrders(BytesN<32>),   // depin_id -> Vec<BytesN<32>> (order IDs)
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAdmin = 3,
    OrderNotFound = 4,
    InvalidDepin = 5,
    InsufficientBalance = 6,
    InvalidStatus = 7,
    Unauthorized = 8,
    InvalidAmount = 9,
    ContractNotSet = 10,
}

#[contractimpl]
impl OrderContract {
    /// Initialize the contract with admin address
    pub fn initialize(env: Env, admin: Address) -> bool {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::OrderCounter, &0u32);
        env.storage().persistent().set(&DataKey::TotalEscrowed, &0i128);
        
        true
    }

    /// Set user profile contract address (admin only)
    pub fn set_user_profile_contract(env: Env, admin: Address, contract_address: Address) -> bool {
        Self::assert_admin(&env, &admin);
        env.storage().persistent().set(&DataKey::UserProfileContract, &contract_address);
        true
    }

    /// Set DePIN registry contract address (admin only)
    pub fn set_depin_registry_contract(env: Env, admin: Address, contract_address: Address) -> bool {
        Self::assert_admin(&env, &admin);
        env.storage().persistent().set(&DataKey::DepinRegistryContract, &contract_address);
        true
    }

    /// Set treasury wallet address (admin only)
    pub fn set_treasury_wallet(env: Env, admin: Address, treasury_address: Address) -> bool {
        Self::assert_admin(&env, &admin);
        env.storage().persistent().set(&DataKey::TreasuryWallet, &treasury_address);
        true
    }

    /// Create a new order with escrow mechanism
    pub fn create_order(
        env: Env,
        user: Address,
        depin_id: BytesN<32>,
        service_type: String,
        duration_hours: u64,
        price_per_hour: i128,
        deployment_chain: String,
        service_params: String,
    ) -> BytesN<32> {
        // Ensure user is authenticated (they signed the transaction)
        user.require_auth();
        
        // Validate inputs
        if duration_hours == 0 || price_per_hour <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }

        // Check if DePIN exists in registry
        let registry_contract: Address = env.storage().persistent()
            .get(&DataKey::DepinRegistryContract)
            .unwrap_or_else(|| panic_with_error!(&env, Error::ContractNotSet));

        let depin_exists: bool = env.invoke_contract(
            &registry_contract,
            &soroban_sdk::symbol_short!("exists"),
            soroban_sdk::vec![&env, depin_id.into_val(&env)]
        );

        if !depin_exists {
            panic_with_error!(&env, Error::InvalidDepin);
        }

        // Calculate total amount
        let total_amount = (duration_hours as i128) * price_per_hour;

        // Check user balance and deduct from user profile
        let profile_contract: Address = env.storage().persistent()
            .get(&DataKey::UserProfileContract)
            .unwrap_or_else(|| panic_with_error!(&env, Error::ContractNotSet));

        let has_sufficient_balance: bool = env.invoke_contract(
            &profile_contract,
            &soroban_sdk::symbol_short!("has_suff"),
            soroban_sdk::vec![&env, user.into_val(&env), total_amount.into_val(&env)]
        );

        if !has_sufficient_balance {
            panic_with_error!(&env, Error::InsufficientBalance);
        }

        // Deduct balance from user
        let _deduct_result: bool = env.invoke_contract(
            &profile_contract,
            &soroban_sdk::symbol_short!("deduct"),
            soroban_sdk::vec![&env, user.into_val(&env), total_amount.into_val(&env)]
        );

        // Generate unique order ID
        let order_id = Self::generate_order_id(&env);

        // Create order
        let order = Order {
            order_id: order_id.clone(),
            user: user.clone(),
            depin_id: depin_id.clone(),
            service_type,
            duration_hours,
            price_per_hour,
            total_amount,
            status: OrderStatus::Pending,
            created_at: env.ledger().timestamp(),
            external_tx_id: None,
            deployment_chain,
            service_params,
            escrowed_amount: total_amount,
        };

        // Store order
        env.storage().persistent().set(&DataKey::Order(order_id.clone()), &order);

        // Update total escrowed amount
        let current_escrowed: i128 = env.storage().persistent()
            .get(&DataKey::TotalEscrowed)
            .unwrap_or(0);
        env.storage().persistent().set(&DataKey::TotalEscrowed, &(current_escrowed + total_amount));

        // Add to user's order list
        Self::add_user_order(&env, &user, &order_id);

        // Add to DePIN's order list
        Self::add_depin_order(&env, &depin_id, &order_id);

        order_id
    }

    /// Update order status (admin only)
    pub fn update_order_status(
        env: Env,
        admin: Address,
        order_id: BytesN<32>,
        new_status: OrderStatus,
        external_tx_id: Option<String>,
    ) -> bool {
        Self::assert_admin(&env, &admin);

        let mut order: Order = env.storage().persistent()
            .get(&DataKey::Order(order_id.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, Error::OrderNotFound));

        order.status = new_status;
        if external_tx_id.is_some() {
            order.external_tx_id = external_tx_id;
        }

        env.storage().persistent().set(&DataKey::Order(order_id), &order);
        true
    }

    /// Complete order and transfer funds to treasury
    pub fn complete_order(env: Env, admin: Address, order_id: BytesN<32>) -> bool {
        Self::assert_admin(&env, &admin);

        let mut order: Order = env.storage().persistent()
            .get(&DataKey::Order(order_id.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, Error::OrderNotFound));

        if order.status != OrderStatus::Deployed {
            panic_with_error!(&env, Error::InvalidStatus);
        }

        // Update order status
        order.status = OrderStatus::Completed;
        let escrowed_amount = order.escrowed_amount;
        order.escrowed_amount = 0;

        env.storage().persistent().set(&DataKey::Order(order_id), &order);

        // Update total escrowed amount
        let current_escrowed: i128 = env.storage().persistent()
            .get(&DataKey::TotalEscrowed)
            .unwrap_or(0);
        env.storage().persistent().set(&DataKey::TotalEscrowed, &(current_escrowed - escrowed_amount));

        // Funds are now considered transferred to treasury
        // (In a real implementation, you might want to track treasury balance)

        true
    }

    /// Refund order (admin only)
    pub fn refund_order(env: Env, admin: Address, order_id: BytesN<32>) -> bool {
        Self::assert_admin(&env, &admin);

        let mut order: Order = env.storage().persistent()
            .get(&DataKey::Order(order_id.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, Error::OrderNotFound));

        if order.status == OrderStatus::Completed {
            panic_with_error!(&env, Error::InvalidStatus);
        }

        let escrowed_amount = order.escrowed_amount;
        if escrowed_amount > 0 {
            // Refund to user profile
            let profile_contract: Address = env.storage().persistent()
                .get(&DataKey::UserProfileContract)
                .unwrap_or_else(|| panic_with_error!(&env, Error::ContractNotSet));

            let _refund_result: bool = env.invoke_contract(
                &profile_contract,
                &soroban_sdk::symbol_short!("refund"),
                soroban_sdk::vec![&env, order.user.into_val(&env), escrowed_amount.into_val(&env)]
            );

            // Update total escrowed amount
            let current_escrowed: i128 = env.storage().persistent()
                .get(&DataKey::TotalEscrowed)
                .unwrap_or(0);
            env.storage().persistent().set(&DataKey::TotalEscrowed, &(current_escrowed - escrowed_amount));

            order.escrowed_amount = 0;
        }

        order.status = match order.status {
            OrderStatus::Pending => OrderStatus::Cancelled,
            _ => OrderStatus::Failed,
        };

        env.storage().persistent().set(&DataKey::Order(order_id), &order);
        true
    }

    /// Cancel order (user only, before deployment)
    pub fn cancel_order(env: Env, user: Address, order_id: BytesN<32>) -> bool {
        user.require_auth();

        let order: Order = env.storage().persistent()
            .get(&DataKey::Order(order_id.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, Error::OrderNotFound));

        if order.user != user {
            panic_with_error!(&env, Error::Unauthorized);
        }

        if order.status != OrderStatus::Pending {
            panic_with_error!(&env, Error::InvalidStatus);
        }

        // Get admin for refund process
        let admin: Address = env.storage().persistent()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));

        Self::refund_order(env, admin, order_id)
    }

    /// Get order details
    pub fn get_order(env: Env, order_id: BytesN<32>) -> Order {
        env.storage().persistent()
            .get(&DataKey::Order(order_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::OrderNotFound))
    }

    /// Get all orders for a user
    pub fn list_user_orders(env: Env, user: Address) -> Vec<BytesN<32>> {
        env.storage().persistent()
            .get(&DataKey::UserOrders(user))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get all orders for a DePIN
    pub fn list_depin_orders(env: Env, depin_id: BytesN<32>) -> Vec<BytesN<32>> {
        env.storage().persistent()
            .get(&DataKey::DepinOrders(depin_id))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get total order count
    pub fn get_order_count(env: Env) -> u32 {
        env.storage().persistent()
            .get(&DataKey::OrderCounter)
            .unwrap_or(0)
    }

    /// Get total escrowed amount
    pub fn get_total_escrowed(env: Env) -> i128 {
        env.storage().persistent()
            .get(&DataKey::TotalEscrowed)
            .unwrap_or(0)
    }

    /// Get treasury wallet address
    pub fn get_treasury_wallet(env: Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::TreasuryWallet)
    }

    // Helper functions
    fn assert_admin(env: &Env, admin: &Address) {
        let stored_admin: Address = env.storage().persistent()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized));
        
        if *admin != stored_admin {
            panic_with_error!(env, Error::NotAdmin);
        }
        admin.require_auth();
    }

    fn generate_order_id(env: &Env) -> BytesN<32> {
        let mut counter: u32 = env.storage().persistent()
            .get(&DataKey::OrderCounter)
            .unwrap_or(0);
        
        counter += 1;
        env.storage().persistent().set(&DataKey::OrderCounter, &counter);
        
        // Create unique ID with counter in first 4 bytes
        let mut id_bytes = [0u8; 32];
        id_bytes[0..4].copy_from_slice(&counter.to_be_bytes());
        
        // Fill remaining bytes with timestamp and random data from ledger
        let timestamp = env.ledger().timestamp();
        id_bytes[4..12].copy_from_slice(&timestamp.to_be_bytes());
        
        // Use sequence number for additional randomness
        let sequence = env.ledger().sequence();
        id_bytes[12..16].copy_from_slice(&sequence.to_be_bytes());
        
        BytesN::from_array(env, &id_bytes)
    }

    fn add_user_order(env: &Env, user: &Address, order_id: &BytesN<32>) {
        let mut user_orders: Vec<BytesN<32>> = env.storage().persistent()
            .get(&DataKey::UserOrders(user.clone()))
            .unwrap_or_else(|| Vec::new(env));
        
        user_orders.push_back(order_id.clone());
        env.storage().persistent().set(&DataKey::UserOrders(user.clone()), &user_orders);
    }

    fn add_depin_order(env: &Env, depin_id: &BytesN<32>, order_id: &BytesN<32>) {
        let mut depin_orders: Vec<BytesN<32>> = env.storage().persistent()
            .get(&DataKey::DepinOrders(depin_id.clone()))
            .unwrap_or_else(|| Vec::new(env));
        
        depin_orders.push_back(order_id.clone());
        env.storage().persistent().set(&DataKey::DepinOrders(depin_id.clone()), &depin_orders);
    }
}

mod test;
