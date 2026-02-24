//! Governance Contract
//!
//! On-chain governance for protocol upgrades and parameter changes.

use merklith_types::{Address, U256};
use std::collections::HashMap;

/// Governance state.
#[derive(Debug)]
pub struct GovernanceContract {
    /// Proposals by ID
    proposals: HashMap<u64, Proposal>,
    /// Next proposal ID
    next_proposal_id: u64,
    /// Voting power (address -> weight)
    voting_power: HashMap<Address, U256>,
    /// Delegations (delegator -> delegate)
    delegations: HashMap<Address, Address>,
    /// Quorum threshold (basis points)
    quorum_bps: u16,
    /// Approval threshold (basis points)
    threshold_bps: u16,
    /// Voting period (blocks)
    voting_period: u64,
}

/// Proposal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalState {
    Pending,
    Active,
    Succeeded,
    Defeated,
    Executed,
    Cancelled,
}

/// Proposal data.
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Proposal ID
    pub id: u64,
    /// Proposer
    pub proposer: Address,
    /// Description
    pub description: String,
    /// Target contract
    pub target: Address,
    /// Call data
    pub call_data: Vec<u8>,
    /// For votes
    pub for_votes: U256,
    /// Against votes
    pub against_votes: U256,
    /// Start block
    pub start_block: u64,
    /// End block
    pub end_block: u64,
    /// Current state
    pub state: ProposalState,
    /// Has voted
    pub has_voted: HashMap<Address, bool>,
}

/// Vote type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteType {
    For,
    Against,
    Abstain,
}

impl GovernanceContract {
    /// Create new governance contract.
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            next_proposal_id: 1,
            voting_power: HashMap::new(),
            delegations: HashMap::new(),
            quorum_bps: 400,    // 4%
            threshold_bps: 5000, // 50%
            voting_period: 100_800, // ~1 week at 6s blocks
        }
    }

    /// Create a proposal.
    pub fn propose(
        &mut self,
        proposer: Address,
        description: String,
        target: Address,
        call_data: Vec<u8>,
        current_block: u64,
    ) -> Result<u64, String> {
        // Check proposer has voting power
        let power = self.get_voting_power(&proposer);
        if power == U256::ZERO {
            return Err("No voting power".to_string());
        }

        let id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let proposal = Proposal {
            id,
            proposer,
            description,
            target,
            call_data,
            for_votes: U256::ZERO,
            against_votes: U256::ZERO,
            start_block: current_block + 1,
            end_block: current_block + self.voting_period,
            state: ProposalState::Pending,
            has_voted: HashMap::new(),
        };

        self.proposals.insert(id, proposal);
        Ok(id)
    }

    /// Cast a vote.
    pub fn cast_vote(
        &mut self,
        voter: Address,
        proposal_id: u64,
        vote: VoteType,
    ) -> Result<(), String> {
        // First, check proposal state and if already voted
        let (is_active, has_voted) = {
            let proposal = self.proposals
                .get(&proposal_id)
                .ok_or("Proposal not found")?;
            (proposal.state == ProposalState::Active, proposal.has_voted.contains_key(&voter))
        };

        if !is_active {
            return Err("Proposal not active".to_string());
        }

        if has_voted {
            return Err("Already voted".to_string());
        }

        let power = self.get_voting_power(&voter);
        
        let proposal = self.proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        match vote {
            VoteType::For => proposal.for_votes += power,
            VoteType::Against => proposal.against_votes += power,
            VoteType::Abstain => {}
        }

        proposal.has_voted.insert(voter, true);
        Ok(())
    }

    /// Start voting (transition from Pending to Active).
    pub fn start_voting(
        &mut self,
        proposal_id: u64,
        current_block: u64,
    ) -> Result<(), String> {
        let proposal = self.proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.state != ProposalState::Pending {
            return Err("Invalid state".to_string());
        }

        if current_block < proposal.start_block {
            return Err("Too early".to_string());
        }

        proposal.state = ProposalState::Active;
        Ok(())
    }

    /// Queue proposal for execution.
    pub fn queue(
        &mut self,
        proposal_id: u64,
        current_block: u64,
    ) -> Result<(), String> {
        let proposal = self.proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.state != ProposalState::Active {
            return Err("Not active".to_string());
        }

        if current_block <= proposal.end_block {
            return Err("Voting not ended".to_string());
        }

        // Check if passed
        let total = proposal.for_votes + proposal.against_votes;
        let quorum = total * U256::from(self.quorum_bps) / U256::from(10000);
        
        if proposal.for_votes <= proposal.against_votes {
            proposal.state = ProposalState::Defeated;
            return Err("Proposal defeated".to_string());
        }

        // Check threshold
        let for_pct = proposal.for_votes * U256::from(10000) / total;
        if for_pct < U256::from(self.threshold_bps) {
            proposal.state = ProposalState::Defeated;
            return Err("Threshold not met".to_string());
        }

        proposal.state = ProposalState::Succeeded;
        Ok(())
    }

    /// Execute a succeeded proposal.
    pub fn execute(
        &mut self,
        proposal_id: u64,
    ) -> Result<(), String> {
        let proposal = self.proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.state != ProposalState::Succeeded {
            return Err("Not succeeded".to_string());
        }

        proposal.state = ProposalState::Executed;
        
        // In real implementation, would execute call_data on target
        Ok(())
    }

    /// Delegate voting power.
    pub fn delegate(
        &mut self,
        delegator: Address,
        delegatee: Address,
    ) {
        self.delegations.insert(delegator, delegatee);
    }

    /// Set voting power.
    pub fn set_voting_power(
        &mut self,
        voter: Address,
        power: U256,
    ) {
        self.voting_power.insert(voter, power);
    }

    /// Get voting power (with delegation).
    pub fn get_voting_power(&self,
        voter: &Address,
    ) -> U256 {
        // Check if delegated to someone else
        if let Some(delegatee) = self.delegations.get(voter) {
            if delegatee != voter {
                return U256::ZERO; // Delegated away
            }
        }

        // Get own power plus delegated power
        let mut power = self.voting_power.get(voter).copied()
            .unwrap_or(U256::ZERO);

        // Add power from delegators
        for (delegator, del) in &self.delegations {
            if del == voter && delegator != voter {
                power += self.voting_power.get(delegator).copied()
                    .unwrap_or(U256::ZERO);
            }
        }

        power
    }

    /// Get proposal.
    pub fn get_proposal(&self,
        id: u64,
    ) -> Option<&Proposal> {
        self.proposals.get(&id)
    }

    /// Get proposal count.
    pub fn proposal_count(&self) -> u64 {
        self.next_proposal_id - 1
    }
}

