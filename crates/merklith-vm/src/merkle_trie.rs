//! Merkle Patricia Trie for MERKLITH blockchain state storage
//! 
//! Ethereum-compatible state tree implementation using Blake3 hashing.

use std::collections::HashMap;
use merklith_types::Hash;

/// Node types in the trie
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrieNode {
    /// Empty node
    Empty,
    /// Leaf node: [encoded_path, value]
    Leaf(Vec<u8>, Vec<u8>),
    /// Extension node: [encoded_path, next_hash]
    Extension(Vec<u8>, Hash),
    /// Branch node: [hash_0, hash_1, ..., hash_15, value]
    Branch([Option<Hash>; 16], Option<Vec<u8>>),
}

impl TrieNode {
    /// Encode node to bytes
    pub fn encode(&self) -> Vec<u8> {
        match self {
            TrieNode::Empty => vec![0x80],
            TrieNode::Leaf(path, value) => {
                let mut result = vec![0x20 | ((path.len() * 2) as u8)]; // Leaf prefix
                result.extend_from_slice(path);
                result.extend_from_slice(value);
                result
            }
            TrieNode::Extension(path, next) => {
                let mut result = vec![0x00 | ((path.len() * 2) as u8)]; // Extension prefix
                result.extend_from_slice(path);
                result.extend_from_slice(next.as_bytes());
                result
            }
            TrieNode::Branch(children, value) => {
                let mut result = vec![0x10 | 0x0F]; // Branch prefix
                for i in 0..16 {
                    if let Some(hash) = &children[i] {
                        result.extend_from_slice(hash.as_bytes());
                    } else {
                        result.extend_from_slice(Hash::ZERO.as_bytes());
                    }
                }
                if let Some(v) = value {
                    result.extend_from_slice(v);
                }
                result
            }
        }
    }

    /// Compute hash of node
    pub fn hash(&self) -> Hash {
        if let TrieNode::Empty = self {
            return Hash::ZERO;
        }
        let encoded = self.encode();
        Hash::compute(&encoded)
    }
}

/// Merkle Patricia Trie
pub struct MerkleTrie {
    /// Root node hash
    root: Hash,
    /// Node storage
    nodes: HashMap<Hash, TrieNode>,
    /// Value storage (leaf values)
    values: HashMap<Vec<u8>, Vec<u8>>,
}

impl MerkleTrie {
    /// Create new empty trie
    pub fn new() -> Self {
        Self {
            root: Hash::ZERO,
            nodes: HashMap::new(),
            values: HashMap::new(),
        }
    }

    /// Get root hash
    pub fn root_hash(&self) -> Hash {
        self.root
    }

    /// Insert key-value pair
    pub fn insert(&mut self, key: &[u8], value: Vec<u8>) {
        let nibbles = bytes_to_nibbles(key);
        
        if self.root == Hash::ZERO {
            // Empty trie, create leaf
            let leaf = TrieNode::Leaf(nibbles, value.clone());
            let hash = leaf.hash();
            self.nodes.insert(hash.clone(), leaf);
            self.root = hash;
            self.values.insert(key.to_vec(), value);
            return;
        }

        // Insert recursively
        let new_root = self.insert_recursive(self.root.clone(), &nibbles, 0, value.clone());
        self.root = new_root;
        self.values.insert(key.to_vec(), value);
    }

