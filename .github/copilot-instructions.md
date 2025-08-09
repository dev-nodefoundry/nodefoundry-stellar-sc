# Copilot Instructions for Soroban DePIN Registry Project

## Project Overview
- This is a multi-contract Soroban project for Stellar, organized as a Rust workspace.
- Each contract (e.g., `depin-registry`) lives in its own directory under `contracts/`, with its own `Cargo.toml` and source/tests in `src/`.
- The main contract, `depin-registry`, manages DePIN nodes, reviews, and ratings using Soroban storage and types.

## Architecture & Patterns
- Contracts use Soroban SDK macros: `#[contract]`, `#[contractimpl]`, and `#[contracttype]`.
- Data is stored using enums (e.g., `DataKey`) and Soroban persistent storage (`env.storage().persistent()`).
- Unique IDs for DePINs are generated using a counter stored in contract state, encoded in the first 4 bytes of a `BytesN<32>`.
- All contract logic is in Rust, with no_std and strict type usage.
- Tests use Soroban's `testutils` for address generation and mock auths.
- Contract clients are generated and used for integration-style tests.

## Developer Workflows
- **Build contracts:**
  - `cargo build --target wasm32v1-none --release` (see `stellar_cmd_help.md` for more)
  - `stellar contract build` (Soroban CLI)
- **Test contracts:**
  - `cargo test` or `cargo test -- --nocapture` (for verbose output)
  - Tests are in each contract's `src/test.rs` and use `mock_all_auths()` for auth simulation.
- **Deploy contracts:**
  - Use Soroban CLI: `stellar contract deploy --wasm ... --network testnet --alias ...`
  - See `stellar_cmd_help.md` for full deployment and invocation examples.
- **Optimize contracts:**
  - `stellar contract optimize --wasm ...`

## Conventions & Patterns
- All contract state keys are defined in a Rust enum (see `DataKey`).
- DePIN IDs are always `BytesN<32>`, with the counter in the first 4 bytes.
- Admin-only actions use an `assert_admin` helper.
- Reviews and ratings are stored as a map from DePIN ID to a vector of `(Address, i32, String)`.
- All contract methods validate input strictly and panic with clear error messages.
- Tests use direct contract client calls and assert on state, error messages, and auths.
- No external dependencies except Soroban SDK.

## Key Files & References
- `contracts/depin-registry/src/lib.rs`: Main contract logic and patterns.
- `contracts/depin-registry/src/test.rs`: Test patterns, client usage, and error handling.
- `stellar_cmd_help.md`: Essential CLI commands for build, test, deploy, and optimize.
- `README.md`: Workspace structure and contract organization.

## Example Patterns
- **Add DePIN:**
  ```rust
  let depin_id = registry.add_depin(&admin, &name, &desc, &uptime, &reliability, &cost);
  ```
- **Test with mock auths:**
  ```rust
  env.mock_all_auths();
  let admin = Address::generate(&env);
  let registry = init_registry(&env, &admin);
  ```
- **Decode DePIN ID counter:**
  ```rust
  fn u32_from_id(id: &BytesN<32>) -> u32 { ... }
  ```

## Integration Points
- No external APIs or services; all logic is on-chain and in Rust.
- CLI integration for deployment and invocation (see `stellar_cmd_help.md`).

---

If any conventions or workflows are unclear, please provide feedback or request more examples.
