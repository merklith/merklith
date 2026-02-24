use merklith_types::Hash;
use crate::hash::hash_pair;

/// Binary Merkle tree using blake3.
pub struct MerkleTree {
    leaves: Vec<Hash>,
    nodes: Vec<Hash>,
}

impl MerkleTree {
    /// Build a Merkle tree from leaf data.
    pub fn from_leaves(leaves: &[Hash]) -> Self {
        if leaves.is_empty() {
            return Self {
                leaves: vec![],
                nodes: vec![],
            };
        }

        // Special case: single leaf is its own root
        if leaves.len() == 1 {
            return Self {
                leaves: leaves.to_vec(),
                nodes: vec![leaves[0]],
            };
        }

        let mut tree_leaves = leaves.to_vec();

        // If odd number of leaves, duplicate the last one
        if tree_leaves.len() % 2 != 0 {
            tree_leaves.push(*tree_leaves.last().unwrap());
        }

        let mut nodes = Vec::new();
        let mut current_level: Vec<Hash> = tree_leaves.clone();

        // Build tree bottom-up
        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let left = &chunk[0];
                let right = if chunk.len() == 2 { &chunk[1] } else { left };
                let parent = hash_pair(left, right);
                next_level.push(parent);
            }

            nodes.extend(current_level.iter().cloned());
            current_level = next_level;
        }

        nodes.push(current_level[0]); // Root

        Self {
            leaves: tree_leaves,
            nodes,
        }
    }

    /// Get the root hash.
    pub fn root(&self) -> Hash {
        self.nodes.last().copied().unwrap_or(Hash::ZERO)
    }

    /// Generate a proof for a leaf at the given index.
    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let leaf = self.leaves[index];
        let mut siblings = Vec::new();
        let mut current_index = index;
        let mut level_size = self.leaves.len();

        // Walk up the tree
        let mut offset = 0;
        while level_size > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < level_size {
                siblings.push(self.nodes[offset + sibling_index]);
            }

            current_index /= 2;
            offset += level_size;
            level_size = (level_size + 1) / 2;
        }

        Some(MerkleProof {
            leaf,
            index,
            siblings,
        })
    }

    /// Number of leaves
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Verify a leaf is in the tree
    pub fn verify_leaf(&self, index: usize, leaf: &Hash) -> bool {
        if index >= self.leaves.len() {
            return false;
        }
        self.leaves[index] == *leaf
    }
}

/// Merkle inclusion proof.
#[derive(Clone, Debug)]
pub struct MerkleProof {
    pub leaf: Hash,
    pub index: usize,
    pub siblings: Vec<Hash>,
}

impl MerkleProof {
    /// Verify this proof against an expected root.
    pub fn verify(&self, root: &Hash) -> bool {
        &self.compute_root() == root
    }

    /// Compute the root from this proof.
    pub fn compute_root(&self) -> Hash {
        let mut current = self.leaf;
        let mut index = self.index;

        for sibling in &self.siblings {
            if index % 2 == 0 {
                current = hash_pair(&current, sibling);
            } else {
                current = hash_pair(sibling, &current);
            }
            index /= 2;
        }

        current
    }

    /// Get proof size (number of siblings)
    pub fn depth(&self) -> usize {
        self.siblings.len()
    }
}

/// Hash two children to get parent node.
pub fn merkle_hash_pair(left: &Hash, right: &Hash) -> Hash {
    hash_pair(left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_empty() {
        let tree = MerkleTree::from_leaves(&[]);
        assert_eq!(tree.root(), Hash::ZERO);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_merkle_tree_single_leaf() {
        let leaf = Hash::compute(b"single");
        let tree = MerkleTree::from_leaves(&[leaf]);

        assert_eq!(tree.root(), leaf);
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_merkle_tree_two_leaves() {
        let leaf1 = Hash::compute(b"leaf1");
        let leaf2 = Hash::compute(b"leaf2");
        let tree = MerkleTree::from_leaves(&[leaf1, leaf2]);

        let expected_root = hash_pair(&leaf1, &leaf2);
        assert_eq!(tree.root(), expected_root);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_merkle_tree_three_leaves() {
        let leaf1 = Hash::compute(b"leaf1");
        let leaf2 = Hash::compute(b"leaf2");
        let leaf3 = Hash::compute(b"leaf3");

        let tree = MerkleTree::from_leaves(&[leaf1, leaf2, leaf3]);

        // Should have 4 leaves (last one duplicated)
        assert_eq!(tree.len(), 4);
    }

    #[test]
    fn test_merkle_proof() {
        let leaves: Vec<Hash> = (0..8)
            .map(|i| Hash::compute(format!("leaf{}", i).as_bytes()))
            .collect();

        let tree = MerkleTree::from_leaves(&leaves);

        // Generate proof for each leaf
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.proof(i).unwrap();

            assert_eq!(proof.leaf, *leaf);
            assert_eq!(proof.index, i);

            // Verify proof
            assert!(proof.verify(&tree.root()));

            // Wrong root should fail
            let wrong_root = Hash::compute(b"wrong");
            assert!(!proof.verify(&wrong_root));
        }
    }

    #[test]
    fn test_merkle_proof_depth() {
        let leaves: Vec<Hash> = (0..8)
            .map(|i| Hash::compute(format!("leaf{}", i).as_bytes()))
            .collect();

        let tree = MerkleTree::from_leaves(&leaves);
        let proof = tree.proof(0).unwrap();

        // 8 leaves = log2(8) = 3 levels of siblings
        assert_eq!(proof.depth(), 3);
    }

    #[test]
    fn test_merkle_proof_out_of_bounds() {
        let leaves: Vec<Hash> = (0..4)
            .map(|i| Hash::compute(format!("leaf{}", i).as_bytes()))
            .collect();

        let tree = MerkleTree::from_leaves(&leaves);

        assert!(tree.proof(4).is_none());
        assert!(tree.proof(100).is_none());
    }

    #[test]
    fn test_merkle_deterministic() {
        let leaves: Vec<Hash> = (0..4)
            .map(|i| Hash::compute(format!("leaf{}", i).as_bytes()))
            .collect();

        let tree1 = MerkleTree::from_leaves(&leaves);
        let tree2 = MerkleTree::from_leaves(&leaves);

        assert_eq!(tree1.root(), tree2.root());

        let proof1 = tree1.proof(0).unwrap();
        let proof2 = tree2.proof(0).unwrap();

        assert_eq!(proof1.siblings, proof2.siblings);
    }

    #[test]
    fn test_verify_leaf() {
        let leaves: Vec<Hash> = (0..4)
            .map(|i| Hash::compute(format!("leaf{}", i).as_bytes()))
            .collect();

        let tree = MerkleTree::from_leaves(&leaves);

        assert!(tree.verify_leaf(0, &leaves[0]));
        assert!(tree.verify_leaf(3, &leaves[3]));

        let wrong_leaf = Hash::compute(b"wrong");
        assert!(!tree.verify_leaf(0, &wrong_leaf));
        assert!(!tree.verify_leaf(10, &leaves[0]));
    }
}
