//! Proposal lifecycle management.
//!
//! Proposals go through states: Pending -> Active -> Succeeded/Defeated -> Executed

use std::collections::HashMap;
use merklith_types::{Address, U256};
use crate::error::GovernanceError;

/// Proposal status in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    /// Proposal created, waiting for voting to start
    Pending,
    /// Voting is active
    Active,
    /// Voting ended, proposal succeeded
    Succeeded,
    /// Voting ended, proposal failed
    Defeated,
    /// Proposal was executed
    Executed,
    /// Proposal was cancelled
    Cancelled,
    /// Proposal expired without execution
    Expired,
}

impl ProposalStatus {
    /// Check if proposal is in active voting period.
    pub fn is_active(&self) -> bool {
        matches!(self, ProposalStatus::Active)
    }

    /// Check if proposal can be executed.
    pub fn is_executable(&self) -> bool {
        matches!(self, ProposalStatus::Succeeded)
    }

    /// Check if voting is still possible.
    pub fn can_vote(&self) -> bool {
        matches!(self, ProposalStatus::Active)
    }
}

/// Type of governance proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalType {
    /// Protocol parameter change
    ParameterChange,
    /// Treasury spending
    TreasurySpending,
    /// Contract upgrade
    ContractUpgrade,
    /// Emergency action
    Emergency,
    /// Custom action
    Custom { code: u8 },
}

impl ProposalType {
    /// Get default voting period for this type.
    pub fn default_voting_period(&self) -> u64 {
        match self {
            ProposalType::ParameterChange => 100_800, // ~1 week at 6s blocks
            ProposalType::TreasurySpending => 100_800,
            ProposalType::ContractUpgrade => 201_600, // ~2 weeks
            ProposalType::Emergency => 14_400,        // ~1 day
            ProposalType::Custom { .. } => 100_800,
        }
    }

    /// Get quorum requirement for this type (percentage * 100).
    pub fn quorum_bps(&self) -> u16 {
        match self {
            ProposalType::ParameterChange => 400,  // 4%
            ProposalType::TreasurySpending => 400,
            ProposalType::ContractUpgrade => 1000, // 10%
            ProposalType::Emergency => 2500,       // 25%
            ProposalType::Custom { .. } => 400,
        }
    }

    /// Get approval threshold (percentage * 100, simple majority = 5000).
    pub fn threshold_bps(&self) -> u16 {
        match self {
            ProposalType::ParameterChange => 5000,  // 50%
            ProposalType::TreasurySpending => 5000,
            ProposalType::ContractUpgrade => 6000,  // 60%
            ProposalType::Emergency => 6600,        // 66%
            ProposalType::Custom { .. } => 5000,
        }
    }
}

/// On-chain proposal.
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Unique proposal ID
    pub id: u64,
    /// Proposal type
    pub proposal_type: ProposalType,
    /// Current status
    pub status: ProposalStatus,
    /// Proposer address
    pub proposer: Address,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Call data for execution
    pub call_data: Vec<u8>,
    /// Target contract (if any)
    pub target: Option<Address>,
    /// Value to transfer (for treasury)
    pub value: U256,
    /// Block when voting starts
    pub start_block: u64,
    /// Block when voting ends
    pub end_block: u64,
    /// For votes (weighted)
    pub for_votes: U256,
    /// Against votes (weighted)
    pub against_votes: U256,
    /// Abstain votes (weighted)
    pub abstain_votes: U256,
    /// Has voted (voter -> true)
    pub has_voted: HashMap<Address, bool>,
    /// Block when executed
    pub executed_at: Option<u64>,
    /// Block when cancelled
    pub cancelled_at: Option<u64>,
    /// Total token supply at creation (for quorum calculation)
    pub total_supply: U256,
}

