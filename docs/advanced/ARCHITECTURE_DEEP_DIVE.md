# Advanced Architecture Documentation

Deep technical analysis of MERKLITH's architecture for developers and researchers.

## Table of Contents
1. [Consensus Algorithm Deep Dive](#1-consensus-algorithm-deep-dive)
2. [State Management Internals](#2-state-management-internals)
3. [Virtual Machine Architecture](#3-virtual-machine-architecture)
4. [Networking Protocol](#4-networking-protocol)
5. [Storage Engine](#5-storage-engine)
6. [Cryptographic Primitives](#6-cryptographic-primitives)
7. [Performance Optimizations](#7-performance-optimizations)
8. [Security Architecture](#8-security-architecture)

---

## 1. Consensus Algorithm Deep Dive

### 1.1 PoC Mathematical Model

**Contribution Score Function:**

```
S(v) = α·Stake(v) + β·Attestation(v) + γ·BlockProd(v) + δ·Network(v)

Where:
- α = 0.40 (stake weight coefficient)
- β = 0.30 (attestation coefficient)
- γ = 0.20 (block production coefficient)
- δ = 0.10 (network contribution coefficient)
```

**Normalization:**
```
Stake_norm(v) = min(Stake(v), Stake_max) / Stake_max
Attestation_norm(v) = Attestations(v) / Attestations_max
BlockProd_norm(v) = Blocks(v) / ExpectedBlocks
Network_norm(v) = RelayScore(v) / AverageRelay
```

### 1.2 Committee Selection Algorithm

**VRF-based Selection:**

```python
def select_committee(validators, epoch_seed, target_size):
    """
    Select validators for committee using VRF
    """
    candidates = []
    
    for validator in validators:
        # Compute VRF output
        vrf_proof, vrf_random = validator.compute_vrf(epoch_seed)
        
        # Calculate selection score
        selection_score = vrf_random / validator.total_score
        
        candidates.append({
            'validator': validator,
            'score': selection_score,
            'proof': vrf_proof
        })
    
    # Sort by selection score (lower is better)
    candidates.sort(key=lambda x: x['score'])
    
    # Return top N validators
    return candidates[:target_size]
```

**Verification:**
```python
def verify_selection(validator, epoch_seed, vrf_proof, claimed_random):
    """
    Verify that validator was correctly selected
    """
    # Verify VRF proof
    assert vrf_verify(validator.public_key, epoch_seed, vrf_proof, claimed_random)
    
    # Verify score calculation
    expected_score = claimed_random / validator.total_score
    assert candidate['score'] == expected_score
    
    return True
```

### 1.3 BFT Finality Gadget

**Casper FFG Implementation:**

```
Checkpoint: Every 100th block (epoch boundary)
Justification: 2/3 attestations for checkpoint
Finalization: 2/3 attestations for checkpoint that justifies another
```

**Safety Proof:**

**Theorem**: Two conflicting checkpoints cannot both be finalized.

**Proof by Contradiction:**
1. Assume checkpoints A and B are finalized, A ≠ B
2. For A to finalize: attestations(A) ≥ 2/3 stake
3. For B to finalize: attestations(B) ≥ 2/3 stake
4. By pigeonhole: attestations(A ∩ B) ≥ 1/3 stake
5. But honest validators won't attest to both
6. Therefore: Byzantine ≥ 1/3 stake
7. **Contradiction!** (safety assumes Byzantine < 1/3)

### 1.4 Fork Choice Rule

**LMD GHOST (Latest Message Driven Greedy Heaviest Observed Sub-Tree):**

```python
def lmd_ghost(head, attestations):
    """
    Choose the head of the chain
    """
    while True:
        children = get_children(head)
        if not children:
            return head
        
        # Select child with most attestation weight
        head = max(children, key=lambda c: get_attestation_weight(c, attestations))
    
    return head
```

**Attestation Weight:**
```
weight(block) = Σ stake(v) for v in validators who attested to block
```

---

## 2. State Management Internals

### 2.1 Merkle Patricia Trie

**Node Types:**

```rust
enum TrieNode {
    Empty,
    Leaf {
        key: Nibbles,
        value: Vec<u8>,
    },
    Extension {
        key: Nibbles,
        child: Box<TrieNode>,
    },
    Branch {
        children: [Option<Box<TrieNode>>; 16],
        value: Option<Vec<u8>>,
    },
}
```

**Key Encoding:**
```
address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0
keccak256(address): 0x1234... (32 bytes)
nibbles: [1, 2, 3, 4, ...] (64 nibbles)
```

**Insertion Complexity:**
- Time: O(log n) where n = number of accounts
- Space: O(log n) for proof

### 2.2 State Transitions

**Atomic State Updates:**

```rust
pub struct StateTransition {
    pub pre_state_root: Hash,
    pub post_state_root: Hash,
    pub changes: Vec<StateChange>,
}

pub enum StateChange {
    BalanceUpdate { address: Address, old: U256, new: U256 },
    NonceIncrement { address: Address, old: u64, new: u64 },
    CodeDeploy { address: Address, code: Vec<u8> },
    StorageWrite { address: Address, slot: U256, old: U256, new: U256 },
}
```

**Reversible Execution:**
```rust
impl State {
    pub fn apply_transition(&mut self, 
                           transition: &StateTransition
                          ) -> Result<(), Error> {
        // Apply atomically
        let checkpoint = self.create_checkpoint();
        
        for change in &transition.changes {
            if let Err(e) = self.apply_change(change) {
                // Rollback on error
                self.restore_checkpoint(checkpoint);
                return Err(e);
            }
        }
        
        Ok(())
    }
}
```

### 2.3 State Pruning

**Archive vs Pruned Nodes:**

**Archive Node:**
- Stores all historical states
- Storage: ~100 GB/year
- Use case: Block explorers, auditors

**Pruned Node:**
- Keeps only last N blocks (default: 256)
- Storage: ~15 GB
- Use case: Validators, most users

**Pruning Strategy:**
```rust
pub struct PruningConfig {
    pub history_blocks: u64,      // Keep last N blocks
    pub checkpoint_interval: u64, // Create state snapshot every N blocks
}

impl State {
    pub fn prune_old_states(&mut self, 
                          current_block: u64,
                          config: &PruningConfig) {
        let cutoff = current_block.saturating_sub(config.history_blocks);
        
        // Remove states before cutoff
        self.state_history.retain(|block, _| *block >= cutoff);
    }
}
```

---

## 3. Virtual Machine Architecture

### 3.1 EVM Compatibility Layer

**Opcode Mapping:**

| EVM Opcode | MERKLITH Implementation | Gas Cost |
|------------|---------------------|----------|
| STOP | 0x00 | 0 |
| ADD | wrapping_add | 3 |
| MUL | wrapping_mul | 5 |
| SSTORE | storage insert/update | 20000/5000 |
| SLOAD | storage read | 200 |
| LOG0-LOG4 | event emission | 375 + 8*bytes |
| CALL | cross-contract call | 700 + memory |
| CREATE | contract deployment | 32000 + init_code |

### 3.2 Gas Metering

**Dynamic Gas Calculation:**

```rust
pub struct GasMeter {
    pub used: u64,
    pub limit: u64,
    pub refunded: u64,
    pub schedule: GasSchedule,
}

impl GasMeter {
    pub fn charge_sstore(
        &mut self,
        is_new: bool,
        original_value: U256,
        current_value: U256,
        new_value: U256,
    ) -> Result<(), OutOfGasError> {
        // EIP-1283: Net gas metering
        if current_value == new_value {
            // No-op
            return Ok(());
        }
        
        if original_value == current_value {
            if original_value == U256::ZERO {
                // Clean write to zero
                self.charge(self.schedule.sstore_set)?;
            } else {
                if new_value == U256::ZERO {
                    // Refund for clearing storage
                    self.refund(self.schedule.sstore_refund);
                }
                self.charge(self.schedule.sstore_reset)?;
            }
        } else {
            // Dirty write
            self.charge(self.schedule.sstore_clean)?;
        }
        
        Ok(())
    }
}
```

### 3.3 Precompiled Contracts

**Ed25519 Verification:**
```
Address: 0x0000000000000000000000000000000000000001
Input: [public_key (32 bytes), message (32 bytes), signature (64 bytes)]
Output: [success (1 byte)]
Gas: 1500
```

**BLS Signature Verification:**
```
Address: 0x0000000000000000000000000000000000000002
Input: [aggregate_pubkey (48 bytes), message (32 bytes), aggregate_sig (96 bytes)]
Output: [success (1 byte)]
Gas: 3000
```

**VRF Verification:**
```
Address: 0x0000000000000000000000000000000000000003
Input: [public_key (32 bytes), seed (32 bytes), proof (64 bytes), output (32 bytes)]
Output: [success (1 byte)]
Gas: 2000
```

---

## 4. Networking Protocol

### 4.1 libp2p Integration

**Protocol Stack:**

```
Transport: TCP, WebSocket, QUIC
Security: Noise, TLS 1.3
Multiplexing: Yamux, mplex
Discovery: Kademlia DHT
PubSub: GossipSub
```

**Custom Protocols:**

**/merklith/status/1.0.0:**
```protobuf
message Status {
    uint64 protocol_version = 1;
    uint64 chain_id = 2;
    bytes genesis_hash = 3;
    uint64 best_block_number = 4;
    bytes best_block_hash = 5;
}
```

**/merklith/block/1.0.0:**
```protobuf
message NewBlock {
    bytes block_hash = 1;
    bytes block_data = 2;
}
```

### 4.2 Gossip Protocol

**Transaction Gossip:**
```rust
impl TransactionGossip {
    pub fn propagate(&self, tx_hash: Hash, exclude: Vec<PeerId>) {
        // Select random subset of peers (fanout = 6)
        let peers = self.peer_manager.get_random_peers(6, exclude);
        
        for peer in peers {
            self.send_transaction(peer, tx_hash);
        }
    }
    
    pub fn handle_seen(&mut self, tx_hash: Hash, from: PeerId) {
        // Don't propagate if already seen
        if self.seen_transactions.contains(&tx_hash) {
            return;
        }
        
        self.seen_transactions.insert(tx_hash);
        
        // Propagate to other peers
        self.propagate(tx_hash, vec![from]);
    }
}
```

**Block Propagation:**
- Full blocks sent to validators
- Headers only to light clients
- Compact blocks (tx hashes only) for efficiency

### 4.3 Synchronization

**Fast Sync Algorithm:**
```python
async def fast_sync(node, bootstrap_peers):
    # 1. Download headers
    headers = await download_headers(bootstrap_peers)
    
    # 2. Verify header chain
    verify_header_chain(headers)
    
    # 3. Download state snapshot at pivot point
    pivot = headers[-1024]  # 1024 blocks from head
    state_snapshot = await download_state(pivot.hash)
    
    # 4. Apply blocks after pivot
    for header in headers[-1024:]:
        block = await download_block(header.hash)
        apply_block(state, block)
    
    return state
```

**Header Verification:**
```rust
fn verify_header(header: &Header, parent: &Header) -> Result<(), Error> {
    // Check parent hash
    ensure!(header.parent_hash == parent.hash, "Invalid parent hash");
    
    // Check timestamp
    ensure!(header.timestamp > parent.timestamp, "Invalid timestamp");
    ensure!(header.timestamp <= now() + 15, "Future timestamp");
    
    // Check difficulty
    let expected_difficulty = calculate_difficulty(parent);
    ensure!(header.difficulty == expected_difficulty, "Invalid difficulty");
    
    // Verify signature
    let validator = recover_validator(header)?;
    ensure!(is_active_validator(validator), "Invalid validator");
    
    Ok(())
}
```

---

## 5. Storage Engine

### 5.1 Database Schema

**Key-Value Store (RocksDB):**

```
Column Families:
- blocks: block_hash → block_data
- headers: block_number → block_hash
- transactions: tx_hash → tx_data
- state: address_hash → account_rlp
- metadata: key → value (chain info, config)
- indexes: various lookup indexes
```

**Block Storage:**
```rust
pub struct BlockStorage {
    db: Arc<DB>,
}

impl BlockStorage {
    pub fn put_block(&self, block: &Block) -> Result<(), Error> {
        let hash = block.hash();
        let data = rlp_encode(block);
        
        // Atomic batch write
        let mut batch = WriteBatch::default();
        
        // Store block
        batch.put_cf(
            self.db.cf_handle("blocks")?,
            hash.as_bytes(),
            &data
        );
        
        // Update index
        batch.put_cf(
            self.db.cf_handle("headers")?,
            &block.number.to_be_bytes(),
            hash.as_bytes()
        );
        
        self.db.write(batch)?;
        Ok(())
    }
}
```

### 5.2 State Caching

**Multi-Level Cache:**

```rust
pub struct StateCache {
    // L1: In-memory hot cache
    l1: RwLock<LruCache<Address, Account>>,
    
    // L2: SSD-based warm cache
    l2: Arc< sled::Tree >,
    
    // L3: Cold storage (DB)
    db: Arc<DB>,
}

impl StateCache {
    pub fn get_account(&self, 
                      address: &Address
                     ) -> Result<Option<Account>, Error> {
        // Try L1 cache
        if let Some(account) = self.l1.read().get(address) {
            return Ok(Some(account.clone()));
        }
        
        // Try L2 cache
        if let Some(data) = self.l2.get(address)? {
            let account: Account = bincode::deserialize(&data)?;
            
            // Promote to L1
            self.l1.write().put(*address, account.clone());
            
            return Ok(Some(account));
        }
        
        // Load from DB
        if let Some(account) = self.load_from_db(address)? {
            // Populate caches
            self.populate_cache(address, &account);
            return Ok(Some(account));
        }
        
        Ok(None)
    }
}
```

---

## 6. Cryptographic Primitives

### 6.1 Ed25519 Implementation

**Key Generation:**
```rust
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};

pub fn generate_keypair() -> Keypair {
    let mut csprng = OsRng {};
    Keypair::generate(&mut csprng)
}
```

**Batch Verification:**
```rust
pub fn batch_verify(messages: &[&[u8]], 
                   signatures: &[Signature],
                   public_keys: &[PublicKey]
                  ) -> Result<(), Error> {
    ensure!(messages.len() == signatures.len());
    ensure!(signatures.len() == public_keys.len());
    
    let mut batch = Ed25519Batch::new();
    
    for ((message, signature), public_key) in messages
        .iter()
        .zip(signatures.iter())
        .zip(public_keys.iter())
    {
        batch.add(message, signature, public_key)?;
    }
    
    batch.verify()
}
```

### 6.2 BLS12-381 Implementation

**Signature Aggregation:**
```rust
use blst::min_pk::*;

pub fn aggregate_signatures(signatures: &[Signature]) -> Result<Signature, Error> {
    let sigs: Vec<&Signature> = signatures.iter().collect();
    let aggregate = Signature::aggregate(&sigs, true)?;
    Ok(aggregate.to_signature())
}

pub fn verify_aggregate(public_keys: &[PublicKey],
                       message: &[u8],
                       aggregate_sig: &Signature
                      ) -> Result<(), Error> {
    let pks: Vec<&PublicKey> = public_keys.iter().collect();
    
    let result = aggregate_sig.verify(true, message, &[], &pks, true);
    
    if result == BLST_ERROR::BLST_SUCCESS {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}
```

### 6.3 Merkle Tree Implementation

**Sparse Merkle Tree:**
```rust
pub struct SparseMerkleTree {
    root: Hash,
    nodes: HashMap<Hash, TreeNode>,
    default_nodes: Vec<Hash>,  // Pre-computed default hashes
}

impl SparseMerkleTree {
    pub fn new(depth: usize) -> Self {
        let default_nodes = compute_default_nodes(depth);
        
        Self {
            root: default_nodes[depth],
            nodes: HashMap::new(),
            default_nodes,
        }
    }
    
    pub fn update(&mut self, 
                  index: u64, 
                  value: Hash
                 ) -> Result<(), Error> {
        let path = get_path(index, self.depth);
        let mut current = value;
        
        for (level, direction) in path.iter().enumerate() {
            let sibling = self.get_sibling(index, level);
            
            current = if *direction == 0 {
                hash_pair(current, sibling)
            } else {
                hash_pair(sibling, current)
            };
            
            self.nodes.insert(current, TreeNode::Branch { left, right });
        }
        
        self.root = current;
        Ok(())
    }
}
```

---

## 7. Performance Optimizations

### 7.1 Parallel Execution

**Transaction Parallelization:**
```rust
pub fn execute_transactions_parallel(
    transactions: Vec<Transaction>,
    state: &mut State,
) -> Vec<Receipt> {
    // 1. Build dependency graph
    let dependency_graph = build_dependency_graph(&transactions);
    
    // 2. Identify independent transactions
    let batches = topological_sort_batches(dependency_graph);
    
    // 3. Execute batches in parallel
    let mut receipts = Vec::new();
    
    for batch in batches {
        let batch_receipts: Vec<Receipt> = batch
            .par_iter()
            .map(|tx| execute_transaction(state, tx))
            .collect();
        
        receipts.extend(batch_receipts);
    }
    
    receipts
}
```

**Dependency Detection:**
```rust
fn build_dependency_graph(txs: &[Transaction]) -> Graph {
    let mut graph = Graph::new();
    
    for (i, tx1) in txs.iter().enumerate() {
        for (j, tx2) in txs.iter().enumerate().skip(i + 1) {
            // Check if transactions conflict
            if conflicts(tx1, tx2) {
                graph.add_edge(i, j);
            }
        }
    }
    
    graph
}

fn conflicts(tx1: &Transaction, tx2: &Transaction) -> bool {
    // Same sender (nonce conflict)
    if tx1.from == tx2.from {
        return true;
    }
    
    // Write-write conflict on same storage slot
    if writes_to_same_slot(tx1, tx2) {
        return true;
    }
    
    false
}
```

### 7.2 JIT Compilation

**WASM JIT:**
```rust
pub struct JitCompiler {
    engine: wasmtime::Engine,
    module_cache: RwLock<LruCache<Hash, Module>>,
}

impl JitCompiler {
    pub fn compile(&self, 
                   code: &[u8]
                  ) -> Result<Arc<Module>, Error> {
        let hash = blake3_hash(code);
        
        // Check cache
        if let Some(module) = self.module_cache.read().get(&hash) {
            return Ok(module.clone());
        }
        
        // Compile
        let module = Module::new(&self.engine, code)?;
        let module = Arc::new(module);
        
        // Cache
        self.module_cache.write().put(hash, module.clone());
        
        Ok(module)
    }
}
```

### 7.3 Memory Pooling

**Object Pool Pattern:**
```rust
pub struct ObjectPool<T> {
    pool: Mutex<Vec<T>>,
    create: Box<dyn Fn() -> T + Send>,
    reset: Box<dyn Fn(&mut T) + Send>,
}

impl<T> ObjectPool<T> {
    pub fn acquire(&self) -> PooledObject<T> {
        let obj = self.pool.lock()
            .pop()
            .unwrap_or_else(|| (self.create)());
        
        PooledObject {
            obj: Some(obj),
            pool: &self.pool,
        }
    }
    
    pub fn release(&self, mut obj: T) {
        (self.reset)(&mut obj);
        self.pool.lock().push(obj);
    }
}

// Usage for EVM stack frames
lazy_static! {
    static ref STACK_FRAME_POOL: ObjectPool<StackFrame> = ObjectPool::new(
        || StackFrame::new(),
        |frame| frame.clear(),
    );
}
```

---

## 8. Security Architecture

### 8.1 Threat Model

**Attackers:**
1. **Byzantine Validators**: Control up to 1/3 of stake
2. **Network Adversaries**: Can partition, delay, or reorder messages
3. **Economic Attackers**: Attempt 51% attacks, bribing
4. **Implementation Attackers**: Exploit bugs, overflow, logic errors

**Defenses:**
1. **Consensus**: BFT with slashing
2. **Cryptography**: Proven primitives (Ed25519, BLS, Blake3)
3. **Economics**: High attack cost via stake requirements
4. **Implementation**: Memory-safe Rust, extensive testing, audits

### 8.2 Formal Verification

**Safety Properties:**
```coq
(* No double spending *)
Theorem no_double_spend:
  forall tx1 tx2 state,
    valid_transaction tx1 state ->
    valid_transaction tx2 state ->
    tx1.from = tx2.from ->
    tx1.nonce <> tx2.nonce.

(* State transition consistency *)
Theorem state_transition_deterministic:
  forall state block state1 state2,
    apply_block state block = state1 ->
    apply_block state block = state2 ->
    state1 = state2.
```

**Liveness Properties:**
```coq
(* Eventually include transaction *)
Theorem eventual_inclusion:
  forall tx mempool,
    valid_transaction tx ->
    add_to_mempool mempool tx ->
    eventually (exists block, tx in block.transactions).
```

### 8.3 Fuzzing

**Continuous Fuzzing:**
```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz transaction decoding
    if let Ok(tx) = Transaction::decode(data) {
        // Should not panic
        let _ = verify_transaction(&tx);
    }
    
    // Fuzz block processing
    if let Ok(block) = Block::decode(data) {
        let mut state = State::default();
        // Should not panic
        let _ = apply_block(&mut state, &block);
    }
});
```

**Fuzzing Targets:**
- Transaction decoding/encoding
- Block processing
- State transitions
- P2P message parsing
- RLP encoding/decoding

### 8.4 Penetration Testing

**Test Categories:**
1. **Consensus Attacks**: Double signing, surround votes
2. **Network Attacks**: Eclipse, partitioning, DDoS
3. **Economic Attacks**: Spam, gas manipulation
4. **Cryptographic Attacks**: Signature forgery, key recovery
5. **Implementation Attacks**: Memory leaks, race conditions

**Tools:**
- AFL++ for fuzzing
- KLEE for symbolic execution
- ProVerif for protocol verification
- Custom chaos testing framework

---

## Conclusion

This advanced architecture documentation provides deep technical insight into MERKLITH's design decisions and implementation details. The combination of:

- **Novel consensus** (PoC)
- **Advanced cryptography** (Ed25519, BLS)
- **Optimized execution** (parallel, JIT)
- **Robust security** (formal verification, fuzzing)

creates a blockchain platform that is fast, secure, and sustainable.

For implementation details, see the source code and API reference.

---

**Document Version**: 1.0  
**Last Updated**: February 24, 2026  
**Maintained By**: MERKLITH Core Team