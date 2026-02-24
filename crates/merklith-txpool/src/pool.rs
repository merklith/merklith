//! Transaction pool management.
//!
//! Maintains pending and queued transactions, ordered by priority.

use std::collections::{HashMap, HashSet};
use merklith_types::{Address, Transaction, Hash, U256};
use dashmap::DashMap;
use crate::error::PoolError;
use crate::validation::{ValidationConfig, ValidationContext, validate_transaction, ValidationResult};
use crate::ordering::{OrderedTransaction, PriorityQueue};
use crate::batch::{TransactionBatcher, BatchConfig};

/// Configuration for the transaction pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of pending transactions
    pub max_pending: usize,
    /// Maximum number of queued transactions (future nonces)
    pub max_queued: usize,
    /// Maximum transactions per account
    pub max_per_account: usize,
    /// Price bump for replacement (percentage, 10 = 10%)
    pub price_bump: u16,
    /// Validation configuration
    pub validation: ValidationConfig,
    /// Batch configuration
    pub batch: BatchConfig,
    /// Enable transaction batching
    pub enable_batching: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_pending: 10_000,
            max_queued: 50_000,
            max_per_account: 64,
            price_bump: 10,
            validation: ValidationConfig::default(),
            batch: BatchConfig::default(),
            enable_batching: true,
        }
    }
}

/// Transaction pool state.
#[derive(Debug)]
pub struct TransactionPool {
    /// Configuration
    config: PoolConfig,
    /// Pending transactions (ready for mining)
    pending: DashMap<Address, PriorityQueue>,
    /// Queued transactions (future nonces)
    queued: DashMap<Address, Vec<OrderedTransaction>>,
    /// All transactions by hash
    all_txs: DashMap<Hash, Transaction>,
    /// Current nonces by address
    current_nonces: DashMap<Address, u64>,
    /// Batcher for micro-transactions
    batcher: Option<TransactionBatcher>,
    /// Pool statistics
    stats: PoolStats,
}

/// Pool statistics.
#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    /// Total pending transactions
    pub pending_count: usize,
    /// Total queued transactions
    pub queued_count: usize,
    /// Total transactions in pool
    pub total_count: usize,
    /// Current gas price floor
    pub gas_price_floor: U256,
}

impl TransactionPool {
    /// Create a new transaction pool.
    pub fn new(config: PoolConfig) -> Self {
        let batcher = if config.enable_batching {
            Some(TransactionBatcher::new(config.batch.clone()))
        } else {
            None
        };

        Self {
            config,
            pending: DashMap::new(),
            queued: DashMap::new(),
            all_txs: DashMap::new(),
            current_nonces: DashMap::new(),
            batcher,
            stats: PoolStats::default(),
        }
    }

    /// Add a transaction to the pool.
    pub fn add_transaction(
        &mut self,
        tx: Transaction,
        context: &ValidationContext,
    ) -> Result<PoolStatus, PoolError> {
        // Validate the transaction
        let validation = validate_transaction(&tx, &self.config.validation, context
        );

        if !validation.valid {
            return Err(validation.error.unwrap_or_else(|| 
                PoolError::InvalidTransaction("Unknown validation error".to_string())
            ));
        }

        let sender = tx.sender()
            .ok_or_else(|| PoolError::InvalidSignature("Could not recover sender".to_string()))?;

        let hash = tx.hash();

        // Check if already exists
        if self.all_txs.contains_key(&hash) {
            return Err(PoolError::DuplicateTransaction(hash.to_string()));
        }

        // Check pool limits
        if self.stats.total_count >= self.config.max_pending + self.config.max_queued {
            return Err(PoolError::PoolFull("Pool capacity reached".to_string()));
        }

        // Check per-account limit
        let account_tx_count = self.count_account_transactions(&sender);
        if account_tx_count >= self.config.max_per_account {
            return Err(PoolError::PoolFull(
                format!("Account {} transaction limit reached", sender)
            ));
        }

        // Get current nonce for sender
        let current_nonce = self.current_nonces
            .get(&sender)
            .map(|n| *n)
            .unwrap_or(validation.expected_nonce);

        // Check for replacement (same nonce)
        if tx.nonce < current_nonce {
            return Err(PoolError::NonceTooLow {
                expected: current_nonce,
                got: tx.nonce,
            });
        }

        // Check if replacing existing transaction
        if let Some(existing) = self.find_by_nonce(&sender, tx.nonce) {
            // Check if new transaction has sufficient price bump
            let min_price = existing.gas_price + 
                (existing.gas_price * U256::from(self.config.price_bump) / U256::from(100));
            
            if tx.gas_price < min_price {
                return Err(PoolError::ReplacementUnderpriced {
                    required: min_price.as_u128(),
                    got: tx.gas_price.as_u128(),
                });
            }

            // Remove old transaction
            self.remove_by_hash(&existing.hash());
        }

        // Store transaction
        self.all_txs.insert(hash, tx.clone());

        // Route to pending or queued
        if tx.nonce == current_nonce {
            // Ready for execution
            self.add_to_pending(sender, tx);
        } else {
            // Future nonce, queue it
            self.add_to_queued(sender, tx);
        }

        self.update_stats();

        Ok(PoolStatus::Accepted { hash })
    }