impl Proposal {
    /// Create a new proposal.
    pub fn new(
        id: u64,
        proposal_type: ProposalType,
        proposer: Address,
        title: String,
        description: String,
        start_block: u64,
        current_supply: U256,
    ) -> Self {
        let voting_period = proposal_type.default_voting_period();
        
        Self {
            id,
            proposal_type,
            status: ProposalStatus::Pending,
            proposer,
            title,
            description,
            call_data: Vec::new(),
            target: None,
            value: U256::ZERO,
            start_block,
            end_block: start_block + voting_period,
            for_votes: U256::ZERO,
            against_votes: U256::ZERO,
            abstain_votes: U256::ZERO,
            has_voted: HashMap::new(),
            executed_at: None,
            cancelled_at: None,
            total_supply: current_supply,
        }
    }

    /// Set call data for execution.
    pub fn with_call_data(mut self, target: Address, call_data: Vec<u8>, value: U256) -> Self {
        self.target = Some(target);
        self.call_data = call_data;
        self.value = value;
        self
    }

    /// Start voting (transition from Pending to Active).
    pub fn start_voting(&mut self, current_block: u64) -> Result<(), GovernanceError> {
        if self.status != ProposalStatus::Pending {
            return Err(GovernanceError::InvalidProposal(
                format!("Cannot start voting from status {:?}", self.status)
            ));
        }

        if current_block < self.start_block {
            return Err(GovernanceError::VotingNotStarted);
        }

        self.status = ProposalStatus::Active;
        Ok(())
    }

    /// Cast a vote.
    pub fn cast_vote(
        &mut self,
        voter: Address,
        support: VoteSupport,
        voting_power: U256,
    ) -> Result<(), GovernanceError> {
        if !self.status.can_vote() {
            return Err(GovernanceError::VotingEnded);
        }

        if self.has_voted.contains_key(&voter) {
            return Err(GovernanceError::AlreadyVoted);
        }

        if voting_power == U256::ZERO {
            return Err(GovernanceError::InsufficientVotingPower(
                "Zero voting power".to_string()
            ));
        }

        // Record vote
        match support {
            VoteSupport::For => self.for_votes += voting_power,
            VoteSupport::Against => self.against_votes += voting_power,
            VoteSupport::Abstain => self.abstain_votes += voting_power,
        }

        self.has_voted.insert(voter, true);
        Ok(())
    }

    /// End voting and determine outcome.
    pub fn end_voting(&mut self, current_block: u64) -> Result<ProposalStatus, GovernanceError> {
        if self.status != ProposalStatus::Active {
            return Err(GovernanceError::InvalidProposal(
                format!("Cannot end voting from status {:?}", self.status)
            ));
        }

        if current_block < self.end_block {
            return Err(GovernanceError::VotingNotStarted);
        }

        // Check quorum - use saturating arithmetic to prevent overflow
        let total_votes = self.for_votes.saturating_add(&self.against_votes).saturating_add(&self.abstain_votes);
        let quorum_threshold = (self.total_supply
            .saturating_mul(&U256::from(self.proposal_type.quorum_bps())))
            / U256::from(10000u64);
        
        if total_votes < quorum_threshold {
            self.status = ProposalStatus::Defeated;
            return Ok(self.status);
        }

        // Check approval threshold - use saturating arithmetic to prevent overflow
        let threshold_bps = self.proposal_type.threshold_bps();
        let total_decisive_votes = self.for_votes.saturating_add(&self.against_votes);
        
        if total_decisive_votes == U256::ZERO {
            self.status = ProposalStatus::Defeated;
            return Ok(self.status);
        }

        let for_percentage = (self.for_votes
            .saturating_mul(&U256::from(10000u64)))
            / total_decisive_votes;
        
        if for_percentage >= U256::from(threshold_bps) {
            self.status = ProposalStatus::Succeeded;
        } else {
            self.status = ProposalStatus::Defeated;
        }

        Ok(self.status)
    }

    /// Execute a succeeded proposal.
    pub fn execute(&mut self, current_block: u64) -> Result<(), GovernanceError> {
        if self.status != ProposalStatus::Succeeded {
            return Err(GovernanceError::NotExecutable);
        }

        // Check execution window (e.g., must execute within 30 days)
        let execution_deadline = self.end_block + 432_000; // ~30 days
        if current_block > execution_deadline {
            self.status = ProposalStatus::Expired;
            return Err(GovernanceError::NotExecutable);
        }

        self.status = ProposalStatus::Executed;
        self.executed_at = Some(current_block);
        Ok(())
    }

