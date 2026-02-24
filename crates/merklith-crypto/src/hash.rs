use merklith_types::Hash;

/// Blake3 hashing utilities.

/// Compute blake3 hash of data
pub fn hash(data: &[u8]) -> Hash {
    Hash::compute(data)
}

/// Compute blake3 hash of multiple data slices
pub fn hash_multi(data: &[&[u8]]) -> Hash {
    Hash::compute_multi(data)
}

/// Incremental hasher for streaming hash computation
pub struct IncrementalHasher {
    hasher: blake3::Hasher,
}

impl IncrementalHasher {
    /// Create a new incremental hasher
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize and return the hash
    pub fn finalize(self) -> Hash {
        Hash::from_bytes(*self.hasher.finalize().as_bytes())
    }

    /// Reset the hasher for reuse
    pub fn reset(&mut self) {
        self.hasher.reset();
    }
}

impl Default for IncrementalHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash two values together (useful for Merkle trees)
pub fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    hash_multi(&[left.as_bytes(), right.as_bytes()])
}

/// Hash with a domain separator
pub fn hash_with_domain(data: &[u8], domain: &str) -> Hash {
    hash_multi(&[domain.as_bytes(), data])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let result = hash(b"test");
        assert!(!result.is_zero());

        // Deterministic
        let result2 = hash(b"test");
        assert_eq!(result, result2);

        // Different input
        let result3 = hash(b"test2");
        assert_ne!(result, result3);
    }

    #[test]
    fn test_hash_multi() {
        let result1 = hash_multi(&[b"hello ", b"world"]);
        let result2 = hash(b"hello world");
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_incremental_hasher() {
        let mut hasher = IncrementalHasher::new();
        hasher.update(b"hello ");
        hasher.update(b"world");
        let result1 = hasher.finalize();

        let result2 = hash(b"hello world");
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_hash_pair() {
        let h1 = hash(b"left");
        let h2 = hash(b"right");

        let result1 = hash_pair(&h1, &h2);
        let result2 = hash_pair(&h2, &h1);

        // Order matters
        assert_ne!(result1, result2);

        // Deterministic
        let result3 = hash_pair(&h1, &h2);
        assert_eq!(result1, result3);
    }

    #[test]
    fn test_hash_with_domain() {
        let result1 = hash_with_domain(b"data", "domain1");
        let result2 = hash_with_domain(b"data", "domain2");

        // Different domains produce different hashes
        assert_ne!(result1, result2);

        // Same domain and data produces same hash
        let result3 = hash_with_domain(b"data", "domain1");
        assert_eq!(result1, result3);
    }

    #[test]
    fn test_hasher_reset() {
        let mut hasher = IncrementalHasher::new();
        hasher.update(b"first");
        hasher.reset();
        hasher.update(b"second");
        let result = hasher.finalize();

        let expected = hash(b"second");
        assert_eq!(result, expected);
    }
}
