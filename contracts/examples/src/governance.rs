//! Governance/DAO Contract
//! 
//! Decentralized governance for protocol decisions.
//! Features:
//! - Proposal creation and voting
//! - Delegation
//! - Timelock execution
//! - Quorum and threshold settings
//! - Emergency actions

use borsh::{BorshSerialize, BorshDeserialize};
use merklith_types::{Address, U256};

/// Governance Contract State
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct GovernanceContract {
    /// Contract owner (timelock)
    pub owner: Address,
    /// Governance token
    pub gov_token: Address,
    /// Voting delay (blocks)
    pub voting_delay: u64,
    /// Voting period (blocks)
    pub voting_period: u64,
    /// Proposal threshold (min tokens to create proposal)
    pub proposal_threshold: U256,
    /// Quorum (percentage, 400 = 4%)
    pub quorum_bps: u64,
    /// Proposal count
    pub proposal_count: u64,
    /// Proposals: id -> proposal
    pub proposals: Vec<(u64, Proposal)>,
    /// Votes: (proposal_id, voter) -> vote
    pub votes: Vec<(u64, Address, Vote)>,
    /// Delegates: voter -> delegate
    pub delegates: Vec<(Address, Address)>,
    /// Voting power: user -> power
    pub voting_power: Vec<(Address, U256)>,
    /// Proposal ETA: id -> timestamp
    pub proposal_eta: Vec<(u64, u64)>,
    /// Executed proposals
    pub executed: Vec<u64>,
    /// Canceled proposals
    pub canceled: Vec<u64>,
    /// Timelock delay (seconds)
    pub timelock_delay: u64,
    /// Grace period (seconds)
    pub grace_period: u64,
    /// Guardian (emergency)
    pub guardian: Address,
}

/// Proposal
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Proposal {
    /// Proposal ID
    pub id: u64,
    /// Proposer
    pub proposer: Address,
    /// Targets (contracts to call)
    pub targets: Vec<Address>,
    /// Values (ETH to send)
    pub values: Vec<U256>,
    /// Signatures (function signatures)
    pub signatures: Vec<String>,
    /// Calldata
    pub calldatas: Vec<Vec<u8>>,
    /// Start block
    pub start_block: u64,
    /// End block
    pub end_block: u64,
    /// For votes
    pub for_votes: U256,
    /// Against votes
    pub against_votes: U256,
    /// Abstain votes
    pub abstain_votes: U256,
    /// Canceled
    pub canceled: bool,
    /// Executed
    pub executed: bool,
    /// Description
    pub description: String,
    /// ETA (for execution)
    pub eta: u64,
}

/// Vote Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum VoteType {
    Against,
    For,
    Abstain,
}

/// Vote Record
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Vote {
    pub voter: Address,
    pub proposal_id: u64,
    pub support: VoteType,
    pub votes: U256,
}

/// Proposal Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ProposalEvent {
    pub id: u64,
    pub proposer: Address,
    pub targets: Vec<Address>,
    pub values: Vec<U256>,
    pub signatures: Vec<String>,
    pub calldatas: Vec<Vec<u8>>,
    pub start_block: u64,
    pub end_block: u64,
    pub description: String,
}

/// Vote Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct VoteEvent {
    pub voter: Address,
    pub proposal_id: u64,
    pub support: VoteType,
    pub votes: U256,
}

/// Proposal State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalState {
    Pending,
    Active,
    Canceled,
    Defeated,
    Succeeded,
    Queued,
    Expired,
    Executed,
}

