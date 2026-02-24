//! State Machine - Real blockchain state transitions with persistence

use merklith_types::{Address, U256, Hash, Transaction};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::str::FromStr;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Block production result
#[derive(Debug, Clone)]
pub struct BlockProductionResult {
    pub block_number: u64,
    pub block_hash: [u8; 32],
    pub transactions_count: usize,
    pub validator_reward: U256,
}

/// State machine errors
#[derive(Debug, Clone)]
pub enum StateError {
    InsufficientBalance,
    InvalidNonce,
    InvalidTransaction(String),
    InvalidBlock(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::InsufficientBalance => write!(f, "Insufficient balance"),
            StateError::InvalidNonce => write!(f, "Invalid nonce"),
            StateError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            StateError::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
        }
    }
}

impl std::error::Error for StateError {}

/// Simple block header for chain tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: [u8; 32],
    pub parent_hash: [u8; 32],
    pub timestamp: u64,
    pub tx_count: usize,
}

/// Account state in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub balance: String,  // hex string
    pub nonce: u64,
    pub code: Vec<u8>,
    pub storage: HashMap<String, String>,  // hex strings
}

impl Default for Account {
    fn default() -> Self {
        Self {
            balance: "0x0".to_string(),
            nonce: 0,
            code: vec![],
            storage: HashMap::new(),
        }
    }
}

impl Account {
    pub fn get_balance(&self) -> U256 {
        U256::from_str(&self.balance).unwrap_or(U256::ZERO)
    }
    
    pub fn set_balance(&mut self, balance: U256) {
        // U256's LowerHex already adds 0x prefix
        self.balance = format!("{:x}", balance);
    }
}

/// Persistent state
#[derive(Debug, Serialize, Deserialize, Default)]
struct StateData {
    accounts: HashMap<String, Account>,
    block_number: u64,
    #[serde(default)]
    block_hash: String,
    total_supply: String,
    #[serde(default)]
    blocks: Vec<BlockInfo>,
}

/// Blockchain state with persistence
#[derive(Debug)]
pub struct State {
    accounts: RwLock<HashMap<Address, Account>>,
    block_number: RwLock<u64>,
    block_hash: RwLock<Hash>,
    total_supply: RwLock<U256>,
    blocks: RwLock<Vec<BlockInfo>>,
    path: PathBuf,
}

impl State {
    pub fn new() -> Self {
        Self::with_path(PathBuf::from("./data/state"))
    }
    
    pub fn with_path(path: PathBuf) -> Self {
        let mut accounts = HashMap::new();
        
        // Devnet: 8 pre-funded accounts with 1,000,000 MERK each
        let genesis_accounts: Vec<&str> = vec![
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
            "0x8ba1f109551bD432803012645Ac136ddd64DBA72",
            "0xdD870fA1b7C4700F2BD7f44238821C26f7392148",
            "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B",
            "0x1aB489E589De6E2F9c9b6B9e2F2b1a4c3d5E6F78",
            "0x2Bc5901A6E4984628Bf12C539f06D5b3369eD0C1",
            "0x3Cd601A7E5985739Bf13D54A107d5b4479fE1D2E",
            "0x4DE710A8E6A96849Cf15D54B208e6C548aF2E3F4",
        ];
        
        // 1,000,000 MERK in Sparks (1 MERK = 10^18 Spark)
        let initial_balance = U256::from(1_000_000u128) * U256::from(1_000_000_000_000_000_000u128);
        let balance_hex = format!("{:x}", initial_balance);  // Without 0x prefix, LowerHex adds it
        
        for addr in genesis_accounts {
            if let Ok(address) = parse_address(addr) {
                accounts.insert(address, Account {
                    balance: balance_hex.clone(),
                    nonce: 0,
                    code: vec![],
                    storage: HashMap::new(),
                });
            }
        }
        
        let state = Self {
            accounts: RwLock::new(accounts),
            block_number: RwLock::new(0),
            block_hash: RwLock::new(Hash::ZERO),
            total_supply: RwLock::new(initial_balance * U256::from(8u64)),
            blocks: RwLock::new(Vec::new()),
            path,
        };
        
        // Try to load from disk
        if let Err(e) = state.load() {
            tracing::info!("Could not load state: {}, using genesis", e);
            // Create genesis block
            state.add_genesis_block();
        }
        
        state
    }
    
    fn add_genesis_block(&self) {
        let genesis = BlockInfo {
            number: 0,
            hash: [0u8; 32],
            parent_hash: [0u8; 32],
            timestamp: 0,
            tx_count: 0,
        };
        self.blocks.write().push(genesis);
    }
    
