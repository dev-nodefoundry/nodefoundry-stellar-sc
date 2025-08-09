# NodeFoundry Platform Contract

The NodeFoundry Platform contract is a comprehensive smart contract that manages user profiles, wallet functionality, and DePIN interactions for the NodeFoundry ecosystem.

## Overview

This contract serves as the central hub for user management and financial operations within the NodeFoundry platform. Users can create profiles, manage their wallet balances, interact with DePINs, and participate in subscription and referral programs.

## Key Features

### 1. User Profile Management
- **User Registration**: Create comprehensive user profiles with verification system
- **Profile Updates**: Update username and email information
- **Verification System**: Admin-controlled user verification
- **Referral System**: Built-in referral program with bonus tracking

### 2. Wallet Management
- **Multi-Token Support**: Support for multiple whitelisted tokens (USDC by default)
- **Deposit/Withdrawal**: Secure fund management with transaction tracking
- **Balance Tracking**: Real-time balance monitoring per token
- **Transaction History**: Complete audit trail of all financial operations

### 3. DePIN Integration
- **Usage Tracking**: Monitor DePIN usage sessions with time-based billing
- **Flexible Pricing**: Configurable pricing models (hourly, monthly, pay-per-use)
- **Automatic Billing**: Real-time cost calculation and payment processing
- **Usage History**: Complete history of DePIN interactions

### 4. Subscription System
- **Multi-Tier Subscriptions**: Basic (Free), Premium ($10), Enterprise ($50)
- **Automatic Billing**: Seamless subscription upgrades with balance deduction
- **Usage Benefits**: Different access levels based on subscription tier

### 5. Loyalty Program
- **Points System**: Earn 1 loyalty point per 1 USDC spent
- **Referral Rewards**: 5% commission on referred user spending
- **Spending Tracking**: Complete spending analytics per user

### 6. Administrative Functions
- **Token Whitelisting**: Admin control over accepted tokens
- **DePIN Pricing**: Flexible pricing configuration for DePIN services
- **Platform Analytics**: Real-time platform statistics and metrics

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

### Transaction
```rust
pub struct Transaction {
    pub tx_id: String,
    pub user_address: Address,
    pub tx_type: String, // "deposit", "withdrawal", "depin_payment", "refund"
    pub amount: i128,
    pub token_address: Address,
    pub depin_id: Option<soroban_sdk::BytesN<32>>,
    pub timestamp: u64,
    pub status: String, // "pending", "completed", "failed"
    pub description: String,
}
```

### DepinUsage
```rust
pub struct DepinUsage {
    pub depin_id: soroban_sdk::BytesN<32>,
    pub user_address: Address,
    pub usage_start: u64,
    pub usage_end: Option<u64>,
    pub total_cost: i128,
    pub usage_type: String, // "hourly", "monthly", "pay-per-use"
    pub is_active: bool,
}
```

## Key Functions

### User Management
- `create_user_profile(user_address, username, email, referral_code)` - Create new user
- `update_user_profile(user_address, username, email)` - Update profile info
- `verify_user(invoker, user_address)` - Admin verification
- `get_user_profile(user_address)` - Retrieve user data

### Wallet Operations
- `deposit_funds(user_address, token_address, amount, tx_id)` - Add funds
- `withdraw_funds(user_address, token_address, amount, tx_id)` - Remove funds
- `get_user_balance(user_address, token_address)` - Check balance

### DePIN Management
- `set_depin_pricing(invoker, depin_id, price_per_hour, pricing_model)` - Admin pricing
- `start_depin_usage(user_address, depin_id, usage_type, token_address)` - Begin usage
- `stop_depin_usage(user_address, usage_id, token_address)` - End usage & pay
- `get_user_usage_history(user_address)` - Usage analytics

### Subscription & Loyalty
- `upgrade_subscription(user_address, tier, token_address)` - Change tier
- `claim_referral_bonus(user_address, token_address)` - Claim rewards

### Administrative
- `whitelist_token(invoker, token_address)` - Add supported token
- `remove_token_whitelist(invoker, token_address)` - Remove token
- `get_platform_stats()` - Platform metrics

## Suggested Additional Features

Based on the core functionality, here are recommended enhancements:

### 1. Advanced Billing Features
- **Prepaid Credits**: Allow users to purchase DePIN credits in advance
- **Usage Limits**: Set spending limits and alerts
- **Auto-Reload**: Automatic balance top-up when below threshold
- **Billing Cycles**: Monthly billing with invoice generation

### 2. Enhanced Security
- **Multi-Signature Wallets**: Enhanced security for large transactions
- **Withdrawal Limits**: Daily/monthly withdrawal restrictions
- **2FA Integration**: Two-factor authentication support
- **Fraud Detection**: Unusual activity monitoring

### 3. Social Features
- **User Reviews**: Rate and review DePIN providers
- **Social Profiles**: Public profile pages with usage stats
- **Achievement System**: Badges for platform milestones
- **Community Forums**: Discussion and support features

### 4. Advanced Analytics
- **Usage Patterns**: AI-powered usage optimization
- **Cost Optimization**: Recommendations for cost savings
- **Performance Metrics**: DePIN performance tracking
- **Predictive Billing**: Forecast monthly costs

### 5. Integration Features
- **API Keys**: Third-party integration support
- **Webhooks**: Real-time event notifications
- **Mobile App Integration**: Deep linking and push notifications
- **Cross-Platform Sync**: Sync across multiple devices

### 6. Business Features
- **Team Accounts**: Corporate account management
- **Bulk Operations**: Batch user management
- **White-Label Support**: Custom branding options
- **Reseller Program**: Partner revenue sharing

## Pricing Recommendations

### Subscription Tiers
- **Basic (Free)**: Limited DePIN access, basic support
- **Premium ($10/month)**: Priority access, advanced analytics, 24/7 support
- **Enterprise ($50/month)**: Custom integrations, dedicated support, bulk discounts

### Transaction Fees
- **Deposit**: Free for amounts > $100, $2 fee for smaller amounts
- **Withdrawal**: $5 flat fee or 1% of amount (whichever is higher)
- **DePIN Usage**: No platform fee (only DePIN provider costs)

### Loyalty Program
- **Points Earning**: 1 point per $1 spent, bonus points for referrals
- **Points Redemption**: 100 points = $1 credit, special offers
- **Tier Benefits**: Increased point earning rates for higher subscription tiers

## Security Considerations

1. **Access Control**: Strict admin-only functions for critical operations
2. **Input Validation**: Comprehensive validation for all user inputs
3. **Balance Checks**: Prevent overdraft and negative balance scenarios
4. **Audit Trail**: Complete transaction history for compliance
5. **Emergency Functions**: Admin ability to pause operations if needed

## Future Roadmap

1. **Cross-Chain Support**: Integrate with other blockchain networks
2. **DeFi Integration**: Yield farming and staking options
3. **NFT Support**: Digital certificates and achievements
4. **Governance Token**: Platform governance and voting rights
5. **AI Integration**: Smart recommendations and automation

This contract provides a solid foundation for the NodeFoundry platform while maintaining flexibility for future enhancements and integrations.
