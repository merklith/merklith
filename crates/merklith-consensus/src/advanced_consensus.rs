//! Advanced Consensus Module - Improved Proof of Contribution
//! 
//! Enhanced consensus with:
//! - Adaptive block time
//! - Multi-dimensional contribution scoring
//! - Dynamic validator sets
//! - Fast finality gadget
//! - BFT-like safety guarantees

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{Duration, Instant};
use merklith_types::{Address, Hash, Block, BlockHeader, U256};
use serde::{Serialize, Deserialize};

/// Advanced consensus configuration
#[derive(Debug, Clone)]
pub struct AdvancedConsensusConfig {
    /// Minimum validators
    pub min_validators: usize,
    /// Maximum validators
    pub max_validators: usize,
    /// Target block time (adaptive)
    pub target_block_time_ms: u64,
    /// Block time adjustment factor
    pub block_time_adjustment_rate: f64,
    /// Finality threshold (2/3)
    pub finality_threshold: f64,
    /// Epoch length (blocks)
    pub epoch_length: u64,
    /// Contribution score decay
    pub contribution_decay: f64,
    /// Minimum contribution score
    pub min_contribution_score: f64,
    /// Max missed blocks before removal
    pub max_missed_blocks: u64,
    /// Enable fast finality
    pub enable_fast_finality: bool,
}

impl Default for AdvancedConsensusConfig {
    fn default() -> Self {
        Self {
            min_validators: 4,
            max_validators: 100,
            target_block_time_ms: 6000, // 6 seconds
            block_time_adjustment_rate: 0.1,
            finality_threshold: 2.0 / 3.0,
            epoch_length: 1000,
            contribution_decay: 0.99,
            min_contribution_score: 10.0,
            max_missed_blocks: 50,
            enable_fast_finality: true,
        }
    }
}

/// Advanced consensus engine
pub struct AdvancedConsensusEngine {
    config: AdvancedConsensusConfig,
    /// Current validators
    validators: Arc<Mutex<Vec<Validator>>>,
    /// Current epoch
    current_epoch: Arc<Mutex<u64>>,
    /// Validator contributions
    contributions: Arc<Mutex<HashMap<Address, ContributionScore>>>,
    /// Block proposals
    proposals: Arc<Mutex<VecDeque<BlockProposal>>>,
    /// Finalized blocks
    finalized_blocks: Arc<Mutex<HashMap<u64, Hash>>>,
    /// Adaptive block time
    adaptive_block_time: Arc<Mutex<u64>>,
    /// Last block time
    last_block_time: Arc<Mutex<Instant>>,
    /// Current proposer index
    proposer_index: Arc<Mutex<usize>>,
    /// Fast finality votes
    finality_votes: Arc<Mutex<HashMap<u64, Vec<FinalityVote>>>>,
}

/// Validator with advanced scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: Address,
    pub public_key: Vec<u8>,
    pub stake: U256,
    pub contribution_score: f64,
    pub reputation_score: f64,
    pub total_blocks_proposed: u64,
    pub total_blocks_missed: u64,
    pub last_active_block: u64,
    pub is_active: bool,
    pub join_time: u64,
}

/// Multi-dimensional contribution score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionScore {
    pub total: f64,
    pub uptime: f64,
    pub proposal_quality: f64,
    pub network_participation: f64,
    pub community_engagement: f64,
    pub last_updated: u64,
}

impl Default for ContributionScore {
    fn default() -> Self {
        Self {
            total: 100.0,
            uptime: 100.0,
            proposal_quality: 100.0,
            network_participation: 100.0,
            community_engagement: 100.0,
            last_updated: 0,
        }
    }
}

/// Block proposal
#[derive(Debug, Clone)]
pub struct BlockProposal {
    pub block: Block,
    pub proposer: Address,
    pub timestamp: Instant,
    pub votes: Vec<Vote>,
    pub status: ProposalStatus,
}

/// Proposal status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Finalized,
}

/// Vote on proposal
#[derive(Debug, Clone)]
pub struct Vote {
    pub validator: Address,
    pub block_hash: Hash,
    pub signature: Vec<u8>,
    pub timestamp: Instant,
}

/// Fast finality vote
#[derive(Debug, Clone)]
pub struct FinalityVote {
    pub validator: Address,
    pub block_number: u64,
    pub block_hash: Hash,
    pub signature: Vec<u8>,
}

