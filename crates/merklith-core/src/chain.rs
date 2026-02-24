//! Chain state management.

use crate::error::CoreError;
use merklith_types::{Block, BlockHeader, Hash};
use std::collections::HashMap;

/// Chain state tracking and fork choice.
pub struct Chain {
    /// Current chain head
    head: Hash,
    /// Current finalized head
    finalized_head: Option<Hash>,
    /// Block headers by hash
    headers: HashMap<Hash, BlockHeader>,
    /// Block numbers by hash
    numbers: HashMap<Hash, u64>,
    /// Children by parent hash
    children: HashMap<Hash, Vec<Hash>>,
}

impl Chain {
    /// Create a new chain with a genesis block.
    pub fn new(genesis: Block) -> Self {
        let mut chain = Self {
            head: genesis.hash(),
            finalized_head: None,
            headers: HashMap::new(),
            numbers: HashMap::new(),
            children: HashMap::new(),
        };

        chain.insert_block(genesis);
        chain
    }

    /// Get the current head hash.
    pub fn head(&self) -> Hash {
        self.head
    }

    /// Get the head block number.
    pub fn head_number(&self) -> u64 {
        self.numbers.get(&self.head).copied().unwrap_or(0)
    }

    /// Get the finalized head.
    pub fn finalized_head(&self) -> Option<Hash> {
        self.finalized_head
    }

    /// Insert a block into the chain.
    pub fn insert_block(
        &mut self,
        block: Block,
    ) {
        let hash = block.hash();
        let number = block.number();

        self.headers.insert(hash, block.header.clone());
        self.numbers.insert(hash, number);

        // Track parent-child relationship
        self.children
            .entry(block.header.parent_hash)
            .or_default()
            .push(hash);
    }

    /// Update the head (fork choice).
    pub fn set_head(
        &mut self,
        hash: Hash,
    ) -> Result<(), CoreError> {
        if !self.headers.contains_key(&hash) {
            return Err(CoreError::BlockNotFound(
                self.numbers.get(&hash).copied().unwrap_or(0)
            ));
        }

        self.head = hash;
        Ok(())
    }

    /// Finalize a block.
    pub fn finalize_block(
        &mut self,
        hash: Hash,
    ) -> Result<(), CoreError> {
        if !self.headers.contains_key(&hash) {
            return Err(CoreError::BlockNotFound(
                self.numbers.get(&hash).copied().unwrap_or(0)
            ));
        }

        self.finalized_head = Some(hash);
        Ok(())
    }

    /// Get a block header by hash.
    pub fn get_header(
        &self,
        hash: &Hash,
    ) -> Option<&BlockHeader> {
        self.headers.get(hash)
    }

    /// Get block number by hash.
    pub fn get_number(
        &self,
        hash: &Hash,
    ) -> Option<u64> {
        self.numbers.get(hash).copied()
    }

    /// Get children of a block.
    pub fn get_children(
        &self,
        hash: &Hash,
    ) -> &[Hash] {
        self.children.get(hash).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check if a block is an ancestor of another.
    pub fn is_ancestor(
        &self,
        ancestor: &Hash,
        descendant: &Hash,
    ) -> bool {
        let mut current = *descendant;

        while current != *ancestor {
            if let Some(header) = self.headers.get(&current) {
                if header.parent_hash == Hash::ZERO {
                    return false; // Reached genesis
                }
                current = header.parent_hash;
            } else {
                return false; // Block not found
            }
        }

        true
    }

    /// Get the canonical chain (from head to genesis).
    pub fn get_canonical_chain(
        &self,
    ) -> Vec<Hash> {
        let mut chain = Vec::new();
        let mut current = self.head;

        while current != Hash::ZERO {
            chain.push(current);

            if let Some(header) = self.headers.get(&current) {
                current = header.parent_hash;
            } else {
                break;
            }
        }

        chain.reverse();
        chain
    }

    /// Get the distance between two blocks.
    pub fn distance(
        &self,
        from: &Hash,
        to: &Hash,
    ) -> Option<u64> {
        let from_number = self.get_number(from)?;
        let to_number = self.get_number(to)?;

        if from_number > to_number {
            Some(from_number - to_number)
        } else {
            Some(to_number - from_number)
        }
    }

    /// Check if a block is finalized.
    pub fn is_finalized(
        &self,
        hash: &Hash,
    ) -> bool {
        if let Some(finalized) = self.finalized_head {
            if *hash == finalized {
                return true;
            }
            return self.is_ancestor(hash, &finalized);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::{Address, BlockHeader};

    fn create_genesis() -> Block {
        let header = BlockHeader::new(
            Hash::ZERO,
            0,
            0,
            30000000,
            Address::ZERO,
        );
        Block::new(header, vec![])
    }

    fn create_block(parent: &Hash, number: u64) -> Block {
        let header = BlockHeader::new(
            *parent,
            number,
            1000 + number,
            30000000,
            Address::ZERO,
        );
        Block::new(header, vec![])
    }

    #[test]
    fn test_chain_creation() {
        let genesis = create_genesis();
        let chain = Chain::new(genesis.clone());

        assert_eq!(chain.head(), genesis.hash());
        assert_eq!(chain.head_number(), 0);
    }

    #[test]
    fn test_insert_block() {
        let genesis = create_genesis();
        let mut chain = Chain::new(genesis);

        let block1 = create_block(&chain.head(), 1);
        chain.insert_block(block1.clone());

        assert_eq!(chain.get_number(&block1.hash()), Some(1));
    }

    #[test]
    fn test_set_head() {
        let genesis = create_genesis();
        let mut chain = Chain::new(genesis);

        let block1 = create_block(&chain.head(), 1);
        chain.insert_block(block1.clone());

        chain.set_head(block1.hash()).unwrap();
        assert_eq!(chain.head(), block1.hash());
    }

    #[test]
    fn test_is_ancestor() {
        let genesis = create_genesis();
        let mut chain = Chain::new(genesis.clone());

        let block1 = create_block(&genesis.hash(), 1);
        let block2 = create_block(&block1.hash(), 2);

        chain.insert_block(block1.clone());
        chain.insert_block(block2.clone());

        assert!(chain.is_ancestor(&genesis.hash(), &block2.hash()));
        assert!(chain.is_ancestor(&block1.hash(), &block2.hash()));
        assert!(!chain.is_ancestor(&block2.hash(), &block1.hash()));
    }

    #[test]
    fn test_canonical_chain() {
        let genesis = create_genesis();
        let mut chain = Chain::new(genesis.clone());

        let block1 = create_block(&genesis.hash(), 1);
        let block2 = create_block(&block1.hash(), 2);

        chain.insert_block(block1.clone());
        chain.insert_block(block2.clone());
        chain.set_head(block2.hash()).unwrap();

        let canonical = chain.get_canonical_chain();
        assert_eq!(canonical.len(), 3);
        assert_eq!(canonical[0], genesis.hash());
        assert_eq!(canonical[1], block1.hash());
        assert_eq!(canonical[2], block2.hash());
    }
}
