//! Consensus - Proof of Contribution (PoC) consensus
//!
//! Validators are selected based on their contributions to the network.

use std::collections::HashMap;

pub mod validator {
    pub use super::{Validator, ValidatorSet};
}

pub mod poc {
    pub use super::{Contribution, ContributionTracker, PoCScore, ContributionType};
}

pub mod attestation {
    pub use super::{Attestation, AttestationPool, AttestationStatus};
}

/// Validator information
#[derive(Debug, Clone)]
pub struct Validator {
    pub address: merklith_types::Address,
    pub stake: u64,
}

/// Consensus error
#[derive(Debug, Clone)]
pub enum ConsensusError {
    InvalidBlock(String),
    InvalidSignature,
    NotValidator,
    InsufficientContribution,
}

impl std::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusError::InvalidBlock(e) => write!(f, "Invalid block: {}", e),
            ConsensusError::InvalidSignature => write!(f, "Invalid signature"),
            ConsensusError::NotValidator => write!(f, "Not a validator"),
            ConsensusError::InsufficientContribution => write!(f, "Insufficient contribution score"),
        }
    }
}

impl std::error::Error for ConsensusError {}

/// Types of contributions that earn PoC score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContributionType {
    BlockProduction,
    Attestation,
    TransactionRelay,
    PeerDiscovery,
    DataAvailability,
}

/// A single contribution record
#[derive(Debug, Clone)]
pub struct Contribution {
    pub contributor: merklith_types::Address,
    pub contribution_type: ContributionType,
    pub weight: u64,
    pub block_number: u64,
    pub timestamp: u64,
}

/// PoC score for a validator
#[derive(Debug, Clone, Default)]
pub struct PoCScore {
    pub total: u64,
    pub block_production: u64,
    pub attestations: u64,
    pub relayed_txs: u64,
    pub discovered_peers: u64,
    pub data_availability: u64,
}

impl PoCScore {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn total(&self) -> u64 {
        self.total
    }
    
    /// Add a contribution with weight. Uses saturating arithmetic to prevent overflow.
    pub fn add_contribution(&mut self, contribution_type: ContributionType, weight: u64) {
        self.total = self.total.saturating_add(weight);
        match contribution_type {
            ContributionType::BlockProduction => {
                self.block_production = self.block_production.saturating_add(weight);
            }
            ContributionType::Attestation => {
                self.attestations = self.attestations.saturating_add(weight);
            }
            ContributionType::TransactionRelay => {
                self.relayed_txs = self.relayed_txs.saturating_add(weight);
            }
            ContributionType::PeerDiscovery => {
                self.discovered_peers = self.discovered_peers.saturating_add(weight);
            }
            ContributionType::DataAvailability => {
                self.data_availability = self.data_availability.saturating_add(weight);
            }
        }
    }
    
    /// Decay scores by factor/divisor. Uses checked division to prevent panic on zero divisor.
    pub fn decay(&mut self, factor: u64, divisor: u64) {
        if divisor == 0 {
            return;
        }
        self.total = self.total / divisor * factor;
        self.block_production = self.block_production / divisor * factor;
        self.attestations = self.attestations / divisor * factor;
        self.relayed_txs = self.relayed_txs / divisor * factor;
        self.discovered_peers = self.discovered_peers / divisor * factor;
        self.data_availability = self.data_availability / divisor * factor;
    }
    
    /// Get percentage contribution for each category
    pub fn get_percentages(&self) -> Option<ContributionPercentages> {
        if self.total == 0 {
            return None;
        }
        Some(ContributionPercentages {
            block_production: (self.block_production as f64 / self.total as f64 * 100.0),
            attestations: (self.attestations as f64 / self.total as f64 * 100.0),
            relayed_txs: (self.relayed_txs as f64 / self.total as f64 * 100.0),
            discovered_peers: (self.discovered_peers as f64 / self.total as f64 * 100.0),
            data_availability: (self.data_availability as f64 / self.total as f64 * 100.0),
        })
    }
}

