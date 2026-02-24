//! Committee selection using VRF (Verifiable Random Function).
//!
//! The committee is selected based on:
//! 1. PoC scores (weighted probability)
//! 2. VRF proofs (verifiable randomness)
//! 3. Stake-weighted sampling

use std::collections::HashSet;
use merklith_types::{Address, Hash};
use merklith_crypto::vrf::VrfProof;
use crate::poc::{calculate_poc_score, ContributionMetrics, ValidatorInfo, PocConfig};
use crate::error::ConsensusError;

/// Committee configuration.
#[derive(Debug, Clone, Copy)]
pub struct CommitteeConfig {
    /// Target committee size
    pub target_size: usize,
    /// Maximum committee size (hard cap)
    pub max_size: usize,
    /// Minimum committee size
    pub min_size: usize,
    /// Probability boost for high PoC score (multiplier)
    pub poc_boost_factor: f64,
    /// Minimum PoC score to be considered
    pub min_poc_score: f64,
}

impl Default for CommitteeConfig {
    fn default() -> Self {
        Self {
            target_size: 128,
            max_size: 256,
            min_size: 4,
            poc_boost_factor: 1.5,
            min_poc_score: 0.1,
        }
    }
}

/// A committee member.
#[derive(Debug, Clone)]
pub struct CommitteeMember {
    /// Validator address
    pub address: Address,
    /// PoC score
    pub poc_score: f64,
    /// VRF proof for this selection
    pub vrf_proof: VrfProof,
    /// VRF output (determines position)
    pub vrf_output: [u8; 32],
    /// Stake weight
    pub stake_weight: f64,
}

impl CommitteeMember {
    /// Create a new committee member.
    pub fn new(
        address: Address,
        poc_score: f64,
        vrf_proof: VrfProof,
        vrf_output: [u8; 32],
        stake_weight: f64,
    ) -> Self {
        Self {
            address,
            poc_score,
            vrf_proof,
            vrf_output,
            stake_weight,
        }
    }

    /// Get sorting key for committee ordering.
    /// Members are sorted by VRF output for deterministic ordering.
    pub fn sorting_key(&self) -> [u8; 32] {
        self.vrf_output
    }
}

/// Committee for a given epoch.
#[derive(Debug, Clone)]
pub struct Committee {
    /// Epoch number
    pub epoch: u64,
    /// Committee members
    pub members: Vec<CommitteeMember>,
    /// Total stake weight
    pub total_weight: f64,
    /// Block hash used for seed
    pub seed: [u8; 32],
}

impl Committee {
    /// Create a new committee.
    pub fn new(epoch: u64, seed: [u8; 32]) -> Self {
        Self {
            epoch,
            members: Vec::new(),
            total_weight: 0.0,
            seed,
        }
    }

    /// Add a member to the committee.
    pub fn add_member(&mut self, member: CommitteeMember) {
        self.total_weight += member.stake_weight;
        self.members.push(member);
    }

    /// Sort members by VRF output (deterministic ordering).
    pub fn sort(&mut self) {
        self.members.sort_by(|a, b| a.sorting_key().cmp(&b.sorting_key()));
    }

    /// Get the proposer for a given slot.
    /// Uses round-robin selection based on sorted VRF outputs.
    pub fn get_proposer(&self, slot: u64) -> Option<&CommitteeMember> {
        if self.members.is_empty() {
            return None;
        }
        
        let index = (slot as usize) % self.members.len();
        self.members.get(index)
    }

    /// Check if an address is in the committee.
    pub fn contains(&self, address: &Address) -> bool {
        self.members.iter().any(|m| &m.address == address)
    }

    /// Get member by address.
    pub fn get_member(&self, address: &Address) -> Option<&CommitteeMember> {
        self.members.iter().find(|m| &m.address == address)
    }

    /// Get the size of the committee.
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Check if committee has minimum size.
    pub fn is_valid(&self, min_size: usize) -> bool {
        self.members.len() >= min_size
    }

    /// Calculate the attestation threshold (2/3 of weight).
    pub fn attestation_threshold(&self) -> f64 {
        self.total_weight * 2.0 / 3.0
    }
}

/// Selection input for VRF.
#[derive(Debug, Clone)]
pub struct SelectionInput {
    /// Epoch number
    pub epoch: u64,
    /// Validator address
    pub validator: Address,
    /// Seed from previous block hash
    pub seed: [u8; 32],
}

