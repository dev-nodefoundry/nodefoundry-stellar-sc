# User Profile Contract

This contract manages user profiles and wallet functionality for the NodeFoundry platform. It's designed to work alongside the order management contract.

## Features

### User Management
- **Create User Profiles**: Register users with username, email, and optional referral codes
- **Update Profiles**: Modify user information
- **User Verification**: Admin can verify users
- **Referral System**: Users can refer others and earn referral codes

### Wallet Management
- **Multi-token Support**: Support for whitelisted tokens (USDC by default)
- **Deposits & Withdrawals**: Users can manage their token balances
- **Balance Tracking**: Real-time balance management
- **Admin Token Control**: Whitelist/remove tokens

### Subscription System
- **Three Tiers**: Basic (Free), Premium (10 USDC), Enterprise (50 USDC)
- **Automatic Billing**: Subscription costs deducted from user balance
- **Loyalty Points**: Users earn points based on spending

### Order Contract Integration
- **Balance Deduction**: Order contract can deduct user balances
- **Refund System**: Order contract can refund users
- **Balance Validation**: Check if users have sufficient funds
- **User Validation**: Verify user existence

## Data Structures

### UserProfile
```rust
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
```

### PlatformStats
```rust
pub struct PlatformStats {
    pub total_users: u32,
    pub total_deposits: i128,
    pub total_withdrawals: i128,
    pub active_subscriptions: u32,
}
```

## Key Functions

### User Management
- `initialize(admin, usdc_token)` - Initialize contract
- `create_user_profile(user_address, username, email, referral_code)` - Create user
- `update_user_profile(user_address, username, email)` - Update user info
- `verify_user(invoker, user_address)` - Admin verify user
- `get_user_profile(user_address)` - Get user details

### Wallet Management
- `deposit_funds(user_address, token_address, amount)` - Deposit tokens
- `withdraw_funds(user_address, token_address, amount)` - Withdraw tokens
- `get_user_balance(user_address, token_address)` - Check balance

### Order Contract Integration
- `deduct_balance(user_address, token_address, amount)` - Deduct for orders
- `refund_balance(user_address, token_address, amount)` - Refund orders
- `user_exists(user_address)` - Check if user exists
- `has_sufficient_balance(user_address, token_address, amount)` - Check funds

### Subscription Management
- `upgrade_subscription(user_address, tier, token_address)` - Change tier

### Admin Functions
- `whitelist_token(invoker, token_address)` - Add supported token
- `remove_token_whitelist(invoker, token_address)` - Remove token
- `get_platform_stats()` - Get platform statistics

## Integration with Order Contract

The User Profile contract provides essential services for the order management system:

1. **User Validation**: Order contract can verify users exist before processing orders
2. **Balance Management**: Order contract can check balances and deduct payments
3. **Refund Processing**: Order contract can refund users when orders are cancelled
4. **Loyalty Tracking**: User spending and loyalty points are automatically updated

## Usage Flow

1. **User Registration**:
   ```rust
   let referral_code = contract.create_user_profile(user_address, username, email, None);
   ```

2. **Deposit Funds**:
   ```rust
   contract.deposit_funds(user_address, usdc_token, amount);
   ```

3. **Order Processing** (from order contract):
   ```rust
   // Check if user exists and has sufficient balance
   if user_profile_contract.user_exists(user_address) && 
      user_profile_contract.has_sufficient_balance(user_address, token, amount) {
       // Deduct payment
       user_profile_contract.deduct_balance(user_address, token, amount);
   }
   ```

4. **Refund Processing** (from order contract):
   ```rust
   user_profile_contract.refund_balance(user_address, token, refund_amount);
   ```

## Security Features

- **Admin Controls**: Critical functions restricted to admin
- **User Validation**: All operations require valid user profiles
- **Token Whitelist**: Only approved tokens can be used
- **Balance Verification**: Prevents overdrafts and insufficient funds
- **Input Validation**: Username and email cannot be empty

## Future Enhancements

- Enhanced referral bonus system
- Time-based subscription management
- Advanced loyalty point features
- Integration with reputation system
- Multi-signature admin controls
