//! Enterprise-Grade Proof of Contribution Consensus
//! 
//! Features:
//! - Validator staking with slashing
//! - Byzantine Fault Tolerance (BFT)
//! - Multi-dimensional contribution scoring
//! - Anti-spam and anti-sybil protection
//! - Block finality with justification

use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use sha3::{Sha3_256, Digest};
use serde::{Serialize, Deserialize};

// Constants for security
const MIN_STAKE_AMOUNT: u64 = 1_000_000; // 1M MERK minimum stake
const BLOCK_PRODUCTION_TIMEOUT_MS: u64 = 6000; // 6 seconds
const FINALITY_THRESHOLD: usize = 2; // 2/3 validators needed
const MAX_VALIDATORS: usize = 100;
const SLASH_PERCENTAGE: u8 = 10; // 10% slash for misbehavior
const STAKE_LOCK_PERIOD: u64 = 86400 * 7; // 7 days lock after unstake

/// Validator status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Jailed,
    Unbonding,
}

/// Validator with full staking info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    pub stake: u64,
    pub status: ValidatorStatus,
    pub joined_at: u64,
    pub last_produced_block: u64,
    pub total_blocks_produced: u64,
    pub missed_blocks: u64,
    pub contribution_score: u64,
    pub slash_count: u32,
    pub rewards: u64,
    pub unbonding_end: Option<u64>,
    pub public_key: Vec<u8>,
}

impl Validator {
    pub fn new(address: String, stake: u64, public_key: Vec<u8>) -> Result<Self, ConsensusError> {
        if stake < MIN_STAKE_AMOUNT {
            return Err(ConsensusError::InsufficientStake(stake, MIN_STAKE_AMOUNT));
        }
        
        let now = current_timestamp();
        
        Ok(Self {
            address,
            stake,
            status: ValidatorStatus::Active,
            joined_at: now,
            last_produced_block: 0,
            total_blocks_produced: 0,
            missed_blocks: 0,
            contribution_score: 0,
            slash_count: 0,
            rewards: 0,
            unbonding_end: None,
            public_key,
        })
    }
    
    /// Check if validator can produce blocks
    pub fn is_active(&self) -> bool {
        matches!(self.status, ValidatorStatus::Active)
    }
    
    /// Calculate voting power based on stake and contribution
    pub fn voting_power(&self) -> u64 {
        // Formula: stake * (1 + contribution_score / 10000)
        // This rewards validators who contribute more to the network
        let contribution_multiplier = 1.0 + (self.contribution_score as f64 / 10000.0);
        (self.stake as f64 * contribution_multiplier) as u64
    }
    
    /// Slash validator for misbehavior
    pub fn slash(&mut self, reason: &str) -> u64 {
        let slash_amount = self.stake / SLASH_PERCENTAGE as u64;
        self.stake -= slash_amount;
        self.slash_count += 1;
        
        if self.slash_count >= 3 {
            self.status = ValidatorStatus::Jailed;
        }
        
        tracing::warn!(
            "Validator {} slashed {} MERK for: {}",
            self.address,
            slash_amount,
            reason
        );
        
        slash_amount
    }
}

/// Block proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProposal {
    pub block_number: u64,
    pub block_hash: String,
    pub proposer: String,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub parent_hash: String,
}

/// Vote on a block proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub block_hash: String,
    pub validator: String,
    pub vote_type: VoteType,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteType {
    Prevote,
    Precommit,
}

/// Block justification (finality proof)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Justification {
    pub block_hash: String,
    pub block_number: u64,
    pub votes: Vec<Vote>,
    pub total_voting_power: u64,
    pub timestamp: u64,
}

impl Justification {
    pub fn new(block_hash: String, block_number: u64) -> Self {
        Self {
            block_hash,
            block_number,
            votes: Vec::new(),
            total_voting_power: 0,
            timestamp: current_timestamp(),
        }
    }
    
    pub fn add_vote(&mut self, vote: Vote, voting_power: u64) {
        self.votes.push(vote);
        self.total_voting_power += voting_power;
    }
    
    /// Check if justification reaches finality threshold
    pub fn is_final(&self, total_stake: u64) -> bool {
        // Need 2/3 of total voting power
        let threshold = (total_stake * 2) / 3;
        self.total_voting_power >= threshold
    }
}

/// Consensus errors
#[derive(Debug, Clone)]
pub enum ConsensusError {
    InsufficientStake(u64, u64),
    InvalidValidator(String),
    DoubleSigning(String),
    InvalidBlock(String),
    InvalidSignature,
    Timeout(String),
    NotEnoughValidators(usize, usize),
    AlreadyVoted(String),
    ValidatorNotFound(String),
}

