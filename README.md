# NodeFoundry DePIN Platform

**Stellar Soroban Smart Contracts for Decentralized Physical Infrastructure Networks**

![Stellar](https://img.shields.io/badge/Stellar-Soroban-blue)
![Rust](https://img.shields.io/badge/Rust-22.0.8-orange)
![License](https://img.shields.io/badge/License-MIT-green)
![Status](https://img.shields.io/badge/Status-Work%20in%20Progress-yellow)

> ⚠️ **WORK IN PROGRESS** - This repository contains development code for the NodeFoundry DePIN Platform. The smart contracts are currently in active development and testing phase. Not recommended for production use yet.

## Overview

NodeFoundry is a comprehensive DePIN (Decentralized Physical Infrastructure Networks) marketplace platform built on Stellar using Soroban smart contracts. The platform provides a complete infrastructure for managing DePIN providers, user profiles, reputation systems, and order processing.

**Current Status**: 🚧 Development Phase - Core contracts implemented with comprehensive testing

## 🏗️ Architecture

This repository contains **4 core smart contracts** that work together to create a complete DePIN marketplace:

```text
contracts/
├── depin-registry/     # Core DePIN provider management
├── reputation/         # Rating and review system
├── user-profile/       # User accounts and wallet management
└── order/             # Order processing with escrow
```

## ✨ Features

- **🏢 DePIN Registry**: Manage infrastructure providers with validation
- **⭐ Reputation System**: User reviews and ratings for trust building
- **👤 User Profiles**: Comprehensive user management with wallet functionality
- **📦 Order Management**: Secure order processing with escrow mechanism
- **🔐 Admin Controls**: Comprehensive administrative functions

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- Stellar CLI (soroban)
- Soroban SDK 22.0.8

### Installation

```bash
# Clone the repository
git clone https://github.com/dev-nodefoundry/nodefoundry-stellar-sc.git
cd nodefoundry-stellar-sc

# Build all contracts
cargo build --target wasm32v1-none --release

# Run tests
cargo test
```

### Deploy to Stellar

```bash
# Build and optimize contracts
stellar contract build

# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32v1-none/release/depin_registry.wasm \
  --network testnet \
  --alias depin-registry
```

> ⚠️ **Development Notice**: These contracts are currently in development and testing phase. Only deploy to testnet for development purposes. Do not use in production environments.

## 📋 Contract Details

| Contract | Purpose | Key Features |
|----------|---------|--------------|
| **DePIN Registry** | Provider management | CRUD operations, validation, status management |
| **Reputation** | Trust system | Reviews, ratings, average calculations |
| **User Profile** | User management | Profiles, wallets, subscriptions, referrals |
| **Order** | Transaction processing | Escrow, order lifecycle, provider integration |

## 🧪 Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific contract
cd contracts/depin-registry && cargo test
```

## 📚 Documentation

- **[Technical Specification](docs/TECHNICAL_SPECIFICATION_final_clean.pdf)** - Complete platform documentation
- **[Contract Details](CONTRACTS_README.md)** - Detailed contract documentation
- **[Stellar Commands](docs/stellar_cmd_help.md)** - CLI reference

## 🏛️ Project Structure

```text
.
├── contracts/
│   ├── depin-registry/    # DePIN provider management
│   ├── reputation/        # Rating and review system  
│   ├── user-profile/      # User account management
│   └── order/            # Order processing
├── docs/                 # Documentation and PDFs
├── README.md            # This file
├── CONTRACTS_README.md  # Detailed contract docs
└── Cargo.toml          # Workspace configuration
```

## 🔧 Development

### Adding New Contracts

1. Create new directory in `contracts/`
2. Add to workspace in root `Cargo.toml`
3. Implement using Soroban patterns
4. Add comprehensive tests

### Code Style

- Follow Rust conventions
- Use Soroban SDK patterns
- Comprehensive error handling
- Clear documentation

## 🌐 Network Support

- **Stellar Testnet**: Ready for deployment
- **Stellar Mainnet**: Production ready
- **Local Development**: Full offline testing

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🚀 Development Roadmap

### ✅ Completed (Current Phase)
- [x] Core smart contract architecture
- [x] DePIN Registry contract with full CRUD operations
- [x] Reputation system with ratings and reviews
- [x] User profile management with wallet functionality
- [x] Order processing with escrow mechanism
- [x] Comprehensive test suite (38+ test cases)
- [x] Technical specification documentation

### 🚧 In Development
- [ ] Contract optimization and gas efficiency improvements
- [ ] Enhanced error handling and validation
- [ ] Integration testing between contracts
- [ ] Security audit preparation

### 📋 Planned Features
- [ ] Frontend interface development
- [ ] Additional DePIN service integrations
- [ ] Advanced analytics and reporting
- [ ] Multi-chain bridge support
- [ ] Governance token implementation
- [ ] Mainnet deployment

### ⚠️ Known Limitations
- Contracts are in development phase - not audited
- Limited to testnet deployment currently
- Frontend interface not yet developed
- Some advanced features are placeholder implementations

---

**Built with ❤️ by NodeFoundry Team**