/// Percentage breakdown of contribution types
#[derive(Debug, Clone, Copy)]
pub struct ContributionPercentages {
    pub block_production: f64,
    pub attestations: f64,
    pub relayed_txs: f64,
    pub discovered_peers: f64,
    pub data_availability: f64,
}

/// Tracks contributions for PoC consensus
#[derive(Debug, Clone)]
pub struct ContributionTracker {
    scores: HashMap<merklith_types::Address, PoCScore>,
    contribution_history: Vec<Contribution>,
    last_decay_block: u64,
    decay_interval: u64,
}

impl ContributionTracker {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            contribution_history: Vec::new(),
            last_decay_block: 0,
            decay_interval: 1000,
        }
    }
    
    pub fn record_contribution(&mut self, contribution: Contribution) {
        let score = self.scores.entry(contribution.contributor).or_default();
        score.add_contribution(contribution.contribution_type, contribution.weight);
        self.contribution_history.push(contribution);
    }
    
    pub fn get_score(&self, address: &merklith_types::Address) -> PoCScore {
        self.scores.get(address).cloned().unwrap_or_default()
    }
    
    pub fn record_block_production(&mut self, proposer: merklith_types::Address, block_number: u64) {
        self.record_contribution(Contribution {
            contributor: proposer,
            contribution_type: ContributionType::BlockProduction,
            weight: 100,
            block_number,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });
    }
    
    pub fn record_attestation(&mut self, attester: merklith_types::Address, block_number: u64) {
        self.record_contribution(Contribution {
            contributor: attester,
            contribution_type: ContributionType::Attestation,
            weight: 10,
            block_number,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });
    }
    
    pub fn record_tx_relay(&mut self, relayer: merklith_types::Address, block_number: u64) {
        self.record_contribution(Contribution {
            contributor: relayer,
            contribution_type: ContributionType::TransactionRelay,
            weight: 1,
            block_number,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });
    }
    
    pub fn maybe_decay(&mut self, current_block: u64) {
        if current_block >= self.last_decay_block + self.decay_interval {
            for score in self.scores.values_mut() {
                score.decay(9, 10);
            }
            self.last_decay_block = current_block;
            self.contribution_history.retain(|c| c.block_number > current_block.saturating_sub(10000));
        }
    }
    
    pub fn get_top_contributors(&self, n: usize) -> Vec<(merklith_types::Address, u64)> {
        let mut contributors: Vec<_> = self.scores.iter()
            .map(|(addr, score)| (*addr, score.total()))
            .collect();
        contributors.sort_by(|a, b| b.1.cmp(&a.1));
        contributors.into_iter().take(n).collect()
    }
    
    pub fn total_contributions(&self) -> u64 {
        self.scores.values().map(|s| s.total()).sum()
    }
}

impl Default for ContributionTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Attestation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestationStatus {
    Pending,
    Finalized,
    Rejected,
}

/// A committee attestation for a block
#[derive(Debug, Clone)]
pub struct Attestation {
    pub block_number: u64,
    pub block_hash: [u8; 32],
    pub attester: merklith_types::Address,
    pub signature: Vec<u8>,
    pub timestamp: u64,
    pub status: AttestationStatus,
}

impl Attestation {
    pub fn new(
        block_number: u64,
        block_hash: [u8; 32],
        attester: merklith_types::Address,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            block_number,
            block_hash,
            attester,
            signature,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            status: AttestationStatus::Pending,
        }
    }
    
    pub fn signing_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.block_number.to_le_bytes());
        msg.extend_from_slice(&self.block_hash);
        msg
    }
}

/// Pool to collect and aggregate attestations
#[derive(Debug, Clone, Default)]
pub struct AttestationPool {
    attestations: HashMap<u64, Vec<Attestation>>,
    finalized_blocks: HashMap<u64, [u8; 32]>,
    finality_threshold: usize,
}