    /// Get account balance
    pub fn balance(&self, address: &Address) -> U256 {
        let accounts = self.accounts.read();
        accounts.get(address).map(|a| a.get_balance()).unwrap_or(U256::ZERO)
    }
    
    /// Get account nonce
    pub fn nonce(&self, address: &Address) -> u64 {
        let accounts = self.accounts.read();
        accounts.get(address).map(|a| a.nonce).unwrap_or(0)
    }
    
    /// Transfer tokens between accounts
    pub fn transfer(&self, from: &Address, to: &Address, amount: U256) -> Result<Hash, String> {
        let mut accounts = self.accounts.write();
        
        // Get sender state in a single read to ensure consistency
        let (sender_balance, sender_nonce) = accounts.get(from)
            .map(|a| (a.get_balance(), a.nonce))
            .unwrap_or((U256::ZERO, 0));
        
        // Check balance
        if sender_balance < amount {
            return Err(format!("Insufficient balance: have {}, need {}", sender_balance, amount));
        }
        
        // Compute tx hash before modifying
        let new_nonce = sender_nonce + 1;
        let tx_hash = self.compute_tx_hash(from, to, amount, new_nonce);
        
        // Update sender
        if let Some(sender) = accounts.get_mut(from) {
            sender.set_balance(sender_balance - amount);
            sender.nonce = new_nonce;
        }
        
        // Get receiver balance AFTER sender update (from updated HashMap)
        let receiver_balance = accounts.get(to)
            .map(|a| a.get_balance())
            .unwrap_or(U256::ZERO);
        
        // Update receiver
        if let Some(receiver) = accounts.get_mut(to) {
            receiver.set_balance(receiver_balance + amount);
        } else {
            accounts.insert(*to, Account {
                balance: format!("{:x}", amount),
                nonce: 0,
                code: vec![],
                storage: HashMap::new(),
            });
        }
        
        // Persist to disk BEFORE releasing lock to prevent race conditions
        if let Err(e) = self.persist() {
            drop(accounts);
            return Err(format!("Transfer succeeded but failed to persist state: {}", e));
        }
        
        drop(accounts);
        
        Ok(tx_hash)
    }
    
    /// Get current block number
    pub fn block_number(&self) -> u64 {
        *self.block_number.read()
    }
    
    /// Get current block hash
    pub fn block_hash(&self) -> Hash {
        *self.block_hash.read()
    }
    
