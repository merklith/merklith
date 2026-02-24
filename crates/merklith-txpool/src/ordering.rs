//! Transaction ordering and prioritization.
//!
//! Orders transactions by:
//! 1. Gas price (higher = better)
//! 2. Time in pool (older = better, for fairness)
//! 3. Sender nonce (lower = better)

use merklith_types::{Transaction, U256};
use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

/// Priority score for a transaction.
#[derive(Debug, Clone, Copy)]
pub struct PriorityScore {
    /// Effective gas price (wei)
    pub gas_price: U256,
    /// Time when transaction entered pool (unix timestamp)
    pub timestamp: u64,
    /// Sender nonce
    pub nonce: u64,
}

impl PriorityScore {
    /// Create a new priority score.
    pub fn new(gas_price: U256, nonce: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            gas_price,
            timestamp,
            nonce,
        }
    }

    /// Create with explicit timestamp.
    pub fn with_timestamp(gas_price: U256, nonce: u64, timestamp: u64) -> Self {
        Self {
            gas_price,
            timestamp,
            nonce,
        }
    }
}

impl PartialEq for PriorityScore {
    fn eq(&self, other: &Self) -> bool {
        self.gas_price == other.gas_price && 
        self.timestamp == other.timestamp &&
        self.nonce == other.nonce
    }
}

impl Eq for PriorityScore {}

impl PartialOrd for PriorityScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityScore {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by gas price (higher is better)
        match other.gas_price.cmp(&self.gas_price) {
            Ordering::Equal => {}
            other => return other,
        }

        // Then compare by timestamp (older is better)
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => {}
            other => return other,
        }

        // Finally compare by nonce (lower is better)
        self.nonce.cmp(&other.nonce)
    }
}

/// Priority ordering for transactions.
#[derive(Debug)]
pub struct TransactionOrdering {
    /// Whether to use time-based prioritization
    pub use_time_priority: bool,
    /// Age bonus factor (priority boost per second in pool)
    pub age_bonus_factor: f64,
}

impl Default for TransactionOrdering {
    fn default() -> Self {
        Self {
            use_time_priority: true,
            age_bonus_factor: 0.01, // 1% per 100 seconds
        }
    }
}

impl TransactionOrdering {
    /// Create a new ordering configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate effective priority for a transaction.
    ///
    /// Considers gas price and time in pool.
    pub fn calculate_priority(
        &self,
        gas_price: U256,
        timestamp: u64,
        current_time: u64,
    ) -> U256 {
        if !self.use_time_priority {
            return gas_price;
        }

        // Calculate age bonus
        let age = current_time.saturating_sub(timestamp);
        let age_bonus = 1.0 + (age as f64 * self.age_bonus_factor);

        // Apply bonus to gas price
        // Note: This is a simplified version
        let adjusted_price = (gas_price.as_u128() as f64 * age_bonus) as u128;
        U256::from(adjusted_price)
    }

    /// Compare two transactions for ordering.
    pub fn compare(
        &self,
        a_gas_price: U256,
    a_timestamp: u64,
        a_nonce: u64,
        b_gas_price: U256,
        b_timestamp: u64,
        b_nonce: u64,
    ) -> Ordering {
        let score_a = PriorityScore::with_timestamp(a_gas_price, a_nonce, a_timestamp);
        let score_b = PriorityScore::with_timestamp(b_gas_price, b_nonce, b_timestamp);

        score_a.cmp(&score_b)
    }
}

/// Transaction with ordering metadata.
#[derive(Debug, Clone)]
pub struct OrderedTransaction {
    /// The transaction
    pub transaction: Transaction,
    /// When it entered the pool
    pub timestamp: u64,
    /// Priority score (cached)
    pub priority: PriorityScore,
}

impl OrderedTransaction {
    /// Create a new ordered transaction.
    pub fn new(transaction: Transaction) -> Self {
        let priority = PriorityScore::new(transaction.gas_price, transaction.nonce);
        let timestamp = priority.timestamp;

        Self {
            transaction,
            timestamp,
            priority,
        }
    }

    /// Create with explicit timestamp.
    pub fn with_timestamp(transaction: Transaction, timestamp: u64) -> Self {
        let priority = PriorityScore::with_timestamp(transaction.gas_price, transaction.nonce, timestamp);

        Self {
            transaction,
            timestamp,
            priority,
        }
    }

    /// Update priority (e.g., when gas price changes).
    pub fn update_priority(&mut self) {
        self.priority = PriorityScore::with_timestamp(
            self.transaction.gas_price,
            self.transaction.nonce,
            self.timestamp,
        );
    }
}

impl PartialEq for OrderedTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for OrderedTransaction {}

impl PartialOrd for OrderedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// Batch ordering for efficient retrieval.
///
/// Maintains transactions sorted by priority.
#[derive(Debug)]
pub struct PriorityQueue {
    transactions: Vec<OrderedTransaction>,
    ordering: TransactionOrdering,
}

