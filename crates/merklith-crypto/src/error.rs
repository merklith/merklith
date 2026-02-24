use thiserror::Error;

/// Errors that can occur in cryptographic operations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CryptoError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Invalid private key")]
    InvalidPrivateKey,

    #[error("Signature verification failed")]
    VerificationFailed,

    #[error("BLS aggregation failed: {0}")]
    BLSAggregationError(String),

    #[error("VRF proof invalid")]
    VRFProofInvalid,

    #[error("Merkle proof invalid")]
    MerkleProofInvalid,

    #[error("Keystore error: {0}")]
    KeystoreError(String),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Invalid mnemonic")]
    InvalidMnemonic,

    #[error("RNG error: {0}")]
    RngError(String),

    #[error("Invalid seed length: expected 32, got {0}")]
    InvalidSeedLength(usize),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<std::io::Error> for CryptoError {
    fn from(e: std::io::Error) -> Self {
        CryptoError::KeystoreError(e.to_string())
    }
}

impl From<ed25519_dalek::SignatureError> for CryptoError {
    fn from(_: ed25519_dalek::SignatureError) -> Self {
        CryptoError::InvalidSignature
    }
}