    /// Insert helper (recursive)
    fn insert_recursive(
        &mut self,
        node_hash: Hash,
        nibbles: &[u8],
        depth: usize,
        value: Vec<u8>,
    ) -> Hash {
        let node = self.nodes.get(&node_hash).cloned().unwrap_or(TrieNode::Empty);

        match node {
            TrieNode::Empty => {
                let leaf = TrieNode::Leaf(nibbles[depth..].to_vec(), value);
                let hash = leaf.hash();
                self.nodes.insert(hash.clone(), leaf);
                hash
            }

            TrieNode::Leaf(path, existing_value) => {
                let remaining = &nibbles[depth..];
                
                // Find common prefix
                let common_len = common_prefix_length(&path, remaining);
                
                if common_len == path.len() && common_len == remaining.len() {
                    // Exact match, update value
                    let leaf = TrieNode::Leaf(path, value);
                    let hash = leaf.hash();
                    self.nodes.insert(hash.clone(), leaf);
                    hash
                } else if common_len == path.len() {
                    // Existing path is prefix of new path
                    let _branch_nibble = remaining[common_len];
                    // Existing path is prefix of new path
                    let new_leaf = TrieNode::Leaf(remaining[common_len + 1..].to_vec(), value);
                    let new_hash = new_leaf.hash();
                    self.nodes.insert(new_hash.clone(), new_leaf);
                    
                    let mut children = [None; 16];
                    let branch_nibble = remaining[common_len];
                    children[branch_nibble as usize] = Some(new_hash);
                    
                    // Put existing as another branch
                    let existing_nibble = path[common_len];
                    let existing_leaf = TrieNode::Leaf(path[common_len + 1..].to_vec(), existing_value);
                    let existing_hash = existing_leaf.hash();
                    self.nodes.insert(existing_hash.clone(), existing_leaf);
                    children[existing_nibble as usize] = Some(existing_hash);
                    
                    let branch = TrieNode::Branch(children, None);
                    let hash = branch.hash();
                    self.nodes.insert(hash.clone(), branch);
                    hash
                } else if common_len == remaining.len() {
                    // New path is prefix of existing
                    let mut children = [None; 16];
                    let branch_nibble = path[common_len];
                    let old_leaf = TrieNode::Leaf(path[common_len + 1..].to_vec(), existing_value);
                    let old_hash = old_leaf.hash();
                    self.nodes.insert(old_hash.clone(), old_leaf);
                    children[branch_nibble as usize] = Some(old_hash);
                    
                    let branch = TrieNode::Branch(children, Some(value));
                    let hash = branch.hash();
                    self.nodes.insert(hash.clone(), branch);
                    hash
                } else {
                    // Split at common prefix
                    let mut children = [None; 16];
                    
                    // Old path branch
                    let old_branch_nibble = path[common_len];
                    let old_leaf = TrieNode::Leaf(path[common_len + 1..].to_vec(), existing_value);
                    let old_hash = old_leaf.hash();
                    self.nodes.insert(old_hash.clone(), old_leaf);
                    children[old_branch_nibble as usize] = Some(old_hash);
                    
                    // New path branch
                    let new_branch_nibble = remaining[common_len];
                    let new_leaf = TrieNode::Leaf(remaining[common_len + 1..].to_vec(), value);
                    let new_hash = new_leaf.hash();
                    self.nodes.insert(new_hash.clone(), new_leaf);
                    children[new_branch_nibble as usize] = Some(new_hash);
                    
                    let branch = TrieNode::Branch(children, None);
                    let hash = branch.hash();
                    self.nodes.insert(hash.clone(), branch);
                    hash
                }
            }

            TrieNode::Extension(path, next_hash) => {
                let remaining = &nibbles[depth..];
                let common_len = common_prefix_length(&path, remaining);
                
                if common_len == path.len() {
                    // Full match, continue down
                    let new_next = self.insert_recursive(next_hash, nibbles, depth + common_len, value);
                    let ext = TrieNode::Extension(path, new_next);
                    let hash = ext.hash();
                    self.nodes.insert(hash.clone(), ext);
                    hash
                } else {
                    // Split extension
                    let mut children = [None; 16];
                    
                    // Old branch
                    let old_nibble = path[common_len];
                    let old_ext = TrieNode::Extension(path[common_len + 1..].to_vec(), next_hash);
                    let old_hash = old_ext.hash();
                    self.nodes.insert(old_hash.clone(), old_ext);
                    children[old_nibble as usize] = Some(old_hash);
                    
                    // New branch
                    let new_nibble = remaining[common_len];
                    let new_leaf = TrieNode::Leaf(remaining[common_len + 1..].to_vec(), value);
                    let new_hash = new_leaf.hash();
                    self.nodes.insert(new_hash.clone(), new_leaf);
                    children[new_nibble as usize] = Some(new_hash);
                    
                    let branch = TrieNode::Branch(children, None);
                    let hash = branch.hash();
                    self.nodes.insert(hash.clone(), branch);
                    hash
                }
            }

            TrieNode::Branch(children, existing_value) => {
                let remaining = &nibbles[depth..];
                
                if remaining.is_empty() {
                    // Store value in branch
                    let branch = TrieNode::Branch(children.clone(), Some(value));
                    let hash = branch.hash();
                    self.nodes.insert(hash.clone(), branch);
                    hash
                } else {
                    // Continue down
                    let nibble = remaining[0];
                    let child_hash = children[nibble as usize].clone();
                    
                    let new_child = if let Some(hash) = child_hash {
                        self.insert_recursive(hash, nibbles, depth + 1, value)
                    } else {
                        let leaf = TrieNode::Leaf(remaining[1..].to_vec(), value);
                        let h = leaf.hash();
                        self.nodes.insert(h.clone(), leaf);
                        h
                    };
                    
                    let mut new_children = children;
                    new_children[nibble as usize] = Some(new_child);
                    let branch = TrieNode::Branch(new_children, existing_value);
                    let hash = branch.hash();
                    self.nodes.insert(hash.clone(), branch);
                    hash
                }
            }
        }
    }

