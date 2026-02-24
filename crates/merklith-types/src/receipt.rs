use crate::address::Address;
use crate::hash::Hash;
use crate::u256::U256;
use std::fmt;

/// Result of executing a transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct TransactionReceipt {
    /// Hash of the transaction
    pub tx_hash: Hash,
    /// Index of the transaction in the block
    pub tx_index: u32,
    /// Hash of the block containing this TX
    pub block_hash: Hash,
    /// Number of the block containing this TX
    pub block_number: u64,
    /// Address of the sender
    pub from: Address,
    /// Address of the recipient (None for contract creation)
    pub to: Option<Address>,
    /// Address of created contract (if contract creation)
    pub contract_address: Option<Address>,
    /// Whether the transaction succeeded
    pub status: bool,
    /// Gas used by this transaction
    pub gas_used: u64,
    /// Cumulative gas used in the block up to and including this TX
    pub cumulative_gas_used: u64,
    /// Effective gas price paid
    pub effective_gas_price: U256,
    /// Logs emitted during execution
    pub logs: Vec<Log>,
    /// Bloom filter for log topics (for efficient filtering)
    pub logs_bloom: [u8; 256],
    /// Revert reason (if failed, optional ABI-encoded reason)
    pub revert_reason: Option<Vec<u8>>,
}

impl Default for TransactionReceipt {
    fn default() -> Self {
        Self {
            tx_hash: Hash::ZERO,
            tx_index: 0,
            block_hash: Hash::ZERO,
            block_number: 0,
            from: Address::ZERO,
            to: None,
            contract_address: None,
            status: false,
            gas_used: 0,
            cumulative_gas_used: 0,
            effective_gas_price: U256::ZERO,
            logs: Vec::new(),
            logs_bloom: [0u8; 256],
            revert_reason: None,
        }
    }
}

impl TransactionReceipt {
    /// Create a new receipt
    pub fn new(
        tx_hash: Hash,
        tx_index: u32,
        block_hash: Hash,
        block_number: u64,
        from: Address,
        to: Option<Address>,
        status: bool,
        gas_used: u64,
    ) -> Self {
        Self {
            tx_hash,
            tx_index,
            block_hash,
            block_number,
            from,
            to,
            contract_address: None,
            status,
            gas_used,
            cumulative_gas_used: 0,
            effective_gas_price: U256::ONE,
            logs: Vec::new(),
            logs_bloom: [0u8; 256],
            revert_reason: None,
        }
    }

    /// Check if transaction succeeded
    pub fn is_success(&self) -> bool {
        self.status
    }

    /// Check if transaction failed
    pub fn is_failure(&self) -> bool {
        !self.status
    }

    /// Add a log
    pub fn add_log(&mut self, log: Log) {
        self.logs.push(log);
    }

    /// Set contract address (for contract creation)
    pub fn set_contract_address(&mut self, address: Address) {
        self.contract_address = Some(address);
    }

    /// Set revert reason
    pub fn set_revert_reason(&mut self, reason: Vec<u8>) {
        self.revert_reason = Some(reason);
    }

    /// Calculate the receipts root for a list of receipts
    pub fn calculate_root(receipts: &[Self]) -> Hash {
        // Simple implementation - in production use proper trie
        if receipts.is_empty() {
            return Hash::ZERO;
        }

        let mut data = Vec::new();
        for receipt in receipts {
            data.extend_from_slice(receipt.tx_hash.as_bytes());
        }
        Hash::compute(&data)
    }
}

/// Event log emitted by a smart contract.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct Log {
    /// Address of the contract that emitted this log
    pub address: Address,
    /// Indexed topics (max 4, first is usually event signature hash)
    pub topics: Vec<Hash>,
    /// Non-indexed event data
    pub data: Vec<u8>,
    /// Log index within the block
    pub log_index: u32,
    /// Transaction index within the block
    pub tx_index: u32,
}