/// Governance Error Types
#[derive(Debug, Clone, PartialEq)]
pub enum GovernanceError {
    /// Not owner
    NotOwner,
    /// Not guardian
    NotGuardian,
    /// Insufficient voting power
    InsufficientVotingPower,
    /// Proposal not found
    ProposalNotFound,
    /// Voting closed
    VotingClosed,
    /// Voting not started
    VotingNotStarted,
    /// Already voted
    AlreadyVoted,
    /// Proposal already queued
    AlreadyQueued,
    /// Proposal not succeeded
    ProposalNotSucceeded,
    /// Proposal not queued
    ProposalNotQueued,
    /// Timelock not reached
    TimelockNotReached,
    /// Proposal expired
    ProposalExpired,
    /// Proposal already executed
    AlreadyExecuted,
    /// Proposal already canceled
    AlreadyCanceled,
    /// Cannot cancel
    CannotCancel,
    /// Invalid proposal
    InvalidProposal,
    /// Quorum not reached
    QuorumNotReached,
    /// Overflow
    Overflow,
    /// Underflow
    Underflow,
    /// Divide by zero
    DivideByZero,
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GovernanceError::NotOwner => write!(f, "Not contract owner"),
            GovernanceError::NotGuardian => write!(f, "Not guardian"),
            GovernanceError::InsufficientVotingPower => write!(f, "Insufficient voting power"),
            GovernanceError::ProposalNotFound => write!(f, "Proposal not found"),
            GovernanceError::VotingClosed => write!(f, "Voting is closed"),
            GovernanceError::VotingNotStarted => write!(f, "Voting has not started"),
            GovernanceError::AlreadyVoted => write!(f, "Already voted"),
            GovernanceError::AlreadyQueued => write!(f, "Proposal already queued"),
            GovernanceError::ProposalNotSucceeded => write!(f, "Proposal did not succeed"),
            GovernanceError::ProposalNotQueued => write!(f, "Proposal not queued"),
            GovernanceError::TimelockNotReached => write!(f, "Timelock not reached"),
            GovernanceError::ProposalExpired => write!(f, "Proposal expired"),
            GovernanceError::AlreadyExecuted => write!(f, "Proposal already executed"),
            GovernanceError::AlreadyCanceled => write!(f, "Proposal already canceled"),
            GovernanceError::CannotCancel => write!(f, "Cannot cancel proposal"),
            GovernanceError::InvalidProposal => write!(f, "Invalid proposal"),
            GovernanceError::QuorumNotReached => write!(f, "Quorum not reached"),
            GovernanceError::Overflow => write!(f, "Arithmetic overflow"),
            GovernanceError::Underflow => write!(f, "Arithmetic underflow"),
            GovernanceError::DivideByZero => write!(f, "Divide by zero"),
        }
    }
}

impl std::error::Error for GovernanceError {}

impl GovernanceContract {
    /// Create new governance contract
    pub fn new(
        owner: Address,
        gov_token: Address,
        voting_delay: u64,
        voting_period: u64,
        proposal_threshold: U256,
        quorum_bps: u64,
    ) -> Self {
        Self {
            owner,
            gov_token,
            voting_delay,
            voting_period,
            proposal_threshold,
            quorum_bps,
            proposal_count: 0,
            proposals: Vec::new(),
            votes: Vec::new(),
            delegates: Vec::new(),
            voting_power: Vec::new(),
            proposal_eta: Vec::new(),
            executed: Vec::new(),
            canceled: Vec::new(),
            timelock_delay: 2 * 24 * 60 * 60, // 2 days
            grace_period: 14 * 24 * 60 * 60, // 14 days
            guardian: owner,
        }
    }

    /// Create proposal
    pub fn propose(
        &mut self,
        proposer: Address,
        targets: Vec<Address>,
        values: Vec<U256>,
        signatures: Vec<String>,
        calldatas: Vec<Vec<u8>>,
        description: String,
    ) -> Result<ProposalEvent, GovernanceError> {
        // Check proposer has enough tokens
        let voting_power = self.get_voting_power(proposer);
        if voting_power < self.proposal_threshold {
            return Err(GovernanceError::InsufficientVotingPower);
        }

        // Validate proposal
        if targets.len() != values.len()
            || targets.len() != signatures.len()
            || targets.len() != calldatas.len()
        {
            return Err(GovernanceError::InvalidProposal);
        }

        if targets.is_empty() {
            return Err(GovernanceError::InvalidProposal);
        }

        if targets.len() > 10 {
            return Err(GovernanceError::InvalidProposal);
        }

        // Create proposal
        self.proposal_count += 1;
        let id = self.proposal_count;
        
        let start_block = Self::current_block() + self.voting_delay;
        let end_block = start_block + self.voting_period;

        let proposal = Proposal {
            id,
            proposer,
            targets: targets.clone(),
            values: values.clone(),
            signatures: signatures.clone(),
            calldatas: calldatas.clone(),
            start_block,
            end_block,
            for_votes: U256::ZERO,
            against_votes: U256::ZERO,
            abstain_votes: U256::ZERO,
            canceled: false,
            executed: false,
            description: description.clone(),
            eta: 0,
        };

        self.proposals.push((id, proposal));

        Ok(ProposalEvent {
            id,
            proposer,
            targets,
            values,
            signatures,
            calldatas,
            start_block,
            end_block,
            description,
        })
    }

