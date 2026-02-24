//! Micro-transaction batching for fee efficiency.
//!
//! Batches multiple small transactions into a single transaction,
//! reducing per-transaction overhead costs.

use merklith_types::{Address, Transaction, TransactionType, U256, Hash};
use crate::error::PoolError;
use std::collections::HashMap;

/// Configuration for transaction batching.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of transactions in a batch
    pub max_batch_size: usize,
    /// Maximum total gas for a batch
    pub max_batch_gas: u64,
    /// Minimum number of transactions to form a batch
    pub min_batch_size: usize,
    /// Maximum time to wait before forming a batch (seconds)
    pub max_wait_time_secs: u64,
    /// Maximum value per individual transaction in batch
    pub max_tx_value: U256,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            max_batch_gas: 1_000_000,
            min_batch_size: 4,
            max_wait_time_secs: 60,
            max_tx_value: U256::from(1_000_000_000_000_000_000u128), // 1 MERK
        }
    }
}

/// A single micro-transaction in a batch.
#[derive(Debug, Clone)]
pub struct MicroTransaction {
    /// Original transaction
    pub transaction: Transaction,
    /// Original sender
    pub sender: Address,
    /// Submission timestamp
    pub submitted_at: u64,
}

/// A batch of micro-transactions.
#[derive(Debug, Clone)]
pub struct TransactionBatch {
    /// Unique batch ID
    pub id: u64,
    /// Transactions in this batch
    pub transactions: Vec<MicroTransaction>,
    /// Total gas used by batch
    pub total_gas: u64,
    /// Combined signature data
    pub aggregated_signatures: Vec<u8>,
    /// Batch creation timestamp
    pub created_at: u64,
    /// Whether batch is ready for submission
    pub ready: bool,
}

impl TransactionBatch {
    /// Create a new empty batch.
    pub fn new(id: u64, created_at: u64) -> Self {
        Self {
            id,
            transactions: Vec::new(),
            total_gas: 0,
            aggregated_signatures: Vec::new(),
            created_at,
            ready: false,
        }
    }

    /// Add a transaction to the batch.
    pub fn add(
        &mut self,
        micro_tx: MicroTransaction,
        gas_cost: u64,
        config: &BatchConfig,
    ) -> Result<(), PoolError> {
        // Check batch size limit
        if self.transactions.len() >= config.max_batch_size {
            return Err(PoolError::BatchError(
                "Batch size limit reached".to_string()
            ));
        }

        // Check gas limit
        if self.total_gas + gas_cost > config.max_batch_gas {
            return Err(PoolError::BatchError(
                "Batch gas limit would be exceeded".to_string()
            ));
        }

        self.transactions.push(micro_tx);
        self.total_gas += gas_cost;

        // Mark as ready if we've reached minimum size
        if self.transactions.len() >= config.min_batch_size {
            self.ready = true;
        }

        Ok(())
    }

    /// Check if batch is full.
    pub fn is_full(&self, config: &BatchConfig) -> bool {
        self.transactions.len() >= config.max_batch_size ||
        self.total_gas >= config.max_batch_gas
    }

    /// Check if batch has timed out.
    pub fn is_timed_out(&self, current_time: u64, config: &BatchConfig) -> bool {
        current_time >= self.created_at + config.max_wait_time_secs
    }

    /// Get number of transactions.
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Calculate total value being transferred.
    pub fn total_value(&self) -> U256 {
        self.transactions.iter()
            .map(|mt| mt.transaction.value)
            .fold(U256::ZERO, |acc, v| acc + v)
    }

    /// Generate the batch transaction.
    ///
    /// This creates a single meta-transaction that contains all batched
    /// transactions, significantly reducing per-tx overhead.
    pub fn to_transaction(&self,
        batch_sender: Address,
        gas_price: U256,
        nonce: u64,
    ) -> Transaction {
        // Serialize all batched transactions
        let mut batch_data = Vec::new();
        batch_data.extend_from_slice(&self.id.to_le_bytes());
        batch_data.extend_from_slice(&self.transactions.len().to_le_bytes());

        for micro_tx in &self.transactions {
            // Add each transaction's hash to the batch data
            let tx_hash = micro_tx.transaction.hash();
            batch_data.extend_from_slice(tx_hash.as_bytes());
        }

        Transaction {
            tx_type: TransactionType::Batch,
            nonce,
            gas_price,
            gas_limit: self.total_gas + 21_000, // Base cost + batch overhead
            to: Some(batch_sender),
            value: self.total_value(),
            data: batch_data,
            v: 0,
            r: U256::ZERO,
            s: U256::ZERO,
            chain_id: Some(1),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: None,
        }
    }
}

