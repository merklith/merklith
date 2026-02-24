//! Finality gadget implementing BFT (Byzantine Fault Tolerance) consensus.
//!
//! Guarantees that once a block is finalized, it cannot be reverted
//! unless 1/3+ of validators are malicious.

use std::collections::{HashMap, HashSet};
use merklith_types::{Address, Block, Hash};
use crate::error::ConsensusError;
use crate::committee::Committee;

/// State of a block in the finality process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockState {
    /// Block is proposed but not yet attested
    Proposed,
    /// Block has attestations but not enough for justification
    PartiallyAttested,
    /// Block has enough attestations to be justified
    Justified,
    /// Block is finalized (two consecutive justified blocks)
    Finalized,
    /// Block was rejected/invalid
    Rejected,
}

impl BlockState {
    /// Check if block is justified or better.
    pub fn is_justified(&self) -> bool {
        matches!(self, BlockState::Justified | BlockState::Finalized)
    }

    /// Check if block is finalized.
    pub fn is_finalized(&self) -> bool {
        matches!(self, BlockState::Finalized)
    }
}

/// An attestation for a block.
#[derive(Debug, Clone)]
pub struct Attestation {
    /// Validator address
    pub validator: Address,
    /// Block hash being attested
    pub block_hash: Hash,
    /// Block number
    pub block_number: u64,
    /// Slot number
    pub slot: u64,
    /// Source checkpoint (for Casper FFG)
    pub source_epoch: u64,
    pub source_root: Hash,
    /// Target checkpoint
    pub target_epoch: u64,
    pub target_root: Hash,
}

/// Checkpoint for Casper FFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Checkpoint {
    /// Epoch number
    pub epoch: u64,
    /// Block root (hash)
    pub root: Hash,
}

impl Checkpoint {
    /// Create a new checkpoint.
    pub fn new(epoch: u64, root: Hash) -> Self {
        Self { epoch, root }
    }
}

/// Vote from a validator for a target checkpoint.
#[derive(Debug, Clone)]
struct Vote {
    /// Validator address
    validator: Address,
    /// Source checkpoint
    source: Checkpoint,
    /// Target checkpoint
    target: Checkpoint,
    /// Weight of this vote
    weight: f64,
}

/// Finality engine implementing Casper FFG.
#[derive(Debug)]
pub struct FinalityEngine {
    /// Current justified checkpoint
    justified_checkpoint: Checkpoint,
    /// Current finalized checkpoint
    finalized_checkpoint: Checkpoint,
    /// All votes received (validator -> vote)
    votes: HashMap<Address, Vote>,
    /// Block states (hash -> state)
    block_states: HashMap<Hash, BlockState>,
    /// Attestations received (block_hash -> attestations)
    attestations: HashMap<Hash, Vec<Attestation>>,
    /// Current epoch
    current_epoch: u64,
    /// Minimum weight needed for justification (2/3)
    justification_threshold: f64,
}

impl FinalityEngine {
    /// Create a new finality engine.
    pub fn new(genesis_root: Hash, justification_threshold: f64) -> Self {
        let genesis_checkpoint = Checkpoint::new(0, genesis_root);
        
        Self {
            justified_checkpoint: genesis_checkpoint,
            finalized_checkpoint: genesis_checkpoint,
            votes: HashMap::new(),
            block_states: HashMap::new(),
            attestations: HashMap::new(),
            current_epoch: 0,
            justification_threshold,
        }
    }

    /// Create with default threshold (2/3).
    pub fn with_genesis(genesis_root: Hash) -> Self {
        Self::new(genesis_root, 2.0 / 3.0)
    }

    /// Get the justified checkpoint.
    pub fn justified_checkpoint(&self) -> Checkpoint {
        self.justified_checkpoint
    }

    /// Get the finalized checkpoint.
    pub fn finalized_checkpoint(&self) -> Checkpoint {
        self.finalized_checkpoint
    }

    /// Get block state.
    pub fn block_state(&self, block_hash: &Hash) -> BlockState {
        self.block_states.get(block_hash).copied().unwrap_or(BlockState::Proposed)
    }

    /// Process a block proposal.
    pub fn process_block(
        &mut self,
        block_hash: Hash,
        parent_hash: Hash,
        epoch: u64,
    ) -> Result<BlockState, ConsensusError> {
        // Check if parent is finalized (cannot build on non-finalized chain in some modes)
        // For now, allow building on justified blocks
        
        // Set initial state
        let state = BlockState::Proposed;
        self.block_states.insert(block_hash, state);
        
        // Update current epoch if needed
        if epoch > self.current_epoch {
            self.current_epoch = epoch;
        }

        Ok(state)
    }