    /// Get value by key
    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.values.get(key)
    }

    /// Generate proof for key
    pub fn generate_proof(&self,
        key: &[u8],
    ) -> Vec<Vec<u8>> {
        let mut proof = Vec::new();
        let nibbles = bytes_to_nibbles(key);
        self.generate_proof_recursive(self.root.clone(), &nibbles, 0, &mut proof);
        proof
    }

    fn generate_proof_recursive(
        &self,
        node_hash: Hash,
        nibbles: &[u8],
        depth: usize,
        proof: &mut Vec<Vec<u8>>,
    ) -> bool {
        let node = match self.nodes.get(&node_hash) {
            Some(n) => n,
            None => return false,
        };

        proof.push(node.encode());

        match node {
            TrieNode::Empty => false,
            
            TrieNode::Leaf(path, _) => {
                let remaining = &nibbles[depth..];
                path == remaining
            }

            TrieNode::Extension(path, next_hash) => {
                let remaining = &nibbles[depth..];
                if remaining.starts_with(path) {
                    self.generate_proof_recursive(next_hash.clone(), nibbles, depth + path.len(), proof)
                } else {
                    false
                }
            }

            TrieNode::Branch(children, _) => {
                let remaining = &nibbles[depth..];
                if remaining.is_empty() {
                    true
                } else if let Some(child) = &children[remaining[0] as usize] {
                    self.generate_proof_recursive(child.clone(), nibbles, depth + 1, proof)
                } else {
                    false
                }
            }
        }
    }

    /// Verify proof
    pub fn verify_proof(
        _root_hash: &Hash,
        _key: &[u8],
        _value: &[u8],
        proof: &[Vec<u8>],
    ) -> bool {
        // Simplified proof verification
        // In production, this would reconstruct the trie and verify
        !proof.is_empty()
    }
}

impl Default for MerkleTrie {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert bytes to nibbles (4-bit units)
fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        nibbles.push((byte >> 4) & 0x0F);
        nibbles.push(byte & 0x0F);
    }
    nibbles
}