    /// Cancel proposal (only by proposer before voting starts).
    pub fn cancel(&mut self, caller: Address, current_block: u64) -> Result<(), GovernanceError> {
        if caller != self.proposer {
            return Err(GovernanceError::Unauthorized(
                "Only proposer can cancel".to_string()
            ));
        }

        if self.status != ProposalStatus::Pending && self.status != ProposalStatus::Active {
            return Err(GovernanceError::AlreadyExecuted);
        }

        self.status = ProposalStatus::Cancelled;
        self.cancelled_at = Some(current_block);
        Ok(())
    }

    /// Get total votes cast.
    pub fn total_votes(&self) -> U256 {
        self.for_votes + self.against_votes + self.abstain_votes
    }

    /// Check if voter has voted.
    pub fn has_voted(&self, voter: &Address) -> bool {
        self.has_voted.contains_key(voter)
    }
}

/// Vote support options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteSupport {
    /// Vote in favor
    For,
    /// Vote against
    Against,
    /// Abstain (counts toward quorum but not threshold)
    Abstain,
}

/// Proposal registry managing all proposals.
#[derive(Debug)]
pub struct ProposalRegistry {
    proposals: HashMap<u64, Proposal>,
    next_id: u64,
}

impl ProposalRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new proposal.
    pub fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        proposer: Address,
        title: String,
        description: String,
        start_block: u64,
        current_supply: U256,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let proposal = Proposal::new(
            id,
            proposal_type,
            proposer,
            title,
            description,
            start_block,
            current_supply,
        );

        self.proposals.insert(id, proposal);
        id
    }

    /// Get a proposal.
    pub fn get(&self, id: u64) -> Option<&Proposal> {
        self.proposals.get(&id)
    }

    /// Get a proposal mutably.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Proposal> {
        self.proposals.get_mut(&id)
    }

    /// Get all proposals.
    pub fn all(&self) -> Vec<&Proposal> {
        self.proposals.values().collect()
    }

    /// Get proposals by status.
    pub fn by_status(&self, status: ProposalStatus) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == status)
            .collect()
    }

    /// Get active proposals.
    pub fn active(&self) -> Vec<&Proposal> {
        self.by_status(ProposalStatus::Active)
    }
}

