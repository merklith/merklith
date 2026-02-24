use thiserror::Error;

/// Errors that can occur in the transaction pool.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum PoolError {
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Transaction already exists: {0}")]
    DuplicateTransaction(String),

    #[error("Transaction pool full: {0}")]
    PoolFull(String),

    #[error("Insufficient balance: have {have}, want {want}")]
    InsufficientBalance { have: u128, want: u128 },

    #[error("Nonce too low: expected {expected}, got {got}")]
    NonceTooLow { expected: u64, got: u64 },

    #[error("Nonce too high: expected {expected}, got {got}")]
    NonceTooHigh { expected: u64, got: u64 },

    #[error("Gas price too low: minimum {minimum}, got {got}")]
    GasPriceTooLow { minimum: u128, got: u128 },

    #[error("Gas limit exceeded: limit {limit}, got {got}")]
    GasLimitExceeded { limit: u64, got: u64 },

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Chain ID mismatch: expected {expected}, got {got}")]
    ChainIdMismatch { expected: u64, got: u64 },

    #[error("Transaction too large: {size} bytes, limit {limit}")]
    TransactionTooLarge { size: usize, limit: usize },

    #[error("Account is not an EOA")]
    NotEoaAccount,

    #[error("Nonce gap detected: missing nonce {nonce} for account {account}")]
    NonceGap { nonce: u64, account: String },

    #[error("Replacement transaction underpriced: need {required}, got {got}")]
    ReplacementUnderpriced { required: u128, got: u128 },

    #[error("Batch error: {0}")]
    BatchError(String),

    #[error("Invalid batch: {0}")]
    InvalidBatch(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PoolError::InvalidTransaction("test".to_string());
        assert!(err.to_string().contains("Invalid transaction"));
    }

    #[test]
    fn test_nonce_error() {
        let err = PoolError::NonceTooLow { expected: 10, got: 5 };
        assert!(err.to_string().contains("10"));
        assert!(err.to_string().contains("5"));
    }
}