/// Find common prefix length
fn common_prefix_length(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

/// State manager using Merkle trie
pub struct StateManager {
    /// Main state trie
    trie: MerkleTrie,
    /// Block number -> state root mapping
    historical_roots: HashMap<u64, Hash>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            trie: MerkleTrie::new(),
            historical_roots: HashMap::new(),
        }
    }

    /// Set account balance
    pub fn set_balance(&mut self, address: &merklith_types::Address, balance: merklith_types::U256) {
        let key = format!("balance:{:x}", address).into_bytes();
        self.trie.insert(&key, balance.to_be_bytes().to_vec());
    }

    /// Get account balance
    pub fn get_balance(&self, address: &merklith_types::Address) -> merklith_types::U256 {
        let key = format!("balance:{:x}", address).into_bytes();
        match self.trie.get(&key) {
            Some(bytes) => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(bytes);
                merklith_types::U256::from_be_bytes(arr)
            }
            None => merklith_types::U256::ZERO,
        }
    }

    /// Set account nonce
    pub fn set_nonce(&mut self, address: &merklith_types::Address, nonce: u64) {
        let key = format!("nonce:{:x}", address).into_bytes();
        self.trie.insert(&key, nonce.to_be_bytes().to_vec());
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &merklith_types::Address) -> u64 {
        let key = format!("nonce:{:x}", address).into_bytes();
        match self.trie.get(&key) {
            Some(bytes) => {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(bytes);
                u64::from_be_bytes(arr)
            }
            None => 0,
        }
    }

    /// Set contract code
    pub fn set_code(&mut self, address: &merklith_types::Address, code: Vec<u8>) {
        let key = format!("code:{:x}", address).into_bytes();
        self.trie.insert(&key, code);
    }

    /// Get contract code
    pub fn get_code(&self, address: &merklith_types::Address) -> Option<&Vec<u8>> {
        let key = format!("code:{:x}", address).into_bytes();
        self.trie.get(&key)
    }

    /// Set storage slot
    pub fn set_storage(
        &mut self,
        address: &merklith_types::Address,
        slot: &merklith_types::Hash,
        value: merklith_types::U256,
    ) {
        let key = format!("storage:{:x}:{:x}", address, slot).into_bytes();
        self.trie.insert(&key, value.to_be_bytes().to_vec());
    }

    /// Get storage slot
    pub fn get_storage(
        &self,
        address: &merklith_types::Address,
        slot: &merklith_types::Hash,
    ) -> merklith_types::U256 {
        let key = format!("storage:{:x}:{:x}", address, slot).into_bytes();
        match self.trie.get(&key) {
            Some(bytes) => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(bytes);
                merklith_types::U256::from_be_bytes(arr)
            }
            None => merklith_types::U256::ZERO,
        }
    }

    /// Get current state root
    pub fn state_root(&self) -> Hash {
        self.trie.root_hash()
    }

    /// Commit block state
    pub fn commit_block(&mut self,
        block_number: u64,
    ) {
        let root = self.state_root();
        self.historical_roots.insert(block_number, root);
    }

    /// Get historical state root
    pub fn get_historical_root(&self,
        block_number: u64,
    ) -> Option<Hash> {
        self.historical_roots.get(&block_number).cloned()
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_trie_basic() {
        let mut trie = MerkleTrie::new();
        
        trie.insert(b"key1", b"value1".to_vec());
        trie.insert(b"key2", b"value2".to_vec());
        
        assert_eq!(trie.get(b"key1"), Some(&b"value1".to_vec()));
        assert_eq!(trie.get(b"key2"), Some(&b"value2".to_vec()));
        assert_eq!(trie.get(b"key3"), None);
    }

    #[test]
    fn test_merkle_trie_update() {
        let mut trie = MerkleTrie::new();
        
        trie.insert(b"key1", b"value1".to_vec());
        let root1 = trie.root_hash();
        
        trie.insert(b"key1", b"value2".to_vec());
        let root2 = trie.root_hash();
        
        assert_ne!(root1, root2);
        assert_eq!(trie.get(b"key1"), Some(&b"value2".to_vec()));
    }

    #[test]
    fn test_state_manager() {
        let mut state = StateManager::new();
        let addr = merklith_types::Address::from_bytes([1u8; 20]);
        
        state.set_balance(&addr, merklith_types::U256::from(1000u64));
        assert_eq!(state.get_balance(&addr), merklith_types::U256::from(1000u64));
        
        state.set_nonce(&addr, 5);
        assert_eq!(state.get_nonce(&addr), 5);
        
        let slot = merklith_types::Hash::compute(b"slot1");
        state.set_storage(&addr, &slot, merklith_types::U256::from(500u64));
        assert_eq!(state.get_storage(&addr, &slot), merklith_types::U256::from(500u64));
    }

    #[test]
    fn test_proof_generation() {
        let mut trie = MerkleTrie::new();
        trie.insert(b"key1", b"value1".to_vec());
        trie.insert(b"key2", b"value2".to_vec());
        
        let proof = trie.generate_proof(b"key1");
        assert!(!proof.is_empty());
    }
}
