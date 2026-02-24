# MERKLITH Technical Documentation - Intermediate Level

## Table of Contents
1. [System Architecture](#1-system-architecture)
2. [Proof of Contribution Consensus](#2-proof-of-contribution-consensus)
3. [Transaction Flow](#3-transaction-flow)
4. [State Management](#4-state-management)
5. [Block Production](#5-block-production)
6. [Cryptography](#6-cryptography)
7. [RPC API](#7-rpc-api)
8. [Security Features](#8-security-features)
9. [Scalability](#9-scalability)
10. [DevOps & Monitoring](#10-devops--monitoring)

---

## 1. System Architecture

### 1.1 Layer Structure

```
┌─────────────────────────────────────┐
│         Application Layer           │
│  (CLI, Web Wallet, Block Explorer) │
├─────────────────────────────────────┤
│         RPC Layer                   │
│  (JSON-RPC API, WebSocket)         │
├─────────────────────────────────────┤
│         Consensus Layer             │
│  (PoC Engine, Validator Set)       │
├─────────────────────────────────────┤
│         Execution Layer             │
│  (VM, State Machine, EVM-compat)   │
├─────────────────────────────────────┤
│         Storage Layer               │
│  (State DB, Block Store, Trie)     │
├─────────────────────────────────────┤
│         Network Layer               │
│  (P2P, Gossip, Sync)               │
└─────────────────────────────────────┘
```

### 1.2 Crate Organization

| Crate | Purpose | Importance |
|-------|---------|------------|
| `merklith-types` | Core data types (Address, U256, Hash) | ⭐⭐⭐⭐⭐ |
| `merklith-crypto` | Cryptographic operations | ⭐⭐⭐⭐⭐ |
| `merklith-core` | State machine, block production | ⭐⭐⭐⭐⭐ |
| `merklith-consensus` | PoC consensus, validator management | ⭐⭐⭐⭐⭐ |
| `merklith-vm` | Smart contract execution | ⭐⭐⭐⭐ |
| `merklith-storage` | Persistent data storage | ⭐⭐⭐⭐ |
| `merklith-rpc` | JSON-RPC API | ⭐⭐⭐⭐ |
| `merklith-network` | P2P communication | ⭐⭐⭐ |
| `merklith-txpool` | Transaction pool | ⭐⭐⭐ |
| `merklith-cli` | Command line interface | ⭐⭐ |

---

## 2. Proof of Contribution (PoC) Consensus

### 2.1 Why PoC?

**Problems with PoW (Proof of Work):**
- High energy consumption (Bitcoin uses 150 TWh/year)
- Centralization (large mining pools dominate)
- Slow transaction confirmation (10+ minutes)

**Problems with PoS (Proof of Stake):**
- "Rich get richer" problem
- High entry barrier (32 ETH minimum)
- Nothing-at-stake problem

**PoC Advantages:**
- Low energy (no CPU mining required)
- Fair distribution (everyone can contribute)
- Fast finality (6 seconds)
- Multi-dimensional rewards (not just stake, all contributions)

### 2.2 PoC Score Calculation

```rust
// Contribution Score Formula
score = (stake_weight * 0.4) + 
        (attestation_count * 10 * 0.3) + 
        (block_production_count * 100 * 0.2) + 
        (peer_relay_count * 5 * 0.1)
```

**Parameters:**
- `stake_weight`: Amount staked by validator (max 40% impact)
- `attestation_count`: Number of blocks validated (10 points each)
- `block_production_count`: Blocks produced (100 points each)
- `peer_relay_count`: Transactions relayed (5 points each)

### 2.3 Validator Lifecycle

```
1. Register → 2. Active → 3. Contributing → 4. Reward/Slash → 5. Exit
     ↓            ↓              ↓                  ↓            ↓
  Min stake    Proposing      Attesting         Incentives   Unbonding
  (1000 ANV)   Blocks         Voting            /Penalties   Period
```

**Unbonding Period:** 14 days (for slashing risk)

---

## 3. Transaction Flow

### 3.1 Transaction Structure

```rust
pub struct Transaction {
    pub chain_id: u64,        // Network ID (MERKLITH: 1337)
    pub nonce: u64,           // Sender's transaction count
    pub to: Option<Address>,  // Recipient (None = contract creation)
    pub value: U256,          // Amount to send (wei)
    pub gas_limit: u64,       // Maximum gas limit
    pub max_fee_per_gas: U256, // Base gas fee
    pub max_priority_fee: U256, // Priority fee (tip)
    pub data: Vec<u8>,        // Contract call data
}
```

### 3.2 Transaction Validation Steps

1. **Format Check:** Is the transaction struct valid?
2. **Signature Verification:** Is Ed25519 signature valid?
3. **Nonce Check:** Is this the expected nonce?
4. **Balance Check:** Does sender have enough balance?
5. **Gas Check:** Is gas limit sufficient?
6. **Replay Protection:** Has this been processed before?

### 3.3 Transaction Pool (Mempool)

```rust
pub struct TxPool {
    pending: HashMap<Hash, Transaction>,  // Pending transactions
    queued: HashMap<Address, Vec<Transaction>>, // By nonce
    max_size: usize,  // Maximum pool size (10,000 tx)
}
```

**Gas Price Strategy:**
- Higher gas price = faster processing
- EIP-1559 dynamic base fee adjustment
- Priority fee goes to validator

---

## 4. State Management

### 4.1 Account Model

MERKLITH uses Ethereum-compatible **Account Model**:

```rust
pub struct Account {
    pub balance: U256,      // Balance (wei)
    pub nonce: u64,         // Transaction count
    pub code: Vec<u8>,      // Smart contract code (if any)
    pub storage: HashMap<U256, U256>, // Contract storage
}
```

**EOA (Externally Owned Account):** Normal user wallets  
**Contract Account:** Smart contract addresses

### 4.2 State Transition Function

```rust
// Apply block to transition state
fn apply_block(state: &mut State, block: &Block) -> Result<(), Error> {
    // 1. Distribute block rewards
    distribute_block_rewards(state, block)?;
    
    // 2. Apply transactions sequentially
    for tx in &block.transactions {
        apply_transaction(state, tx)?;
    }
    
    // 3. Burn gas fees
    burn_gas_fees(state, block)?;
    
    // 4. Reward validators
    reward_validators(state, block)?;
    
    Ok(())
}
```

### 4.3 State Root Calculation

All accounts stored in Merkle Patricia Trie:

```
State Root = MerkleRoot([
    (address1, account1_hash),
    (address2, account2_hash),
    ...
])
```

**Benefits:**
- Data integrity (entire state in one hash)
- Light client support (Merkle proofs)
- Efficient updates (O(log n))

---

## 5. Block Production

### 5.1 Block Structure

```rust
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<Receipt>,
}

pub struct BlockHeader {
    pub number: u64,           // Block number
    pub hash: Hash,            // Block hash
    pub parent_hash: Hash,     // Previous block hash
    pub timestamp: u64,        // Unix timestamp
    pub state_root: Hash,      // State root hash
    pub tx_root: Hash,         // Transaction root (Merkle)
    pub receipts_root: Hash,   // Receipt root hash
    pub gas_used: u64,         // Total gas used
    pub gas_limit: u64,        // Block gas limit
    pub validator: Address,    // Block producer
    pub signature: Signature,  // Validator signature
}
```

### 5.2 Block Production Process

```rust
fn produce_block(
    &self,
    parent: &BlockHeader,
    transactions: Vec<Transaction>,
) -> Result<Block, Error> {
    // 1. Determine block number
    let number = parent.number + 1;
    
    // 2. Clone state (sandbox)
    let mut state = self.state.clone();
    
    // 3. Apply transactions
    let mut receipts = Vec::new();
    for tx in transactions {
        let receipt = apply_transaction(&mut state, &tx)?;
        receipts.push(receipt);
    }
    
    // 4. Calculate state root
    let state_root = state.compute_root();
    
    // 5. Build block header
    let header = BlockHeader {
        number,
        parent_hash: parent.hash,
        timestamp: current_timestamp(),
        state_root,
        // ... other fields
    };
    
    // 6. Sign block
    let signature = self.validator_key.sign(&header.hash());
    
    Ok(Block { header, transactions, receipts })
}
```

### 5.3 Block Timing

**Target Block Time:** 6 seconds  
**Epoch Length:** 100 blocks (10 minutes)  
**Difficulty Adjustment:** Every epoch

**Formula:**
```
if actual_time > target_time:
    decrease_difficulty()
else:
    increase_difficulty()
```

---

## 6. Cryptography

### 6.1 Ed25519 Signatures

**Why Ed25519 (instead of secp256k1)?**
- Faster verification
- More secure (timing attack resistant)
- Compact signatures (64 bytes)

**Signature Flow:**
```rust
// 1. Create transaction hash
let tx_hash = blake3_hash(transaction_bytes);

// 2. Sign with private key
let signature = ed25519_sign(private_key, tx_hash);

// 3. Broadcast to network
broadcast(transaction, signature);

// 4. Validator verifies
assert!(ed25519_verify(public_key, tx_hash, signature));
```

### 6.2 BLS Aggregation

Combine multiple signatures into one:

```rust
// Aggregate 100 validator signatures
let aggregate_sig = bls_aggregate(
    &[validator1_sig, validator2_sig, ..., validator100_sig]
);

// Verify all with single check
assert!(bls_verify_aggregate(
    &[validator1_pk, validator2_pk, ..., validator100_pk],
    message,
    aggregate_sig
));
```

**Benefits:**
- Store 1 signature instead of 100
- O(1) verification instead of O(n)
- Smaller block size

---

## 7. RPC API

### 7.1 JSON-RPC Endpoints

**Basic Network Info:**
```bash
# Get Chain ID
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_chainId","params":[],"id":1}'

# Get current block number
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"merklith_blockNumber","params":[],"id":1}'
```

**Balance Query:**
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"merklith_getBalance",
    "params":["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"],
    "id":1
  }'
```

**Transfer:**
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"merklith_transfer",
    "params":[
      "0xSender...",
      "0xRecipient...",
      "0xDE0B6B3A7640000",
      "0x0",
      "0xSignature...",
      "0xPublicKey..."
    ],
    "id":1
  }'
```

### 7.2 WebSocket Subscriptions

**Real-time Notifications:**
```javascript
const ws = new WebSocket('ws://localhost:8546');

// Listen for new blocks
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  id: 1,
  method: 'eth_subscribe',
  params: ['newHeads']
}));

ws.onmessage = (event) => {
  const block = JSON.parse(event.data);
  console.log('New block:', block.number);
};
```

---

## 8. Security Features

### 8.1 Slashing Conditions

Validators are penalized for bad behavior:

**Double Signing:**
- Signing two different blocks at same height
- Penalty: 100% of stake slashed

**Surround Vote:**
- Inconsistent attestations
- Penalty: 50% of stake slashed

**Downtime:**
- Offline for 100+ blocks
- Penalty: Small stake loss

### 8.2 Replay Protection

```rust
pub struct ReplayProtection {
    seen_nonces: HashMap<Address, u64>,
    ttl: Duration,  // 1 hour
}

impl ReplayProtection {
    pub fn check(&mut self, tx: &Transaction) -> Result<(), SecurityError> {
        let last_nonce = self.seen_nonces.get(&tx.from).copied().unwrap_or(0);
        
        if tx.nonce != last_nonce {
            return Err(SecurityError::InvalidNonce);
        }
        
        self.seen_nonces.insert(tx.from, last_nonce + 1);
        Ok(())
    }
}
```

### 8.3 Rate Limiting

```rust
pub struct RateLimiter {
    requests: HashMap<IpAddr, Vec<Instant>>,
    limit: usize,  // 100 requests
    window: Duration,  // 1 minute
}
```

---

## 9. Scalability

### 9.1 Sharding Plan (Future)

```
Shard 0: EOA Accounts
Shard 1: Smart Contracts
Shard 2: DeFi Protocols
Shard 3: NFTs
...
```

**Cross-Shard Communication:**
- Asynchronous message passing
- State root aggregation
- Validator committee rotation

### 9.2 Layer 2 Integration

**ZK-Rollup Support:**
- SNARK/STARK proof verification
- L2 → L1 bridge
- Data availability sampling

---

## 10. DevOps & Monitoring

### 10.1 Running a Node

```bash
# Production node
./merklith-node \
  --rpc-port 8545 \
  --p2p-port 30303 \
  --validator \
  --chain-id 1337 \
  --data-dir /var/merklith/data \
  --log-level info

# Docker
docker run -p 8545:8545 -v merklith-data:/data merklith/node:latest \
  --validator \
  --chain-id 1337
```

### 10.2 Metrics

**Prometheus Metrics:**
```
merklith_block_height
merklith_transactions_total
merklith_gas_used
merklith_validators_active
merklith_peer_count
merklith_memory_usage
```

### 10.3 Monitoring

**Grafana Dashboard:**
- Block production time
- Transaction throughput
- Validator performance
- Network health

---

## Next Steps

- Read the Advanced Architecture documentation
- Check the API Reference
- Read the Whitepaper