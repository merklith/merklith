use thiserror::Error;

/// Errors that can occur in consensus operations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ConsensusError {
    #[error("Invalid validator: {0}")]
    InvalidValidator(String),

    #[error("Insufficient stake: {0}")]
    InsufficientStake(String),

    #[error("Invalid proof: {0}")]
    InvalidProof(String),

    #[error("VRF verification failed: {0}")]
    VrfVerificationFailed(String),

    #[error("Committee selection failed: {0}")]
    CommitteeSelectionFailed(String),

    #[error("Not a committee member: {0}")]
    NotCommitteeMember(String),

    #[error("Double proposal detected: validator {0} at slot {1}")]
    DoubleProposal(String, u64),

    #[error("Double attestation detected: validator {0} for block {1}")]
    DoubleAttestation(String, String),

    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Invalid attestation: {0}")]
    InvalidAttestation(String),

    #[error("Finality threshold not reached: {0}")]
    FinalityNotReached(String),

    #[error("Fork choice error: {0}")]
    ForkChoiceError(String),

    #[error("Slashing condition: {0}")]
    SlashingCondition(String),

    #[error("Validator already exists: {0}")]
    ValidatorAlreadyExists(String),

    #[error("Validator not found: {0}")]
    ValidatorNotFound(String),

    #[error("Invalid epoch: expected {expected}, got {actual}")]
    InvalidEpoch { expected: u64, actual: u64 },

    #[error("Invalid slot: expected {expected}, got {actual}")]
    InvalidSlot { expected: u64, actual: u64 },

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ConsensusError::InvalidValidator("test".to_string());
        assert!(err.to_string().contains("Invalid validator"));
    }

    #[test]
    fn test_double_proposal_error() {
        let err = ConsensusError::DoubleProposal("0x1234...".to_string(), 100);
        let msg = err.to_string();
        assert!(msg.contains("Double proposal"));
        assert!(msg.contains("0x1234"));
        assert!(msg.contains("100"));
    }
}