/// Batcher managing transaction batching.
#[derive(Debug)]
pub struct TransactionBatcher {
    /// Active batches (grouped by destination/recipient type)
    batches: HashMap<BatchKey, TransactionBatch>,
    /// Completed batches ready for submission
    completed: Vec<TransactionBatch>,
    /// Configuration
    config: BatchConfig,
    /// Next batch ID
    next_batch_id: u64,
}

/// Key for grouping transactions into batches.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct BatchKey {
    /// Target shard or zone (for sharded chains)
    pub shard: u32,
    /// Transaction type category
    pub category: u8,
}

impl TransactionBatcher {
    /// Create a new batcher.
    pub fn new(config: BatchConfig) -> Self {
        Self {
            batches: HashMap::new(),
            completed: Vec::new(),
            config,
            next_batch_id: 1,
        }
    }

    /// Create with default config.
    pub fn default() -> Self {
        Self::new(BatchConfig::default())
    }

    /// Submit a transaction for batching.
    pub fn submit(
        &mut self,
        transaction: Transaction,
        sender: Address,
        submitted_at: u64,
    ) -> Result<BatchStatus, PoolError> {
        // Validate transaction is suitable for batching
        if transaction.value > self.config.max_tx_value {
            return Err(PoolError::BatchError(
                "Transaction value too large for batching".to_string()
            ));
        }

        if transaction.tx_type != TransactionType::Legacy {
            return Err(PoolError::BatchError(
                "Only legacy transactions supported for batching".to_string()
            ));
        }

        let micro_tx = MicroTransaction {
            transaction,
            sender,
            submitted_at,
        };

        // Calculate gas cost
        let gas_cost = estimate_gas_cost(&micro_tx.transaction
        );

        // Determine batch key
        let key = BatchKey {
            shard: 0, // For now, all in same shard
            category: 0,
        };

        // Find or create batch
        let batch = self.batches.entry(key.clone()).or_insert_with(|| {
            TransactionBatch::new(self.next_batch_id, submitted_at)
        });

        // Try to add to current batch
        if let Err(_) = batch.add(micro_tx.clone(), gas_cost, &self.config) {
            // Current batch is full, create new one
            let old_batch = self.batches.remove(&key).unwrap();
            if !old_batch.is_empty() {
                self.completed.push(old_batch);
            }

            let mut new_batch = TransactionBatch::new(self.next_batch_id, submitted_at);
            self.next_batch_id += 1;
            new_batch.add(micro_tx, gas_cost, &self.config)?;
            self.batches.insert(key, new_batch);
        }

        // Check if batch is now ready
        let batch = self.batches.get(&key).unwrap();
        if batch.ready {
            return Ok(BatchStatus::Ready {
                batch_id: batch.id,
                tx_count: batch.len(),
            });
        }

        Ok(BatchStatus::Pending {
            batch_id: batch.id,
            tx_count: batch.len(),
        })
    }

    /// Get completed batches.
    pub fn get_completed(&mut self) -> Vec<TransactionBatch> {
        // Also check for timed out batches
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let timed_out: Vec<BatchKey> = self.batches
            .iter()
            .filter(|(_, batch)| batch.is_timed_out(current_time, &self.config) && !batch.is_empty())
            .map(|(k, _)| k.clone())
            .collect();

        for key in timed_out {
            if let Some(batch) = self.batches.remove(&key) {
                self.completed.push(batch);
            }
        }

        std::mem::take(&mut self.completed)
    }

    /// Get batch by ID.
    pub fn get_batch(&self, batch_id: u64) -> Option<&TransactionBatch> {
        self.batches.values()
            .chain(self.completed.iter())
            .find(|b| b.id == batch_id)
    }

    /// Get number of pending batches.
    pub fn pending_count(&self) -> usize {
        self.batches.len()
    }

    /// Get number of completed batches.
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }
}

impl Default for TransactionBatcher {
    fn default() -> Self {
        Self::default()
    }
}

/// Status of a batch submission.
#[derive(Debug, Clone)]
pub enum BatchStatus {
    /// Transaction added to pending batch
    Pending {
        batch_id: u64,
        tx_count: usize,
    },
    /// Batch is ready for submission
    Ready {
        batch_id: u64,
        tx_count: usize,
    },
}

