//! Trie implementation with database backend.

use crate::db::{ColumnFamily, Database};
use crate::error::StorageError;
use crate::trie::{Nibbles, TrieNode};
use merklith_types::Hash;
use std::collections::HashMap;
use std::sync::Arc;

/// Merkle Patricia Trie for state storage.
#[derive(Clone)]
pub struct Trie {
    /// Current root hash
    root: Hash,
    /// Database for persistent storage
    db: Arc<Database>,
    /// Cache of dirty nodes (modified but not committed)
    dirty_nodes: HashMap<Hash, TrieNode>,
}

impl Trie {
    /// Create a new empty trie.
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            root: Hash::ZERO,
            db,
            dirty_nodes: HashMap::new(),
        }
    }

    /// Create a trie from an existing root.
    pub fn from_root(db: Arc<Database>, root: Hash) -> Self {
        Self {
            root,
            db,
            dirty_nodes: HashMap::new(),
        }
    }

    /// Get the current root hash.
    pub fn root(&self) -> Hash {
        self.root
    }

    /// Get a value by key.
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        if self.root.is_zero() {
            return Ok(None);
        }

        let nibbles = Nibbles::from_bytes(key);
        self.get_recursive(&self.root, &nibbles)
    }

    fn get_recursive(
        &self,
        node_hash: &Hash,
        remaining: &Nibbles,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        if remaining.is_empty() {
            // This shouldn't happen in normal traversal
            return Ok(None);
        }

        let node = self.get_node(node_hash)?;

        match node {
            TrieNode::Empty => Ok(None),
            TrieNode::Leaf { key_end, value } => {
                if &key_end == remaining {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
            TrieNode::Extension { prefix, child } => {
                if remaining.0.starts_with(&prefix.0) {
                    let new_remaining = remaining.skip(prefix.len());
                    self.get_recursive(&child, &new_remaining)
                } else {
                    Ok(None)
                }
            }
            TrieNode::Branch { children, value } => {
                if remaining.is_empty() {
                    Ok(value)
                } else {
                    let nibble = remaining.first().unwrap() as usize;
                    if let Some(child_hash) = &children[nibble] {
                        let new_remaining = remaining.skip(1);
                        self.get_recursive(child_hash, &new_remaining)
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }

    /// Insert or update a value at the given key.
    pub fn insert(&mut self, key: &[u8], value: Vec<u8>) -> Result<Hash, StorageError> {
        let nibbles = Nibbles::from_bytes(key);
        let (new_root, _) = self.insert_recursive(self.root, &nibbles, value)?;
        self.root = new_root;
        Ok(self.root)
    }

    fn insert_recursive(
        &mut self,
        node_hash: Hash,
        remaining: &Nibbles,
        value: Vec<u8>,
    ) -> Result<(Hash, bool), StorageError> {
        if node_hash.is_zero() {
            // Create new leaf node
            let node = TrieNode::Leaf {
                key_end: remaining.clone(),
                value,
            };
            let hash = node.hash();
            self.dirty_nodes.insert(hash, node);
            return Ok((hash, true));
        }

        let node = self.get_node(&node_hash)?;

        match node {
            TrieNode::Empty => {
                let new_node = TrieNode::Leaf {
                    key_end: remaining.clone(),
                    value,
                };
                let hash = new_node.hash();
                self.dirty_nodes.insert(hash, new_node);
                Ok((hash, true))
            }
            TrieNode::Leaf { key_end, value: old_value } => {
                if key_end == *remaining {
                    // Update existing leaf
                    let new_node = TrieNode::Leaf {
                        key_end: remaining.clone(),
                        value,
                    };
                    let hash = new_node.hash();
                    self.dirty_nodes.insert(hash, new_node);
                    Ok((hash, true))
                } else {
                    // Split into branch
                    self.split_leaf(node_hash, &key_end, &old_value, remaining, value)
                }
            }
            TrieNode::Extension { prefix, child } => {
                if remaining.0.starts_with(&prefix.0) {
                    // Continue down the extension
                    let new_remaining = remaining.skip(prefix.len());
                    let (new_child, modified) = self.insert_recursive(child, &new_remaining, value)?;
                    if modified {
                        let new_node = TrieNode::Extension {
                            prefix: prefix.clone(),
                            child: new_child,
                        };
                        let hash = new_node.hash();
                        self.dirty_nodes.insert(hash, new_node);
                        Ok((hash, true))
                    } else {
                        Ok((node_hash, false))
                    }
                } else {
                    // Split extension
                    self.split_extension(node_hash, &prefix, &child, remaining, value)
                }
            }
            TrieNode::Branch { children, value: branch_value } => {
                if remaining.is_empty() {
                    // Update value at this branch
                    let new_node = TrieNode::Branch {
                        children: children.clone(),
                        value: Some(value),
                    };
                    let hash = new_node.hash();
                    self.dirty_nodes.insert(hash, new_node);
                    Ok((hash, true))
                } else {
                    // Navigate to appropriate child
                    let nibble = remaining.first().unwrap() as usize;
                    let child_hash = children[nibble].unwrap_or(Hash::ZERO);
                    let new_remaining = remaining.skip(1);
                    
                    let (new_child, modified) = self.insert_recursive(child_hash, &new_remaining, value)?;
                    
                    if modified {
                        let mut new_children = children;
                        new_children[nibble] = Some(new_child);
                        
                        let new_node = TrieNode::Branch {
                            children: new_children,
                            value: branch_value,
                        };
                        let hash = new_node.hash();
                        self.dirty_nodes.insert(hash, new_node);
                        Ok((hash, true))
                    } else {
                        Ok((node_hash, false))
                    }
                }
            }
        }
    }

    fn split_leaf(
        &mut self,
        _old_hash: Hash,
        old_key: &Nibbles,
        old_value: &Vec<u8>,
        new_key: &Nibbles,
        new_value: Vec<u8>,
    ) -> Result<(Hash, bool), StorageError> {
        let common = old_key.common_prefix(new_key);
        
        if common == 0 {
            // Create branch at root
            let old_nibble = old_key.first().unwrap() as usize;
            let new_nibble = new_key.first().unwrap() as usize;
            
            let mut children: [Option<Hash>; 16] = Default::default();
            
            // Old leaf
            if old_key.len() == 1 {
                let leaf = TrieNode::Leaf {
                    key_end: Nibbles(vec![]),
                    value: old_value.clone(),
                };
                let hash = leaf.hash();
                self.dirty_nodes.insert(hash, leaf);
                children[old_nibble] = Some(hash);
            } else {
                let leaf = TrieNode::Leaf {
                    key_end: old_key.skip(1),
                    value: old_value.clone(),
                };
                let hash = leaf.hash();
                self.dirty_nodes.insert(hash, leaf);
                children[old_nibble] = Some(hash);
            }
            
            // New leaf
            if new_key.len() == 1 {
                let leaf = TrieNode::Leaf {
                    key_end: Nibbles(vec![]),
                    value: new_value,
                };
                let hash = leaf.hash();
                self.dirty_nodes.insert(hash, leaf);
                children[new_nibble] = Some(hash);
            } else {
                let leaf = TrieNode::Leaf {
                    key_end: new_key.skip(1),
                    value: new_value,
                };
                let hash = leaf.hash();
                self.dirty_nodes.insert(hash, leaf);
                children[new_nibble] = Some(hash);
            }
            
            let branch = TrieNode::Branch {
                children,
                value: None,
            };
            let hash = branch.hash();
            self.dirty_nodes.insert(hash, branch);
            Ok((hash, true))
        } else {
            // Create extension + branch
            let prefix = old_key.slice(0, common);
            let old_suffix = old_key.skip(common);
            let new_suffix = new_key.skip(common);
            
            let (branch_hash, _) = self.split_leaf(
                Hash::ZERO,
                &old_suffix,
                old_value,
                &new_suffix,
                new_value,
            )?;
            
            let ext = TrieNode::Extension {
                prefix,
                child: branch_hash,
            };
            let hash = ext.hash();
            self.dirty_nodes.insert(hash, ext);
            Ok((hash, true))
        }
    }

    fn split_extension(
        &mut self,
        _old_hash: Hash,
        prefix: &Nibbles,
        child: &Hash,
        new_key: &Nibbles,
        value: Vec<u8>,
    ) -> Result<(Hash, bool), StorageError> {
        let common = prefix.common_prefix(new_key);
        
        if common == 0 {
            // Create branch
            let prefix_nibble = prefix.first().unwrap() as usize;
            let key_nibble = new_key.first().unwrap() as usize;
            
            let mut children: [Option<Hash>; 16] = Default::default();
            
            // Old extension becomes child at prefix_nibble
            if prefix.len() == 1 {
                children[prefix_nibble] = Some(*child);
            } else {
                let ext = TrieNode::Extension {
                    prefix: prefix.skip(1),
                    child: *child,
                };
                let hash = ext.hash();
                self.dirty_nodes.insert(hash, ext);
                children[prefix_nibble] = Some(hash);
            }
            
            // New leaf
            if new_key.len() == 1 {
                let leaf = TrieNode::Leaf {
                    key_end: Nibbles(vec![]),
                    value,
                };
                let hash = leaf.hash();
                self.dirty_nodes.insert(hash, leaf);
                children[key_nibble] = Some(hash);
            } else {
                let leaf = TrieNode::Leaf {
                    key_end: new_key.skip(1),
                    value,
                };
                let hash = leaf.hash();
                self.dirty_nodes.insert(hash, leaf);
                children[key_nibble] = Some(hash);
            }
            
            let branch = TrieNode::Branch {
                children,
                value: None,
            };
            let hash = branch.hash();
            self.dirty_nodes.insert(hash, branch);
            Ok((hash, true))
        } else {
            // Create nested extensions
            let new_prefix = prefix.slice(0, common);
            let old_suffix = prefix.skip(common);
            let new_suffix = new_key.skip(common);
            
            // Create branch for the split
            let (branch_hash, _) = self.split_extension(
                Hash::ZERO,
                &old_suffix,
                child,
                &new_suffix,
                value,
            )?;
            
            let ext = TrieNode::Extension {
                prefix: new_prefix,
                child: branch_hash,
            };
            let hash = ext.hash();
            self.dirty_nodes.insert(hash, ext);
            Ok((hash, true))
        }
    }

    /// Delete a key from the trie.
    pub fn delete(&mut self,
        key: &[u8],
    ) -> Result<Hash, StorageError> {
        let nibbles = Nibbles::from_bytes(key);
        let (new_root, _) = self.delete_recursive(self.root, &nibbles)?;
        self.root = new_root;
        Ok(self.root)
    }

    fn delete_recursive(
        &mut self,
        node_hash: Hash,
        remaining: &Nibbles,
    ) -> Result<(Hash, bool), StorageError> {
        if node_hash.is_zero() {
            return Ok((Hash::ZERO, false));
        }

        let node = self.get_node(&node_hash)?;

        match node {
            TrieNode::Empty => Ok((Hash::ZERO, false)),
            TrieNode::Leaf { key_end, .. } => {
                if key_end == *remaining {
                    Ok((Hash::ZERO, true))
                } else {
                    Ok((node_hash, false))
                }
            }
            TrieNode::Extension { prefix, child } => {
                if remaining.0.starts_with(&prefix.0) {
                    let new_remaining = remaining.skip(prefix.len());
                    let (new_child, modified) = self.delete_recursive(child, &new_remaining)?;
                    if modified {
                        if new_child.is_zero() {
                            Ok((Hash::ZERO, true))
                        } else {
                            let new_node = TrieNode::Extension {
                                prefix: prefix.clone(),
                                child: new_child,
                            };
                            let hash = new_node.hash();
                            self.dirty_nodes.insert(hash, new_node);
                            Ok((hash, true))
                        }
                    } else {
                        Ok((node_hash, false))
                    }
                } else {
                    Ok((node_hash, false))
                }
            }
            TrieNode::Branch { children, value } => {
                if remaining.is_empty() {
                    // Delete value at this branch
                    let new_node = TrieNode::Branch {
                        children: children.clone(),
                        value: None,
                    };
                    let hash = new_node.hash();
                    self.dirty_nodes.insert(hash, new_node);
                    Ok((hash, true))
                } else {
                    let nibble = remaining.first().unwrap() as usize;
                    if let Some(child_hash) = children[nibble] {
                        let new_remaining = remaining.skip(1);
                        let (new_child, modified) = self.delete_recursive(child_hash, &new_remaining)?;
                        
                        if modified {
                            let mut new_children = children.clone();
                            if new_child.is_zero() {
                                new_children[nibble] = None;
                            } else {
                                new_children[nibble] = Some(new_child);
                            }
                            
                            let new_node = TrieNode::Branch {
                                children: new_children,
                                value: value.clone(),
                            };
                            let hash = new_node.hash();
                            self.dirty_nodes.insert(hash, new_node);
                            Ok((hash, true))
                        } else {
                            Ok((node_hash, false))
                        }
                    } else {
                        Ok((node_hash, false))
                    }
                }
            }
        }
    }

    /// Get a node from cache or database.
    fn get_node(&self,
        hash: &Hash,
    ) -> Result<TrieNode, StorageError> {
        if let Some(node) = self.dirty_nodes.get(hash) {
            return Ok(node.clone());
        }

        if let Some(bytes) = self.db.get(ColumnFamily::StateTrie, hash.as_bytes())? {
            TrieNode::decode(&bytes)
        } else {
            Ok(TrieNode::Empty)
        }
    }

    /// Commit all dirty nodes to the database.
    pub fn commit(&mut self) -> Result<Hash, StorageError> {
        let mut batch = self.db.new_write_batch();
        
        for (hash, node) in &self.dirty_nodes {
            batch.put(
                ColumnFamily::StateTrie,
                hash.as_bytes(),
                &node.encode(),
            )?;
        }
        
        self.db.batch_write(batch)?;
        self.dirty_nodes.clear();
        
        Ok(self.root)
    }

    /// Revert all uncommitted changes.
    pub fn revert(&mut self) {
        self.dirty_nodes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseConfig;
    use tempfile::TempDir;

    fn create_test_trie() -> (Trie, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = DatabaseConfig::default();
        let db = Arc::new(Database::open(temp_dir.path(), &config).unwrap());
        let trie = Trie::new(db);
        (trie, temp_dir)
    }

    #[test]
    fn test_trie_insert_and_get() {
        let (mut trie, _temp) = create_test_trie();
        
        trie.insert(b"key1", vec![1, 2, 3]).unwrap();
        trie.insert(b"key2", vec![4, 5, 6]).unwrap();
        
        assert_eq!(trie.get(b"key1").unwrap(), Some(vec![1, 2, 3]));
        assert_eq!(trie.get(b"key2").unwrap(), Some(vec![4, 5, 6]));
        assert_eq!(trie.get(b"key3").unwrap(), None);
    }

    #[test]
    fn test_trie_update() {
        let (mut trie, _temp) = create_test_trie();
        
        trie.insert(b"key1", vec![1, 2, 3]).unwrap();
        trie.insert(b"key1", vec![7, 8, 9]).unwrap();
        
        assert_eq!(trie.get(b"key1").unwrap(), Some(vec![7, 8, 9]));
    }

    #[test]
    fn test_trie_delete() {
        let (mut trie, _temp) = create_test_trie();
        
        trie.insert(b"key1", vec![1, 2, 3]).unwrap();
        trie.insert(b"key2", vec![4, 5, 6]).unwrap();
        
        trie.delete(b"key1").unwrap();
        
        assert_eq!(trie.get(b"key1").unwrap(), None);
        assert_eq!(trie.get(b"key2").unwrap(), Some(vec![4, 5, 6]));
    }

    #[test]
    fn test_trie_commit() {
        let (mut trie, _temp) = create_test_trie();
        
        trie.insert(b"key1", vec![1, 2, 3]).unwrap();
        let root1 = trie.commit().unwrap();
        
        trie.insert(b"key2", vec![4, 5, 6]).unwrap();
        let root2 = trie.commit().unwrap();
        
        assert_ne!(root1, root2);
        
        // Revert and check root is back
        let trie2 = Trie::from_root(trie.db.clone(), root1);
        assert_eq!(trie2.get(b"key1").unwrap(), Some(vec![1, 2, 3]));
        assert_eq!(trie2.get(b"key2").unwrap(), None);
    }

    #[test]
    fn test_trie_revert() {
        let (mut trie, _temp) = create_test_trie();
        
        trie.insert(b"key1", vec![1, 2, 3]).unwrap();
        let root_before = trie.root;
        
        trie.insert(b"key2", vec![4, 5, 6]).unwrap();
        trie.revert();
        
        assert_eq!(trie.root, root_before);
        assert_eq!(trie.get(b"key2").unwrap(), None);
    }
}
