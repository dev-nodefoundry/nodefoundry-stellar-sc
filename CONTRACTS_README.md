# DePIN Smart Contracts

This project contains two separate Stellar smart contracts for managing DePIN (Decentralized Physical Infrastructure Networks):

## 1. DePIN Registry Contract (`contracts/depin-registry/`)

This contract manages the core DePIN infrastructure data.

### Features:
- **Admin Management**: Only admins can add, update, or remove DePINs
- **DePIN CRUD Operations**: Create, read, update, and delete DePIN entries
- **Status Management**: Enable/disable DePINs
- **Validation**: Input validation for all DePIN parameters

### Key Functions:
- `initialize(admin)` - Initialize contract with admin
- `add_depin(invoker, name, description, uptime, reliability, cost)` - Add new DePIN
- `update_depin(invoker, depin_id, name, description, uptime, reliability, cost)` - Update existing DePIN
- `remove_depin(invoker, depin_id)` - Remove DePIN
- `set_depin_status(invoker, depin_id, status)` - Change DePIN status
- `get_depin(depin_id)` - Get DePIN details
- `list_depins()` - List all DePIN IDs
- `get_depin_count()` - Get total count
- `depin_exists(depin_id)` - Check if DePIN exists

### Data Structure:
```rust
type DePIN = (
    soroban_sdk::BytesN<32>, // ID
    String,                  // Name
    String,                  // Description
    bool,                    // Status (active/inactive)
    i32,                     // Uptime (0-100)
    i32,                     // Reliability (0-100)
    i32,                     // Cost
);
```

## 2. Reputation Contract (`contracts/reputation/`)

This contract manages user ratings and reviews for DePINs.

### Features:
- **User Reviews**: Any user can rate and review DePINs
- **Rating Statistics**: Calculate averages, min/max ratings
- **Review Management**: Update or replace user reviews
- **Cross-Contract Integration**: References DePIN registry for validation

### Key Functions:
- `initialize(admin, depin_registry_address)` - Initialize with DePIN registry reference
- `rate_and_review_depin(invoker, depin_id, rating, review)` - Add/update user review
- `get_reviews(depin_id)` - Get all reviews for a DePIN
- `get_average_rating(depin_id)` - Get average rating
- `get_rating_stats(depin_id)` - Get comprehensive rating statistics
- `get_review_count(depin_id)` - Get number of reviews
- `remove_depin_reviews(invoker, depin_id)` - Admin function to clean up reviews

### Data Structure:
```rust
type Review = (
    Address, // Reviewer address
    i32,     // Rating (1-5)
    String,  // Review text
);
```

## Building and Testing

### Build all contracts:
```bash
cargo build
```

### Build individual contracts:
```bash
cd contracts/depin-registry && make build
cd contracts/reputation && make build
```

### Run tests:
```bash
cd contracts/depin-registry && make test
cd contracts/reputation && make test
```

## Architecture Benefits

1. **Separation of Concerns**: Core DePIN data is separate from user-generated content
2. **Modularity**: Each contract can be upgraded independently
3. **Scalability**: Reputation data can scale without affecting core registry
4. **Security**: Different access controls for different functionalities
5. **Flexibility**: Easy to add new reputation features without touching core registry

## Usage Pattern

1. Deploy DePIN Registry contract first
2. Deploy Reputation contract with registry address
3. Admin adds DePINs to registry
4. Users interact with both contracts (read from registry, write to reputation)

## Cross-Contract Integration

The reputation contract is designed to validate DePIN existence by calling the registry contract. In the current implementation, this validation is stubbed out but can be implemented using Stellar's contract invocation features.
