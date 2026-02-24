# MERKLITH Blockchain

<p align="center">
  <img src="docs/images/merklith-logo.png" alt="MERKLITH Blockchain" width="200"/>
</p>

<p align="center">
  <strong>"Building Trust Block by Block"</strong> - A Complete Layer 1 Blockchain in Rust
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0%20OR%20MIT-blue.svg" alt="License"></a>
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.75%2B-orange.svg" alt="Rust Version"></a>
  <a href="#testing"><img src="https://img.shields.io/badge/Tests-244%20passed-brightgreen.svg" alt="Tests"></a>
  <a href="https://github.com/merklith/merklith/releases"><img src="https://img.shields.io/badge/Version-0.1.0-blue.svg" alt="Version"></a>
</p>

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Documentation](#documentation)
- [Project Structure](#project-structure)
- [RPC API](#rpc-api)
- [Consensus Mechanism](#consensus-mechanism)
- [Testing](#testing)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Overview

MERKLITH is a high-performance Layer 1 blockchain built from scratch in Rust. Unlike Ethereum clones, MERKLITH introduces innovative technologies including Proof of Contribution (PoC) consensus, ed25519 signatures, and a unique economic model that rewards actual network contributions.

### What Makes MERKLITH Different?

| Feature | MERKLITH | Ethereum |
|---------|-------|----------|
| **Hash Function** | Blake3 | Keccak256 |
| **Signatures** | Ed25519 | Secp256k1 |
| **Consensus** | Proof of Contribution | Proof of Stake |
| **Address Format** | Bech32m | Hex (0x...) |
| **RPC Namespace** | `merklith_*` / `eth_*` | `eth_*` |
| **Block Time** | 2-6 seconds | 12 seconds |
| **Smart Contracts** | MerklithVM (bytecode) | EVM |

## Key Features

### 🔐 Advanced Cryptography
- **Ed25519 Signatures**: Faster and more secure than secp256k1
- **BLS12-381**: For committee attestations and signature aggregation
- **Blake3 Hashing**: 3x faster than Keccak256 with parallel processing
- **VRF (Verifiable Random Functions)**: For fair proposer selection

### 🏛️ Proof of Contribution (PoC) Consensus
Validators earn scores through actual network contributions:

| Activity | Points | Description |
|----------|--------|-------------|
| Block Production | +100 | Creating and proposing valid blocks |
| Attestations | +10 | Signing and validating blocks |
| Transaction Relay | +1 | Propagating transactions |
| Peer Discovery | +1 | Connecting new nodes to the network |
| Data Availability | +1 | Storing and serving historical data |

Proposers are selected based on weighted PoC scores, ensuring the most contributing validators lead consensus.

### 💼 Built-in Wallet & Keystore
- AES-256-GCM encrypted keystores
- Argon2id password hashing
- Hardware wallet support (future)
- Multi-account management

### 🖥️ TUI Block Explorer
Interactive terminal-based blockchain explorer with:
- Real-time block updates
- Transaction inspection
- Account balance queries
- Vim-style keyboard navigation

### 🛠️ Developer Tools
- Comprehensive CLI for all operations
- Rust and TypeScript SDKs
- JSON-RPC API compatible with Ethereum tools
- Docker deployment support

## Quick Start

### Prerequisites
- Rust 1.75+ with Cargo
- 4GB RAM minimum
- 10GB free disk space

### Installation

```bash
# Clone the repository
git clone https://github.com/merklith/merklith.git
cd merklith

# Build in release mode (optimized)
cargo build --release

# Binaries will be available at:
# ./target/release/merklith-node    (Full node)
# ./target/release/merklith         (CLI tool)
```

### Run a Single Node

```bash
# Start a validator node
./target/release/merklith-node --chain-id 1337 --validator

# Output:
# [INFO] MERKLITH Node v0.1.0 - Where Trust is Forged
# [INFO] Chain ID: 1337
# [INFO] RPC Server: http://0.0.0.0:8545
# [INFO] P2P Network: listening on 0.0.0.0:30303
# [INFO] Block production started
```

### Test the RPC

```bash
# Get chain ID
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_chainId",
    "params": [],
    "id": 1
  }'
# Response: {"jsonrpc":"2.0","result":"0x539","id":1}

# Get latest block number
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_blockNumber",
    "params": [],
    "id": 1
  }'

# Check balance
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getBalance",
    "params": ["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb", "latest"],
    "id": 1
  }'
```

### Launch TUI Explorer

```bash
# Interactive terminal explorer
./target/release/merklith explorer --rpc http://localhost:8545
```

Keyboard shortcuts:
- `b` - View blocks
- `t` - View transactions  
- `r` - Refresh data
- `h` - Help
- `q` - Quit

## Installation

### From Source

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/merklith/merklith.git
cd merklith
cargo build --release

# Add to PATH (optional)
sudo cp target/release/merklith* /usr/local/bin/
```

### Docker

```bash
# Build Docker image
docker build -t merklith-blockchain .

# Run container
docker run -p 8545:8545 -p 30303:30303 merklith-blockchain

# Or use docker-compose for multi-node setup
docker-compose up -d
```

### Pre-built Binaries

Download from [Releases](https://github.com/merklith/merklith/releases):
- Linux: `merklith-linux-x86_64.tar.gz`
- macOS: `merklith-darwin-x86_64.tar.gz`
- Windows: `merklith-windows-x86_64.zip`

## Documentation

- **[CLI Guide](docs/CLI_GUIDE.md)** - Complete command-line interface documentation
- **[TUI Explorer](docs/EXPLORER.md)** - Terminal UI block explorer guide
- **[RPC API](docs/API.md)** - JSON-RPC methods and examples
- **[Architecture](docs/ARCHITECTURE.md)** - System design and components
- **[Contributing](CONTRIBUTING.md)** - How to contribute to MERKLITH

## Project Structure

```
merklith/
├── crates/
│   ├── merklith-types/          # Core blockchain types
│   │   ├── U256, Address, Hash
│   │   ├── Block, Transaction, Receipt
│   │   └── Serialization (Borsh, JSON)
│   │
│   ├── merklith-crypto/         # Cryptographic primitives
│   │   ├── Ed25519 signatures
│   │   ├── BLS12-381 for attestations
│   │   ├── Blake3 hashing
│   │   ├── VRF (randomness)
│   │   └── Merkle trees
│   │
│   ├── merklith-storage/        # Persistent storage
│   │   ├── JSON-based state DB
│   │   └── Block store
│   │
│   ├── merklith-core/           # Blockchain core
│   │   ├── State machine
│   │   ├── Block builder
│   │   ├── Chain management
│   │   └── Fee market
│   │
│   ├── merklith-vm/             # Virtual machine
│   │   ├── Bytecode interpreter
│   │   ├── Gas metering
│   │   └── Reentrancy protection
│   │
│   ├── merklith-consensus/      # PoC consensus
│   │   ├── Attestation pool
│   │   ├── Committee selection
│   │   ├── Finality gadget
│   │   └── Slashing logic
│   │
│   ├── merklith-txpool/         # Transaction pool
│   │   ├── Pending transactions
│   │   └── Batch processing
│   │
│   ├── merklith-network/        # P2P networking
│   │   ├── TCP transport
│   │   ├── Peer discovery
│   │   └── Block gossip
│   │
│   ├── merklith-rpc/            # RPC server
│   │   ├── HTTP/JSON-RPC
│   │   └── 25+ methods
│   │
│   ├── merklith-node/           # Full node
│   │   ├── Node runtime
│   │   ├── Block production
│   │   └── Metrics/telemetry
│   │
│   ├── merklith-cli/            # Command-line tool
│   │   ├── Wallet management
│   │   ├── Transaction signing
│   │   ├── Query operations
│   │   └── TUI explorer
│   │
│   └── merklith-governance/     # On-chain governance
│       ├── Proposal system
│       ├── Voting with token lock
│       ├── Delegation
│       └── Treasury management
│
├── sdk/
│   ├── merklith-sdk-rs/         # Rust SDK
│   └── merklith-sdk-ts/         # TypeScript SDK
│
├── contracts/                # System contracts
│   ├── Bridge.sol
│   ├── Governance.sol
│   ├── Staking.sol
│   └── Treasury.sol
│
├── docs/                     # Documentation
│   ├── CLI_GUIDE.md
│   ├── API.md
│   ├── EXPLORER.md
│   └── ARCHITECTURE.md
│
├── tests/                    # Integration tests
│   ├── consensus_tests.rs
│   ├── storage_tests.rs
│   └── throughput_tests.rs
│
├── config/                   # Network configurations
│   ├── mainnet.toml
│   ├── testnet.toml
│   └── devnet.toml
│
├── docker/                   # Docker configurations
│   ├── Dockerfile
│   └── docker-compose.yml
│
└── benches/                  # Performance benchmarks
    ├── consensus_bench.rs
    └── storage_bench.rs
```

## RPC API

MERKLITH supports both native `merklith_*` and Ethereum-compatible `eth_*` RPC methods.

### Chain Methods

| Method | Description | Example |
|--------|-------------|---------|
| `eth_chainId` | Get chain ID | `{"method":"eth_chainId","params":[]}` |
| `eth_blockNumber` | Get latest block | `{"method":"eth_blockNumber","params":[]}` |
| `eth_getBlockByNumber` | Get block by number | `{"method":"eth_getBlockByNumber","params":["0x1",true]}` |
| `eth_getBlockByHash` | Get block by hash | `{"method":"eth_getBlockByHash","params":["0x...",true]}` |
| `eth_syncing` | Check sync status | `{"method":"eth_syncing","params":[]}` |

### Account Methods

| Method | Description | Example |
|--------|-------------|---------|
| `eth_getBalance` | Get account balance | `{"method":"eth_getBalance","params":["0x...","latest"]}` |
| `eth_getTransactionCount` | Get nonce | `{"method":"eth_getTransactionCount","params":["0x...","latest"]}` |
| `eth_getCode` | Get contract code | `{"method":"eth_getCode","params":["0x...","latest"]}` |
| `eth_getStorageAt` | Get storage slot | `{"method":"eth_getStorageAt","params":["0x...","0x0","latest"]}` |

### Transaction Methods

| Method | Description | Example |
|--------|-------------|---------|
| `eth_sendTransaction` | Send transaction | `{"method":"eth_sendTransaction","params":[{"from":"0x...","to":"0x...","value":"0x..."}]}` |
| `eth_sendRawTransaction` | Send signed tx | `{"method":"eth_sendRawTransaction","params":["0x..."]}` |
| `eth_getTransactionByHash` | Get transaction | `{"method":"eth_getTransactionByHash","params":["0x..."]}` |
| `eth_getTransactionReceipt` | Get receipt | `{"method":"eth_getTransactionReceipt","params":["0x..."]}` |
| `eth_gasPrice` | Get gas price | `{"method":"eth_gasPrice","params":[]}` |
| `eth_estimateGas` | Estimate gas | `{"method":"eth_estimateGas","params":[{"to":"0x...","data":"0x..."}]}` |

### Contract Methods

| Method | Description |
|--------|-------------|
| `eth_call` | Call contract (read-only) |
| `merklith_deployContract` | Deploy new contract |

### Consensus Methods

| Method | Description |
|--------|-------------|
| `merklith_createAttestation` | Submit attestation |
| `merklith_getValidators` | Get validator set |
| `merklith_getAttestations` | Get block attestations |

See [docs/API.md](docs/API.md) for complete documentation.

## Consensus Mechanism

### Proof of Contribution (PoC)

PoC rewards validators for actual network contributions, not just stake.

#### Contribution Types

```rust
pub enum ContributionType {
    BlockProduction,    // +100 points
    Attestation,        // +10 points
    TransactionRelay,   // +1 point
    PeerDiscovery,      // +1 point
    DataAvailability,   // +1 point
}
```

#### Validator Selection

Proposers are selected probabilistically based on their PoC scores:

```
Selection Probability = (Validator Score) / (Total Network Score)
```

#### Attestation Flow

1. Block proposed by selected validator
2. Committee members attest to block validity
3. BLS signatures aggregated for efficiency
4. Block finalized after threshold attestations

### Security Properties

- **BFT Finality**: 2/3+ attestations required for finality
- **Slashing**: Malicious validators lose stake
- **Inactivity Leak**: Inactive validators slowly lose score
- **Random Proposer Selection**: VRF ensures fairness

## Testing

### Test Suite

```bash
# Run all library tests
cargo test --lib --workspace

# Run specific crate tests
cargo test -p merklith-types
cargo test -p merklith-crypto
cargo test -p merklith-consensus

# Run integration tests
cargo test --test integration

# Run with output
cargo test --lib -- --nocapture
```

### Test Coverage

| Crate | Tests | Coverage |
|-------|-------|----------|
| merklith-types | 84 | Core types, serialization |
| merklith-crypto | 38 | Signatures, hashing, keys |
| merklith-consensus | 9 | Attestations, PoC |
| merklith-core | 14 | State, blocks, fees |
| merklith-vm | 27 | Bytecode, gas |
| merklith-storage | 8 | Persistence |
| merklith-txpool | 10 | Pool management |
| merklith-rpc | 15 | JSON-RPC |
| merklith-governance | 39 | Voting, treasury |
| **Total** | **244** | **All passing** |

### Benchmarks

```bash
# Run benchmarks
cargo bench

# Results:
# Block production: ~2ms
# Signature verification: ~50μs
# Transaction processing: ~100μs
# State root calculation: ~1ms (1000 accounts)
```

## Development

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs))
- Cargo
- Git

### Build Commands

```bash
# Development build (fast compilation)
cargo build

# Release build (optimized)
cargo build --release

# Build specific crate
cargo build -p merklith-node
cargo build -p merklith-cli

# Check compilation without building
cargo check
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test --lib

# Check documentation
cargo doc --no-deps
```

### Multi-Node Development Setup

```bash
# Terminal 1: Bootstrap node
./target/release/merklith-node \
  --chain-id 1337 \
  --validator \
  --p2p-port 30303 \
  --rpc-port 8545 \
  --data-dir ./data/node1

# Terminal 2: Second validator
./target/release/merklith-node \
  --chain-id 1337 \
  --validator \
  --p2p-port 30304 \
  --rpc-port 8546 \
  --data-dir ./data/node2 \
  --bootnodes /ip4/127.0.0.1/tcp/30303/p2p/12D3Koo...

# Terminal 3: Full node (non-validator)
./target/release/merklith-node \
  --chain-id 1337 \
  --p2p-port 30305 \
  --rpc-port 8547 \
  --data-dir ./data/node3 \
  --bootnodes /ip4/127.0.0.1/tcp/30303/p2p/12D3Koo...
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/merklith-node

# Trace-level logging
RUST_LOG=trace ./target/release/merklith-node

# Log to file
RUST_LOG=info ./target/release/merklith-node 2>&1 | tee merklith.log
```

## Contributing

We welcome contributions from the community! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Code of Conduct
- Development workflow
- Submitting pull requests
- Reporting issues
- Code style guidelines

### Quick Contribution Guide

```bash
# 1. Fork the repository
# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/merklith.git

# 3. Create a branch
git checkout -b feature/my-feature

# 4. Make changes and commit
git add .
git commit -m "feat: add my feature"

# 5. Push and create PR
git push origin feature/my-feature
```

## Roadmap

### Phase 1: Core (✅ Complete)
- [x] Basic blockchain structure
- [x] PoC consensus
- [x] Transaction pool
- [x] JSON-RPC API
- [x] CLI tool
- [x] TUI explorer

### Phase 2: Production (In Progress)
- [ ] P2P block sync
- [ ] Merkle trie state
- [ ] Validator staking
- [ ] Mainnet deployment
- [ ] Block explorer web UI

### Phase 3: Ecosystem
- [ ] Bridge contracts
- [ ] DeFi primitives
- [ ] Governance DAO
- [ ] Mobile wallet
- [ ] Hardware wallet support

## Security

For security concerns, please open a private issue on GitHub.

## Acknowledgments

- Inspired by Ethereum, Solana, and other innovative blockchains
- Built with Rust, Tokio, and other excellent open-source projects
- Thanks to all contributors and the community

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

---

<p align="center">
  <strong>Where Trust is Forged</strong>
</p>

<p align="center">
  Built with ❤️ by the MERKLITH team and contributors
</p>