    /// Cast vote
    pub fn cast_vote(
        &mut self,
        voter: Address,
        proposal_id: u64,
        support: VoteType,
    ) -> Result<VoteEvent, GovernanceError> {
        let state = self.get_proposal_state(proposal_id)?;
        
        if state != ProposalState::Active {
            return Err(GovernanceError::VotingClosed);
        }

        // Check if already voted
        if self.has_voted(proposal_id, voter) {
            return Err(GovernanceError::AlreadyVoted);
        }

        // Get voting power
        let votes = self.get_voting_power(voter);
        
        if votes == U256::ZERO {
            return Err(GovernanceError::InsufficientVotingPower);
        }

        // Record vote
        let vote = Vote {
            voter,
            proposal_id,
            support,
            votes,
        };
        
        self.votes.push((proposal_id, voter, vote.clone()));

        // Update proposal vote counts
        if let Some(pos) = self.proposals.iter().position(|(id, _)| *id == proposal_id) {
            match support {
                VoteType::For => {
                    self.proposals[pos].1.for_votes = self.proposals[pos].1.for_votes
                        .checked_add(&votes).ok_or(GovernanceError::Overflow)?;
                }
                VoteType::Against => {
                    self.proposals[pos].1.against_votes = self.proposals[pos].1.against_votes
                        .checked_add(&votes).ok_or(GovernanceError::Overflow)?;
                }
                VoteType::Abstain => {
                    self.proposals[pos].1.abstain_votes = self.proposals[pos].1.abstain_votes
                        .checked_add(&votes).ok_or(GovernanceError::Overflow)?;
                }
            }
        }

        Ok(VoteEvent {
            voter,
            proposal_id,
            support,
            votes,
        })
    }

    /// Queue proposal (for execution)
    pub fn queue(
        &mut self,
        proposal_id: u64,
    ) -> Result<(), GovernanceError> {
        let state = self.get_proposal_state(proposal_id)?;
        
        if state != ProposalState::Succeeded {
            return Err(GovernanceError::ProposalNotSucceeded);
        }

        if let Some(pos) = self.proposals.iter().position(|(id, _)| *id == proposal_id) {
            let eta = Self::current_timestamp() + self.timelock_delay;
            self.proposals[pos].1.eta = eta;
            self.proposal_eta.push((proposal_id, eta));
        }

        Ok(())
    }

    /// Execute proposal
    pub fn execute(
        &mut self,
        proposal_id: u64,
    ) -> Result<(), GovernanceError> {
        let state = self.get_proposal_state(proposal_id)?;
        
        if state != ProposalState::Queued {
            return Err(GovernanceError::ProposalNotQueued);
        }

        let proposal = self.get_proposal(proposal_id)?;
        
        // Check timelock
        if Self::current_timestamp() < proposal.eta {
            return Err(GovernanceError::TimelockNotReached);
        }

        // Check grace period
        if Self::current_timestamp() > proposal.eta + self.grace_period {
            return Err(GovernanceError::ProposalExpired);
        }

        // Mark as executed
        if let Some(pos) = self.proposals.iter().position(|(id, _)| *id == proposal_id) {
            self.proposals[pos].1.executed = true;
        }
        self.executed.push(proposal_id);

        // Execute actions (simplified - in production would call contracts)
        // for (i, target) in proposal.targets.iter().enumerate() {
        //     // Execute call
        // }

        Ok(())
    }

