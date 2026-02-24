use crate::address::Address;
use crate::hash::Hash;
use crate::signature::{BLSSignature, Ed25519Signature};
use crate::transaction::SignedTransaction;
use crate::u256::U256;
use std::fmt;

/// Block header containing metadata and consensus information.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct BlockHeader {
    /// Hash of the parent block header
    pub parent_hash: Hash,
    /// Root hash of the state trie after executing this block
    pub state_root: Hash,
    /// Root hash of the transactions trie
    pub transactions_root: Hash,
    /// Root hash of the receipts trie
    pub receipts_root: Hash,
    /// Block number (height)
    pub number: u64,
    /// Unix timestamp (seconds)
    pub timestamp: u64,
    /// Maximum gas allowed in this block
    pub gas_limit: u64,
    /// Total gas used by all transactions in this block
    pub gas_used: u64,
    /// Base fee per gas unit (EIP-1559 style, burned)
    pub base_fee_per_gas: U256,
    /// Address of the block proposer
    pub proposer: Address,
    /// BLS aggregate signature from committee attestors
    pub attestation_aggregate: BLSSignature,
    /// Bitmap of which committee members attested
    pub attestation_bitmap: Vec<u8>,
    /// Current epoch number
    pub epoch: u64,
    /// Number of attestations (for quick validation)
    pub attestation_count: u32,
    /// Block proposer's ed25519 signature over the header
    pub proposer_signature: Ed25519Signature,
    /// Extra data (max 32 bytes, proposer can include arbitrary data)
    pub extra_data: Vec<u8>,
}

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        parent_hash: Hash,
        number: u64,
        timestamp: u64,
        gas_limit: u64,
        proposer: Address,
    ) -> Self {
        Self {
            parent_hash,
            state_root: Hash::ZERO,
            transactions_root: Hash::ZERO,
            receipts_root: Hash::ZERO,
            number,
            timestamp,
            gas_limit,
            gas_used: 0,
            base_fee_per_gas: U256::ONE,
            proposer,
            attestation_aggregate: BLSSignature::default(),
            attestation_bitmap: Vec::new(),
            epoch: number / 1000,
            attestation_count: 0,
            proposer_signature: Ed25519Signature::default(),
            extra_data: Vec::new(),
        }
    }

    /// Compute the hash of this block header (excluding signatures)
    pub fn compute_hash(&self,
    ) -> Hash {
        // Serialize signable fields
        let mut data = Vec::new();
        data.extend_from_slice(self.parent_hash.as_bytes());
        data.extend_from_slice(self.state_root.as_bytes());
        data.extend_from_slice(self.transactions_root.as_bytes());
        data.extend_from_slice(self.receipts_root.as_bytes());
        data.extend_from_slice(&self.number.to_le_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.gas_limit.to_le_bytes());
        data.extend_from_slice(&self.gas_used.to_le_bytes());
        data.extend_from_slice(&self.base_fee_per_gas.to_le_bytes());
        data.extend_from_slice(self.proposer.as_bytes());
        data.extend_from_slice(&self.epoch.to_le_bytes());
        data.extend_from_slice(&self.extra_data);
        Hash::compute(&data)
    }

    /// Get the hash that the proposer should sign
    pub fn signing_hash(&self) -> Hash {
        self.compute_hash()
    }

    /// Check if this is a genesis block
    pub fn is_genesis(&self) -> bool {
        self.number == 0
    }

    /// Check if this is a checkpoint block (every 100 blocks)
    pub fn is_checkpoint(&self) -> bool {
        self.number % 100 == 0
    }

    /// Check if this is an epoch boundary (every 1000 blocks)
    pub fn is_epoch_boundary(&self) -> bool {
        self.number % 1000 == 0
    }

    /// Set extra data (max 32 bytes)
    pub fn set_extra_data(&mut self, data: Vec<u8>,
    ) -> Result<(), crate::error::TypesError> {
        if data.len() > 32 {
            return Err(crate::error::TypesError::ExtraDataTooLong {
                max: 32,
                actual: data.len(),
            });
        }
        self.extra_data = data;
        Ok(())
    }
}

/// Complete block with header and transactions.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<SignedTransaction>,
}