    /// Get pending transactions for mining.
    pub fn get_pending(&self,
        max_transactions: usize,
    ) -> Vec<Transaction> {
        let mut result = Vec::with_capacity(max_transactions);

        // Collect from all accounts
        for entry in self.pending.iter() {
            let queue = entry.value();
            
            // Get highest priority from each account
            if let Some(ordered) = queue.peek() {
                result.push(ordered.transaction.clone());
                
                if result.len() >= max_transactions {
                    break;
                }
            }
        }

        // Sort by gas price
        result.sort_by(|a, b| b.gas_price.cmp(&a.gas_price));

        result.truncate(max_transactions);
        result
    }

    /// Get a transaction by hash.
    pub fn get_transaction(&self, hash: &Hash) -> Option<Transaction> {
        self.all_txs.get(hash).map(|t| t.clone())
    }

    /// Remove a transaction by hash.
    pub fn remove_by_hash(
        &mut self, hash: &Hash) -> Option<Transaction> {
        let tx = self.all_txs.remove(hash).map(|(_, t)| t)?;
        
        if let Some(sender) = tx.sender() {
            // Remove from pending or queued
            self.pending.remove_if(&sender, |_, queue| {
                queue.remove_if(|ot| ot.transaction.hash() == *hash).is_some()
            });

            if let Some(mut queued) = self.queued.get_mut(&sender) {
                queued.retain(|ot| ot.transaction.hash() != *hash);
            }
        }

        self.update_stats();
        Some(tx)
    }

    /// Promote queued transactions to pending.
    ///
    /// Call this when a block is mined to update nonces.
    pub fn promote_transactions(
        &mut self,
        address: &Address,
        new_nonce: u64,
    ) {
        self.current_nonces.insert(*address, new_nonce);

        // Check if any queued transactions can be promoted
        if let Some(queued) = self.queued.get_mut(address) {
            let to_promote: Vec<OrderedTransaction> = queued
                .iter()
                .filter(|ot| ot.transaction.nonce == new_nonce)
                .cloned()
                .collect();

            for ordered in to_promote {
                queued.retain(|ot| ot.transaction.hash() != ordered.transaction.hash());
                self.add_ordered_to_pending(*address, ordered);
            }
        }

        self.update_stats();
    }

    /// Get pool statistics.
    pub fn stats(&self) -> PoolStats {
        self.stats.clone()
    }

    /// Add transaction to pending queue.
    fn add_to_pending(
        &mut self,
        address: Address,
        tx: Transaction,
    ) {
        let ordered = OrderedTransaction::new(tx);
        self.add_ordered_to_pending(address, ordered);
    }

    /// Add ordered transaction to pending.
    fn add_ordered_to_pending(
        &mut self,
        address: Address,
        ordered: OrderedTransaction,
    ) {
        let mut queue = self.pending.entry(address).or_insert_with(PriorityQueue::new);
        queue.push(ordered);
    }