impl PriorityQueue {
    /// Create a new priority queue.
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            ordering: TransactionOrdering::default(),
        }
    }

    /// Create with custom ordering.
    pub fn with_ordering(ordering: TransactionOrdering) -> Self {
        Self {
            transactions: Vec::new(),
            ordering,
        }
    }

    /// Add a transaction to the queue.
    pub fn push(&mut self, tx: OrderedTransaction) {
        self.transactions.push(tx);
        self.heapify_up(self.transactions.len() - 1);
    }

    /// Remove and return the highest priority transaction.
    pub fn pop(&mut self) -> Option<OrderedTransaction> {
        if self.transactions.is_empty() {
            return None;
        }

        let last = self.transactions.len() - 1;
        self.transactions.swap(0, last);
        let result = self.transactions.pop();

        if !self.transactions.is_empty() {
            self.heapify_down(0);
        }

        result
    }

    /// Peek at the highest priority transaction.
    pub fn peek(&self) -> Option<&OrderedTransaction> {
        self.transactions.first()
    }

    /// Get the number of transactions.
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Remove a specific transaction by predicate.
    pub fn remove_if<F>(&mut self, predicate: F) -> Option<OrderedTransaction>
    where
        F: Fn(&OrderedTransaction) -> bool,
    {
        if let Some(index) = self.transactions.iter().position(predicate) {
            let last = self.transactions.len() - 1;
            self.transactions.swap(index, last);
            let result = self.transactions.pop();

            if index < self.transactions.len() {
                self.heapify_up(index);
                self.heapify_down(index);
            }

            return result;
        }

        None
    }

    /// Heapify up from index.
    fn heapify_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if self.transactions[index].priority <= self.transactions[parent].priority {
                break;
            }
            self.transactions.swap(index, parent);
            index = parent;
        }
    }

    /// Heapify down from index.
    fn heapify_down(&mut self, mut index: usize) {
        let len = self.transactions.len();

        loop {
            let left = 2 * index + 1;
            let right = 2 * index + 2;
            let mut largest = index;

            if left < len && self.transactions[left].priority > self.transactions[largest].priority {
                largest = left;
            }

            if right < len && self.transactions[right].priority > self.transactions[largest].priority {
                largest = right;
            }

            if largest == index {
                break;
            }

            self.transactions.swap(index, largest);
            index = largest;
        }
    }
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::{Transaction, TransactionType, Address};

    fn create_tx(gas_price: u64, nonce: u64) -> Transaction {
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

    #[test]
    fn test_priority_score_ordering() {
        // Higher gas price = higher priority
        let score1 = PriorityScore::with_timestamp(U256::from(100), 0, 1000);
        let score2 = PriorityScore::with_timestamp(U256::from(200), 0, 1000);

        assert!(score2 > score1);

        // Same gas price, older = higher priority
        let score3 = PriorityScore::with_timestamp(U256::from(100), 0, 500);
        let score4 = PriorityScore::with_timestamp(U256::from(100), 0, 1000);

        assert!(score3 > score4);

        // Same gas price and time, lower nonce = higher priority
        let score5 = PriorityScore::with_timestamp(U256::from(100), 1, 1000);
        let score6 = PriorityScore::with_timestamp(U256::from(100), 2, 1000);

        assert!(score5 > score6);
    }

    #[test]
    fn test_priority_queue() {
        let mut queue = PriorityQueue::new();

        let tx1 = OrderedTransaction::with_timestamp(create_tx(100, 0), 1000);
        let tx2 = OrderedTransaction::with_timestamp(create_tx(200, 0), 1000);
        let tx3 = OrderedTransaction::with_timestamp(create_tx(150, 0), 1000);

        queue.push(tx1);
        queue.push(tx2);
        queue.push(tx3);

        // Should pop in order: 200, 150, 100
        let first = queue.pop().unwrap();
        assert_eq!(first.transaction.gas_price, U256::from(200));

        let second = queue.pop().unwrap();
        assert_eq!(second.transaction.gas_price, U256::from(150));

        let third = queue.pop().unwrap();
        assert_eq!(third.transaction.gas_price, U256::from(100));

        assert!(queue.is_empty());
    }

    #[test]
    fn test_priority_queue_with_time() {
        let mut queue = PriorityQueue::new();

        // Same gas price, different times
        let tx1 = OrderedTransaction::with_timestamp(create_tx(100, 0), 1000);
        let tx2 = OrderedTransaction::with_timestamp(create_tx(100, 0), 500);

        queue.push(tx1);
        queue.push(tx2);

        // Older transaction should come first
        let first = queue.pop().unwrap();
        assert_eq!(first.timestamp, 500);
    }

    #[test]
    fn test_priority_queue_remove() {
        let mut queue = PriorityQueue::new();

        let tx1 = OrderedTransaction::with_timestamp(create_tx(100, 0), 1000);
        let tx2 = OrderedTransaction::with_timestamp(create_tx(200, 1), 1000);

        queue.push(tx1.clone());
        queue.push(tx2);

        // Remove by nonce
        let removed = queue.remove_if(|ot| ot.transaction.nonce == 0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().transaction.nonce, 0);

        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_transaction_ordering_calculation() {
        let ordering = TransactionOrdering::default();

        let now = 1000;
        let old_ts = 500;

        // Same gas price, older should get boost
        let priority_old = ordering.calculate_priority(
            U256::from(100),
            old_ts,
            now,
        );
        let priority_new = ordering.calculate_priority(
            U256::from(100),
            now,
            now,
        );

        assert!(priority_old > priority_new);
    }

    #[test]
    fn test_ordered_transaction_cmp() {
        let tx1 = OrderedTransaction::with_timestamp(create_tx(200, 0), 1000);
        let tx2 = OrderedTransaction::with_timestamp(create_tx(100, 0), 1000);

        assert!(tx1 > tx2);
        assert!(tx2 < tx1);
    }
}
