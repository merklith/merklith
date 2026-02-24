//! Slashing conditions for misbehaving validators.
//!
//! Implements detection and handling of:
//! - Double proposal (proposing two blocks at same slot)
//! - Double attestation (attesting to conflicting blocks)
//! - Surround voting (attesting to non-monotonic checkpoints)

use std::collections::{HashMap, HashSet};
use merklith_types::{Address, Hash};
use crate::error::ConsensusError;
use crate::validator::{Validator, ValidatorStatus};

/// Slashing configuration.
#[derive(Debug, Clone, Copy)]
pub struct SlashingConfig {
    /// Minimum stake to slash (percentage, 0-100)
    pub min_slash_percent: u8,
    /// Maximum stake to slash (percentage, 0-100)
    pub max_slash_percent: u8,
    /// Whistleblower reward (percentage of slashed amount)
    pub whistleblower_reward_percent: u8,
    /// Proposer reward (percentage of slashed amount)
    pub proposer_reward_percent: u8,
    /// Exit delay after slashing (epochs)
    pub exit_delay_epochs: u64,
    /// Withdrawal delay after slashing (epochs)
    pub withdrawal_delay_epochs: u64,
}

impl Default for SlashingConfig {
    fn default() -> Self {
        Self {
            min_slash_percent: 1,
            max_slash_percent: 100,
            whistleblower_reward_percent: 4,
            proposer_reward_percent: 1,
            exit_delay_epochs: 0,
            withdrawal_delay_epochs: 8192, // ~36 days at 6s blocks
        }
    }
}

/// Type of slashable offense.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashableOffense {
    /// Proposed two blocks at the same slot
    DoubleProposal,
    /// Attested to two different blocks at the same target
    DoubleAttestation,
    /// Attested to non-monotonic checkpoints (surround vote)
    SurroundVote,
    /// Invalid block proposal
    InvalidBlock,
    /// Invalid attestation
    InvalidAttestation,
}

impl SlashableOffense {
    /// Get the slash percentage for this offense.
    pub fn slash_percent(&self) -> u8 {
        match self {
            SlashableOffense::DoubleProposal => 100,
            SlashableOffense::DoubleAttestation => 100,
            SlashableOffense::SurroundVote => 100,
            SlashableOffense::InvalidBlock => 50,
            SlashableOffense::InvalidAttestation => 25,
        }
    }

    /// Get description of the offense.
    pub fn description(&self) -> &'static str {
        match self {
            SlashableOffense::DoubleProposal => "Double proposal at same slot",
            SlashableOffense::DoubleAttestation => "Double attestation for same target",
            SlashableOffense::SurroundVote => "Surround vote (non-monotonic checkpoints)",
            SlashableOffense::InvalidBlock => "Invalid block proposal",
            SlashableOffense::InvalidAttestation => "Invalid attestation",
        }
    }
}

/// Record of a block proposal for detecting double proposals.
#[derive(Debug, Clone)]
pub struct ProposalRecord {
    pub validator: Address,
    pub slot: u64,
    pub block_hash: Hash,
    pub epoch: u64,
}

/// Record of an attestation for detecting double attestations.
#[derive(Debug, Clone)]
pub struct AttestationRecord {
    pub validator: Address,
    pub slot: u64,
    pub target_epoch: u64,
    pub target_root: Hash,
    pub source_epoch: u64,
    pub source_root: Hash,
}

/// Slashing detector tracking validator behavior.
#[derive(Debug)]
pub struct SlashingDetector {
    /// Proposals by (validator, slot)
    proposals: HashMap<(Address, u64), ProposalRecord>,
    /// Attestations by (validator, target_epoch)
    attestations: HashMap<(Address, u64), AttestationRecord>,
    /// Validators already slashed (to prevent double slashing)
    slashed_validators: HashSet<Address>,
    /// Slashing configuration
    config: SlashingConfig,
}

