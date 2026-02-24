//! Transaction Pool - Manages pending transactions
//!
//! This module provides transaction pooling and validation.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_size: usize,
    pub max_per_account: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 5000,
            max_per_account: 100,
        }
    }
}

/// Transaction pool error
#[derive(Debug, Clone)]
pub enum PoolError {
    PoolFull,
    AccountLimit,
    InvalidTransaction(String),
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::PoolFull => write!(f, "Transaction pool is full"),
            PoolError::AccountLimit => write!(f, "Account transaction limit reached"),
            PoolError::InvalidTransaction(e) => write!(f, "Invalid transaction: {}", e),
        }
    }
}

impl std::error::Error for PoolError {}

/// Transaction pool
#[derive(Debug)]
pub struct TransactionPool {
    config: PoolConfig,
    transactions: Arc<Mutex<HashMap<String, merklith_types::Transaction>>>,
    pending: Arc<Mutex<Vec<String>>>,
}

impl TransactionPool {
    /// Create a new transaction pool
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            transactions: Arc::new(Mutex::new(HashMap::new())),
            pending: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a transaction to the pool
    pub fn add_transaction(
        &self,
        tx: merklith_types::Transaction,
    ) -> Result<String, PoolError> {
        let mut transactions = self.transactions.lock();
        let mut pending = self.pending.lock();

        // Check pool size
        if transactions.len() >= self.config.max_size {
            return Err(PoolError::PoolFull);
        }

        // Create a simple hash from nonce and chain_id
        let hash = format!("tx_{}_{}", tx.nonce, tx.chain_id);

        if transactions.contains_key(&hash) {
            return Err(PoolError::InvalidTransaction(
                "Transaction already exists".to_string(),
            ));
        }

        transactions.insert(hash.clone(), tx);
        pending.push(hash.clone());

        Ok(hash)
    }

    /// Get a transaction by hash
    pub fn get_transaction(
        &self,
        hash: &str,
    ) -> Option<merklith_types::Transaction> {
        let transactions = self.transactions.lock();
        transactions.get(hash).cloned()
    }

    /// Get pending transactions up to limit
    pub fn get_pending(&self,
        limit: usize,
    ) -> Vec<merklith_types::Transaction> {
        let transactions = self.transactions.lock();
        let pending = self.pending.lock();

        pending
            .iter()
            .take(limit)  // Respect the limit to prevent unbounded memory growth
            .filter_map(|hash| transactions.get(hash).cloned())
            .collect()
    }

    /// Remove a transaction from the pool
    pub fn remove_transaction(&self,
        hash: &str) {
        let mut transactions = self.transactions.lock();
        let mut pending = self.pending.lock();

        transactions.remove(hash);
        pending.retain(|h| h != hash);
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        let transactions = self.transactions.lock();
        transactions.len()
    }
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new(PoolConfig::default())
    }
}

pub mod pool {
    pub use super::{PoolConfig, PoolError, TransactionPool};
}

// Re-export for convenience
pub use pool::*;

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::{Transaction, Address, U256};

    fn create_test_transaction(nonce: u64) -> Transaction {
        Transaction::new(
            1, // chain_id
            nonce,
            Some(Address::ZERO),
            U256::from(1000u64),
            21000,
            U256::from(1u64),
            U256::from(1u64),
        )
    }

    #[test]
    fn test_pool_creation() {
        let pool = TransactionPool::new(PoolConfig::default());
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_add_transaction() {
        let pool = TransactionPool::new(PoolConfig::default());
        let tx = create_test_transaction(0);
        
        let hash = pool.add_transaction(tx).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_get_transaction() {
        let pool = TransactionPool::new(PoolConfig::default());
        let tx = create_test_transaction(0);
        
        let hash = pool.add_transaction(tx.clone()).unwrap();
        let retrieved = pool.get_transaction(&hash).unwrap();
        
        assert_eq!(retrieved.nonce, tx.nonce);
        assert_eq!(retrieved.chain_id, tx.chain_id);
    }

    #[test]
    fn test_get_nonexistent_transaction() {
        let pool = TransactionPool::new(PoolConfig::default());
        let result = pool.get_transaction("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_transaction() {
        let pool = TransactionPool::new(PoolConfig::default());
        let tx = create_test_transaction(0);
        
        let hash = pool.add_transaction(tx).unwrap();
        assert_eq!(pool.size(), 1);
        
        pool.remove_transaction(&hash);
        assert_eq!(pool.size(), 0);
        
        let retrieved = pool.get_transaction(&hash);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_get_pending_transactions() {
        let pool = TransactionPool::new(PoolConfig::default());
        
        let tx1 = create_test_transaction(0);
        let tx2 = create_test_transaction(1);
        
        pool.add_transaction(tx1).unwrap();
        pool.add_transaction(tx2).unwrap();
        
        let pending = pool.get_pending(10);
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_duplicate_transaction() {
        let pool = TransactionPool::new(PoolConfig::default());
        let tx = create_test_transaction(0);
        
        pool.add_transaction(tx.clone()).unwrap();
        
        let result = pool.add_transaction(tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_full() {
        let config = PoolConfig {
            max_size: 2,
            max_per_account: 100,
        };
        let pool = TransactionPool::new(config);
        
        pool.add_transaction(create_test_transaction(0)).unwrap();
        pool.add_transaction(create_test_transaction(1)).unwrap();
        
        let result = pool.add_transaction(create_test_transaction(2));
        assert!(matches!(result, Err(PoolError::PoolFull)));
    }

    #[test]
    fn test_pool_default() {
        let pool: TransactionPool = Default::default();
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_pool_error_display() {
        let err1 = PoolError::PoolFull;
        assert!(format!("{}", err1).contains("full"));
        
        let err2 = PoolError::AccountLimit;
        assert!(format!("{}", err2).contains("limit"));
        
        let err3 = PoolError::InvalidTransaction("test".to_string());
        assert!(format!("{}", err3).contains("test"));
    }
}