impl AttestationPool {
    pub fn new() -> Self {
        Self {
            attestations: HashMap::new(),
            finalized_blocks: HashMap::new(),
            finality_threshold: 2,
        }
    }
    
    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.finality_threshold = threshold;
        self
    }
    
    pub fn add_attestation(&mut self, attestation: Attestation) -> bool {
        let block_number = attestation.block_number;
        
        if self.finalized_blocks.contains_key(&block_number) {
            return false;
        }
        
        let attestations = self.attestations.entry(block_number).or_default();
        
        for existing in attestations.iter() {
            if existing.attester == attestation.attester {
                return false;
            }
        }
        
        attestations.push(attestation);
        true
    }
    
    pub fn check_finality(&mut self, block_number: u64, block_hash: [u8; 32]) -> bool {
        if self.finalized_blocks.contains_key(&block_number) {
            return true;
        }
        
        let count = self.attestations.get(&block_number)
            .map(|v| v.len())
            .unwrap_or(0);
        
        if count >= self.finality_threshold {
            for att in self.attestations.entry(block_number).or_default() {
                att.status = AttestationStatus::Finalized;
            }
            self.finalized_blocks.insert(block_number, block_hash);
            return true;
        }
        
        false
    }
    
    pub fn is_finalized(&self, block_number: u64) -> bool {
        self.finalized_blocks.contains_key(&block_number)
    }
    
    pub fn get_attestation_count(&self, block_number: u64) -> usize {
        self.attestations.get(&block_number).map(|v| v.len()).unwrap_or(0)
    }
    
    pub fn get_attestations(&self, block_number: u64) -> Vec<&Attestation> {
        self.attestations.get(&block_number)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
    
    pub fn get_finalized_blocks(&self) -> Vec<(u64, [u8; 32])> {
        let mut blocks: Vec<_> = self.finalized_blocks.iter()
            .map(|(n, h)| (*n, *h))
            .collect();
        blocks.sort_by_key(|(n, _)| *n);
        blocks
    }
    
    pub fn latest_finalized(&self) -> Option<(u64, [u8; 32])> {
        self.finalized_blocks.iter()
            .max_by_key(|(n, _)| *n)
            .map(|(n, h)| (*n, *h))
    }
    
    pub fn prune_old_attestations(&mut self, current_block: u64, keep_blocks: u64) {
        self.attestations.retain(|&block_num, _| block_num + keep_blocks >= current_block);
    }
}

/// Validator set with PoC scoring
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    validators: HashMap<merklith_types::Address, u64>,
    contribution_tracker: ContributionTracker,
}

impl ValidatorSet {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            contribution_tracker: ContributionTracker::new(),
        }
    }

    pub fn add_validator(&mut self, address: merklith_types::Address, stake: u64) {
        self.validators.insert(address, stake);
    }

    pub fn is_validator(&self, address: &merklith_types::Address) -> bool {
        self.validators.contains_key(address)
    }

    pub fn len(&self) -> usize {
        self.validators.len()
    }

    pub fn select_proposer(&self, block_number: u64) -> Option<merklith_types::Address> {
        let validators: Vec<_> = self.validators.keys().cloned().collect();
        if validators.is_empty() {
            return None;
        }
        let index = (block_number as usize) % validators.len();
        Some(validators[index])
    }
    
    pub fn select_proposer_poc(&self, block_number: u64) -> Option<merklith_types::Address> {
        if self.validators.is_empty() {
            return None;
        }
        
        let total_contrib = self.contribution_tracker.total_contributions();
        
        if total_contrib == 0 {
            return self.select_proposer(block_number);
        }
        
        let mut cumulative = 0u64;
        let target = block_number % total_contrib.max(1);
        
        for (addr, _) in &self.validators {
            let score = self.contribution_tracker.get_score(addr).total();
            cumulative += score;
            if cumulative > target {
                return Some(*addr);
            }
        }
        
        self.validators.keys().next().copied()
    }
    
    pub fn contribution_tracker(&self) -> &ContributionTracker {
        &self.contribution_tracker
    }
    
    pub fn contribution_tracker_mut(&mut self) -> &mut ContributionTracker {
        &mut self.contribution_tracker
    }
    
    pub fn get_validator_score(&self, address: &merklith_types::Address) -> PoCScore {
        self.contribution_tracker.get_score(address)
    }
}