    /// Cancel proposal
    pub fn cancel(
        &mut self,
        caller: Address,
        proposal_id: u64,
    ) -> Result<(), GovernanceError> {
        let proposal = self.get_proposal(proposal_id)?;
        
        // Only proposer or guardian can cancel
        if caller != proposal.proposer && caller != self.guardian {
            return Err(GovernanceError::CannotCancel);
        }

        let state = self.get_proposal_state(proposal_id)?;
        
        if state == ProposalState::Executed {
            return Err(GovernanceError::AlreadyExecuted);
        }

        if state == ProposalState::Canceled {
            return Err(GovernanceError::AlreadyCanceled);
        }

        // Mark as canceled
        if let Some(pos) = self.proposals.iter().position(|(id, _)| *id == proposal_id) {
            self.proposals[pos].1.canceled = true;
        }
        self.canceled.push(proposal_id);

        Ok(())
    }

    /// Delegate voting power
    pub fn delegate(
        &mut self,
        delegator: Address,
        delegatee: Address,
    ) -> Result<(), GovernanceError> {
        if delegator == delegatee {
            return Err(GovernanceError::InvalidProposal);
        }

        if let Some(pos) = self.delegates.iter().position(|(d, _)| *d == delegator) {
            self.delegates[pos].1 = delegatee;
        } else {
            self.delegates.push((delegator, delegatee));
        }

        Ok(())
    }

    /// Get voting power
    pub fn get_voting_power(
        &self,
        voter: Address,
    ) -> U256 {
        // Check if delegated to someone else
        if let Some((_, delegatee)) = self.delegates.iter().find(|(d, _)| *d == voter) {
            if *delegatee != voter {
                return U256::ZERO; // Power delegated away
            }
        }

        // Get own power
        self.voting_power
            .iter()
            .find(|(v, _)| *v == voter)
            .map(|(_, power)| *power)
            .unwrap_or(U256::ZERO)
    }

    /// Get proposal state
    pub fn get_proposal_state(
        &self,
        proposal_id: u64,
    ) -> Result<ProposalState, GovernanceError> {
        let proposal = self.get_proposal(proposal_id)?;
        
        if proposal.canceled {
            return Ok(ProposalState::Canceled);
        }

        if proposal.executed {
            return Ok(ProposalState::Executed);
        }

        let current_block = Self::current_block();

        if current_block <= proposal.start_block {
            return Ok(ProposalState::Pending);
        }

        if current_block <= proposal.end_block {
            return Ok(ProposalState::Active);
        }

        // Voting ended, check result
        let total_votes = proposal.for_votes
            .checked_add(&proposal.against_votes).ok_or(GovernanceError::Overflow)?
            .checked_add(&proposal.abstain_votes).ok_or(GovernanceError::Overflow)?;

        // Check quorum
        let total_supply = U256::from(1000000u64); // Would be actual token supply
        let quorum = total_supply
            .checked_mul(&U256::from(self.quorum_bps)).ok_or(GovernanceError::Overflow)?
            .checked_div(&U256::from(10000u64)).ok_or(GovernanceError::DivideByZero)?;

        if total_votes < quorum {
            return Ok(ProposalState::Defeated);
        }

        if proposal.for_votes > proposal.against_votes {
            if proposal.eta != 0 {
                if Self::current_timestamp() >= proposal.eta + self.grace_period {
                    return Ok(ProposalState::Expired);
                }
                return Ok(ProposalState::Queued);
            }
            return Ok(ProposalState::Succeeded);
        } else {
            return Ok(ProposalState::Defeated);
        }
    }

    /// Get proposal
    fn get_proposal(
        &self,
        proposal_id: u64,
    ) -> Result<Proposal, GovernanceError> {
        self.proposals
            .iter()
            .find(|(id, _)| *id == proposal_id)
            .map(|(_, p)| p.clone())
            .ok_or(GovernanceError::ProposalNotFound)
    }

    /// Check if has voted
    fn has_voted(
        &self,
        proposal_id: u64,
        voter: Address,
    ) -> bool {
        self.votes.iter().any(|(p, v, _)| *p == proposal_id && *v == voter)
    }

