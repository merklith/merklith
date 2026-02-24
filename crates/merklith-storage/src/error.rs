use thiserror::Error;

/// Errors that can occur in storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Key not found")]
    KeyNotFound,

    #[error("Invalid column family: {0}")]
    InvalidColumnFamily(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Trie error: {0}")]
    TrieError(String),

    #[error("State root mismatch: expected {expected}, got {actual}")]
    StateRootMismatch { expected: String, actual: String },

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl From<rocksdb::Error> for StorageError {
    fn from(e: rocksdb::Error) -> Self {
        StorageError::Database(e.to_string())
    }
}

impl From<borsh::io::Error> for StorageError {
    fn from(e: borsh::io::Error) -> Self {
        StorageError::Serialization(e.to_string())
    }
}

impl From<merklith_types::TypesError> for StorageError {
    fn from(e: merklith_types::TypesError) -> Self {
        StorageError::Deserialization(e.to_string())
    }
}