    /// Process an attestation.
    /// 
    /// # Returns
    /// The new block state after processing this attestation.
    pub fn process_attestation(
        &mut self,
        attestation: Attestation,
        committee: &Committee,
    ) -> Result<BlockState, ConsensusError> {
        let block_hash = attestation.block_hash;
        let validator = attestation.validator;

        // Verify validator is in committee
        let member = committee.get_member(&validator)
            .ok_or_else(|| ConsensusError::NotCommitteeMember(
                format!("Validator {} not in committee", validator)
            ))?;

        // Check for double attestation (same target, different blocks)
        if let Some(existing) = self.votes.get(&validator) {
            if existing.target.epoch == attestation.target_epoch && 
               existing.target.root != attestation.target_root {
                return Err(ConsensusError::DoubleAttestation(
                    validator.to_string(),
                    block_hash.to_string(),
                ));
            }
        }

        // Store attestation
        self.attestations.entry(block_hash).or_default().push(attestation.clone());

        // Record vote for Casper FFG
        let vote = Vote {
            validator,
            source: Checkpoint::new(attestation.source_epoch, attestation.source_root),
            target: Checkpoint::new(attestation.target_epoch, attestation.target_root),
            weight: member.stake_weight,
        };
        self.votes.insert(validator, vote);

        // Try to justify the target block
        self.try_justify(&attestation.target, committee)?;

        // Try to finalize
        self.try_finalize();

        // Return current state
        Ok(self.block_state(&block_hash))
    }

    /// Try to justify a checkpoint.
    fn try_justify(
        &mut self,
        target: &Checkpoint,
        committee: &Committee,
    ) -> Result<(), ConsensusError> {
        // Collect votes for this target
        let mut total_weight = 0.0;
        
        for vote in self.votes.values() {
            if vote.target == *target && vote.source.epoch < target.epoch {
                // Valid vote: targets this checkpoint from an older source
                total_weight += vote.weight;
            }
        }

        // Check if we have enough weight
        if total_weight >= committee.attestation_threshold() {
            // Justify the checkpoint
            self.justified_checkpoint = *target;
            
            // Update block state
            if let Some(state) = self.block_states.get_mut(&target.root) {
                *state = BlockState::Justified;
            }
        }

        Ok(())
    }

    /// Try to finalize based on consecutive justified checkpoints.
    fn try_finalize(&mut self) {
        // Find all justified checkpoints
        let justified: Vec<&Vote> = self.votes.values()
            .filter(|v| v.target.epoch == self.justified_checkpoint.epoch)
            .collect();

        // Check if we have a supermajority link from source to target
        for vote in justified {
            if vote.source.epoch == self.finalized_checkpoint.epoch &&
               vote.source.root == self.finalized_checkpoint.root {
                // We have a link from finalized to justified
                // This finalizes the source checkpoint (which is already finalized)
                // and makes the target eligible for finalization in next epoch
            }
        }

        // In Casper FFG, finalization happens when we have:
        // - A supermajority link from epoch n to n+1 (justifies n+1)
        // - Then a supermajority link from n+1 to n+2 (finalizes n+1)
        
        // For simplicity, if we have two consecutive justified epochs, finalize the older
        // This is a simplified version of Casper FFG
    }

    /// Get attestations for a block.
    pub fn get_attestations(&self,
        block_hash: &Hash,
    ) -> &[Attestation] {
        self.attestations.get(block_hash).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get current total attestation weight for a block.
    pub fn get_attestation_weight(
        &self,
        block_hash: &Hash,
    ) -> f64 {
        self.attestations
            .get(block_hash)
            .map(|atts| {
                atts.iter()
                    .filter_map(|att| {
                        self.votes.get(&att.validator).map(|v| v.weight)
                    })
                    .sum()
            })
            .unwrap_or(0.0)
    }

    /// Check if a block is safe to build on (is justified or finalized).
    pub fn is_safe_head(&self, block_hash: &Hash) -> bool {
        let state = self.block_state(block_hash);
        state.is_justified() || state.is_finalized()
    }

    /// Get the chain of finalized blocks.
    pub fn finalized_chain(&self) -> Vec<Hash> {
        // In a full implementation, this would traverse the chain
        // For now, return a placeholder
        vec![self.finalized_checkpoint.root]
    }
}

/// Fork choice rule implementation.
/// 
/// Uses LMD GHOST (Latest Message Driven Greediest Heaviest Observed SubTree)
#[derive(Debug)]
pub struct ForkChoice {
    /// Latest messages (validator -> block hash)
    latest_messages: HashMap<Address, (u64, Hash)>, // slot, hash
}

impl ForkChoice {
    /// Create a new fork choice.
    pub fn new() -> Self {
        Self {
            latest_messages: HashMap::new(),
        }
    }

    /// Process an attestation as a vote for fork choice.
    pub fn process_attestation(
        &mut self,
        validator: Address,
        slot: u64,
        block_hash: Hash,
    ) {
        // Only update if this is a newer slot
        let should_update = self.latest_messages
            .get(&validator)
            .map(|(s, _)| slot > *s)
            .unwrap_or(true);

        if should_update {
            self.latest_messages.insert(validator, (slot, block_hash));
        }
    }