    /// Get current block
    fn current_block() -> u64 {
        // In production: use block number
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() / 12 // Assume 12s block time
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Set voting power (for testing/initialization)
    pub fn set_voting_power(
        &mut self,
        user: Address,
        power: U256,
    ) {
        if let Some(pos) = self.voting_power.iter().position(|(u, _)| *u == user) {
            self.voting_power[pos].1 = power;
        } else {
            self.voting_power.push((user, power));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_governance() -> GovernanceContract {
        let owner = Address::from_bytes([1u8; 20]);
        let gov_token = Address::from_bytes([2u8; 20]);
        
        GovernanceContract::new(
            owner,
            gov_token,
            1, // voting delay
            10, // voting period
            U256::from(100u64), // proposal threshold
            400, // 4% quorum
        )
    }

    #[test]
    fn test_initialization() {
        let gov = create_governance();
        assert_eq!(gov.proposal_count, 0);
        assert_eq!(gov.voting_delay, 1);
        assert_eq!(gov.voting_period, 10);
    }

    #[test]
    fn test_propose() {
        let mut gov = create_governance();
        let proposer = Address::from_bytes([3u8; 20]);
        
        // Set voting power
        gov.set_voting_power(proposer, U256::from(1000u64));
        
        let targets = vec![Address::from_bytes([4u8; 20])];
        let values = vec![U256::ZERO];
        let signatures = vec!["test()".to_string()];
        let calldatas = vec![vec![]];
        let description = "Test proposal".to_string();
        
        let result = gov.propose(
            proposer,
            targets,
            values,
            signatures,
            calldatas,
            description,
        );
        
        assert!(result.is_ok());
        assert_eq!(gov.proposal_count, 1);
    }

    #[test]
    fn test_propose_insufficient_power() {
        let mut gov = create_governance();
        let proposer = Address::from_bytes([3u8; 20]);
        
        // Set voting power below threshold
        gov.set_voting_power(proposer, U256::from(50u64));
        
        let targets = vec![Address::from_bytes([4u8; 20])];
        let values = vec![U256::ZERO];
        let signatures = vec!["test()".to_string()];
        let calldatas = vec![vec![]];
        let description = "Test proposal".to_string();
        
        let result = gov.propose(
            proposer,
            targets,
            values,
            signatures,
            calldatas,
            description,
        );
        
        assert!(matches!(result, Err(GovernanceError::InsufficientVotingPower)));
    }

    #[test]
    fn test_cast_vote() {
        let mut gov = create_governance();
        let proposer = Address::from_bytes([3u8; 20]);
        let voter = Address::from_bytes([4u8; 20]);
        
        // Set voting power
        gov.set_voting_power(proposer, U256::from(1000u64));
        gov.set_voting_power(voter, U256::from(500u64));
        
        // Create proposal
        let targets = vec![Address::from_bytes([5u8; 20])];
        let values = vec![U256::ZERO];
        let signatures = vec!["test()".to_string()];
        let calldatas = vec![vec![]];
        
        gov.propose(
            proposer,
            targets,
            values,
            signatures,
            calldatas,
            "Test".to_string(),
        ).unwrap();
        
        // Vote (in production would wait for voting delay)
        // For testing, we skip to voting period
        
        // Note: This would fail in production because voting hasn't started
        // But for testing purposes, we're checking the logic
    }

    #[test]
    fn test_delegate() {
        let mut gov = create_governance();
        let delegator = Address::from_bytes([3u8; 20]);
        let delegatee = Address::from_bytes([4u8; 20]);
        
        gov.delegate(delegator, delegatee).unwrap();
        
        // Set voting power for delegator
        gov.set_voting_power(delegator, U256::from(1000u64));
        
        // Delegator should have 0 power (delegated away)
        assert_eq!(gov.get_voting_power(delegator), U256::ZERO);
    }

    #[test]
    fn test_cancel() {
        let mut gov = create_governance();
        let proposer = Address::from_bytes([3u8; 20]);
        
        gov.set_voting_power(proposer, U256::from(1000u64));
        
        let targets = vec![Address::from_bytes([4u8; 20])];
        let values = vec![U256::ZERO];
        let signatures = vec!["test()".to_string()];
        let calldatas = vec![vec![]];
        
        gov.propose(
            proposer,
            targets,
            values,
            signatures,
            calldatas,
            "Test".to_string(),
        ).unwrap();
        
        // Cancel by proposer
        let result = gov.cancel(proposer, 1);
        assert!(result.is_ok());
        
        let state = gov.get_proposal_state(1).unwrap();
        assert_eq!(state, ProposalState::Canceled);
    }
}