    /// Increment block number (called when block is produced)
    /// Returns the new block hash
    pub fn increment_block(&self) -> [u8; 32] {
        let (new_hash, _block_info) = {
            let mut block = self.block_number.write();
            let mut hash = self.block_hash.write();
            let mut blocks = self.blocks.write();
            
            *block += 1;
            let parent = *hash;
            
            // Compute new block hash using blake3
            let new_hash = self.compute_block_hash(*block, parent.as_bytes());
            *hash = Hash::from_bytes(new_hash);
            
            // Store block info
            let block_info = BlockInfo {
                number: *block,
                hash: new_hash,
                parent_hash: *parent.as_bytes(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                tx_count: 0,
            };
            blocks.push(block_info.clone());
            
            (new_hash, block_info)
        };
        
        // Persist (outside of lock scope)
        let _ = self.persist();
        
        new_hash
    }
    
    /// Produce a new block with reward for the validator
    /// 
    /// Block reward structure:
    /// - Base reward: Fixed amount for producing a block
    /// - Tx fees: Variable based on transaction gas used
    /// - Activity bonus: Extra reward if network is active (has transactions)
    /// 
    /// Strategy:
    /// - Regular blocks (with txs): Full reward + fees
    /// - Hourly heartbeat blocks: Base reward (even if empty) for security
    /// - Skip empty blocks between heartbeats: No reward, save space
    pub fn produce_block(
        &self,
        validator: &Address,
        transactions: Vec<Transaction>,
        is_heartbeat: bool,
    ) -> Result<BlockProductionResult, StateError> {
        // Acquire write lock early to prevent race conditions
        let mut block_number_guard = self.block_number.write();
        let block_number = *block_number_guard + 1;
        
        // Calculate rewards
        let base_reward = U256::from(2_000_000_000_000_000_000u128); // 2 MERK
        
        // Transaction fees calculation (using max_fee_per_gas) with overflow protection
        let tx_fees: U256 = transactions.iter()
            .filter_map(|tx| {
                tx.max_fee_per_gas.checked_mul(&U256::from(tx.gas_limit))
            })
            .fold(U256::ZERO, |acc, fee| acc.saturating_add(&fee));
        
        // Activity bonus: Extra 1 MERK if we have transactions
        let activity_bonus = if !transactions.is_empty() {
            U256::from(1_000_000_000_000_000_000u128) // 1 MERK bonus
        } else {
            U256::ZERO
        };
        
        // Heartbeat blocks get reduced reward (but still something for security)
        let heartbeat_multiplier = if is_heartbeat && transactions.is_empty() {
            U256::from(50u64) / U256::from(100u64) // 50% for empty heartbeat
        } else {
            U256::ONE
        };
        
        let total_reward = (base_reward + tx_fees + activity_bonus) * heartbeat_multiplier;
        
        // Execute transactions
        for tx in &transactions {
            if let Some(to) = tx.to {
                match self.transfer(&self.get_sender(tx), &to, tx.value) {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::warn!("Transaction failed in block production: {}", e);
                        // Continue with other transactions
                    }
                }
            }
        }
        
        // Mint reward to validator
        self.mint_to_validator(validator, total_reward)?;
        
        // Create and store block - inline increment_block logic to avoid race conditions
        let new_hash = {
            let mut hash = self.block_hash.write();
            let mut blocks = self.blocks.write();
            
            // Increment block number
            *block_number_guard += 1;
            let parent = *hash;
            
            // Compute new block hash using blake3
            let new_hash = self.compute_block_hash(*block_number_guard, parent.as_bytes());
            *hash = Hash::from_bytes(new_hash);
            
            // Store block info
            let block_info = BlockInfo {
                number: *block_number_guard,
                hash: new_hash,
                parent_hash: *parent.as_bytes(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                tx_count: transactions.len(),
            };
            blocks.push(block_info);
            
            new_hash
        };
        
        // Persist (outside of lock scope)
        let _ = self.persist();
        
        tracing::info!(
            "Block #{} produced by {}: {} txs, reward: {} MERK (base: {}, fees: {}, bonus: {})",
            block_number,
            hex::encode(validator),
            transactions.len(),
            total_reward / U256::from(1_000_000_000_000_000_000u128),
            base_reward / U256::from(1_000_000_000_000_000_000u128),
            tx_fees / U256::from(1_000_000_000_000_000_000u128),
            activity_bonus / U256::from(1_000_000_000_000_000_000u128)
        );
        
        Ok(BlockProductionResult {
            block_number,
            block_hash: new_hash,
            transactions_count: transactions.len(),
            validator_reward: total_reward,
        })
    }
    
    /// Get sender from transaction (simplified - should verify signature)
    fn get_sender(&self, tx: &Transaction) -> Address {
        // In a real implementation, recover sender from signature
        // For now, return a dummy address
        Address::from_bytes([0u8; 20])
    }
    
    /// Mint new coins to validator as block reward
    fn mint_to_validator(&self, validator: &Address, amount: U256) -> Result<(), StateError> {
        let mut accounts = self.accounts.write();
        
        // Get or create validator account
        let validator_account = accounts.entry(*validator).or_insert_with(|| Account {
            balance: "0x0".to_string(),
            nonce: 0,
            code: vec![],
            storage: HashMap::new(),
        });
        
        let current_balance = validator_account.get_balance();
        validator_account.set_balance(current_balance + amount);
        
        // Update total supply
        let mut total_supply = self.total_supply.write();
        *total_supply += amount;
        
        Ok(())
    }
    
    /// Add a block from network sync
    pub fn add_block(&self, number: u64, hash: [u8; 32], parent_hash: [u8; 32]) -> bool {
        let current = *self.block_number.read();
        
        // Only accept if it extends our chain
        if number != current + 1 {
            tracing::debug!("Rejecting block #{} (expected #{})", number, current + 1);
            return false;
        }
        
        // Verify parent hash
        if parent_hash != *self.block_hash.read().as_bytes() {
            tracing::debug!("Block #{} has wrong parent hash", number);
            return false;
        }
        
        // Accept the block (in a separate scope to release locks before persist)
        {
            *self.block_number.write() = number;
            *self.block_hash.write() = Hash::from_bytes(hash);
            
            self.blocks.write().push(BlockInfo {
                number,
                hash,
                parent_hash,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                tx_count: 0,
            });
        }
        
        let _ = self.persist();
        tracing::info!("Added block #{} from network", number);
        true
    }
    
    /// Get block by number
    pub fn get_block(&self, number: u64) -> Option<BlockInfo> {
        let blocks = self.blocks.read();
        blocks.iter().find(|b| b.number == number).cloned()
    }
    
    /// Check if we have a block with given hash
    pub fn has_block(&self, hash: &[u8; 32]) -> bool {
        let blocks = self.blocks.read();
        blocks.iter().any(|b| &b.hash == hash)
    }
    
    fn compute_block_hash(&self, number: u64, parent_hash: &[u8; 32]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(parent_hash);
        hasher.update(&number.to_le_bytes());
        hasher.update(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0).to_le_bytes());
        *hasher.finalize().as_bytes()
    }
    
