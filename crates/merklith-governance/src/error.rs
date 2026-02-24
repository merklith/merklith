use thiserror::Error;

/// Errors that can occur in governance operations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum GovernanceError {
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),

    #[error("Proposal not found: {0}")]
    ProposalNotFound(u64),

    #[error("Proposal already executed")]
    AlreadyExecuted,

    #[error("Proposal not yet executable")]
    NotExecutable,

    #[error("Voting period not started")]
    VotingNotStarted,

    #[error("Voting period ended")]
    VotingEnded,

    #[error("Insufficient voting power: {0}")]
    InsufficientVotingPower(String),

    #[error("Already voted")]
    AlreadyVoted,

    #[error("Invalid delegation: {0}")]
    InvalidDelegation(String),

    #[error("Delegation cycle detected")]
    DelegationCycle,

    #[error("Self-delegation not allowed")]
    SelfDelegation,

    #[error("Invalid lock duration")]
    InvalidLockDuration,

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Treasury operation failed: {0}")]
    TreasuryError(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Quorum not reached: {actual} < {required}")]
    QuorumNotReached { actual: u64, required: u64 },

    #[error("Threshold not reached")]
    ThresholdNotReached,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GovernanceError::InvalidProposal("test".to_string());
        assert!(err.to_string().contains("Invalid proposal"));
    }

    #[test]
    fn test_quorum_error() {
        let err = GovernanceError::QuorumNotReached { actual: 100, required: 200 };
        assert!(err.to_string().contains("100"));
        assert!(err.to_string().contains("200"));
    }
}