    /// Add transaction to queued (future nonce).
    fn add_to_queued(
        &mut self,
        address: Address,
        tx: Transaction,
    ) {
        let ordered = OrderedTransaction::new(tx);
        let mut queued = self.queued.entry(address).or_insert_with(Vec::new);
        queued.push(ordered);
        // Sort by nonce
        queued.sort_by_key(|ot| ot.transaction.nonce);
    }

    /// Find transaction by nonce.
    fn find_by_nonce(
        &self,
        address: &Address,
        nonce: u64,
    ) -> Option<Transaction> {
        // Check pending
        if let Some(queue) = self.pending.get(address) {
            if let Some(ordered) = queue.peek() {
                if ordered.transaction.nonce == nonce {
                    return Some(ordered.transaction.clone());
                }
            }
        }

        // Check queued
        if let Some(queued) = self.queued.get(address) {
            for ordered in queued.iter() {
                if ordered.transaction.nonce == nonce {
                    return Some(ordered.transaction.clone());
                }
            }
        }

        None
    }

    /// Count transactions for an account.
    fn count_account_transactions(
        &self,
        address: &Address,
    ) -> usize {
        let pending_count = self.pending
            .get(address)
            .map(|q| q.len())
            .unwrap_or(0);
        
        let queued_count = self.queued
            .get(address)
            .map(|q| q.len())
            .unwrap_or(0);

        pending_count + queued_count
    }

    /// Update pool statistics.
    fn update_stats(&mut self,
    ) {
        let pending_count: usize = self.pending.iter()
            .map(|e| e.value().len())
            .sum();
        
        let queued_count: usize = self.queued.iter()
            .map(|e| e.value().len())
            .sum();

        self.stats = PoolStats {
            pending_count,
            queued_count,
            total_count: self.all_txs.len(),
            gas_price_floor: self.calculate_gas_price_floor(),
        };
    }

    /// Calculate the minimum gas price in the pool.
    fn calculate_gas_price_floor(&self,
    ) -> U256 {
        let mut min_price = U256::MAX;

        for entry in self.pending.iter() {
            if let Some(ordered) = entry.value().peek() {
                if ordered.transaction.gas_price < min_price {
                    min_price = ordered.transaction.gas_price;
                }
            }
        }

        if min_price == U256::MAX {
            min_price = U256::ZERO;
        }

        min_price
    }

    /// Get all pending transaction hashes.
    pub fn pending_hashes(&self,
    ) -> Vec<Hash> {
        self.all_txs.iter()
            .map(|e| *e.key())
            .collect()
    }

    /// Clear all transactions.
    pub fn clear(&mut self,
    ) {
        self.pending.clear();
        self.queued.clear();
        self.all_txs.clear();
        self.current_nonces.clear();
        self.update_stats();
    }
}

/// Status of adding a transaction.
#[derive(Debug, Clone)]
pub enum PoolStatus {
    /// Transaction accepted into pool
    Accepted {
        hash: Hash,
    },
    /// Transaction queued (future nonce)
    Queued {
        hash: Hash,
    },
    /// Transaction replaced existing
    Replaced {
        old_hash: Hash,
        new_hash: Hash,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::{Transaction, TransactionType, Address, ChainConfig, U256};
    use merklith_core::state::AccountState;

    fn create_test_tx(nonce: u64, gas_price: u64) -> Transaction {
        Transaction {
            tx_type: TransactionType::Legacy,
            nonce,
            gas_price: U256::from(gas_price),
            gas_limit: 100_000,
            to: Some(Address::from_bytes([1u8; 20])),
            value: U256::from(1000u64),
            data: vec![],
            v: 0,
            r: U256::ZERO,
            s: U256::ZERO,
            chain_id: Some(1),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: None,
        }
    }

    fn create_context() -> ValidationContext<'static> {
        let account_state = Box::leak(Box::new(AccountState::new()));
        let chain_config = Box::leak(Box::new(ChainConfig::default()));

        ValidationContext::new(
            account_state,
            100,
            1000,
            U256::from(1_000_000_000u64),
            chain_config,
        )
    }

    #[test]
    fn test_add_transaction() {
        let config = PoolConfig::default();
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        let tx = create_test_tx(0, 10_000_000_000);
        let result = pool.add_transaction(tx, &context);

        assert!(result.is_ok());
        assert_eq!(pool.stats().total_count, 1);
    }