    /// Get all accounts (for debugging)
    pub fn all_accounts(&self) -> Vec<(Address, U256)> {
        let accounts = self.accounts.read();
        accounts.iter().map(|(k, v)| (*k, v.get_balance())).collect()
    }
    
    /// Deploy a contract
    pub fn deploy_contract(&self, from: &Address, code: Vec<u8>) -> Result<Address, String> {
        let mut accounts = self.accounts.write();
        
        let nonce = accounts.get(from).map(|a| a.nonce).unwrap_or(0);
        
        // Compute contract address from sender + nonce
        let contract_addr = self.compute_contract_address(from, nonce);
        
        // Increment sender nonce
        if let Some(sender) = accounts.get_mut(from) {
            sender.nonce = nonce + 1;
        }
        
        // Create contract account
        accounts.insert(contract_addr, Account {
            balance: "0x0".to_string(),
            nonce: 0,
            code,
            storage: HashMap::new(),
        });
        
        drop(accounts);
        
        // Persist
        let _ = self.persist();
        
        tracing::info!("Deployed contract at {}", hex::encode(contract_addr));
        Ok(contract_addr)
    }
    
    /// Get contract code
    pub fn get_code(&self, address: &Address) -> Vec<u8> {
        let accounts = self.accounts.read();
        accounts.get(address).map(|a| a.code.clone()).unwrap_or_default()
    }
    
    /// Set contract storage
    pub fn set_storage(&self, address: &Address, key: [u8; 32], value: [u8; 32]) {
        let mut accounts = self.accounts.write();
        if let Some(account) = accounts.get_mut(address) {
            account.storage.insert(hex::encode(key), hex::encode(value));
        }
        drop(accounts);
        let _ = self.persist();
    }
    
