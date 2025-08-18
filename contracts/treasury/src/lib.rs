#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, Env, Address, Symbol, IntoVal,
    symbol_short,
    panic_with_error,
};

#[contracttype]
pub enum DataKey {
    AdminContract,                // Address of the admin contract
    AssetBalance(Address),        // token address -> i128
    TotalReceived(Address),       // token address -> i128
    TotalWithdrawn(Address),      // token address -> i128
    LastWithdrawal(u64),          // timestamp
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AdminNotSet = 1,
    NotAdmin = 2,
    InsufficientBalance = 3,
    InvalidAmount = 4,
}

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    // Initialize with admin contract address (can only be set once)
    pub fn initialize(env: Env, admin_contract: Address) {
        if env.storage().persistent().has(&DataKey::AdminContract) {
            panic_with_error!(&env, Error::AdminNotSet);
        }
        env.storage().persistent().set(&DataKey::AdminContract, &admin_contract);
    }

    // Deposit tokens into treasury (called by order contract)
    pub fn deposit(env: Env, token: Address, from: Address, amount: i128) {
        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        // Transfer tokens from user/order contract to treasury
        soroban_sdk::token::Client::new(&env, &token)
            .transfer(&from, &env.current_contract_address(), &amount);

        let bal = Self::get_balance_internal(&env, &token);
        let new_bal = bal + amount;
        env.storage().persistent().set(&DataKey::AssetBalance(token.clone()), &new_bal);

        let total = env.storage().persistent().get(&DataKey::TotalReceived(token.clone())).unwrap_or(0i128) + amount;
        env.storage().persistent().set(&DataKey::TotalReceived(token.clone()), &total);

        env.events().publish(
            (symbol_short!("deposit"), token.clone()),
            (from, amount, new_bal),
        );
    }

    // Withdraw tokens (user can withdraw their own, admin contract can withdraw platform funds)
    pub fn withdraw(env: Env, token: Address, to: Address, amount: i128, is_admin: bool) {
        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }

        let bal = Self::get_balance_internal(&env, &token);
        if bal < amount {
            panic_with_error!(&env, Error::InsufficientBalance);
        }

        // If admin withdrawal, must be called by admin contract
        if is_admin {
            let admin_contract: Address = env.storage().persistent().get(&DataKey::AdminContract).expect("admin not set");
            admin_contract.require_auth();
        } else {
            // User withdrawal: only the recipient can withdraw
            to.require_auth();
        }

        let new_bal = bal - amount;
        env.storage().persistent().set(&DataKey::AssetBalance(token.clone()), &new_bal);

        let total = env.storage().persistent().get(&DataKey::TotalWithdrawn(token.clone())).unwrap_or(0i128) + amount;
        env.storage().persistent().set(&DataKey::TotalWithdrawn(token.clone()), &total);

        env.storage().persistent().set(&DataKey::LastWithdrawal(env.ledger().timestamp()), &env.ledger().timestamp());

        // Transfer tokens from treasury to recipient
        soroban_sdk::token::Client::new(&env, &token)
            .transfer(&env.current_contract_address(), &to, &amount);

        env.events().publish(
            (symbol_short!("withdraw"), token.clone()),
            (to, amount, new_bal, is_admin),
        );
    }

    // Getters
    pub fn get_balance(env: Env, token: Address) -> i128 {
        Self::get_balance_internal(&env, &token)
    }

    pub fn get_total_received(env: Env, token: Address) -> i128 {
        env.storage().persistent().get(&DataKey::TotalReceived(token)).unwrap_or(0)
    }

    pub fn get_total_withdrawn(env: Env, token: Address) -> i128 {
        env.storage().persistent().get(&DataKey::TotalWithdrawn(token)).unwrap_or(0)
    }

    pub fn get_admin_contract(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::AdminContract).expect("admin not set")
    }

    // Internal helper
    fn get_balance_internal(env: &Env, token: &Address) -> i128 {
        env.storage().persistent().get(&DataKey::AssetBalance(token.clone())).unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