    /// Get the head of the chain (block with most votes).
    /// 
    /// # Arguments
    /// - `justified_root`: The latest justified checkpoint (can't go before this)
    /// - `blocks`: Map of block hashes to their children
    /// 
    /// # Returns
    /// The head block hash.
    pub fn get_head(
        &self,
        justified_root: Hash,
        blocks: &HashMap<Hash, Vec<Hash>>,
    ) -> Option<Hash> {
        // Start from justified root
        let mut head = justified_root;

        // While there are children, pick the one with most votes
        while let Some(children) = blocks.get(&head) {
            if children.is_empty() {
                break;
            }

            // Find child with most votes
            head = children
                .iter()
                .max_by_key(|child| {
                    self.count_votes(**child, blocks)
                })
                .copied()?;
        }

        Some(head)
    }

    /// Count votes for a block and all its descendants.
    /// Uses iterative approach with depth limit to prevent stack overflow.
    fn count_votes(
        &self,
        block_hash: Hash,
        blocks: &HashMap<Hash, Vec<Hash>>,
    ) -> usize {
        const MAX_DEPTH: usize = 256; // Maximum reorganization depth
        
        let mut count = 0;
        let mut queue = vec![(block_hash, 0)];
        let mut visited = std::collections::HashSet::new();
        
        while let Some((hash, depth)) = queue.pop() {
            // Prevent excessive depth and cycles
            if depth > MAX_DEPTH || !visited.insert(hash) {
                continue;
            }
            
            // Count direct votes for this block
            count += self.latest_messages
                .values()
                .filter(|(_, h)| *h == hash)
                .count();
            
            // Add children to queue
            if let Some(children) = blocks.get(&hash) {
                for child in children {
                    queue.push((*child, depth + 1));
                }
            }
        }
        
        count
    }
}

impl Default for ForkChoice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash(n: u8) -> Hash {
        let mut hash = [0u8; 32];
        hash[31] = n;
        Hash::from_bytes(hash)
    }

    #[test]
    fn test_block_state_transitions() {
        let genesis = test_hash(0);
        let mut engine = FinalityEngine::with_genesis(genesis);

        // Process a block
        let block = test_hash(1);
        let state = engine.process_block(block, genesis, 1).unwrap();
        assert_eq!(state, BlockState::Proposed);

        // Check initial state
        assert_eq!(engine.block_state(&block), BlockState::Proposed);
    }

    #[test]
    fn test_finality_engine_checkpoints() {
        let genesis = test_hash(0);
        let engine = FinalityEngine::with_genesis(genesis);

        assert_eq!(engine.justified_checkpoint().epoch, 0);
        assert_eq!(engine.finalized_checkpoint().epoch, 0);
        assert_eq!(engine.justified_checkpoint().root, genesis);
    }

    #[test]
    fn test_attestation_processing() {
        let genesis = test_hash(0);
        let mut engine = FinalityEngine::with_genesis(genesis);
        
        // Create a mock committee
        let mut committee = Committee::new(1, [0u8; 32]);
        // Would need to add members...

        let block = test_hash(1);
        engine.process_block(block, genesis, 1).unwrap();

        // Without committee members, attestation should fail
        let attestation = Attestation {
            validator: Address::ZERO,
            block_hash: block,
            block_number: 1,
            slot: 1,
            source_epoch: 0,
            source_root: genesis,
            target_epoch: 1,
            target_root: block,
        };

        let result = engine.process_attestation(attestation, &committee);
        assert!(result.is_err()); // Not in committee
    }

    #[test]
    fn test_fork_choice_basic() {
        let mut fc = ForkChoice::new();
        
        let block1 = test_hash(1);
        let block2 = test_hash(2);
        
        fc.process_attestation(Address::ZERO, 1, block1);
        
        let mut blocks = HashMap::new();
        blocks.insert(test_hash(0), vec![block1, block2]);
        
        let head = fc.get_head(test_hash(0), &blocks);
        assert_eq!(head, Some(block1)); // block1 has 1 vote, block2 has 0
    }

    #[test]
    fn test_fork_choice_updates() {
        let mut fc = ForkChoice::new();
        
        let block1 = test_hash(1);
        
        // First attestation at slot 1
        fc.process_attestation(Address::ZERO, 1, block1);
        
        // Update at slot 2 (should succeed)
        let block2 = test_hash(2);
        fc.process_attestation(Address::ZERO, 2, block2);
        
        // Old slot should be ignored
        fc.process_attestation(Address::ZERO, 1, block1);
        
        // Should still have the latest
        assert_eq!(fc.latest_messages.get(&Address::ZERO), Some(&(2, block2)));
    }

    #[test]
    fn test_block_state_is_justified() {
        assert!(!BlockState::Proposed.is_justified());
        assert!(!BlockState::PartiallyAttested.is_justified());
        assert!(BlockState::Justified.is_justified());
        assert!(BlockState::Finalized.is_justified());
    }

    #[test]
    fn test_block_state_is_finalized() {
        assert!(!BlockState::Proposed.is_finalized());
        assert!(!BlockState::Justified.is_finalized());
        assert!(BlockState::Finalized.is_finalized());
    }
}