    /// Get contract storage
    pub fn get_storage(&self, address: &Address, key: [u8; 32]) -> Option<[u8; 32]> {
        let accounts = self.accounts.read();
        accounts.get(address)
            .and_then(|a| a.storage.get(&hex::encode(key)))
            .and_then(|v| hex::decode(v).ok())
            .filter(|v| v.len() == 32)
            .map(|v| {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&v);
                arr
            })
    }
    
    /// Increment nonce for an address
    pub fn increment_nonce(&self, address: &Address) {
        let mut accounts = self.accounts.write();
        if let Some(account) = accounts.get_mut(address) {
            account.nonce += 1;
        }
        drop(accounts);
        let _ = self.persist();
    }
    
    fn compute_contract_address(&self, from: &Address, nonce: u64) -> Address {
        let mut hasher = blake3::Hasher::new();
        hasher.update(from.as_bytes());
        hasher.update(&nonce.to_le_bytes());
        let hash = hasher.finalize();
        Address::from_slice(&hash.as_bytes()[12..]).unwrap_or(Address::ZERO)
    }
    
    /// Persist state to disk
    fn persist(&self) -> Result<(), String> {
        fs::create_dir_all(&self.path).map_err(|e| e.to_string())?;
        
        let accounts = self.accounts.read();
        let accounts_map: HashMap<String, Account> = accounts
            .iter()
            .map(|(k, v)| (hex::encode(k), v.clone()))
            .collect();
        
        let blocks = self.blocks.read();
        
        let data = StateData {
            accounts: accounts_map,
            block_number: *self.block_number.read(),
            block_hash: hex::encode(self.block_hash.read().as_bytes()),
            total_supply: format!("0x{}", *self.total_supply.read()),
            blocks: blocks.clone(),
        };
        
        let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
        let file = self.path.join("state.json");
        fs::write(&file, json).map_err(|e| e.to_string())?;
        
        tracing::debug!("State persisted to {:?}", file);
        Ok(())
    }
    
    /// Load state from disk
    fn load(&self) -> Result<(), String> {
        let file = self.path.join("state.json");
        if !file.exists() {
            return Err("No state file".to_string());
        }
        
        let json = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let data: StateData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        
        let mut accounts = self.accounts.write();
        accounts.clear();
        
        for (addr_hex, account) in data.accounts {
            if let Ok(addr) = parse_address(&format!("0x{}", addr_hex)) {
                accounts.insert(addr, account);
            }
        }
        
        *self.block_number.write() = data.block_number;
        
        // Load block hash
        if let Ok(hash_bytes) = hex::decode(&data.block_hash) {
            if hash_bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&hash_bytes);
                *self.block_hash.write() = Hash::from_bytes(arr);
            }
        }
        
        // Load blocks
        *self.blocks.write() = data.blocks;
        
        tracing::info!("Loaded state from disk: {} accounts, block {}", accounts.len(), data.block_number);
        Ok(())
    }
    
    fn compute_tx_hash(&self, from: &Address, to: &Address, amount: U256, nonce: u64) -> Hash {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash as StdHash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        from.hash(&mut hasher);
        to.hash(&mut hasher);
        amount.hash(&mut hasher);
        nonce.hash(&mut hasher);
        
        let h = hasher.finish();
        let mut arr = [0u8; 32];
        arr[..8].copy_from_slice(&h.to_le_bytes());
        Hash::from_bytes(arr)
    }
    
    /// ⚠️ IMPORTANT: We NEVER delete blocks from the chain!
    /// Block chain is immutable by definition.
    /// 
    /// This function only removes old transaction DETAILS (not blocks)
    /// to save space, while keeping block headers for chain integrity.
    /// 
    /// Real pruning strategy:
    /// - Keep ALL block headers (number, hash, parent_hash, timestamp)
    /// - Keep latest state (account balances)
    /// - Archive old transaction details (optional)
    pub fn prune_old_transactions(&self, before_block: u64) -> Result<usize, String> {
        // NOTE: In a real implementation, we would:
        // 1. Move old transaction details to cold storage
        // 2. Keep only block headers in hot storage
        // 3. State remains available for all blocks
        
        // For now, we just log the recommendation
        let blocks = self.blocks.read();
        let tx_count: usize = blocks
            .iter()
            .filter(|b| b.number < before_block)
            .map(|b| b.tx_count)
            .sum();
        
        tracing::info!(
            "Recommendation: Archive {} transactions from blocks before #{}",
            tx_count,
            before_block
        );
        
        Ok(tx_count)
    }
    
    /// Get storage stats
    pub fn storage_stats(&self) -> StorageStats {
        let blocks = self.blocks.read();
        let accounts = self.accounts.read();
        
        // IMPORTANT: All block headers are kept
        // Only transaction details can be archived
        StorageStats {
            total_blocks: blocks.len(),
            total_accounts: accounts.len(),
            latest_block: blocks.last().map(|b| b.number).unwrap_or(0),
            estimated_size_mb: self.estimate_size(),
            chain_integrity: "IMMUTABLE".to_string(),
        }
    }
    
    /// Estimate storage size in MB
    /// Block headers are small (~200 bytes each)
    /// Transaction data is the bulk of storage
    fn estimate_size(&self) -> u64 {
        let blocks = self.blocks.read();
        let accounts = self.accounts.read();
        
        // Block headers: ~200 bytes per block (immutable, kept forever)
        // Accounts: ~200 bytes per account
        let headers_size = blocks.len() * 200;
        let accounts_size = accounts.len() * 200;
        
        ((headers_size + accounts_size) / (1024 * 1024)) as u64
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_blocks: usize,
    pub total_accounts: usize,
    pub latest_block: u64,
    pub estimated_size_mb: u64,
    pub chain_integrity: String,
}

fn parse_address(s: &str) -> Result<Address, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s).map_err(|e: hex::FromHexError| e.to_string())?;
    Address::from_slice(&bytes).map_err(|e| e.to_string())
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_creation() {
        let state = State::new();
        assert_eq!(state.block_number(), 0);
    }
    
    #[test]
    fn test_transfer() {
        // Use temp directory for test
        let temp_dir = std::env::temp_dir().join(format!("merklith_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        let state = State::with_path(temp_dir.clone());
        
        let from = parse_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0").unwrap();
        let to = parse_address("0x0000000000000000000000000000000000000001").unwrap();
        
        let initial = state.balance(&from);
        println!("Initial balance: {}", initial);
        assert!(initial > U256::ZERO, "Genesis account should have balance, got {}", initial);
        
        let result = state.transfer(&from, &to, U256::from(1000));
        println!("Transfer result: {:?}", result);
        assert!(result.is_ok(), "Transfer should succeed");
        
        let from_after = state.balance(&from);
        let to_after = state.balance(&to);
        println!("From after: {}, To after: {}", from_after, to_after);
        println!("Expected from: {}, Expected to: {}", initial - U256::from(1000), U256::from(1000));
        
        assert_eq!(from_after, initial - U256::from(1000));
        assert_eq!(to_after, U256::from(1000));
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
