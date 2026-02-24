//! Block database for block and receipt storage.

use crate::db::{ColumnFamily, Database};
use crate::error::StorageError;
use merklith_types::{Block, BlockHeader, Hash, SignedTransaction, TransactionReceipt};
use borsh::{BorshDeserialize, BorshSerialize};
use std::sync::Arc;

/// Block storage with indexing.
pub struct BlockDB {
    db: Arc<Database>,
}

impl BlockDB {
    /// Create a new BlockDB.
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    // Block storage

    /// Store a block.
    pub fn put_block(&self,
        block: &Block,
    ) -> Result<(), StorageError> {
        let hash = block.hash();
        let number = block.number();

        // Store header
        let header_data = borsh::to_vec(&block.header)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        self.db.put(ColumnFamily::Headers, hash.as_bytes(), &header_data)?;

        // Store body (transactions)
        let body_data = borsh::to_vec(&block.transactions)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        self.db.put(ColumnFamily::Bodies, hash.as_bytes(), &body_data)?;

        // Index by block number
        self.db.put(
            ColumnFamily::BlockIndex,
            &number.to_le_bytes(),
            hash.as_bytes(),
        )?;

        // Index transactions
        for (idx, tx) in block.transactions.iter().enumerate() {
            let tx_hash = tx.hash();
            let location = (hash, idx as u32);
            let location_data = borsh::to_vec(&location)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            self.db.put(ColumnFamily::TxIndex, tx_hash.as_bytes(), &location_data)?;
        }

        Ok(())
    }

    /// Get a block by hash.
    pub fn get_block_by_hash(
        &self,
        hash: &Hash,
    ) -> Result<Option<Block>, StorageError> {
        let header = match self.get_header_by_hash(hash)? {
            Some(h) => h,
            None => return Ok(None),
        };

        let transactions = match self.get_block_transactions(hash)? {
            Some(txs) => txs,
            None => return Ok(None),
        };

        Ok(Some(Block {
            header,
            transactions,
        }))
    }

    /// Get a block by number.
    pub fn get_block_by_number(
        &self,
        number: u64,
    ) -> Result<Option<Block>, StorageError> {
        let hash = match self.get_block_hash_by_number(number)? {
            Some(h) => h,
            None => return Ok(None),
        };

        self.get_block_by_hash(&hash)
    }

    /// Get block header by hash.
    pub fn get_header_by_hash(
        &self,
        hash: &Hash,
    ) -> Result<Option<BlockHeader>, StorageError> {
        if let Some(data) = self.db.get(ColumnFamily::Headers, hash.as_bytes())? {
            let header: BlockHeader = borsh::from_slice(&data)
                .map_err(|e| StorageError::Deserialization(e.to_string()))?;
            Ok(Some(header))
        } else {
            Ok(None)
        }
    }

    /// Get block header by number.
    pub fn get_header_by_number(
        &self,
        number: u64,
    ) -> Result<Option<BlockHeader>, StorageError> {
        let hash = match self.get_block_hash_by_number(number)? {
            Some(h) => h,
            None => return Ok(None),
        };

        self.get_header_by_hash(&hash)
    }

    fn get_block_transactions(
        &self,
        hash: &Hash,
    ) -> Result<Option<Vec<SignedTransaction>>, StorageError> {
        if let Some(data) = self.db.get(ColumnFamily::Bodies, hash.as_bytes())? {
            let transactions: Vec<SignedTransaction> = borsh::from_slice(&data)
                .map_err(|e| StorageError::Deserialization(e.to_string()))?;
            Ok(Some(transactions))
        } else {
            Ok(None)
        }
    }