impl SlashingDetector {
    /// Create a new slashing detector.
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            attestations: HashMap::new(),
            slashed_validators: HashSet::new(),
            config: SlashingConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: SlashingConfig) -> Self {
        Self {
            proposals: HashMap::new(),
            attestations: HashMap::new(),
            slashed_validators: HashSet::new(),
            config,
        }
    }

    /// Check if validator has been slashed.
    pub fn is_slashed(&self, validator: &Address) -> bool {
        self.slashed_validators.contains(validator)
    }

    /// Record a proposal and check for double proposal.
    pub fn check_proposal(
        &mut self,
        validator: Address,
        slot: u64,
        block_hash: Hash,
        epoch: u64,
    ) -> Result<(), (SlashableOffense, ProposalRecord)> {
        // Skip if already slashed
        if self.slashed_validators.contains(&validator) {
            return Ok(());
        }

        let key = (validator, slot);
        let new_record = ProposalRecord {
            validator,
            slot,
            block_hash,
            epoch,
        };

        // Check for existing proposal at same slot
        if let Some(existing) = self.proposals.get(&key) {
            if existing.block_hash != block_hash {
                // Double proposal detected!
                return Err((SlashableOffense::DoubleProposal, existing.clone()));
            }
        }

        // Record this proposal
        self.proposals.insert(key, new_record);
        Ok(())
    }

    /// Record an attestation and check for slashable offenses.
    pub fn check_attestation(
        &mut self,
        validator: Address,
        slot: u64,
        source_epoch: u64,
        source_root: Hash,
        target_epoch: u64,
        target_root: Hash,
    ) -> Result<(), (SlashableOffense, AttestationRecord)> {
        // Skip if already slashed
        if self.slashed_validators.contains(&validator) {
            return Ok(());
        }

        let key = (validator, target_epoch);
        let new_record = AttestationRecord {
            validator,
            slot,
            target_epoch,
            target_root,
            source_epoch,
            source_root,
        };

        // Check for existing attestation at same target
        if let Some(existing) = self.attestations.get(&key) {
            if existing.target_root != target_root {
                // Double attestation!
                return Err((SlashableOffense::DoubleAttestation, existing.clone()));
            }
        }

        // Check for surround votes
        for (_, existing) in self.attestations.iter().filter(|(k, _)| k.0 == validator) {
            // Check if new vote surrounds existing
            if self.is_surround_vote(existing, &new_record) {
                return Err((SlashableOffense::SurroundVote, existing.clone()));
            }
            // Check if existing surrounds new
            if self.is_surround_vote(&new_record, existing) {
                return Err((SlashableOffense::SurroundVote, existing.clone()));
            }
        }

        // Record this attestation
        self.attestations.insert(key, new_record);
        Ok(())
    }

    /// Check if vote_a surrounds vote_b.
    /// 
    /// A surrounds B if:
    /// - A.source < B.source && A.target > B.target
    fn is_surround_vote(
        &self,
        vote_a: &AttestationRecord,
        vote_b: &AttestationRecord,
    ) -> bool {
        vote_a.source_epoch < vote_b.source_epoch && 
        vote_a.target_epoch > vote_b.target_epoch
    }

    /// Mark validator as slashed.
    pub fn slash_validator(&mut self, validator: Address) {
        self.slashed_validators.insert(validator);
    }

    /// Clean up old records (call periodically).
    pub fn prune_old_records(
        &mut self,
        current_epoch: u64,
        keep_epochs: u64,
    ) {
        let cutoff = current_epoch.saturating_sub(keep_epochs);

        self.proposals.retain(|_, record| record.epoch >= cutoff);
        self.attestations.retain(|_, record| record.target_epoch >= cutoff);
    }

    /// Get number of tracked proposals.
    pub fn proposal_count(&self) -> usize {
        self.proposals.len()
    }

    /// Get number of tracked attestations.
    pub fn attestation_count(&self) -> usize {
        self.attestations.len()
    }
}

impl Default for SlashingDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Slash a validator and distribute rewards.
pub fn slash_validator(
    validator: &mut Validator,
    offense: SlashableOffense,
    config: &SlashingConfig,
    epoch: u64,
) -> Result<SlashingResult, ConsensusError> {
    if !validator.status.can_be_slashed() {
        return Err(ConsensusError::SlashingCondition(
            format!("Validator {} cannot be slashed in status {:?}", 
                validator.address, validator.status)
        ));
    }

    let slash_percent = offense.slash_percent();
    let slash_amount = validator.stake
        .checked_mul(slash_percent as u64)
        .and_then(|v| v.checked_div(100))
        .ok_or_else(|| ConsensusError::SlashingCondition("Slash calculation overflow".to_string()))?;

    // Calculate rewards with overflow protection
    let whistleblower_reward = slash_amount
        .checked_mul(config.whistleblower_reward_percent as u64)
        .and_then(|v| v.checked_div(100))
        .unwrap_or(0);
    let proposer_reward = slash_amount
        .checked_mul(config.proposer_reward_percent as u64)
        .and_then(|v| v.checked_div(100))
        .unwrap_or(0);
    let burn_amount = slash_amount.saturating_sub(whistleblower_reward).saturating_sub(proposer_reward);

    // Apply slashing
    validator.stake -= slash_amount;
    validator.slash(epoch);

    Ok(SlashingResult {
        slashed_validator: validator.address,
        offense,
        slash_amount,
        whistleblower_reward,
        proposer_reward,
        burn_amount,
        epoch,
    })
}

/// Result of a slashing operation.
#[derive(Debug, Clone)]
pub struct SlashingResult {
    /// Validator that was slashed
    pub slashed_validator: Address,
    /// Offense committed
    pub offense: SlashableOffense,
    /// Total amount slashed
    pub slash_amount: U256,
    /// Reward for whistleblower
    pub whistleblower_reward: U256,
    /// Reward for block proposer
    pub proposer_reward: U256,
    /// Amount burned
    pub burn_amount: U256,
    /// Epoch of slashing
    pub epoch: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::U256;