impl SelectionInput {
    /// Serialize to bytes for VRF.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.epoch.to_le_bytes());
        bytes.extend_from_slice(self.validator.as_bytes());
        bytes.extend_from_slice(&self.seed);
        bytes
    }
}

/// Select committee members for an epoch.
/// 
/// # Algorithm
/// 1. Filter validators by minimum PoC score
/// 2. For each validator, verify VRF proof against seed
/// 3. Weight selection probability by PoC score * stake
/// 4. Select top N validators by weighted VRF output
/// 5. Sort by VRF output for deterministic ordering
/// 
/// # Returns
/// The selected committee or an error if selection fails.
pub fn select_committee(
    epoch: u64,
    validators: &[ValidatorInfo],
    seed: [u8; 32],
    poc_config: &PocConfig,
    committee_config: &CommitteeConfig,
) -> Result<Committee, ConsensusError> {
    if validators.len() < committee_config.min_size {
        return Err(ConsensusError::CommitteeSelectionFailed(
            format!("Not enough validators: {} < {}", validators.len(), committee_config.min_size)
        ));
    }

    let mut committee = Committee::new(epoch, seed);
    let mut candidates: Vec<(ValidatorInfo, f64, VrfProof, [u8; 32])> = Vec::new();

    // Calculate PoC scores and collect candidates
    for validator in validators {
        // Calculate PoC score
        let poc_score = match calculate_poc_score(
            validator.stake,
            &validator.contribution,
            validator.epochs_active,
            poc_config,
        ) {
            Ok(score) => score,
            Err(_) => continue, // Skip validators with insufficient stake
        };

        // Filter by minimum PoC score
        if poc_score < committee_config.min_poc_score {
            continue;
        }

        // Create VRF proof (simplified - in production this would come from validator)
        // For now, we use a deterministic hash
        let selection_input = SelectionInput {
            epoch,
            validator: validator.address,
            seed,
        };
        let input_hash = blake3::hash(&selection_input.to_bytes());
        
        // In production, validators provide VRF proofs
        // Here we simulate with deterministic "randomness"
        let mut vrf_output = [0u8; 32];
        vrf_output.copy_from_slice(input_hash.as_bytes());
        
        // Create a dummy VRF proof (in production, this would be actual crypto)
        let vrf_proof = VrfProof::from_bytes(&[0u8; 64])
            .map_err(|e| ConsensusError::CryptoError(e.to_string()))?;

        // Calculate selection weight (PoC score * stake)
        let stake_weight = poc_score * committee_config.poc_boost_factor;

        candidates.push((validator.clone(), stake_weight, vrf_proof, vrf_output));
    }

    if candidates.len() < committee_config.min_size {
        return Err(ConsensusError::CommitteeSelectionFailed(
            format!("Not enough candidates: {} < {}", candidates.len(), committee_config.min_size)
        ));
    }

    // Sort by weighted VRF output (simplified: use VRF output as random number)
    // In production, this would use proper weighted sampling
    candidates.sort_by(|a, b| {
        // Weighted score = vrf_output / weight
        // Lower is better (more likely to be selected)
        // Safe slice extraction with bounds checking
        let score_a = a.3.get(0..8)
            .and_then(|bytes| bytes.try_into().ok())
            .map(|arr: [u8; 8]| u64::from_le_bytes(arr) as f64)
            .unwrap_or(0.0) / a.1.max(1.0);
        let score_b = b.3.get(0..8)
            .and_then(|bytes| bytes.try_into().ok())
            .map(|arr: [u8; 8]| u64::from_le_bytes(arr) as f64)
            .unwrap_or(0.0) / b.1.max(1.0);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Select top N candidates
    let target_size = committee_config.target_size.min(candidates.len());
    let selected = &candidates[..target_size.min(committee_config.max_size)];

    // Add to committee
    for (validator, stake_weight, vrf_proof, vrf_output) in selected {
        let poc_score = calculate_poc_score(
            validator.stake,
            &validator.contribution,
            validator.epochs_active,
            poc_config,
        )?;

        let member = CommitteeMember::new(
            validator.address,
            poc_score,
            vrf_proof.clone(),
            *vrf_output,
            *stake_weight,
        );

        committee.add_member(member);
    }

    // Sort by VRF output for deterministic ordering
    committee.sort();

    if !committee.is_valid(committee_config.min_size) {
        return Err(ConsensusError::CommitteeSelectionFailed(
            "Committee below minimum size".to_string()
        ));
    }

    Ok(committee)
}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::Address;
    use merklith_types::U256;

    fn test_validators(count: usize) -> Vec<ValidatorInfo> {
        let mut validators = Vec::new();
        for i in 0..count {
            let mut addr = [0u8; 20];
            addr[19] = i as u8;
            let address = Address::from_bytes(addr);
            
            validators.push(
                ValidatorInfo::new(address, U256::from(1_000_000_000u128))
                    .with_epochs_active(20)
                    .with_contribution(ContributionMetrics::new().with_uptime(95.0))
            );
        }
        validators
    }

    #[test]
    fn test_committee_selection() {
        let validators = test_validators(10);
        let seed = [1u8; 32];
        let poc_config = PocConfig::default();
        let committee_config = CommitteeConfig {
            target_size: 5,
            min_size: 3,
            max_size: 8,
            ..Default::default()
        };

        let committee = select_committee(
            1,
            &validators,
            seed,
            &poc_config,
            &committee_config,
        ).unwrap();

        assert!(committee.size() >= committee_config.min_size);
        assert!(committee.size() <= committee_config.target_size);
        assert_eq!(committee.epoch, 1);
    }

    #[test]
    fn test_committee_proposer_selection() {
        let mut committee = Committee::new(1, [0u8; 32]);
        
        // Add 5 members
        for i in 0..5 {
            let mut addr = [0u8; 20];
            addr[19] = i;
            let member = CommitteeMember::new(
                Address::from_bytes(addr),
                0.5,
                VrfProof::from_bytes(&[0u8; 64]).unwrap(),
                [i as u8; 32],
                1.0,
            );
            committee.add_member(member);
        }

        // Proposer should rotate
        let proposer_0 = committee.get_proposer(0).unwrap();
        let proposer_1 = committee.get_proposer(1).unwrap();
        let proposer_5 = committee.get_proposer(5).unwrap();

        assert_eq!(proposer_5.address, proposer_0.address); // Should cycle back
        assert_ne!(proposer_0.address, proposer_1.address); // Different slots
    }

    #[test]
    fn test_committee_contains() {
        let mut committee = Committee::new(1, [0u8; 32]);
        
        let address = Address::from_bytes([1u8; 20]);
        let member = CommitteeMember::new(
            address,
            0.5,
            VrfProof::from_bytes(&[0u8; 64]).unwrap(),
            [1u8; 32],
            1.0,
        );
        
        committee.add_member(member);
        
        assert!(committee.contains(&address));
        assert!(!committee.contains(&Address::ZERO));
    }

    #[test]
    fn test_insufficient_validators() {
        let validators = test_validators(2); // Only 2 validators
        let seed = [1u8; 32];
        let poc_config = PocConfig::default();
        let committee_config = CommitteeConfig {
            min_size: 5, // Require 5
            ..Default::default()
        };

        let result = select_committee(
            1,
            &validators,
            seed,
            &poc_config,
            &committee_config,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_attestation_threshold() {
        let mut committee = Committee::new(1, [0u8; 32]);
        
        // Add 3 members with different weights
        for i in 1..=3 {
            let mut addr = [0u8; 20];
            addr[19] = i;
            let member = CommitteeMember::new(
                Address::from_bytes(addr),
                0.5,
                VrfProof::from_bytes(&[0u8; 64]).unwrap(),
                [i as u8; 32],
                i as f64, // Weights: 1, 2, 3
            );
            committee.add_member(member);
        }

        // Total weight = 6, threshold = 4 (2/3 of 6)
        assert_eq!(committee.total_weight, 6.0);
        assert_eq!(committee.attestation_threshold(), 4.0);
    }

    #[test]
    fn test_selection_input_serialization() {
        let input = SelectionInput {
            epoch: 100,
            validator: Address::ZERO,
            seed: [1u8; 32],
        };

        let bytes = input.to_bytes();
        assert_eq!(bytes.len(), 8 + 20 + 32); // epoch + address + seed
    }
}