/// Estimate gas cost for a transaction.
fn estimate_gas_cost(tx: &Transaction) -> u64 {
    // Base cost
    let mut gas = 21_000u64;

    // Data cost
    for byte in &tx.data {
        if *byte == 0 {
            gas += 4;
        } else {
            gas += 16;
        }
    }

    // Value transfer cost (if any)
    if tx.value > U256::ZERO {
        gas += 9_000;
    }

    gas
}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::{Transaction, TransactionType, Address, U256};

    fn create_micro_tx(value: u64) -> Transaction {
        Transaction {
            tx_type: TransactionType::Legacy,
            nonce: 0,
            gas_price: U256::from(10_000_000_000u64),
            gas_limit: 21_000,
            to: Some(Address::from_bytes([1u8; 20])),
            value: U256::from(value),
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

    #[test]
    fn test_batch_creation() {
        let mut batch = TransactionBatch::new(1, 1000);
        assert!(batch.is_empty());
        assert!(!batch.ready);

        let config = BatchConfig::default();

        // Add transactions
        for i in 0..4 {
            let micro_tx = MicroTransaction {
                transaction: create_micro_tx(100),
                sender: Address::from_bytes([i as u8; 20]),
                submitted_at: 1000,
            };

            batch.add(micro_tx, 21_000, &config).unwrap();
        }

        assert_eq!(batch.len(), 4);
        assert!(batch.ready); // Should be ready after min_batch_size
    }

    #[test]
    fn test_batch_size_limit() {
        let config = BatchConfig {
            max_batch_size: 2,
            ..Default::default()
        };

        let mut batch = TransactionBatch::new(1, 1000);

        // Add 2 transactions (at limit)
        for i in 0..2 {
            let micro_tx = MicroTransaction {
                transaction: create_micro_tx(100),
                sender: Address::from_bytes([i as u8; 20]),
                submitted_at: 1000,
            };

            batch.add(micro_tx, 21_000, &config).unwrap();
        }

        assert!(batch.is_full(&config));

        // Third should fail
        let micro_tx = MicroTransaction {
            transaction: create_micro_tx(100),
            sender: Address::from_bytes([3u8; 20]),
            submitted_at: 1000,
        };

        assert!(batch.add(micro_tx, 21_000, &config).is_err());
    }

    #[test]
    fn test_batcher() {
        let config = BatchConfig {
            min_batch_size: 3,
            max_batch_size: 5,
            ..Default::default()
        };

        let mut batcher = TransactionBatcher::new(config);

        // Add 2 transactions - should be pending
        for i in 0..2 {
            let tx = create_micro_tx(100);
            let status = batcher.submit(tx, Address::from_bytes([i as u8; 20]), 1000).unwrap();
            
            match status {
                BatchStatus::Pending { tx_count, .. } => {
                    assert_eq!(tx_count, i as usize + 1);
                }
                _ => panic!("Expected pending status"),
            }
        }

        // Add 3rd transaction - should be ready
        let tx = create_micro_tx(100);
        let status = batcher.submit(tx, Address::from_bytes([3u8; 20]), 1000).unwrap();
        
        match status {
            BatchStatus::Ready { tx_count, .. } => {
                assert_eq!(tx_count, 3);
            }
            _ => panic!("Expected ready status"),
        }
    }

    #[test]
    fn test_batch_total_value() {
        let mut batch = TransactionBatch::new(1, 1000);
        let config = BatchConfig::default();

        // Add transactions with different values
        for i in 1..=3 {
            let micro_tx = MicroTransaction {
                transaction: create_micro_tx(i * 100),
                sender: Address::from_bytes([i as u8; 20]),
                submitted_at: 1000,
            };

            batch.add(micro_tx, 21_000, &config).unwrap();
        }

        // Total should be 100 + 200 + 300 = 600
        assert_eq!(batch.total_value(), U256::from(600u64));
    }

    #[test]
    fn test_value_too_large() {
        let config = BatchConfig {
            max_tx_value: U256::from(500u64),
            ..Default::default()
        };

        let mut batcher = TransactionBatcher::new(config);

        // Transaction with value 1000 > max 500
        let tx = create_micro_tx(1000);
        let result = batcher.submit(tx, Address::ZERO, 1000);

        assert!(result.is_err());
    }

    #[test]
    fn test_batch_timeout() {
        let config = BatchConfig {
            max_wait_time_secs: 60,
            min_batch_size: 10, // High minimum so it won't be ready
            ..Default::default()
        };

        let mut batch = TransactionBatch::new(1, 1000);

        // Add a transaction
        let micro_tx = MicroTransaction {
            transaction: create_micro_tx(100),
            sender: Address::ZERO,
            submitted_at: 1000,
        };
        batch.add(micro_tx, 21_000, &config).unwrap();

        // Not timed out yet
        assert!(!batch.is_timed_out(1050, &config));

        // Timed out after 60+ seconds
        assert!(batch.is_timed_out(1061, &config));
    }
}