/// Consensus events
#[derive(Debug, Clone)]
pub enum ConsensusEvent {
    BlockProposed(BlockProposal),
    BlockAccepted(Block),
    BlockFinalized(Block),
    ValidatorAdded(Validator),
    ValidatorRemoved(Address),
    EpochChanged(u64),
}

/// Consensus statistics
#[derive(Debug, Clone, Serialize)]
pub struct ConsensusStats {
    pub current_epoch: u64,
    pub active_validators: usize,
    pub total_stake: U256,
    pub avg_block_time_ms: u64,
    pub last_finalized_block: u64,
    pub safety_threshold: f64,
}

impl AdvancedConsensusEngine {
    /// Create new consensus engine
    pub fn new(config: AdvancedConsensusConfig) -> Self {
        Self {
            config,
            validators: Arc::new(Mutex::new(Vec::new())),
            current_epoch: Arc::new(Mutex::new(0)),
            contributions: Arc::new(Mutex::new(HashMap::new())),
            proposals: Arc::new(Mutex::new(VecDeque::new())),
            finalized_blocks: Arc::new(Mutex::new(HashMap::new())),
            adaptive_block_time: Arc::new(Mutex::new(6000)),
            last_block_time: Arc::new(Mutex::new(Instant::now())),
            proposer_index: Arc::new(Mutex::new(0)),
            finality_votes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize with genesis validators
    pub fn initialize(
        &self,
        genesis_validators: Vec<Validator>,
    ) {
        let mut validators = self.validators.lock().unwrap();
        *validators = genesis_validators.clone();
        
        let mut contributions = self.contributions.lock().unwrap();
        for validator in genesis_validators {
            contributions.insert(
                validator.address,
                ContributionScore::default()
            );
        }
    }

    /// Select next proposer (weighted by contribution score)
    pub fn select_proposer(
        &self,
    ) -> Option<Address> {
        let validators = self.validators.lock().unwrap();
        let contributions = self.contributions.lock().unwrap();
        
        if validators.is_empty() {
            return None;
        }
        
        // Filter active validators with sufficient score
        let active_validators: Vec<&Validator> = validators
            .iter()
            .filter(|v| {
                v.is_active && 
                contributions.get(&v.address)
                    .map(|c| c.total >= self.config.min_contribution_score)
                    .unwrap_or(false)
            })
            .collect();
        
        if active_validators.is_empty() {
            return None;
        }
        
        // Weighted random selection based on contribution score
        let total_score: f64 = active_validators
            .iter()
            .map(|v| contributions.get(&v.address).map(|c| c.total).unwrap_or(0.0))
            .sum();
        
        let mut index = self.proposer_index.lock().unwrap();
        *index = (*index + 1) % active_validators.len();
        
        // Simple round-robin for now (can be weighted)
        Some(active_validators[*index].address)
    }

    /// Submit block proposal
    pub fn submit_proposal(
        &self,
        block: Block,
        proposer: Address,
    ) -> Result<(), String> {
        // Verify proposer
        if !self.is_valid_proposer(proposer) {
            return Err("Invalid proposer".to_string());
        }
        
        let proposal = BlockProposal {
            block: block.clone(),
            proposer,
            timestamp: Instant::now(),
            votes: Vec::new(),
            status: ProposalStatus::Pending,
        };
        
        let mut proposals = self.proposals.lock().unwrap();
        proposals.push_back(proposal);
        
        // Limit pending proposals
        if proposals.len() > 10 {
            proposals.pop_front();
        }
        
        Ok(())
    }

    /// Vote on proposal
    pub fn vote(
        &self,
        validator: Address,
        block_hash: Hash,
        signature: Vec<u8>,
    ) -> Result<(), String> {
        if !self.is_validator(validator) {
            return Err("Not a validator".to_string());
        }
        
        let mut proposals = self.proposals.lock().unwrap();
        
        for proposal in proposals.iter_mut() {
            if proposal.block.hash() == block_hash {
                // Check for duplicate vote
                if proposal.votes.iter().any(|v| v.validator == validator) {
                    return Err("Already voted".to_string());
                }
                
                proposal.votes.push(Vote {
                    validator,
                    block_hash,
                    signature,
                    timestamp: Instant::now(),
                });
                
                // Check if proposal has enough votes
                let total_stake = self.get_total_stake();
                let vote_stake: U256 = proposal.votes
                    .iter()
                    .filter_map(|v| self.get_validator_stake(v.validator))
                    .fold(U256::ZERO, |acc, stake| acc + stake);
                
                let ratio = vote_stake.as_u128() as f64 / total_stake.as_u128() as f64;
                
                if ratio >= self.config.finality_threshold {
                    proposal.status = ProposalStatus::Accepted;
                }
                
                return Ok(());
            }
        }
        
        Err("Proposal not found".to_string())
    }

    /// Submit fast finality vote
    pub fn submit_finality_vote(
        &self,
        vote: FinalityVote,
    ) -> Result<(), String> {
        if !self.config.enable_fast_finality {
            return Err("Fast finality not enabled".to_string());
        }
        
        if !self.is_validator(vote.validator) {
            return Err("Not a validator".to_string());
        }
        
        let mut votes = self.finality_votes.lock().unwrap();
        let block_votes = votes.entry(vote.block_number).or_insert_with(Vec::new);
        
        // Check for duplicate
        if block_votes.iter().any(|v| v.validator == vote.validator) {
            return Err("Already voted for finality".to_string());
        }
        
        block_votes.push(vote.clone());
        
        // Check if we have enough votes for finality
        let total_stake = self.get_total_stake();
        let finality_stake: U256 = block_votes
            .iter()
            .filter_map(|v| self.get_validator_stake(v.validator))
            .fold(U256::ZERO, |acc, stake| acc + stake);
        
        let ratio = finality_stake.as_u128() as f64 / total_stake.as_u128() as f64;
        
        if ratio >= self.config.finality_threshold {
            // Finalize block
            let mut finalized = self.finalized_blocks.lock().unwrap();
            finalized.insert(vote.block_number, vote.block_hash);
        }
        
        Ok(())
    }

    /// Update contribution scores
    pub fn update_contributions(
        &self,
        block_number: u64,
    ) {
        let mut contributions = self.contributions.lock().unwrap();
        let validators = self.validators.lock().unwrap();
        
        for validator in validators.iter() {
            let score = contributions.entry(validator.address).or_default();
            
            // Decay old scores
            score.total *= self.config.contribution_decay;
            score.uptime *= self.config.contribution_decay;
            score.proposal_quality *= self.config.contribution_decay;
            score.network_participation *= self.config.contribution_decay;
            
            // Update based on performance
            if validator.total_blocks_proposed > 0 {
                let success_rate = 1.0 - (validator.total_blocks_missed as f64 
                    / validator.total_blocks_proposed as f64);
                score.uptime = score.uptime.max(0.0) * 0.9 + success_rate * 100.0 * 0.1;
            }
            
            // Recalculate total
            score.total = (score.uptime + score.proposal_quality + 
                score.network_participation + score.community_engagement) / 4.0;
            
            score.last_updated = block_number;
        }
    }

    /// Adjust block time based on network conditions
    pub fn adjust_block_time(&self) {
        let last = *self.last_block_time.lock().unwrap();
        let elapsed = last.elapsed().as_millis() as u64;
        
        let mut adaptive = self.adaptive_block_time.lock().unwrap();
        let target = self.config.target_block_time_ms;
        
        // Adjust towards target
        let error = elapsed as f64 - target as f64;
        let adjustment = error * self.config.block_time_adjustment_rate;
        
        *adaptive = ((*adaptive as f64 - adjustment) as u64)
            .clamp(target / 2, target * 2);
    }

    /// Get next block time
    pub fn get_next_block_time(&self,
    ) -> Duration {
        let adaptive = *self.adaptive_block_time.lock().unwrap();
        Duration::from_millis(adaptive)
    }

    /// Start new epoch
    pub fn start_new_epoch(&self,
    ) {
        let mut epoch = self.current_epoch.lock().unwrap();
        *epoch += 1;
        
        // Update validator set
        self.update_validator_set();
    }

    /// Update validator set (add/remove based on performance)
    fn update_validator_set(&self,
    ) {
        let mut validators = self.validators.lock().unwrap();
        let contributions = self.contributions.lock().unwrap();
        
        // Remove underperforming validators
        validators.retain(|v| {
            let score = contributions.get(&v.address)
                .map(|c| c.total)
                .unwrap_or(0.0);
            
            v.total_blocks_missed < self.config.max_missed_blocks &&
            score >= self.config.min_contribution_score
        });
        
        // Ensure minimum validators
        if validators.len() < self.config.min_validators {
            // In production: trigger emergency protocol
        }
    }

    /// Get consensus statistics
    pub fn get_stats(&self,
    ) -> ConsensusStats {
        let validators = self.validators.lock().unwrap();
        let epoch = *self.current_epoch.lock().unwrap();
        let finalized = self.finalized_blocks.lock().unwrap();
        let adaptive = *self.adaptive_block_time.lock().unwrap();
        
        let active_count = validators.iter().filter(|v| v.is_active).count();
        let total_stake: U256 = validators
            .iter()
            .map(|v| v.stake)
            .fold(U256::ZERO, |acc, s| acc + s);
        
        let last_finalized = finalized.keys().copied().max().unwrap_or(0);
        
        ConsensusStats {
            current_epoch: epoch,
            active_validators: active_count,
            total_stake,
            avg_block_time_ms: adaptive,
            last_finalized_block: last_finalized,
            safety_threshold: self.config.finality_threshold,
        }
    }

    // Helper methods
    fn is_validator(&self,
        address: Address,
    ) -> bool {
        let validators = self.validators.lock().unwrap();
        validators.iter().any(|v| v.address == address && v.is_active)
    }

    fn is_valid_proposer(&self,
        address: Address,
    ) -> bool {
        // In production: verify against selected proposer
        self.is_validator(address)
    }

    fn get_validator_stake(
        &self,
        address: Address,
    ) -> Option<U256> {
        let validators = self.validators.lock().unwrap();
        validators.iter()
            .find(|v| v.address == address)
            .map(|v| v.stake)
    }

    fn get_total_stake(&self,
    ) -> U256 {
        let validators = self.validators.lock().unwrap();
        validators.iter()
            .filter(|v| v.is_active)
            .map(|v| v.stake)
            .fold(U256::ZERO, |acc, s| acc + s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator(id: u8) -> Validator {
        Validator {
            address: Address::from_bytes([id; 20]),
            public_key: vec![id; 32],
            stake: U256::from(1000u64),
            contribution_score: 100.0,
            reputation_score: 100.0,
            total_blocks_proposed: 0,
            total_blocks_missed: 0,
            last_active_block: 0,
            is_active: true,
            join_time: 0,
        }
    }

    #[test]
    fn test_consensus_initialization() {
        let config = AdvancedConsensusConfig::default();
        let engine = AdvancedConsensusEngine::new(config);
        
        let validators = vec![
            create_test_validator(1),
            create_test_validator(2),
            create_test_validator(3),
            create_test_validator(4),
        ];
        
        engine.initialize(validators);
        
        let stats = engine.get_stats();
        assert_eq!(stats.active_validators, 4);
    }

    #[test]
    fn test_proposer_selection() {
        let config = AdvancedConsensusConfig::default();
        let engine = AdvancedConsensusEngine::new(config);
        
        let validators = vec![
            create_test_validator(1),
            create_test_validator(2),
        ];
        
        engine.initialize(validators);
        
        let proposer1 = engine.select_proposer();
        assert!(proposer1.is_some());
        
        let proposer2 = engine.select_proposer();
        assert!(proposer2.is_some());
        
        // Should rotate
        assert_ne!(proposer1, proposer2);
    }

    #[test]
    fn test_block_time_adjustment() {
        let config = AdvancedConsensusConfig::default();
        let engine = AdvancedConsensusEngine::new(config);
        
        let initial = engine.get_next_block_time();
        engine.adjust_block_time();
        let adjusted = engine.get_next_block_time();
        
        // Should be within reasonable bounds
        assert!(adjusted >= Duration::from_millis(3000));
        assert!(adjusted <= Duration::from_millis(12000));
    }

    #[test]
    fn test_contribution_update() {
        let config = AdvancedConsensusConfig::default();
        let engine = AdvancedConsensusEngine::new(config);
        
        let validators = vec![
            create_test_validator(1),
        ];
        
        engine.initialize(validators);
        engine.update_contributions(100);
        
        // Contribution should be decayed but still positive
        let contributions = engine.contributions.lock().unwrap();
        let score = contributions.get(&Address::from_bytes([1u8; 20])).unwrap();
        assert!(score.total > 0.0);
        assert!(score.total <= 100.0);
    }
}
