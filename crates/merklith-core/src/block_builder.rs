//! Block building for proposers.

use crate::fee_market::calculate_base_fee;
use merklith_types::{Address, Block, BlockHeader, ChainConfig, SignedTransaction, TransactionReceipt, U256};

/// Block builder for creating new blocks.
pub struct BlockBuilder {
    /// Parent block header
    parent: BlockHeader,
    /// Chain configuration
    config: ChainConfig,
    /// Pending transactions
    pending_txs: Vec<SignedTransaction>,
    /// Built receipts
    receipts: Vec<TransactionReceipt>,
    /// Gas used so far
    gas_used: u64,
    /// Block value (sum of fees)
    block_value: U256,
}

impl BlockBuilder {
    /// Create a new block builder.
    pub fn new(
        parent: &BlockHeader,
        config: ChainConfig,
    ) -> Self {
        Self {
            parent: parent.clone(),
            config,
            pending_txs: Vec::new(),
            receipts: Vec::new(),
            gas_used: 0,
            block_value: U256::ZERO,
        }
    }

    /// Try to add a transaction to the block.
    /// Returns the receipt if successful.
    pub fn add_transaction(
        &mut self,
        tx: SignedTransaction,
        receipt: TransactionReceipt,
    ) -> Result<(), BuilderError> {
        // Check gas limit
        if self.gas_used + receipt.gas_used > self.config.gas_limit {
            return Err(BuilderError::GasLimitExceeded);
        }

        // Add transaction
        self.gas_used += receipt.gas_used;
        
        // Calculate fee contribution
        let fee = receipt.effective_gas_price * U256::from(receipt.gas_used);
        self.block_value = self.block_value + fee;
        
        self.pending_txs.push(tx);
        self.receipts.push(receipt);

        Ok(())
    }

    /// Get current gas used.
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    /// Get remaining gas.
    pub fn gas_remaining(&self) -> u64 {
        self.config.gas_limit - self.gas_used
    }

    /// Get the number of transactions.
    pub fn tx_count(&self) -> usize {
        self.pending_txs.len()
    }

    /// Get the total block value.
    pub fn block_value(&self) -> U256 {
        self.block_value
    }

    /// Finalize the block and produce a complete Block.
    pub fn finalize(
        self,
        proposer: Address,
        timestamp: u64,
        extra_data: Vec<u8>,
    ) -> Result<Block, BuilderError> {
        // Calculate base fee
        let base_fee = calculate_base_fee(
            &self.parent.base_fee_per_gas,
            self.parent.gas_used,
            self.config.gas_target,
            &self.config,
        );

        // Build header
        let mut header = BlockHeader::new(
            self.parent.compute_hash(),
            self.parent.number + 1,
            timestamp,
            self.config.gas_limit,
            proposer,
        );

        header.gas_used = self.gas_used;
        header.base_fee_per_gas = base_fee;
        header.extra_data = extra_data;

        // Calculate transaction root (simplified - should use proper trie)
        // header.transactions_root = calculate_tx_root(&self.pending_txs);

        // Calculate receipts root (simplified)
        // header.receipts_root = calculate_receipts_root(&self.receipts);

        Ok(Block {
            header,
            transactions: self.pending_txs,
        })
    }

    /// Get pending transactions.
    pub fn pending_transactions(&self,
    ) -> &[SignedTransaction] {
        &self.pending_txs
    }

    /// Get receipts.
    pub fn receipts(&self,
    ) -> &[TransactionReceipt] {
        &self.receipts
    }
}

/// Block builder errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderError {
    GasLimitExceeded,
    InvalidTransaction,
    StateError,
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::GasLimitExceeded => write!(f, "Gas limit exceeded"),
            BuilderError::InvalidTransaction => write!(f, "Invalid transaction"),
            BuilderError::StateError => write!(f, "State error"),
        }
    }
}

impl std::error::Error for BuilderError {}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::Hash;

    #[test]
    fn test_block_builder_creation() {
        let parent = BlockHeader::new(
            Hash::ZERO,
            0,
            1000,
            30000000,
            Address::ZERO,
        );

        let config = ChainConfig::mainnet();
        let builder = BlockBuilder::new(&parent, config);

        assert_eq!(builder.gas_used(), 0);
        assert_eq!(builder.tx_count(), 0);
    }

    #[test]
    fn test_add_transaction() {
        let parent = BlockHeader::new(
            Hash::ZERO,
            0,
            1000,
            30000000,
            Address::ZERO,
        );

        let config = ChainConfig::mainnet();
        let mut builder = BlockBuilder::new(&parent, config);

        // Create a mock transaction
        let tx = SignedTransaction::new(
            merklith_types::Transaction::new(
                1, 0, Some(Address::ZERO), U256::ZERO, 21000,
                U256::from(10u64), U256::from(1u64),
            ),
            merklith_types::Ed25519Signature::from_bytes([0u8; 64]),
            merklith_types::Ed25519PublicKey::from_bytes([0u8; 32]),
        );

        let receipt = TransactionReceipt::new(
            tx.hash(), 0, Hash::ZERO, 1, Address::ZERO, None, true, 21000,
        );

        builder.add_transaction(tx, receipt).unwrap();
        assert_eq!(builder.tx_count(), 1);
        assert_eq!(builder.gas_used(), 21000);
    }
}
