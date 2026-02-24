//! Error types for the SDK.

use thiserror::Error;

/// SDK result type.
pub type Result<T> = std::result::Result<T, SdkError>;

/// SDK errors.
#[derive(Error, Debug, Clone)]
pub enum SdkError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// RPC error
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Invalid address
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid transaction
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Contract error
    #[error("Contract error: {0}")]
    Contract(String),

    /// Wallet error
    #[error("Wallet error: {0}")]
    Wallet(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Timeout
    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<reqwest::Error> for SdkError {
    fn from(e: reqwest::Error) -> Self {
        SdkError::Connection(e.to_string())
    }
}

impl From<serde_json::Error> for SdkError {
    fn from(e: serde_json::Error) -> Self {
        SdkError::Serialization(e.to_string())
    }
}