impl Block {
    /// Create a new block
    pub fn new(header: BlockHeader, transactions: Vec<SignedTransaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    /// Get the block hash
    pub fn hash(&self) -> Hash {
        self.header.compute_hash()
    }

    /// Get the block number
    pub fn number(&self) -> u64 {
        self.header.number
    }

    /// Get transaction count
    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }

    /// Check if this is a genesis block
    pub fn is_genesis(&self) -> bool {
        self.header.is_genesis()
    }

    /// Check if this is a checkpoint block
    pub fn is_checkpoint(&self) -> bool {
        self.header.is_checkpoint()
    }

    /// Calculate total gas used from transactions
    pub fn calculate_gas_used(&self) -> u64 {
        // This would sum actual gas used from receipts
        // For now, return header value
        self.header.gas_used
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block {{ number: {}, hash: {}, txs: {}, gas_used: {} }}",
            self.number(),
            self.hash(),
            self.tx_count(),
            self.header.gas_used
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::Ed25519PublicKey;

    #[test]
    fn test_block_header_new() {
        let parent_hash = Hash::compute(b"parent");
        let proposer = Address::from_bytes([1u8; 20]);

        let header = BlockHeader::new(
            parent_hash,
            1, // number
            1000, // timestamp
            30000000, // gas_limit
            proposer,
        );

        assert_eq!(header.number, 1);
        assert_eq!(header.parent_hash, parent_hash);
        assert_eq!(header.proposer, proposer);
        assert!(!header.is_genesis());
        assert!(!header.is_checkpoint());
        assert!(!header.is_epoch_boundary());
    }

    #[test]
    fn test_block_header_genesis() {
        let header = BlockHeader::new(
            Hash::ZERO,
            0, // Genesis block
            0,
            30000000,
            Address::ZERO,
        );

        assert!(header.is_genesis());
    }

    #[test]
    fn test_block_header_checkpoint() {
        let header = BlockHeader::new(
            Hash::ZERO,
            100, // Checkpoint
            0,
            30000000,
            Address::ZERO,
        );

        assert!(header.is_checkpoint());
        assert!(!header.is_epoch_boundary());
    }

    #[test]
    fn test_block_header_epoch() {
        let header = BlockHeader::new(
            Hash::ZERO,
            1000, // Epoch boundary
            0,
            30000000,
            Address::ZERO,
        );

        assert!(header.is_epoch_boundary());
        assert!(header.is_checkpoint());
    }

    #[test]
    fn test_block_header_hash() {
        let parent_hash = Hash::compute(b"parent");
        let proposer = Address::from_bytes([1u8; 20]);

        let header = BlockHeader::new(
            parent_hash,
            1,
            1000,
            30000000,
            proposer,
        );

        let hash = header.compute_hash();
        assert!(!hash.is_zero());

        // Deterministic
        let hash2 = header.compute_hash();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_block_new() {
        let header = BlockHeader::new(
            Hash::ZERO,
            1,
            1000,
            30000000,
            Address::ZERO,
        );

        let block = Block::new(header, vec![]);

        assert_eq!(block.number(), 1);
        assert_eq!(block.tx_count(), 0);
        assert!(!block.hash().is_zero());
    }

    #[test]
    fn test_block_with_transactions() {
        use crate::transaction::{Transaction, SignedTransaction};

        let header = BlockHeader::new(
            Hash::ZERO,
            1,
            1000,
            30000000,
            Address::ZERO,
        );

        let tx = Transaction::new(
            1,
            0,
            Some(Address::ZERO),
            U256::from(1000u64),
            21000,
            U256::from(10u64),
            U256::from(1u64),
        );

        let sig = crate::signature::Ed25519Signature::from_bytes([1u8; 64]);
        let pk = Ed25519PublicKey::from_bytes([2u8; 32]);
        let signed_tx = SignedTransaction::new(tx, sig, pk);

        let block = Block::new(header, vec![signed_tx]);

        assert_eq!(block.tx_count(), 1);
    }

    #[test]
    fn test_block_header_extra_data() {
        let mut header = BlockHeader::new(
            Hash::ZERO,
            1,
            1000,
            30000000,
            Address::ZERO,
        );

        // Valid extra data
        assert!(header.set_extra_data(vec![1u8; 32]).is_ok());
        assert_eq!(header.extra_data.len(), 32);

        // Too long
        assert!(header.set_extra_data(vec![1u8; 33]).is_err());
    }
}
