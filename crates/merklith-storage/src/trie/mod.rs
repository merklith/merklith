//! Merkle Patricia Trie implementation.
//!
//! This module provides a modified Merkle Patricia Trie using blake3 hashing.
//! It is the core data structure for Merklith's state storage.

use crate::error::StorageError;
use merklith_types::Hash;
use merklith_crypto::hash::hash_pair;
use std::collections::HashMap;

pub mod trie;

/// Nibble-based key path for trie traversal (4 bits per nibble).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nibbles(Vec<u8>);

impl Nibbles {
    /// Create nibbles from bytes (each byte becomes 2 nibbles).
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut nibbles = Vec::with_capacity(bytes.len() * 2);
        for byte in bytes {
            nibbles.push(byte >> 4);       // High nibble
            nibbles.push(byte & 0x0F);    // Low nibble
        }
        Self(nibbles)
    }

    /// Convert nibbles back to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((self.0.len() + 1) / 2);
        for chunk in self.0.chunks(2) {
            if chunk.len() == 2 {
                bytes.push((chunk[0] << 4) | chunk[1]);
            } else {
                bytes.push(chunk[0] << 4);
            }
        }
        bytes
    }

    /// Get the length in nibbles.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a slice of nibbles.
    pub fn slice(&self, start: usize, end: usize) -> Self {
        Self(self.0[start..end.min(self.0.len())].to_vec())
    }

    /// Get first nibble.
    pub fn first(&self) -> Option<u8> {
        self.0.first().copied()
    }

    /// Skip first n nibble.
    pub fn skip(&self, n: usize) -> Self {
        if n >= self.0.len() {
            Self(vec![])
        } else {
            Self(self.0[n..].to_vec())
        }
    }

    /// Common prefix length with another nibbles.
    pub fn common_prefix(&self, other: &Nibbles) -> usize {
        self.0.iter()
            .zip(other.0.iter())
            .take_while(|(a, b)| a == b)
            .count()
    }
}

/// Trie node types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrieNode {
    /// Empty node
    Empty,
    /// Leaf node: remaining path + value
    Leaf { key_end: Nibbles, value: Vec<u8> },
    /// Extension node: shared prefix + child hash
    Extension { prefix: Nibbles, child: Hash },
    /// Branch node: 16 children (one per nibble) + optional value
    Branch { children: [Option<Hash>; 16], value: Option<Vec<u8>> },
}

impl TrieNode {
    /// Compute the hash of this node.
    pub fn hash(&self) -> Hash {
        let encoded = self.encode();
        Hash::compute(&encoded)
    }

