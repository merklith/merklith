use crate::error::StorageError;
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use std::path::Path;
use std::sync::Arc;

/// Column families for organized data storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnFamily {
    /// Block headers: block_hash → BlockHeader
    Headers,
    /// Block bodies: block_hash → Vec<SignedTransaction>
    Bodies,
    /// Receipts: block_hash → Vec<TransactionReceipt>
    Receipts,
    /// State trie nodes: node_hash → encoded_node
    StateTrie,
    /// Contract code: code_hash → wasm_bytecode
    ContractCode,
    /// Block number index: block_number → block_hash
    BlockIndex,
    /// Transaction index: tx_hash → (block_hash, tx_index)
    TxIndex,
    /// Metadata: key → value (chain head, latest finalized, etc.)
    Metadata,
    /// Validator data: address → ValidatorState
    Validators,
    /// Governance: proposal_id → ProposalState
    Governance,
}

impl ColumnFamily {
    fn name(&self) -> &'static str {
        match self {
            ColumnFamily::Headers => "headers",
            ColumnFamily::Bodies => "bodies",
            ColumnFamily::Receipts => "receipts",
            ColumnFamily::StateTrie => "state_trie",
            ColumnFamily::ContractCode => "contract_code",
            ColumnFamily::BlockIndex => "block_index",
            ColumnFamily::TxIndex => "tx_index",
            ColumnFamily::Metadata => "metadata",
            ColumnFamily::Validators => "validators",
            ColumnFamily::Governance => "governance",
        }
    }

    fn all() -> Vec<ColumnFamily> {
        vec![
            ColumnFamily::Headers,
            ColumnFamily::Bodies,
            ColumnFamily::Receipts,
            ColumnFamily::StateTrie,
            ColumnFamily::ContractCode,
            ColumnFamily::BlockIndex,
            ColumnFamily::TxIndex,
            ColumnFamily::Metadata,
            ColumnFamily::Validators,
            ColumnFamily::Governance,
        ]
    }
}

/// Database configuration options.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Cache size in MB
    pub cache_size_mb: usize,
    /// Max open files
    pub max_open_files: i32,
    /// Compression type
    pub compression: Compression,
    /// Write buffer size in MB
    pub write_buffer_size_mb: usize,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            cache_size_mb: 512,
            max_open_files: 1024,
            compression: Compression::Lz4,
            write_buffer_size_mb: 64,
        }
    }
}

/// Compression type for database.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    None,
    Snappy,
    Zlib,
    Bz2,
    Lz4,
    Lz4hc,
    Zstd,
}

impl Compression {
    fn to_rocksdb(&self) -> rocksdb::DBCompressionType {
        match self {
            Compression::None => rocksdb::DBCompressionType::None,
            Compression::Snappy => rocksdb::DBCompressionType::Snappy,
            Compression::Zlib => rocksdb::DBCompressionType::Zlib,
            Compression::Bz2 => rocksdb::DBCompressionType::Bz2,
            Compression::Lz4 => rocksdb::DBCompressionType::Lz4,
            Compression::Lz4hc => rocksdb::DBCompressionType::Lz4hc,
            Compression::Zstd => rocksdb::DBCompressionType::Zstd,
        }
    }
}

/// RocksDB wrapper with column family support.
pub struct Database {
    db: Arc<DB>,
}

impl Database {
    /// Open a database at the given path.
    pub fn open(path: &Path, config: &DatabaseConfig) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(config.max_open_files);
        
        let cache = rocksdb::Cache::new_lru_cache(config.cache_size_mb * 1024 * 1024);
        let mut block_opts = rocksdb::BlockBasedOptions::default();
        block_opts.set_block_cache(&cache);
        opts.set_block_based_table_factory(&block_opts);
        
        opts.set_compression_type(config.compression.to_rocksdb());
        opts.set_write_buffer_size(config.write_buffer_size_mb * 1024 * 1024);

