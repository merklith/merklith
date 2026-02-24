# Changelog

All notable changes to the MERKLITH blockchain project will be documented in this file.

## [0.1.0] - 2026-02-23

### Added

#### CLI Tool (merklith-cli)
- **Comprehensive command-line interface** for blockchain interaction
- Wallet management with encrypted keystore (AES-256-GCM + Argon2id)
  - `wallet create` - Create new encrypted wallet
  - `wallet import` - Import from private key
  - `wallet export` - Export private key (with warnings)
  - `wallet remove` - Delete wallet from keystore
  - `wallet list` - Display all wallets
- Account operations
  - `account balance` - Query single account
  - `account balances` - List all wallet balances
  - `account nonce` - Get transaction count
- Transaction commands
  - `tx send` - Send signed transactions
  - `tx get` - Get transaction details
  - `tx wait` - Wait for confirmation
- Query commands
  - `query block` - Get block by number
  - `query block-hash` - Get block by hash
  - `query block-number` - Get latest block
  - `query chain-id` - Get chain ID
  - `query gas-price` - Get current gas price
  - `query node-info` - Get node information
- Contract interaction
  - `contract deploy` - Deploy smart contracts
  - `contract call` - Call contract (read-only)
  - `contract send` - Send transaction to contract
  - `contract code` - Get contract bytecode
- Configuration management
  - `config show` - Display current config
  - `config set` - Set config value
  - `config get` - Get config value
  - `config reset` - Reset to defaults

#### TUI Block Explorer
- **Interactive terminal-based blockchain explorer**
- Real-time block monitoring with auto-refresh
- Multiple views:
  - Blocks list with hash, timestamp, transaction count
  - Block detail view with full JSON data
  - Transactions list with from/to/value
  - Accounts view (placeholder)
  - Search interface
  - Help screen with keyboard shortcuts
- Vim-style navigation (j/k/h/l)
- Keyboard shortcuts:
  - `b` - Blocks view
  - `t` - Transactions view
  - `a` - Accounts view
  - `s` - Search
  - `r` - Refresh data
  - `h` - Help
  - `q/Esc` - Quit

#### Core Blockchain
- Block production with blake3 hash chains
- JSON-based state persistence
- 8 pre-funded genesis accounts (1,000,000 ANV each)
- Parent hash verification for block sync

#### Cryptography
- Ed25519 signatures for transactions
- BLS12-381 aggregate signatures for committee attestations
- blake3 hashing throughout (not keccak256)
- Bech32m address encoding

#### Consensus
- Proof of Contribution (PoC) scoring system
- Contribution types: BlockProduction, Attestation, TransactionRelay, PeerDiscovery, DataAvailability
- Score decay (10% per 1000 blocks)
- Attestation pool with finality threshold
- Weighted proposer selection by PoC score

#### Virtual Machine
- Bytecode interpreter with 15+ opcodes
- Memory operations (MLOAD, MSTORE)
- Storage operations (SLOAD, SSTORE)
- Control flow (JUMP, JUMPI, STOP, RETURN)
- Contract creation and calls

#### RPC API (25 methods)
- Chain queries: `merklith_chainId`, `merklith_blockNumber`, `merklith_getBlockInfo`, `merklith_getBlockByNumber`, `merklith_getBlockChain`, `merklith_getCurrentBlockHash`, `merklith_getChainStats`
- Accounts: `merklith_getBalance`, `merklith_getNonce`, `merklith_accounts`, `merklith_createWallet`
- Transactions: `merklith_transfer`, `merklith_sendRawTransaction`, `merklith_sendSignedTransaction`, `merklith_signAndSendTransaction`, `merklith_getTransactionByHash`
- Gas: `merklith_gasPrice`, `merklith_estimateGas`
- Contracts: `merklith_deployContract`, `merklith_getCode`, `merklith_getStorageAt`, `merklith_call`
- Consensus: `merklith_createAttestation`
- Utility: `merklith_version`, `merklith_syncing`, `web3_clientVersion`

#### Networking
- TCP-based P2P peer connections
- Block broadcast with parent hash
- Network event handling
- Peer discovery framework

#### Types
- Custom U256 implementation with proper Ord comparison
- Address type with Bech32m support
- Hash type (32 bytes, blake3)
- Transaction and SignedTransaction types
- Block and BlockHeader types

### Fixed
- U256 Ord comparison was comparing little-endian limbs incorrectly
- State machine lock scope issues during persistence
- Balance comparison for large transfers

### Test Coverage
- merklith-types: 84 tests
- merklith-crypto: 38 tests
- merklith-consensus: 9 tests
- merklith-core: 14 tests
- merklith-vm: 27 tests
- merklith-storage: 8 tests (NEW)
- merklith-txpool: 10 tests (NEW)
- merklith-rpc: 15 tests (NEW)
- merklith-governance: 39 tests
- **Total: 244 tests passing**

## Technical Specifications

### Chain Parameters
| Parameter | Value |
|-----------|-------|
| Chain ID | 1337 (configurable) |
| Block Time | 2 seconds |
| Gas Limit | 30,000,000 |
| Native Token | ANV |
| Smallest Unit | Spark (10^-18 ANV) |
| Initial Supply | 8,000,000 ANV |

### Cryptographic Choices
| Purpose | Algorithm |
|---------|-----------|
| Hashing | blake3 |
| Transaction Signatures | ed25519 |
| Committee Attestations | BLS12-381 |
| Address Encoding | Bech32m |

### Project Structure
- 15 Rust crates
- Rust SDK
- TypeScript SDK (partial)
- Smart contracts (framework)
