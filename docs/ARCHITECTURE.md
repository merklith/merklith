# MERKLITH Architecture

Technical architecture and system design documentation.

## Table of Contents

- [System Overview](#system-overview)
- [Component Architecture](#component-architecture)
- [Data Flow](#data-flow)
- [Consensus Protocol](#consensus-protocol)
- [Storage Layer](#storage-layer)
- [Network Layer](#network-layer)
- [Virtual Machine](#virtual-machine)
- [Security Model](#security-model)
- [Performance](#performance)
- [Scalability](#scalability)

## System Overview

MERKLITH is a modular blockchain architecture built in Rust, designed for high performance and flexibility.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Application Layer                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │     CLI      │  │    SDKs      │  │   Explorer   │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
└─────────┼─────────────────┼─────────────────┼────────────────┘
          │                 │                 │
          └─────────────────┴─────────────────┘
                            │
┌───────────────────────────┼─────────────────────────────────┐
│                      RPC Layer                             │
│              ┌────────────┴────────────┐                   │
│              │    JSON-RPC Server      │                   │
│              └────────────┬────────────┘                   │
└───────────────────────────┼─────────────────────────────────┘
                            │
┌───────────────────────────┼─────────────────────────────────┐
│                     Node Runtime                           │
│  ┌──────────────┐  ┌──────┴──────┐  ┌──────────────┐       │
│  │  Tx Pool     │  │   Mempool   │  │ Block Builder│       │
│  └──────────────┘  └─────────────┘  └──────┬───────┘       │
└────────────────────────────────────────────┼────────────────┘
                                             │
┌────────────────────────────────────────────┼────────────────┐
│                  Core Engine                               │
│  ┌──────────────┐  ┌──────────────┐  ┌────┴──────────┐     │
│  │ State Machine│  │ Fee Market   │  │  Chain Mgr    │     │
│  └──────────────┘  └──────────────┘  └───────────────┘     │
└─────────────────────────────────────────────────────────────┘
                            │
┌───────────────────────────┼─────────────────────────────────┐
│                  Consensus Layer                           │
│  ┌──────────────┐  ┌──────┴──────┐  ┌──────────────┐       │
│  │     PoC      │  │Attestations │  │   Finality   │       │
│  │   Scoring    │  │   (BLS)     │  │   Gadget     │       │
│  └──────────────┘  └─────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
                            │
┌───────────────────────────┼─────────────────────────────────┐
│                Infrastructure Layer                        │
│  ┌──────────────┐  ┌──────┴──────┐  ┌──────────────┐       │
│  │   Storage    │  │  Network    │  │     VM       │       │
│  │  (JSON/DB)   │  │   (P2P)     │  │  (Bytecode)  │       │
│  └──────────────┘  └─────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

## Component Architecture

### Crate Organization

```
merklith/
├── merklith-types/       # Core domain types
├── merklith-crypto/      # Cryptographic primitives
├── merklith-storage/     # Persistence layer
├── merklith-core/        # Blockchain logic
├── merklith-vm/          # Virtual machine
├── merklith-consensus/   # Consensus protocol
├── merklith-txpool/      # Transaction management
├── merklith-network/     # P2P networking
├── merklith-rpc/         # RPC interface
├── merklith-node/        # Node runtime
├── merklith-cli/         # Command-line tool
└── merklith-governance/  # On-chain governance
```

### merklith-types

Core types implementing the blockchain domain model.

**Key Types**:
- `U256`: 256-bit unsigned integer
- `Address`: 20-byte account address
- `Hash`: 32-byte hash
- `Block`: Block structure
- `Transaction`: Unsigned transaction
- `SignedTransaction`: Signed transaction
- `Receipt`: Transaction receipt

**Design Decisions**:
- Fixed-size arrays for performance
- Custom serialization (Borsh + JSON)
- Zero-copy where possible

### merklith-crypto

Cryptographic primitives optimized for blockchain use.

**Components**:
- **ed25519**: Fast signatures using dalek
- **BLS12-381**: BLS signatures for attestations
- **Blake3**: Fast hashing with parallelization
- **VRF**: Verifiable random functions
- **Merkle**: Merkle tree for data integrity

**Performance**:
- Ed25519 signature: ~50μs
- Blake3 hash: ~100ns per 1KB
- BLS aggregation: O(1) for any number of sigs

### merklith-storage

Persistent storage layer.

**Architecture**:
```
Storage Layer
├── StateDB       # Account state
│   ├── accounts/ # Address → Account
│   └── storage/  # (Address, Slot) → Value
├── BlockStore    # Block data
│   ├── blocks/   # Number → Block
│   └── index/    # Hash → Number
└── MetaStore     # Metadata
    ├── chain/    # Chain info
    └── config/   # Node config
```

**Current Implementation**:
- JSON-based storage (development)
- RocksDB backend exists but is not the default path yet
- In-memory cache layer

### merklith-core

Blockchain core logic.

**State Machine**:
```rust
pub struct State {
    accounts: HashMap<Address, Account>,
    nonce: u64,
    root: Hash,
}

impl State {
    pub fn apply_block(&mut self, block: &Block) -> Result<()>;
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<Receipt>;
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<()>;
}
```

**Block Builder**:
- Transaction selection by gas price
- Priority ordering (FIFO for same price)
- Batch processing for efficiency

### merklith-vm

Byte-code virtual machine.
Current status: the WASM runtime validates module shape and gas bounds, but full WASM execution is still pending engine integration.

**Architecture**:
```
VM Components
├── Runtime
│   ├── Stack (1024 slots max)
│   ├── Memory (expandable)
│   └── Storage (persistent)
├── Gas Meter
│   ├── Static gas per opcode
│   ├── Dynamic gas (SSTORE, etc.)
│   └── Refunds
└── Opcodes
    ├── Arithmetic (ADD, SUB, MUL, DIV)
    ├── Comparison (LT, GT, EQ)
    ├── Stack (PUSH, POP, DUP, SWAP)
    ├── Memory (MLOAD, MSTORE)
    ├── Storage (SLOAD, SSTORE)
    └── Control (JUMP, JUMPI, STOP)
```

**Gas Costs**:
| Operation | Gas Cost |
|-----------|----------|
| ADD/SUB | 3 |
| MUL | 5 |
| DIV/MOD | 5 |
| SLOAD | 200 |
| SSTORE | 20,000 (set), 5,000 (reset) |
| CALL | 700 + memory |

### merklith-consensus

Proof of Contribution consensus.

**PoC Scoring**:
```rust
pub struct PoCScore {
    total: u64,
    block_production: u64,
    attestations: u64,
    relayed_txs: u64,
    discovered_peers: u64,
    data_availability: u64,
}
```

**Attestation Flow**:
1. Block proposed by selected validator
2. Committee attests to block validity
3. BLS signatures aggregated
4. Block finalized after 2/3 attestations

**Validator Selection**:
```
P(validator_i) = score_i / total_score
```

### merklith-txpool

Transaction pool management.

**Pool Structure**:
```
TransactionPool
├── Pending: Vec<Transaction>
│   ├── Sorted by gas price
│   └── Account nonce tracking
├── Queued: Map<Address, Vec<Transaction>>
│   └── Future nonces
└── Config
    ├── max_size: 5000
    └── max_per_account: 100
```

**Validation**:
- Signature verification
- Nonce checking
- Balance sufficient
- Gas limit reasonable

### merklith-network

P2P networking layer.

**Protocol Stack**:
```
Application
├── Block sync
├── Transaction gossip
├── Attestation broadcast
└── Peer discovery

Transport
├── TCP sockets
├── Encryption (Noise)
└── Multiplexing (Yamux)

Discovery
├── Bootstrap nodes
├── Kademlia DHT
└── mDNS (local)
```

**Message Types**:
- `BlockAnnounce`: New block available
- `BlockRequest`: Request block data
- `BlockResponse`: Block data
- `Transaction`: New transaction
- `Attestation`: Validator attestation

### merklith-rpc

JSON-RPC server interface.

**Architecture**:
```
HTTP Server (hyper)
├── Request parsing
├── Method routing
├── Response formatting
└── Error handling

Methods (25+)
├── merklith_chainId, merklith_blockNumber, merklith_getBalance
├── merklith_transfer, merklith_sendSignedTransaction
├── merklith_getBlockByNumber, merklith_getBlockInfo
├── merklith_deployContract, merklith_call
├── merklith_createAttestation (BLS)
└── eth_*/web3_*/net_* compatibility aliases
```

**Concurrency**:
- Async/await (Tokio)
- Connection pooling
- Rate limiting

## Data Flow

### Transaction Lifecycle

```
1. User creates transaction
   ↓
2. Sign with private key
   ↓
3. Submit to node (RPC or P2P)
   ↓
4. Validate (signature, nonce, balance)
   ↓
5. Add to transaction pool
   ↓
6. Validator includes in block
   ↓
7. Execute transaction
   ↓
8. Update state
   ↓
9. Broadcast block
   ↓
10. Other validators attest
    ↓
11. Block finalized
```

### Block Production Flow

```
Validator (every 2-6 seconds)
├── 1. Check if proposer
│   └── VRF-based selection
├── 2. Select transactions
│   ├── Get from pool
│   ├── Sort by gas price
│   └── Apply until full
├── 3. Build block
│   ├── Create header
│   ├── Execute transactions
│   ├── Calculate state root
│   └── Sign block
├── 4. Broadcast block
│   ├── Send to peers
│   └── Announce availability
└── 5. Collect attestations
    ├── Wait for signatures
    ├── Aggregate BLS sigs
    └── Finalize block
```

## Consensus Protocol

### Proof of Contribution (PoC)

**Concept**: Reward actual network contributions, not just stake.

**Contribution Types**:

| Type | Points | Weight | Verification |
|------|--------|--------|--------------|
| Block Production | +100 | High | On-chain |
| Attestation | +10 | Medium | Signature |
| Transaction Relay | +1 | Low | Network trace |
| Peer Discovery | +1 | Low | P2P protocol |
| Data Availability | +1 | Low | Sampling |

**Score Decay**:
```
score_new = score_old * decay_factor
```
Prevents old validators from dominating indefinitely.

**Proposer Selection**:
```rust
pub fn select_proposer(scores: &[PoCScore], seed: &[u8]) -> ValidatorIndex {
    let total: u64 = scores.iter().map(|s| s.total).sum();
    let random = vrf_output_to_u64(seed);
    let target = random % total;
    
    let mut cumulative = 0;
    for (i, score) in scores.iter().enumerate() {
        cumulative += score.total;
        if cumulative >= target {
            return i;
        }
    }
    0
}
```

### BLS Attestations

**Aggregation**: Combine 100 signatures into 1

```
Individual signatures: 100 × 96 bytes = 9,600 bytes
Aggregated signature: 1 × 96 bytes = 96 bytes
Compression: 99% reduction
```

**Security**:
- 2/3+ attestations for finality
- Slashing for double-signing
- Inactivity leak for offline validators

## Storage Layer

### State Storage

**Account Structure**:
```rust
pub struct Account {
    pub balance: U256,
    pub nonce: u64,
    pub code_hash: Hash,
    pub storage_root: Hash,
}
```

**State Root Calculation**:
```
state_root = merkle_root(all_accounts)
```

**Storage Layout**:
- Key: `sha3(address)`
- Value: RLP-encoded account

### Block Storage

**Indexing**:
- Primary: Block number → Block
- Secondary: Block hash → Block number

**Pruning**:
- Keep last 10,000 blocks fully
- Keep headers for older blocks
- Archive mode: Keep everything

## Network Layer

### P2P Protocol

**Connection Management**:
- Max 50 peers by default
- Quality-based peer scoring
- Automatic reconnection

**Message Propagation**:
- Gossip protocol for transactions
- Direct request for blocks
- Epidemic broadcast for attestations

**Sync Protocol**:
1. Discover peers with higher block number
2. Request block headers (backward)
3. Download full blocks
4. Verify and apply

### Security

**Transport**:
- Noise protocol for encryption
- Perfect forward secrecy
- Peer authentication

**DoS Protection**:
- Rate limiting per peer
- Message size limits
- Connection quotas

## Virtual Machine

### Bytecode Format

**Instruction Set**:
```
0x01 STOP       - Halts execution
0x02 ADD        - Pop 2, push sum
0x03 SUB        - Pop 2, push difference
0x04 MUL        - Pop 2, push product
0x05 DIV        - Pop 2, push quotient
0x10 LT         - Pop 2, push 1 if less
0x11 GT         - Pop 2, push 1 if greater
0x14 EQ         - Pop 2, push 1 if equal
0x60 PUSH1      - Push 1-byte value
0x61 PUSH2      - Push 2-byte value
...
0xF0 CREATE     - Create new contract
0xF1 CALL       - Message call
0xF4 DELEGATECALL - Delegate call
```

### Execution Model

**Stack Machine**:
- 1024 max stack depth
- 256-bit word size
- Big-endian byte order

**Memory Model**:
- Byte-addressable
- Expandable (gas cost per word)
- Volatile (cleared after call)

**Storage Model**:
- Key-value (256-bit → 256-bit)
- Persistent
- Expensive to write

## Security Model

### Threat Model

**Assumptions**:
- 2/3+ validators are honest
- Network is partially synchronous
- Adversary has < 1/3 of contribution score

**Attacks**:
- **Double spending**: Prevented by BFT finality
- **Nothing at stake**: Prevented by slashing
- **Long-range**: Prevented by weak subjectivity
- **Censorship**: Prevented by rotating proposers

### Slashing Conditions

**Slashable Offenses**:
1. **Double signing**: Two different blocks at same height
2. **Surround voting**: Attesting to conflicting blocks
3. **Invalid block**: Proposing block with invalid state transition

**Penalties**:
- Loss of all contribution score
- Temporary ban (1 week)
- Reputation damage

### Cryptographic Security

**Signature Schemes**:
- Ed25519: 128-bit security
- BLS12-381: 128-bit security
- VRF: 128-bit security

**Hash Function**:
- Blake3: 256-bit security
- Collision resistant
- Preimage resistant

## Performance

### Benchmarks

**Transaction Processing**:
```
Simple transfer: ~100μs
Contract call: ~500μs
Contract creation: ~1ms
```

**Block Processing**:
```
Empty block: ~2ms
Block with 100 txs: ~15ms
Block with 1000 txs: ~120ms
```

**Signature Verification**:
```
Ed25519 verify: ~50μs
BLS verify (single): ~2ms
BLS verify (aggregated): ~3ms
```

**Network**:
```
Block propagation (1MB): < 1s
Transaction gossip: < 500ms
Sync speed: ~100 blocks/s
```

### Throughput

**Current**:
- TPS: ~100 simple transfers
- Block time: 2-6 seconds
- Block size: ~1MB

**Target**:
- TPS: ~1000
- Block time: 1 second
- Block size: ~10MB

## Scalability

### Horizontal Scaling

**Sharding Roadmap**:
- Phase 1: Data availability sampling
- Phase 2: Execution sharding
- Phase 3: Cross-shard transactions

**Expected TPS**:
- Single chain: ~1000 TPS
- 64 shards: ~64,000 TPS

### Layer 2 Support

**State Channels**:
- Off-chain transaction batches
- On-chain settlement
- Instant finality

**Rollups**:
- Optimistic rollups
- ZK rollups (future)

## Future Improvements

### Short Term (6 months)
- [ ] Make RocksDB the default storage backend
- [ ] Full P2P sync
- [ ] Full WebAssembly execution engine integration
- [ ] Light client support

### Medium Term (1 year)
- [ ] Sharding implementation
- [ ] Cross-chain bridges
- [ ] Privacy features (zk-SNARKs)
- [ ] Governance DAO

### Long Term (2+ years)
- [ ] Quantum-resistant cryptography
- [ ] Formal verification
- [ ] Hardware wallet integration
- [ ] Mobile full nodes

## See Also

- [CLI Guide](CLI_GUIDE.md) - User documentation
- [API Documentation](API.md) - RPC reference
- [Explorer Guide](EXPLORER.md) - TUI explorer