impl Default for GovernanceContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let mut gov = GovernanceContract::new();
        let proposer = Address::ZERO;
        
        gov.set_voting_power(proposer, U256::from(1000u64));
        
        let id = gov.propose(
            proposer,
            "Test proposal".to_string(),
            Address::from_bytes([1u8; 20]),
            vec![1, 2, 3],
            0,
        ).unwrap();
        
        assert_eq!(id, 1);
        assert_eq!(gov.proposal_count(), 1);
    }

    #[test]
    fn test_vote() {
        let mut gov = GovernanceContract::new();
        let proposer = Address::from_bytes([1u8; 20]);
        let voter = Address::from_bytes([2u8; 20]);
        
        gov.set_voting_power(proposer, U256::from(1000u64));
        gov.set_voting_power(voter, U256::from(500u64));
        
        let id = gov.propose(
            proposer,
            "Test".to_string(),
            Address::from_bytes([3u8; 20]),
            vec![],
            0,
        ).unwrap();
        
        // Start voting
        gov.start_voting(id, 1).unwrap();
        
        // Cast vote
        gov.cast_vote(voter, id, VoteType::For).unwrap();
        
        let proposal = gov.get_proposal(id).unwrap();
        assert_eq!(proposal.for_votes, U256::from(500u64));
    }

    #[test]
    fn test_delegation() {
        let mut gov = GovernanceContract::new();
        let alice = Address::from_bytes([1u8; 20]);
        let bob = Address::from_bytes([2u8; 20]);
        
        gov.set_voting_power(alice, U256::from(1000u64));
        gov.set_voting_power(bob, U256::from(500u64));
        
        // Alice delegates to Bob
        gov.delegate(alice, bob);
        
        assert_eq!(gov.get_voting_power(&alice), U256::ZERO);
        assert_eq!(gov.get_voting_power(&bob), U256::from(1500u64));
    }
}
