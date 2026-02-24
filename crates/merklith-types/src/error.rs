use thiserror::Error;

/// Errors that can occur in type operations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum TypesError {
    #[error("Invalid address format: {0}")]
    InvalidAddressFormat(String),

    #[error("Invalid address length: expected 20, got {0}")]
    InvalidAddressLength(usize),

    #[error("Invalid hash length: expected 32, got {0}")]
    InvalidHashLength(usize),

    #[error("Invalid signature length: expected {expected}, got {actual}")]
    InvalidSignatureLength { expected: usize, actual: usize },

    #[error("Invalid public key length: expected {expected}, got {actual}")]
    InvalidPublicKeyLength { expected: usize, actual: usize },

    #[error("U256 overflow")]
    U256Overflow,

    #[error("U256 underflow")]
    U256Underflow,

    #[error("U256 division by zero")]
    U256DivisionByZero,

    #[error("Invalid U256 decimal string: {0}")]
    InvalidU256String(String),

    #[error("Invalid hex: {0}")]
    InvalidHex(String),

    #[error("Bech32 error: {0}")]
    Bech32Error(String),

    #[error("Extra data too long: max {max}, got {actual}")]
    ExtraDataTooLong { max: usize, actual: usize },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid chain ID: {0}")]
    InvalidChainId(u64),

    #[error("Invalid block number: {0}")]
    InvalidBlockNumber(u64),

    #[error("Invalid gas limit: {0}")]
    InvalidGasLimit(u64),

    #[error("Invalid nonce: {0}")]
    InvalidNonce(u64),
}

impl From<hex::FromHexError> for TypesError {
    fn from(e: hex::FromHexError) -> Self {
        TypesError::InvalidHex(e.to_string())
    }
}

impl From<std::array::TryFromSliceError> for TypesError {
    fn from(_: std::array::TryFromSliceError) -> Self {
        TypesError::Serialization("Slice length mismatch".to_string())
    }
}

impl From<std::num::ParseIntError> for TypesError {
    fn from(e: std::num::ParseIntError) -> Self {
        TypesError::InvalidU256String(e.to_string())
    }
}