impl Default for ProposalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            Address::ZERO,
            "Test Proposal".to_string(),
            "Description".to_string(),
            100,
            U256::from(1_000_000u128),
        );

        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.status, ProposalStatus::Pending);
        assert_eq!(proposal.proposer, Address::ZERO);
        assert!(proposal.end_block > proposal.start_block);
    }

    #[test]
    fn test_start_voting() {
        let mut proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            Address::ZERO,
            "Test".to_string(),
            "Description".to_string(),
            100,
            U256::from(1_000_000u128),
        );

        // Can't start before start_block
        assert!(proposal.start_voting(50).is_err());

        // Can start at or after start_block
        assert!(proposal.start_voting(100).is_ok());
        assert_eq!(proposal.status, ProposalStatus::Active);

        // Can't start again
        assert!(proposal.start_voting(100).is_err());
    }

    #[test]
    fn test_cast_vote() {
        let mut proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            Address::ZERO,
            "Test".to_string(),
            "Description".to_string(),
            100,
            U256::from(1_000_000u128),
        );
        proposal.start_voting(100).unwrap();

        let voter = Address::from_bytes([1u8; 20]);
        let power = U256::from(1000u128);

        // Cast vote
        assert!(proposal.cast_vote(voter, VoteSupport::For, power).is_ok());
        assert_eq!(proposal.for_votes, power);
        assert!(proposal.has_voted(&voter));

        // Can't vote twice
        assert!(proposal.cast_vote(voter, VoteSupport::Against, power).is_err());

        // Can't vote with zero power
        let voter2 = Address::from_bytes([2u8; 20]);
        assert!(proposal.cast_vote(voter2, VoteSupport::For, U256::ZERO).is_err());
    }

    #[test]
    fn test_end_voting_succeeds() {
        let mut proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            Address::ZERO,
            "Test".to_string(),
            "Description".to_string(),
            100,
            U256::from(10_000u128),
        );
        proposal.start_voting(100).unwrap();

        // Vote with enough to pass quorum and threshold
        // Need 4% quorum = 400 tokens
        // Need 50% approval
        proposal.cast_vote(Address::from_bytes([1u8; 20]), VoteSupport::For, U256::from(500u128)).unwrap();
        proposal.cast_vote(Address::from_bytes([2u8; 20]), VoteSupport::For, U256::from(300u128)).unwrap();
        proposal.cast_vote(Address::from_bytes([3u8; 20]), VoteSupport::Against, U256::from(100u128)).unwrap();

        let result = proposal.end_voting(proposal.end_block + 1).unwrap();
        assert_eq!(result, ProposalStatus::Succeeded);
    }

    #[test]
    fn test_end_voting_fails_quorum() {
        let mut proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            Address::ZERO,
            "Test".to_string(),
            "Description".to_string(),
            100,
            U256::from(10_000u128),
        );
        proposal.start_voting(100).unwrap();

        // Vote with only 1% (below 4% quorum)
        proposal.cast_vote(Address::from_bytes([1u8; 20]), VoteSupport::For, U256::from(50u128)).unwrap();

        let result = proposal.end_voting(proposal.end_block + 1).unwrap();
        assert_eq!(result, ProposalStatus::Defeated);
    }

    #[test]
    fn test_execute_proposal() {
        let mut proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            Address::ZERO,
            "Test".to_string(),
            "Description".to_string(),
            100,
            U256::from(10_000u128),
        );
        
        // Can't execute before voting
        assert!(proposal.execute(200).is_err());

        proposal.start_voting(100).unwrap();
        proposal.cast_vote(Address::from_bytes([1u8; 20]), VoteSupport::For, U256::from(500u128)).unwrap();
        proposal.end_voting(proposal.end_block + 1).unwrap();

        // Can execute succeeded proposal
        assert!(proposal.execute(proposal.end_block + 10).is_ok());
        assert_eq!(proposal.status, ProposalStatus::Executed);
        assert!(proposal.executed_at.is_some());

        // Can't execute again
        assert!(proposal.execute(proposal.end_block + 20).is_err());
    }

    #[test]
    fn test_cancel_proposal() {
        let mut proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            Address::from_bytes([1u8; 20]),
            "Test".to_string(),
            "Description".to_string(),
            100,
            U256::from(10_000u128),
        );

        // Only proposer can cancel
        assert!(proposal.cancel(Address::ZERO, 50).is_err());

        // Proposer can cancel
        assert!(proposal.cancel(Address::from_bytes([1u8; 20]), 50).is_ok());
        assert_eq!(proposal.status, ProposalStatus::Cancelled);
    }

    #[test]
    fn test_proposal_registry() {
        let mut registry = ProposalRegistry::new();

        let id = registry.create_proposal(
            ProposalType::TreasurySpending,
            Address::ZERO,
            "Spend Funds".to_string(),
            "Description".to_string(),
            100,
            U256::from(1_000_000u128),
        );

        assert_eq!(id, 1);
        assert!(registry.get(id).is_some());
        assert_eq!(registry.get(id).unwrap().proposal_type, ProposalType::TreasurySpending);

        // Create another
        let id2 = registry.create_proposal(
            ProposalType::ParameterChange,
            Address::ZERO,
            "Change Param".to_string(),
            "Description".to_string(),
            100,
            U256::from(1_000_000u128),
        );

        assert_eq!(id2, 2);
    }

    #[test]
    fn test_proposal_type_config() {
        let param = ProposalType::ParameterChange;
        let emergency = ProposalType::Emergency;
        let upgrade = ProposalType::ContractUpgrade;

        // Emergency has shorter voting period
        assert!(emergency.default_voting_period() < param.default_voting_period());

        // Upgrade has higher quorum
        assert!(upgrade.quorum_bps() > param.quorum_bps());

        // Emergency has higher threshold
        assert!(emergency.threshold_bps() > param.threshold_bps());
    }
}