    #[test]
    fn test_duplicate_transaction() {
        let config = PoolConfig::default();
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        let tx = create_test_tx(0, 10_000_000_000);
        pool.add_transaction(tx.clone(), &context).unwrap();

        // Adding same transaction again should fail
        let result = pool.add_transaction(tx, &context);
        assert!(matches!(result, Err(PoolError::DuplicateTransaction(_))));
    }

    #[test]
    fn test_replacement() {
        let config = PoolConfig {
            price_bump: 10,
            ..Default::default()
        };
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        let tx1 = create_test_tx(0, 10_000_000_000);
        pool.add_transaction(tx1.clone(), &context).unwrap();

        // Try to replace with same price - should fail
        let tx2 = create_test_tx(0, 10_000_000_000);
        let result = pool.add_transaction(tx2, &context);
        assert!(matches!(result, Err(PoolError::ReplacementUnderpriced { .. })));

        // Replace with 10% higher price - should succeed
        let tx3 = create_test_tx(0, 11_000_000_000);
        let result = pool.add_transaction(tx3, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pool_limits() {
        let config = PoolConfig {
            max_pending: 2,
            max_per_account: 2,
            ..Default::default()
        };
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        // Add max transactions
        pool.add_transaction(create_test_tx(0, 10_000_000_000), &context).unwrap();
        pool.add_transaction(create_test_tx(1, 10_000_000_000), &context).unwrap();

        // Third should fail
        let result = pool.add_transaction(create_test_tx(2, 10_000_000_000), &context);
        assert!(matches!(result, Err(PoolError::PoolFull(_))));
    }

    #[test]
    fn test_get_pending() {
        let config = PoolConfig::default();
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        // Add transactions with different gas prices
        pool.add_transaction(create_test_tx(0, 5_000_000_000), &context).unwrap();
        pool.add_transaction(create_test_tx(1, 20_000_000_000), &context).unwrap();
        pool.add_transaction(create_test_tx(2, 10_000_000_000), &context).unwrap();

        let pending = pool.get_pending(10);
        
        // Should be sorted by gas price (highest first)
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].gas_price, U256::from(20_000_000_000u64));
        assert_eq!(pending[1].gas_price, U256::from(10_000_000_000u64));
        assert_eq!(pending[2].gas_price, U256::from(5_000_000_000u64));
    }

    #[test]
    fn test_get_transaction() {
        let config = PoolConfig::default();
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        let tx = create_test_tx(0, 10_000_000_000);
        let hash = tx.hash();
        
        pool.add_transaction(tx.clone(), &context).unwrap();

        let retrieved = pool.get_transaction(&hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().nonce, tx.nonce);
    }

    #[test]
    fn test_remove_by_hash() {
        let config = PoolConfig::default();
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        let tx = create_test_tx(0, 10_000_000_000);
        let hash = tx.hash();
        
        pool.add_transaction(tx, &context).unwrap();
        assert_eq!(pool.stats().total_count, 1);

        pool.remove_by_hash(&hash);
        assert_eq!(pool.stats().total_count, 0);
        assert!(pool.get_transaction(&hash).is_none());
    }

    #[test]
    fn test_clear() {
        let config = PoolConfig::default();
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        pool.add_transaction(create_test_tx(0, 10_000_000_000), &context).unwrap();
        pool.add_transaction(create_test_tx(1, 10_000_000_000), &context).unwrap();

        pool.clear();

        assert_eq!(pool.stats().total_count, 0);
        assert_eq!(pool.stats().pending_count, 0);
    }

    #[test]
    fn test_stats() {
        let config = PoolConfig::default();
        let mut pool = TransactionPool::new(config);
        let context = create_context();

        // Initially empty
        let stats = pool.stats();
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.pending_count, 0);
        assert_eq!(stats.queued_count, 0);

        // Add transactions
        pool.add_transaction(create_test_tx(0, 5_000_000_000), &context).unwrap();
        pool.add_transaction(create_test_tx(1, 10_000_000_000), &context).unwrap();

        let stats = pool.stats();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.pending_count, 2);
        assert_eq!(stats.gas_price_floor, U256::from(5_000_000_000u64));
    }
}