impl Log {
    /// Create a new log
    pub fn new(address: Address, topics: Vec<Hash>, data: Vec<u8>) -> Self {
        Self {
            address,
            topics,
            data,
            log_index: 0,
            tx_index: 0,
        }
    }

    /// Get the event signature (first topic)
    pub fn event_signature(&self) -> Option<&Hash> {
        self.topics.first()
    }

    /// Check if log has a specific topic
    pub fn has_topic(&self, topic: &Hash) -> bool {
        self.topics.contains(topic)
    }

    /// Add a topic
    pub fn add_topic(&mut self, topic: Hash) {
        self.topics.push(topic);
    }
}

impl fmt::Display for TransactionReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Receipt {{ tx: {}, status: {}, gas_used: {} }}",
            self.tx_hash,
            if self.status { "success" } else { "failure" },
            self.gas_used
        )
    }
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Log {{ address: {}, topics: {}, data: {} bytes }}",
            self.address,
            self.topics.len(),
            self.data.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_new() {
        let tx_hash = Hash::compute(b"tx");
        let block_hash = Hash::compute(b"block");
        let from = Address::from_bytes([1u8; 20]);

        let receipt = TransactionReceipt::new(
            tx_hash,
            0, // tx_index
            block_hash,
            1, // block_number
            from,
            Some(Address::ZERO),
            true, // success
            21000,
        );

        assert_eq!(receipt.tx_hash, tx_hash);
        assert!(receipt.is_success());
        assert!(!receipt.is_failure());
        assert_eq!(receipt.gas_used, 21000);
    }

    #[test]
    fn test_receipt_failure() {
        let receipt = TransactionReceipt::new(
            Hash::ZERO,
            0,
            Hash::ZERO,
            1,
            Address::ZERO,
            None,
            false, // failure
            21000,
        );

        assert!(!receipt.is_success());
        assert!(receipt.is_failure());
    }

    #[test]
    fn test_receipt_contract_creation() {
        let mut receipt = TransactionReceipt::new(
            Hash::ZERO,
            0,
            Hash::ZERO,
            1,
            Address::ZERO,
            None,
            true,
            100000,
        );

        let contract_addr = Address::from_bytes([2u8; 20]);
        receipt.set_contract_address(contract_addr);

        assert_eq!(receipt.contract_address, Some(contract_addr));
    }

    #[test]
    fn test_receipt_logs() {
        let mut receipt = TransactionReceipt::new(
            Hash::ZERO,
            0,
            Hash::ZERO,
            1,
            Address::ZERO,
            None,
            true,
            21000,
        );

        let log = Log::new(
            Address::ZERO,
            vec![Hash::compute(b"event")],
            vec![1u8, 2u8, 3u8],
        );

        receipt.add_log(log);
        assert_eq!(receipt.logs.len(), 1);
    }

    #[test]
    fn test_log_topics() {
        let topic1 = Hash::compute(b"event");
        let topic2 = Hash::compute(b"indexed1");

        let log = Log::new(
            Address::ZERO,
            vec![topic1, topic2],
            vec![],
        );

        assert_eq!(log.topics.len(), 2);
        assert_eq!(log.event_signature(), Some(&topic1));
        assert!(log.has_topic(&topic1));
        assert!(log.has_topic(&topic2));
    }

    #[test]
    fn test_receipt_root() {
        let receipts = vec![
            TransactionReceipt::new(
                Hash::compute(b"tx1"),
                0,
                Hash::ZERO,
                1,
                Address::ZERO,
                None,
                true,
                21000,
            ),
            TransactionReceipt::new(
                Hash::compute(b"tx2"),
                1,
                Hash::ZERO,
                1,
                Address::ZERO,
                None,
                true,
                21000,
            ),
        ];

        let root = TransactionReceipt::calculate_root(&receipts);
        assert!(!root.is_zero());
    }

    #[test]
    fn test_receipt_root_empty() {
        let root = TransactionReceipt::calculate_root(&[]);
        assert!(root.is_zero());
    }
}
