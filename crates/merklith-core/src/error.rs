use thiserror::Error;

/// Errors that can occur in core operations.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Gas limit exceeded: {used} > {limit}")]
    GasLimitExceeded { used: u64, limit: u64 },

    #[error("Insufficient balance: required {required}, have {have}")]
    InsufficientBalance { required: u64, have: u64 },

    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("Chain ID mismatch: expected {expected}, got {got}")]
    ChainIdMismatch { expected: u64, got: u64 },

    #[error("Fee too low: required {required}, offered {offered}")]
    FeeTooLow { required: u64, offered: u64 },

    #[error("Block not found: {0}")]
    BlockNotFound(u64),

    #[error("Parent block not found: {0}")]
    ParentBlockNotFound(String),

    #[error("State root mismatch: expected {expected}, got {actual}")]
    StateRootMismatch { expected: String, actual: String },

    #[error("Genesis error: {0}")]
    GenesisError(String),

    #[error("Storage error: {0}")]
    Storage(#[from] merklith_storage::StorageError),
}