impl Default for ValidatorSet {
    fn default() -> Self {
        Self::new()
    }
}

/// PoC consensus engine
pub struct ConsensusEngine {
    validator_set: ValidatorSet,
    block_time: u64,
    min_contribution_score: u64,
    attestation_pool: AttestationPool,
}

impl ConsensusEngine {
    pub fn new(validator_set: ValidatorSet, block_time: u64) -> Self {
        Self {
            validator_set,
            block_time,
            min_contribution_score: 10,
            attestation_pool: AttestationPool::new(),
        }
    }
    
    pub fn with_min_contribution(mut self, min_score: u64) -> Self {
        self.min_contribution_score = min_score;
        self
    }
    
    pub fn with_finality_threshold(mut self, threshold: usize) -> Self {
        self.attestation_pool = AttestationPool::new().with_threshold(threshold);
        self
    }

    pub fn validate_block(&self, block: &merklith_types::Block) -> Result<(), ConsensusError> {
        if !self.validator_set.is_validator(&block.header.proposer) {
            return Err(ConsensusError::NotValidator);
        }

        Ok(())
    }

    pub fn next_proposer(&self, block_number: u64) -> Option<merklith_types::Address> {
        self.validator_set.select_proposer_poc(block_number)
    }

    pub fn block_time(&self) -> u64 {
        self.block_time
    }
    
    pub fn record_block_production(&mut self, proposer: merklith_types::Address, block_number: u64) {
        self.validator_set.contribution_tracker_mut()
            .record_block_production(proposer, block_number);
    }
    
    pub fn record_attestation(&mut self, attester: merklith_types::Address, block_number: u64) {
        self.validator_set.contribution_tracker_mut()
            .record_attestation(attester, block_number);
    }
    
    pub fn add_attestation(&mut self, attestation: Attestation) -> bool {
        let attester = attestation.attester;
        let block_number = attestation.block_number;
        let result = self.attestation_pool.add_attestation(attestation);
        if result {
            self.record_attestation(attester, block_number);
        }
        result
    }
    
    pub fn check_finality(&mut self, block_number: u64, block_hash: [u8; 32]) -> bool {
        self.attestation_pool.check_finality(block_number, block_hash)
    }
    
    pub fn is_finalized(&self, block_number: u64) -> bool {
        self.attestation_pool.is_finalized(block_number)
    }
    
    pub fn attestation_count(&self, block_number: u64) -> usize {
        self.attestation_pool.get_attestation_count(block_number)
    }
    
    pub fn latest_finalized(&self) -> Option<(u64, [u8; 32])> {
        self.attestation_pool.latest_finalized()
    }
    
    pub fn attestation_pool(&self) -> &AttestationPool {
        &self.attestation_pool
    }
    
    pub fn validator_set(&self) -> &ValidatorSet {
        &self.validator_set
    }
    