impl std::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusError::InsufficientStake(stake, min) => {
                write!(f, "Insufficient stake: {} (minimum: {})", stake, min)
            }
            ConsensusError::InvalidValidator(addr) => write!(f, "Invalid validator: {}", addr),
            ConsensusError::DoubleSigning(addr) => write!(f, "Double signing detected: {}", addr),
            ConsensusError::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
            ConsensusError::InvalidSignature => write!(f, "Invalid signature"),
            ConsensusError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ConsensusError::NotEnoughValidators(have, need) => {
                write!(f, "Not enough validators: {} (need: {})", have, need)
            }
            ConsensusError::AlreadyVoted(addr) => write!(f, "Already voted: {}", addr),
            ConsensusError::ValidatorNotFound(addr) => write!(f, "Validator not found: {}", addr),
        }
    }
}

impl std::error::Error for ConsensusError {}

/// Enterprise PoC Consensus Engine
pub struct PoCConsensus {
    /// All validators
    validators: Arc<RwLock<HashMap<String, Validator>>>,
    /// Validator set sorted by voting power
    validator_set: Arc<RwLock<Vec<String>>>,
    /// Current round
    current_round: Arc<RwLock<u64>>,
    /// Block proposals for current round
    proposals: Arc<RwLock<HashMap<u64, BlockProposal>>>,
    /// Votes for current round
    votes: Arc<RwLock<HashMap<String, Vec<Vote>>>>,
    /// Finalized blocks with justifications
    finalized_blocks: Arc<RwLock<BTreeMap<u64, Justification>>>,
    /// Slashing events log
    slashing_log: Arc<RwLock<Vec<SlashingEvent>>>,
    /// Total staked amount
    total_stake: Arc<RwLock<u64>>,
    /// My validator address (if running as validator)
    my_address: Option<String>,
}

/// Slashing event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    pub timestamp: u64,
    pub validator: String,
    pub amount: u64,
    pub reason: String,
    pub block_number: u64,
}

/// Contribution record for PoC scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub validator: String,
    pub contribution_type: ContributionType,
    pub amount: u64,
    pub block_number: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContributionType {
    BlockProduction,
    Attestation,
    TransactionRelay,
    PeerDiscovery,
    StorageProvider,
}

impl PoCConsensus {
    pub fn new() -> Self {
        Self {
            validators: Arc::new(RwLock::new(HashMap::new())),
            validator_set: Arc::new(RwLock::new(Vec::new())),
            current_round: Arc::new(RwLock::new(0)),
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            finalized_blocks: Arc::new(RwLock::new(BTreeMap::new())),
            slashing_log: Arc::new(RwLock::new(Vec::new())),
            total_stake: Arc::new(RwLock::new(0)),
            my_address: None,
        }
    }
    
    /// Set my validator address
    pub fn set_validator(&mut self, address: String) {
        self.my_address = Some(address);
    }
    
    /// Register a new validator
    pub fn register_validator(
        &self,
        address: String,
        stake: u64,
        public_key: Vec<u8>,
    ) -> Result<(), ConsensusError> {
        let mut validators = self.validators.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        // Check if already exists
        if validators.contains_key(&address) {
            return Err(ConsensusError::InvalidValidator(format!(
                "Validator {} already exists",
                address
            )));
        }
        
        // Check max validators
        if validators.len() >= MAX_VALIDATORS {
            return Err(ConsensusError::InvalidBlock(
                "Maximum validator limit reached".to_string()
            ));
        }
        
        let validator = Validator::new(address.clone(), stake, public_key)?;
        
        // Update total stake
        let mut total = self.total_stake.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        *total += stake;
        
        // Add to validator set
        validators.insert(address.clone(), validator);
        drop(validators);
        
        self.update_validator_set()?;
        
        tracing::info!("Validator {} registered with stake {}", address, stake);
        
        Ok(())
    }
    
    /// Update validator set sorted by voting power
    fn update_validator_set(&self) -> Result<(), ConsensusError> {
        let validators = self.validators.read().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        let mut set: Vec<(String, u64)> = validators
            .values()
            .filter(|v| v.is_active())
            .map(|v| (v.address.clone(), v.voting_power()))
            .collect();
        
        // Sort by voting power descending
        set.sort_by(|a, b| b.1.cmp(&a.1));
        
        let mut validator_set = self.validator_set.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        *validator_set = set.into_iter().map(|(addr, _)| addr).collect();
        
        Ok(())
    }
    