    fn get_block_hash_by_number(
        &self,
        number: u64,
    ) -> Result<Option<Hash>, StorageError> {
        if let Some(data) = self.db.get(ColumnFamily::BlockIndex, &number.to_le_bytes())? {
            let hash = Hash::from_slice(&data)
                .map_err(|e| StorageError::Deserialization(e.to_string()))?;
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    // Receipt storage

    /// Store receipts for a block.
    pub fn put_receipts(
        &self,
        block_hash: &Hash,
        receipts: &[TransactionReceipt],
    ) -> Result<(), StorageError> {
        let data = borsh::to_vec(receipts)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        self.db.put(ColumnFamily::Receipts, block_hash.as_bytes(), &data)?;
        Ok(())
    }

    /// Get receipts for a block.
    pub fn get_receipts(
        &self,
        block_hash: &Hash,
    ) -> Result<Vec<TransactionReceipt>, StorageError> {
        if let Some(data) = self.db.get(ColumnFamily::Receipts, block_hash.as_bytes())? {
            let receipts: Vec<TransactionReceipt> = borsh::from_slice(&data)
                .map_err(|e| StorageError::Deserialization(e.to_string()))?;
            Ok(receipts)
        } else {
            Ok(vec![])
        }
    }

    /// Get a single transaction receipt.
    pub fn get_receipt(
        &self,
        tx_hash: &Hash,
    ) -> Result<Option<TransactionReceipt>, StorageError> {
        let (block_hash, tx_index) = match self.get_tx_location(tx_hash)? {
            Some(loc) => loc,
            None => return Ok(None),
        };

        let receipts = self.get_receipts(&block_hash)?;
        Ok(receipts.get(tx_index as usize).cloned())
    }

    // Transaction lookup

    /// Get the location of a transaction (block hash + index).
    pub fn get_tx_location(
        &self,
        tx_hash: &Hash,
    ) -> Result<Option<(Hash, u32)>, StorageError> {
        if let Some(data) = self.db.get(ColumnFamily::TxIndex, tx_hash.as_bytes())? {
            let location: (Hash, u32) = borsh::from_slice(&data)
                .map_err(|e| StorageError::Deserialization(e.to_string()))?;
            Ok(Some(location))
        } else {
            Ok(None)
        }
    }

    /// Get a transaction by hash.
    pub fn get_transaction(
        &self,
        tx_hash: &Hash,
    ) -> Result<Option<SignedTransaction>, StorageError> {
        let (block_hash, tx_index) = match self.get_tx_location(tx_hash)? {
            Some(loc) => loc,
            None => return Ok(None),
        };

        let block = match self.get_block_by_hash(&block_hash)? {
            Some(b) => b,
            None => return Ok(None),
        };

        Ok(block.transactions.get(tx_index as usize).cloned())
    }

    // Chain metadata

    /// Get the chain head.
    pub fn get_chain_head(&self,
    ) -> Result<Option<Hash>, StorageError> {
        if let Some(data) = self.db.get(ColumnFamily::Metadata, b"chain_head")? {
            let hash = Hash::from_slice(&data)
                .map_err(|e| StorageError::Deserialization(e.to_string()))?;
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    /// Set the chain head.
    pub fn set_chain_head(
        &self,
        hash: &Hash,
    ) -> Result<(), StorageError> {
        self.db.put(ColumnFamily::Metadata, b"chain_head", hash.as_bytes())?;
        Ok(())
    }

    /// Get the finalized head.
    pub fn get_finalized_head(&self,
    ) -> Result<Option<Hash>, StorageError> {
        if let Some(data) = self.db.get(ColumnFamily::Metadata, b"finalized_head")? {
            let hash = Hash::from_slice(&data)
                .map_err(|e| StorageError::Deserialization(e.to_string()))?;
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    /// Set the finalized head.
    pub fn set_finalized_head(
        &self,
        hash: &Hash,
    ) -> Result<(), StorageError> {
        self.db.put(ColumnFamily::Metadata, b"finalized_head", hash.as_bytes())?;
        Ok(())
    }

    /// Get the latest checkpoint block number.
    pub fn get_latest_checkpoint(&self,
    ) -> Result<Option<u64>, StorageError> {
        if let Some(data) = self.db.get(ColumnFamily::Metadata, b"latest_checkpoint")? {
            let number = u64::from_le_bytes(data.try_into().unwrap_or([0; 8]));
            Ok(Some(number))
        } else {
            Ok(None)
        }
    }

    /// Set the latest checkpoint.
    pub fn set_latest_checkpoint(
        &self,
        number: u64,
    ) -> Result<(), StorageError> {
        self.db.put(
            ColumnFamily::Metadata,
            b"latest_checkpoint",
            &number.to_le_bytes(),
        )?;
        Ok(())
    }

    /// Check if a block exists.
    pub fn has_block(&self,
        hash: &Hash,
    ) -> Result<bool, StorageError> {
        Ok(self.db.get(ColumnFamily::Headers, hash.as_bytes())?.is_some())
    }

    /// Get the current block height.
    pub fn get_height(&self,
    ) -> Result<u64, StorageError> {
        if let Some(head) = self.get_chain_head()? {
            if let Some(header) = self.get_header_by_hash(&head)? {
                return Ok(header.number);
            }
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseConfig;
    use merklith_types::{Address, Transaction, U256, Ed25519PublicKey, Ed25519Signature};
    use tempfile::TempDir;

    fn create_test_block_db() -> (BlockDB, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = DatabaseConfig::default();
        let db = Arc::new(Database::open(temp_dir.path(), &config).unwrap());
        let block_db = BlockDB::new(db);
        (block_db, temp_dir)
    }

    fn create_test_block(number: u64) -> Block {
        let header = BlockHeader::new(
            Hash::ZERO,
            number,
            1000,
            30000000,
            Address::ZERO,
        );

        let tx = Transaction::new(
            1,
            0,
            Some(Address::ZERO),
            U256::from(100u64),
            21000,
            U256::from(10u64),
            U256::from(1u64),
        );

        let sig = Ed25519Signature::from_bytes([1u8; 64]);
        let pk = Ed25519PublicKey::from_bytes([2u8; 32]);
        let signed_tx = SignedTransaction::new(tx, sig, pk);

        Block::new(header, vec![signed_tx])
    }

    #[test]
    fn test_block_storage() {
        let (block_db, _temp) = create_test_block_db();
        let block = create_test_block(1);

        block_db.put_block(&block).unwrap();

        let retrieved = block_db.get_block_by_hash(&block.hash()).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().number(), 1);
    }

    #[test]
    fn test_block_by_number() {
        let (block_db, _temp) = create_test_block_db();
        let block = create_test_block(5);

        block_db.put_block(&block).unwrap();

        let retrieved = block_db.get_block_by_number(5).unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_receipt_storage() {
        let (block_db, _temp) = create_test_block_db();
        let block_hash = Hash::compute(b"block");

        let receipts = vec![
            TransactionReceipt::new(
                Hash::compute(b"tx1"),
                0,
                block_hash,
                1,
                Address::ZERO,
                None,
                true,
                21000,
            ),
        ];

        block_db.put_receipts(&block_hash, &receipts).unwrap();

        let retrieved = block_db.get_receipts(&block_hash).unwrap();
        assert_eq!(retrieved.len(), 1);
    }

    #[test]
    fn test_chain_head() {
        let (block_db, _temp) = create_test_block_db();
        let head = Hash::compute(b"head");

        assert!(block_db.get_chain_head().unwrap().is_none());

        block_db.set_chain_head(&head).unwrap();

        let retrieved = block_db.get_chain_head().unwrap();
        assert_eq!(retrieved, Some(head));
    }

    #[test]
    fn test_transaction_lookup() {
        let (block_db, _temp) = create_test_block_db();
        let block = create_test_block(1);
        let tx_hash = block.transactions[0].hash();

        block_db.put_block(&block).unwrap();

        let location = block_db.get_tx_location(&tx_hash).unwrap();
        assert!(location.is_some());

        let (block_hash, tx_index) = location.unwrap();
        assert_eq!(block_hash, block.hash());
        assert_eq!(tx_index, 0);
    }
}