    pub fn validator_set_mut(&mut self) -> &mut ValidatorSet {
        &mut self.validator_set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_set() {
        let mut set = ValidatorSet::new();
        let addr = merklith_types::Address::from_bytes([1u8; 20]);

        set.add_validator(addr, 1000);
        assert!(set.is_validator(&addr));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_select_proposer() {
        let mut set = ValidatorSet::new();
        let addr1 = merklith_types::Address::from_bytes([1u8; 20]);
        let addr2 = merklith_types::Address::from_bytes([2u8; 20]);

        set.add_validator(addr1, 1000);
        set.add_validator(addr2, 1000);

        let proposer = set.select_proposer(0);
        assert!(proposer.is_some());
    }
    
    #[test]
    fn test_poc_score() {
        let mut score = PoCScore::new();
        assert_eq!(score.total(), 0);
        
        score.add_contribution(ContributionType::BlockProduction, 100);
        assert_eq!(score.total(), 100);
        assert_eq!(score.block_production, 100);
        
        score.add_contribution(ContributionType::Attestation, 10);
        assert_eq!(score.total(), 110);
        assert_eq!(score.attestations, 10);
    }
    
    #[test]
    fn test_contribution_tracker() {
        let mut tracker = ContributionTracker::new();
        let addr = merklith_types::Address::from_bytes([1u8; 20]);
        
        tracker.record_block_production(addr, 1);
        tracker.record_block_production(addr, 2);
        tracker.record_attestation(addr, 2);
        
        let score = tracker.get_score(&addr);
        assert_eq!(score.block_production, 200);
        assert_eq!(score.attestations, 10);
        assert_eq!(score.total(), 210);
    }
    
    #[test]
    fn test_poc_proposer_selection() {
        let mut set = ValidatorSet::new();
        let addr1 = merklith_types::Address::from_bytes([1u8; 20]);
        let addr2 = merklith_types::Address::from_bytes([2u8; 20]);

        set.add_validator(addr1, 1000);
        set.add_validator(addr2, 1000);
        
        set.contribution_tracker_mut().record_block_production(addr1, 1);
        set.contribution_tracker_mut().record_block_production(addr1, 2);
        set.contribution_tracker_mut().record_block_production(addr2, 3);

        let proposer = set.select_proposer_poc(0);
        assert!(proposer.is_some());
        
        let top = set.contribution_tracker().get_top_contributors(10);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, addr1);
    }
    
    #[test]
    fn test_score_decay() {
        let mut score = PoCScore::new();
        score.add_contribution(ContributionType::BlockProduction, 100);
        
        score.decay(9, 10);
        assert_eq!(score.total(), 90);
        assert_eq!(score.block_production, 90);
    }
    
    #[test]
    fn test_attestation_pool() {
        let mut pool = AttestationPool::new().with_threshold(2);
        let addr1 = merklith_types::Address::from_bytes([1u8; 20]);
        let addr2 = merklith_types::Address::from_bytes([2u8; 20]);
        let block_hash = [5u8; 32];
        
        let att1 = Attestation::new(1, block_hash, addr1, vec![1, 2, 3]);
        let att2 = Attestation::new(1, block_hash, addr2, vec![4, 5, 6]);
        
        assert!(pool.add_attestation(att1));
        assert!(pool.add_attestation(att2));
        assert_eq!(pool.get_attestation_count(1), 2);
        
        assert!(pool.check_finality(1, block_hash));
        assert!(pool.is_finalized(1));
    }
    
    #[test]
    fn test_attestation_duplicate_rejected() {
        let mut pool = AttestationPool::new();
        let addr = merklith_types::Address::from_bytes([1u8; 20]);
        let block_hash = [1u8; 32];
        
        let att1 = Attestation::new(1, block_hash, addr, vec![1, 2, 3]);
        let att2 = Attestation::new(1, block_hash, addr, vec![4, 5, 6]);
        
        assert!(pool.add_attestation(att1));
        assert!(!pool.add_attestation(att2));
    }
    
    #[test]
    fn test_consensus_engine_attestations() {
        let mut set = ValidatorSet::new();
        let addr1 = merklith_types::Address::from_bytes([1u8; 20]);
        let addr2 = merklith_types::Address::from_bytes([2u8; 20]);
        let addr3 = merklith_types::Address::from_bytes([3u8; 20]);
        
        set.add_validator(addr1, 1000);
        set.add_validator(addr2, 1000);
        set.add_validator(addr3, 1000);
        
        let mut engine = ConsensusEngine::new(set, 2).with_finality_threshold(2);
        let block_hash = [42u8; 32];
        
        let att1 = Attestation::new(1, block_hash, addr1, vec![1]);
        let att2 = Attestation::new(1, block_hash, addr2, vec![2]);
        
        engine.add_attestation(att1);
        engine.add_attestation(att2);
        
        assert!(engine.check_finality(1, block_hash));
        assert!(engine.is_finalized(1));
        
        let score1 = engine.validator_set().get_validator_score(&addr1);
        let score2 = engine.validator_set().get_validator_score(&addr2);
        assert_eq!(score1.attestations, 10);
        assert_eq!(score2.attestations, 10);
    }
}