    /// Select next block producer deterministically
    pub fn select_producer(&self, block_number: u64) -> Result<String, ConsensusError> {
        let validator_set = self.validator_set.read().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        if validator_set.is_empty() {
            return Err(ConsensusError::NotEnoughValidators(0, 1));
        }
        
        // Deterministic selection using block number
        let index = (block_number as usize) % validator_set.len();
        let producer = validator_set[index].clone();
        
        // Check if producer is actually active
        let validators = self.validators.read().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        if let Some(validator) = validators.get(&producer) {
            if !validator.is_active() {
                // Find next active validator
                for i in 1..validator_set.len() {
                    let idx = (index + i) % validator_set.len();
                    let addr = &validator_set[idx];
                    if let Some(v) = validators.get(addr) {
                        if v.is_active() {
                            return Ok(addr.clone());
                        }
                    }
                }
                return Err(ConsensusError::NotEnoughValidators(0, 1));
            }
        }
        
        Ok(producer)
    }
    
    /// Submit a block proposal
    pub fn submit_proposal(&self, proposal: BlockProposal) -> Result<(), ConsensusError> {
        // Verify proposer is active validator
        let validators = self.validators.read().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        let validator = validators.get(&proposal.proposer).ok_or_else(|| {
            ConsensusError::ValidatorNotFound(proposal.proposer.clone())
        })?;
        
        if !validator.is_active() {
            return Err(ConsensusError::InvalidValidator(
                "Validator not active".to_string()
            ));
        }
        
        // Verify signature (simplified - real impl would use crypto)
        // In production: verify ed25519 signature
        
        let mut proposals = self.proposals.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        proposals.insert(proposal.block_number, proposal);
        
        Ok(())
    }
    
    /// Submit a vote
    pub fn submit_vote(&self, vote: Vote) -> Result<(), ConsensusError> {
        let validators = self.validators.read().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        // Verify validator exists and is active
        let validator = validators.get(&vote.validator).ok_or_else(|| {
            ConsensusError::ValidatorNotFound(vote.validator.clone())
        })?;
        
        if !validator.is_active() {
            return Err(ConsensusError::InvalidValidator(
                "Validator not active".to_string()
            ));
        }
        
        // Check for double voting
        let mut votes = self.votes.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        let vote_list = votes.entry(vote.block_hash.clone()).or_insert_with(Vec::new);
        
        if vote_list.iter().any(|v| v.validator == vote.validator) {
            // Double signing detected! Slash the validator
            drop(validators);
            drop(votes);
            self.slash_validator(
                &vote.validator,
                "Double signing detected",
                0,
            )?;
            return Err(ConsensusError::DoubleSigning(vote.validator.clone()));
        }
        
        vote_list.push(vote);
        
        Ok(())
    }
    
