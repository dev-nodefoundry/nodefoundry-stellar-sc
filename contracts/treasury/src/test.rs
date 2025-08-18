// Treasury contract tests will be added here following the same Soroban test patterns as other contracts.
// Use mock_all_auths, Address::generate, and direct contract client calls for all test cases.

#[cfg(test)]
mod tests {
    use soroban_sdk::{Address, Env, BytesN, String};
    use crate::{TreasuryContract, TreasuryContractClient};

    fn setup(env: &Env) -> (TreasuryContractClient, Address, Address, Address) {
        // Use fixed addresses for test determinism
        let admin_contract = Address::from_string(&String::from_str(env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"));
        let user = Address::from_string(&String::from_str(env, "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBR4"));
        let token = Address::from_string(&String::from_str(env, "GCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC"));
        // Use a fixed contract id for testing
        let contract_id = Address::from_string(&String::from_str(env, "GDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD"));
        let client = TreasuryContractClient::new(env, &contract_id);
        client.initialize(&admin_contract);
        (client, admin_contract, user, token)
    }

    #[test]
    fn test_initialize_and_get_admin() {
        let env = Env::default();
        let (treasury, admin_contract, _, _) = setup(&env);
        assert_eq!(treasury.get_admin_contract(), admin_contract);
    }
    fn test_withdraw_insufficient_balance() {
        let env = Env::default();
        let (treasury, _, user, token) = setup(&env);
        treasury.deposit(&token, &user, &100);
        // Try to withdraw more than balance
        treasury.withdraw(&token, &user, &200, &false);
    }
}