    fn test_address(n: u8) -> Address {
        let mut addr = [0u8; 20];
        addr[19] = n;
        Address::from_bytes(addr)
    }

    fn test_hash(n: u8) -> Hash {
        let mut hash = [0u8; 32];
        hash[31] = n;
        Hash::from_bytes(hash)
    }

    #[test]
    fn test_double_proposal_detection() {
        let mut detector = SlashingDetector::new();
        let validator = test_address(1);

        // First proposal at slot 10
        let result1 = detector.check_proposal(
            validator,
            10,
            test_hash(1),
            1,
        );
        assert!(result1.is_ok());

        // Second proposal at same slot (different block)
        let result2 = detector.check_proposal(
            validator,
            10,
            test_hash(2),
            1,
        );
        assert!(result2.is_err());
        
        let (offense, _) = result2.unwrap_err();
        assert_eq!(offense, SlashableOffense::DoubleProposal);
    }

    #[test]
    fn test_same_proposal_not_double() {
        let mut detector = SlashingDetector::new();
        let validator = test_address(1);

        // First proposal at slot 10
        detector.check_proposal(validator, 10, test_hash(1), 1).unwrap();

        // Same proposal again (should be fine, just duplicate)
        let result = detector.check_proposal(validator, 10, test_hash(1), 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_double_attestation_detection() {
        let mut detector = SlashingDetector::new();
        let validator = test_address(1);

        // First attestation for epoch 10
        detector.check_attestation(
            validator,
            100,
            5,
            test_hash(1),
            10,
            test_hash(10),
        ).unwrap();

        // Second attestation for same epoch (different target)
        let result = detector.check_attestation(
            validator,
            101,
            5,
            test_hash(1),
            10,
            test_hash(11),
        );
        assert!(result.is_err());
        
        let (offense, _) = result.unwrap_err();
        assert_eq!(offense, SlashableOffense::DoubleAttestation);
    }

    #[test]
    fn test_surround_vote_detection() {
        let mut detector = SlashingDetector::new();
        let validator = test_address(1);

        // First vote: source=5, target=10
        detector.check_attestation(
            validator,
            100,
            5,
            test_hash(5),
            10,
            test_hash(10),
        ).unwrap();

        // Surround vote: source=4 (older), target=11 (newer)
        // This surrounds the first vote
        let result = detector.check_attestation(
            validator,
            110,
            4,
            test_hash(4),
            11,
            test_hash(11),
        );
        assert!(result.is_err());
        
        let (offense, _) = result.unwrap_err();
        assert_eq!(offense, SlashableOffense::SurroundVote);
    }

    #[test]
    fn test_slashing_prevents_double_detection() {
        let mut detector = SlashingDetector::new();
        let validator = test_address(1);

        // First proposal
        detector.check_proposal(validator, 10, test_hash(1), 1).unwrap();

        // Slash the validator
        detector.slash_validator(validator);

        // Second proposal should be ignored (validator already slashed)
        let result = detector.check_proposal(validator, 10, test_hash(2), 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_slashable_offense_percentages() {
        assert_eq!(SlashableOffense::DoubleProposal.slash_percent(), 100);
        assert_eq!(SlashableOffense::DoubleAttestation.slash_percent(), 100);
        assert_eq!(SlashableOffense::SurroundVote.slash_percent(), 100);
        assert_eq!(SlashableOffense::InvalidBlock.slash_percent(), 50);
        assert_eq!(SlashableOffense::InvalidAttestation.slash_percent(), 25);
    }

    #[test]
    fn test_validator_slashing() {
        let config = SlashingConfig::default();
        let mut validator = Validator::new(
            test_address(1),
            [0u8; 32],
            U256::from(1_000_000u128),
        );
        validator.activate(0);

        let stake_before = validator.stake;
        
        let result = slash_validator(
            &mut validator,
            SlashableOffense::DoubleProposal,
            &config,
            100,
        ).unwrap();

        assert!(validator.slashed);
        assert_eq!(validator.status, ValidatorStatus::Slashed);
        assert!(validator.stake < stake_before);
        assert_eq!(result.slash_amount, stake_before); // 100% slash
    }

    #[test]
    fn test_prune_old_records() {
        let mut detector = SlashingDetector::new();
        let validator = test_address(1);

        // Add old proposal
        detector.check_proposal(validator, 1, test_hash(1), 1).unwrap();
        
        // Add recent proposal
        detector.check_proposal(validator, 100, test_hash(100), 100).unwrap();

        assert_eq!(detector.proposal_count(), 2);

        // Prune records older than epoch 50
        detector.prune_old_records(100, 50);

        assert_eq!(detector.proposal_count(), 1);
    }
}