    /// Try to finalize a block
    pub fn try_finalize(&self, block_hash: String, block_number: u64) -> Result<bool, ConsensusError> {
        let validators = self.validators.read().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        let votes = self.votes.read().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        let vote_list = votes.get(&block_hash).ok_or_else(|| {
            ConsensusError::InvalidBlock("No votes for this block".to_string())
        })?;
        
        let total_stake = *self.total_stake.read().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        let mut justification = Justification::new(block_hash.clone(), block_number);
        
        for vote in vote_list {
            if let Some(validator) = validators.get(&vote.validator) {
                justification.add_vote(vote.clone(), validator.voting_power());
            }
        }
        
        if justification.is_final(total_stake) {
            let mut finalized = self.finalized_blocks.write().map_err(|_| {
                ConsensusError::InvalidBlock("Lock poisoned".to_string())
            })?;
            
            finalized.insert(block_number, justification);
            
            tracing::info!(
                "Block #{} ({}) finalized with {} voting power",
                block_number,
                format_hash(&block_hash),
                justification.total_voting_power
            );
            
            // Update validator stats
            if let Some(proposal) = self.proposals.read().map_err(|_| {
                ConsensusError::InvalidBlock("Lock poisoned".to_string())
            })?.get(&block_number) {
                if let Some(validator) = validators.get(&proposal.proposer) {
                    // Reward the producer
                    // In production: actually distribute rewards
                }
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Slash a validator
    fn slash_validator(
        &self,
        address: &str,
        reason: &str,
        block_number: u64,
    ) -> Result<u64, ConsensusError> {
        let mut validators = self.validators.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        let validator = validators.get_mut(address).ok_or_else(|| {
            ConsensusError::ValidatorNotFound(address.to_string())
        })?;
        
        let slashed_amount = validator.slash(reason);
        
        // Update total stake
        let mut total = self.total_stake.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        *total -= slashed_amount;
        
        // Log slashing event
        let mut log = self.slashing_log.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        // Add new event and maintain max size to prevent unbounded growth
        log.push(SlashingEvent {
            timestamp: current_timestamp(),
            validator: address.to_string(),
            amount: slashed_amount,
            reason: reason.to_string(),
            block_number,
        });
        
        // Keep only last 10,000 events to prevent OOM
        // Use efficient truncation instead of remove(0) which is O(n)
        const MAX_SLASHING_LOG_SIZE: usize = 10_000;
        if log.len() > MAX_SLASHING_LOG_SIZE {
            let excess = log.len() - MAX_SLASHING_LOG_SIZE;
            // Truncate from front using split_off which is more efficient
            *log = log.split_off(excess);
        }
        
        drop(validators);
        self.update_validator_set()?;
        
        Ok(slashed_amount)
    }
    
    /// Record a contribution
    pub fn record_contribution(&self, contribution: Contribution) -> Result<(), ConsensusError> {
        let mut validators = self.validators.write().map_err(|_| {
            ConsensusError::InvalidBlock("Lock poisoned".to_string())
        })?;
        
        if let Some(validator) = validators.get_mut(&contribution.validator) {
            let points = match contribution.contribution_type {
                ContributionType::BlockProduction => 100,
                ContributionType::Attestation => 10,
                ContributionType::TransactionRelay => 5,
                ContributionType::PeerDiscovery => 20,
                ContributionType::StorageProvider => 50,
            };
            
            validator.contribution_score += points * contribution.amount;
            
            tracing::debug!(
                "Contribution recorded for {}: +{} points",
                contribution.validator,
                points * contribution.amount
            );
        }
        
        drop(validators);
        self.update_validator_set()?;
        
        Ok(())
    }
    
    /// Get validator info
    pub fn get_validator(&self, address: &str) -> Option<Validator> {
        self.validators.read().ok()?.get(address).cloned()
    }
    
    /// Get all validators
    pub fn get_all_validators(&self) -> Vec<Validator> {
        self.validators
            .read()
            .map(|v| v.values().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get validator set
    pub fn get_validator_set(&self) -> Vec<String> {
        self.validator_set.read().map(|v| v.clone()).unwrap_or_default()
    }
    
    /// Check if block is finalized
    pub fn is_finalized(&self, block_number: u64) -> bool {
        self.finalized_blocks
            .read()
            .map(|f| f.contains_key(&block_number))
            .unwrap_or(false)
    }
    
    /// Get justification for a block
    pub fn get_justification(&self, block_number: u64) -> Option<Justification> {
        self.finalized_blocks
            .read()
            .ok()?
            .get(&block_number)
            .cloned()
    }
    
    /// Get total stake
    pub fn get_total_stake(&self) -> u64 {
        self.total_stake.read().map(|t| *t).unwrap_or(0)
    }
    
    /// Get slashing history
    pub fn get_slashing_history(&self) -> Vec<SlashingEvent> {
        self.slashing_log
            .read()
            .map(|l| l.clone())
            .unwrap_or_default()
    }
}

impl Default for PoCConsensus {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn format_hash(hash: &str) -> String {
    if hash.len() > 16 {
        format!("{}...{}", &hash[..8], &hash[hash.len()-8..])
    } else {
        hash.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validator_registration() {
        let consensus = PoCConsensus::new();
        
        let result = consensus.register_validator(
            "0xvalidator1".to_string(),
            MIN_STAKE_AMOUNT,
            vec![1, 2, 3],
        );
        
        assert!(result.is_ok());
        
        let validator = consensus.get_validator("0xvalidator1");
        assert!(validator.is_some());
        assert_eq!(validator.unwrap().stake, MIN_STAKE_AMOUNT);
    }
    
    #[test]
    fn test_insufficient_stake() {
        let consensus = PoCConsensus::new();
        
        let result = consensus.register_validator(
            "0xvalidator1".to_string(),
            MIN_STAKE_AMOUNT - 1,
            vec![1, 2, 3],
        );
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_producer_selection() {
        let consensus = PoCConsensus::new();
        
        // Register 3 validators
        for i in 0..3 {
            consensus.register_validator(
                format!("0xvalidator{}", i),
                MIN_STAKE_AMOUNT,
                vec![i as u8],
            ).unwrap();
        }
        
        let producer = consensus.select_producer(0).unwrap();
        assert!(producer.starts_with("0xvalidator"));
    }
}