        let cf_descriptors: Vec<ColumnFamilyDescriptor> = ColumnFamily::all()
            .into_iter()
            .map(|cf| {
                let mut cf_opts = Options::default();
                cf_opts.set_compression_type(config.compression.to_rocksdb());
                ColumnFamilyDescriptor::new(cf.name(), cf_opts)
            })
            .collect();

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)?;
        
        Ok(Self { db: Arc::new(db) })
    }

    /// Get a value from the database.
    pub fn get(&self, cf: ColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let cf_handle = self.db.cf_handle(cf.name())
            .ok_or_else(|| StorageError::InvalidColumnFamily(cf.name().to_string()))?;
        
        let result = self.db.get_cf(&cf_handle, key)?;
        Ok(result)
    }

    /// Put a value into the database.
    pub fn put(&self, cf: ColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let cf_handle = self.db.cf_handle(cf.name())
            .ok_or_else(|| StorageError::InvalidColumnFamily(cf.name().to_string()))?;
        
        self.db.put_cf(&cf_handle, key, value)?;
        Ok(())
    }

    /// Delete a value from the database.
    pub fn delete(&self, cf: ColumnFamily, key: &[u8]) -> Result<(), StorageError> {
        let cf_handle = self.db.cf_handle(cf.name())
            .ok_or_else(|| StorageError::InvalidColumnFamily(cf.name().to_string()))?;
        
        self.db.delete_cf(&cf_handle, key)?;
        Ok(())
    }

    /// Perform a batch write.
    pub fn batch_write(&self, batch: WriteBatch) -> Result<(), StorageError> {
        self.db.write(batch.inner)?;
        Ok(())
    }

    /// Create a new write batch.
    pub fn new_write_batch(&self) -> WriteBatch {
        WriteBatch::new(self.db.clone())
    }

    /// Compact a column family.
    pub fn compact(&self, cf: ColumnFamily) -> Result<(), StorageError> {
        let cf_handle = self.db.cf_handle(cf.name())
            .ok_or_else(|| StorageError::InvalidColumnFamily(cf.name().to_string()))?;
        
        self.db.compact_range_cf(&cf_handle, None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }

    /// Create a snapshot of the database.
    pub fn snapshot(&self) -> DatabaseSnapshot {
        DatabaseSnapshot {
            snapshot: self.db.snapshot(),
            db: self.db.clone(),
        }
    }
}

/// Write batch for atomic operations.
pub struct WriteBatch {
    inner: rocksdb::WriteBatch,
    db: Arc<DB>,
}

impl WriteBatch {
    fn new(db: Arc<DB>) -> Self {
        Self {
            inner: rocksdb::WriteBatch::default(),
            db,
        }
    }

    /// Put a value into the batch.
    pub fn put(&mut self, cf: ColumnFamily, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let cf_handle = self.db.cf_handle(cf.name())
            .ok_or_else(|| StorageError::InvalidColumnFamily(cf.name().to_string()))?;
        
        self.inner.put_cf(&cf_handle, key, value);
        Ok(())
    }

    /// Delete a value in the batch.
    pub fn delete(&mut self, cf: ColumnFamily, key: &[u8]) -> Result<(), StorageError> {
        let cf_handle = self.db.cf_handle(cf.name())
            .ok_or_else(|| StorageError::InvalidColumnFamily(cf.name().to_string()))?;
        
        self.inner.delete_cf(&cf_handle, key);
        Ok(())
    }

    /// Get the batch size.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if batch is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// Database snapshot for consistent reads.
pub struct DatabaseSnapshot<'a> {
    snapshot: rocksdb::Snapshot<'a>,
    db: Arc<DB>,
}

impl<'a> DatabaseSnapshot<'a> {
    /// Get a value from the snapshot.
    pub fn get(&self, cf: ColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let cf_handle = self.db.cf_handle(cf.name())
            .ok_or_else(|| StorageError::InvalidColumnFamily(cf.name().to_string()))?;
        
        let result = self.snapshot.get_cf(&cf_handle, key)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = DatabaseConfig::default();
        let db = Database::open(temp_dir.path(), &config).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_database_open() {
        let (_db, _temp) = create_test_db();
    }

    #[test]
    fn test_put_and_get() {
        let (db, _temp) = create_test_db();
        
        let key = b"test_key";
        let value = b"test_value";
        
        db.put(ColumnFamily::Metadata, key, value).unwrap();
        
        let result = db.get(ColumnFamily::Metadata, key).unwrap();
        assert_eq!(result, Some(value.to_vec()));
    }

    #[test]
    fn test_delete() {
        let (db, _temp) = create_test_db();
        
        let key = b"test_key";
        let value = b"test_value";
        
        db.put(ColumnFamily::Metadata, key, value).unwrap();
        db.delete(ColumnFamily::Metadata, key).unwrap();
        
        let result = db.get(ColumnFamily::Metadata, key).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_batch_write() {
        let (db, _temp) = create_test_db();
        
        let mut batch = db.new_write_batch();
        batch.put(ColumnFamily::Metadata, b"key1", b"value1").unwrap();
        batch.put(ColumnFamily::Metadata, b"key2", b"value2").unwrap();
        batch.delete(ColumnFamily::Metadata, b"key1").unwrap();
        
        db.batch_write(batch).unwrap();
        
        assert_eq!(db.get(ColumnFamily::Metadata, b"key1").unwrap(), None);
        assert_eq!(db.get(ColumnFamily::Metadata, b"key2").unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_snapshot() {
        let (db, _temp) = create_test_db();
        
        let key = b"test_key";
        let value1 = b"value1";
        
        db.put(ColumnFamily::Metadata, key, value1).unwrap();
        
        let snapshot = db.snapshot();
        
        // Update the value
        let value2 = b"value2";
        db.put(ColumnFamily::Metadata, key, value2).unwrap();
        
        // Snapshot should still see the old value
        let snap_result = snapshot.get(ColumnFamily::Metadata, key).unwrap();
        assert_eq!(snap_result, Some(value1.to_vec()));
        
        // Current db should see the new value
        let current_result = db.get(ColumnFamily::Metadata, key).unwrap();
        assert_eq!(current_result, Some(value2.to_vec()));
    }
}