    /// Encode the node to bytes.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            TrieNode::Empty => vec![0],
            TrieNode::Leaf { key_end, value } => {
                let mut encoded = vec![1];
                encoded.extend_from_slice(&key_end.to_bytes());
                encoded.push(0); // Separator
                encoded.extend_from_slice(value);
                encoded
            }
            TrieNode::Extension { prefix, child } => {
                let mut encoded = vec![2];
                encoded.extend_from_slice(&prefix.to_bytes());
                encoded.push(0); // Separator
                encoded.extend_from_slice(child.as_bytes());
                encoded
            }
            TrieNode::Branch { children, value } => {
                let mut encoded = vec![3];
                for child in children.iter() {
                    if let Some(hash) = child {
                        encoded.extend_from_slice(hash.as_bytes());
                    } else {
                        encoded.extend_from_slice(&[0u8; 32]);
                    }
                }
                if let Some(v) = value {
                    encoded.push(1);
                    encoded.extend_from_slice(v);
                } else {
                    encoded.push(0);
                }
                encoded
            }
        }
    }

    /// Decode a node from bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, StorageError> {
        if bytes.is_empty() {
            return Ok(TrieNode::Empty);
        }

        match bytes[0] {
            0 => Ok(TrieNode::Empty),
            1 => {
                // Leaf node
                let sep_pos = bytes[1..].iter().position(|b| *b == 0)
                    .ok_or_else(|| StorageError::Deserialization("Invalid leaf node".to_string()))?;
                let key_end = Nibbles::from_bytes(&bytes[1..1+sep_pos]);
                let value = bytes[1+sep_pos+1..].to_vec();
                Ok(TrieNode::Leaf { key_end, value })
            }
            2 => {
                // Extension node
                let sep_pos = bytes[1..].iter().position(|b| *b == 0)
                    .ok_or_else(|| StorageError::Deserialization("Invalid extension node".to_string()))?;
                let prefix = Nibbles::from_bytes(&bytes[1..1+sep_pos]);
                let child = Hash::from_slice(&bytes[1+sep_pos+1..1+sep_pos+1+32])
                    .map_err(|e| StorageError::Deserialization(e.to_string()))?;
                Ok(TrieNode::Extension { prefix, child })
            }
            3 => {
                // Branch node
                if bytes.len() < 1 + 16 * 32 + 1 {
                    return Err(StorageError::Deserialization("Invalid branch node".to_string()));
                }
                let mut children: [Option<Hash>; 16] = Default::default();
                for i in 0..16 {
                    let start = 1 + i * 32;
                    let hash_bytes = &bytes[start..start+32];
                    if hash_bytes.iter().all(|b| *b == 0) {
                        children[i] = None;
                    } else {
                        children[i] = Some(Hash::from_slice(hash_bytes)
                            .map_err(|e| StorageError::Deserialization(e.to_string()))?);
                    }
                }
                let value = if bytes[1 + 16 * 32] == 1 {
                    Some(bytes[1 + 16 * 32 + 1..].to_vec())
                } else {
                    None
                };
                Ok(TrieNode::Branch { children, value })
            }
            _ => Err(StorageError::Deserialization(format!("Unknown node type: {}", bytes[0]))),
        }
    }

    /// Check if this is an empty node.
    pub fn is_empty(&self) -> bool {
        matches!(self, TrieNode::Empty)
    }
}

impl Default for TrieNode {
    fn default() -> Self {
        TrieNode::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nibbles_from_bytes() {
        let bytes = vec![0xAB, 0xCD];
        let nibbles = Nibbles::from_bytes(&bytes);
        assert_eq!(nibbles.0, vec![0xA, 0xB, 0xC, 0xD]);
    }

    #[test]
    fn test_nibbles_to_bytes() {
        let nibbles = Nibbles(vec![0xA, 0xB, 0xC, 0xD]);
        let bytes = nibbles.to_bytes();
        assert_eq!(bytes, vec![0xAB, 0xCD]);
    }

    #[test]
    fn test_nibbles_common_prefix() {
        let n1 = Nibbles(vec![0x1, 0x2, 0x3, 0x4]);
        let n2 = Nibbles(vec![0x1, 0x2, 0x5, 0x6]);
        assert_eq!(n1.common_prefix(&n2), 2);
    }

    #[test]
    fn test_node_encode_decode() {
        // Leaf node
        let leaf = TrieNode::Leaf {
            key_end: Nibbles(vec![0x1, 0x2]),
            value: vec![0xAB, 0xCD],
        };
        let encoded = leaf.encode();
        let decoded = TrieNode::decode(&encoded).unwrap();
        assert_eq!(leaf, decoded);

        // Extension node
        let ext = TrieNode::Extension {
            prefix: Nibbles(vec![0x1, 0x2]),
            child: Hash::compute(b"child"),
        };
        let encoded = ext.encode();
        let decoded = TrieNode::decode(&encoded).unwrap();
        assert_eq!(ext, decoded);

        // Branch node
        let branch = TrieNode::Branch {
            children: Default::default(),
            value: Some(vec![0xAB]),
        };
        let encoded = branch.encode();
        let decoded = TrieNode::decode(&encoded).unwrap();
        assert_eq!(branch, decoded);
    }

    #[test]
    fn test_empty_node() {
        let empty = TrieNode::Empty;
        assert!(empty.is_empty());
        let encoded = empty.encode();
        assert_eq!(encoded, vec![0]);
    }
}
