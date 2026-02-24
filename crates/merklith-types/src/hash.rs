use crate::error::TypesError;
use std::fmt;
use std::str::FromStr;

/// 32-byte hash value (blake3 digest).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Hash([u8; 32]);

impl Hash {
    pub const ZERO: Self = Self([0u8; 32]);
    pub const LEN: usize = 32;

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create from a byte slice
    pub fn from_slice(slice: &[u8]) -> Result<Self, TypesError> {
        if slice.len() != 32 {
            return Err(TypesError::InvalidHashLength(slice.len()));
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Compute blake3 hash of data
    pub fn compute(data: &[u8]) -> Self {
        Self(*blake3::hash(data).as_bytes())
    }

    /// Compute blake3 hash of multiple data slices
    pub fn compute_multi(data: &[&[u8]]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for chunk in data {
            hasher.update(chunk);
        }
        Self(*hasher.finalize().as_bytes())
    }

    /// Check if hash is zero
    pub fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self)
    }
}

impl fmt::LowerHex for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl fmt::UpperHex for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode_upper(self.0))
    }
}

impl FromStr for Hash {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = if s.starts_with("0x") || s.starts_with("0X") {
            &s[2..]
        } else {
            s
        };

        let bytes = hex::decode(s)?;
        Self::from_slice(&bytes)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_compute() {
        let hash = Hash::compute(b"hello world");
        assert!(!hash.is_zero());

        // Deterministic
        let hash2 = Hash::compute(b"hello world");
        assert_eq!(hash, hash2);

        // Different input = different output
        let hash3 = Hash::compute(b"hello world!");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_hash_compute_multi() {
        let hash1 = Hash::compute_multi(&[b"hello ", b"world"]);
        let hash2 = Hash::compute(b"hello world");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_hex_roundtrip() {
        let hash = Hash::compute(b"test");
        let hex = hash.to_string();
        let parsed: Hash = hex.parse().unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_hash_zero() {
        assert!(Hash::ZERO.is_zero());
        assert!(!Hash::compute(b"test").is_zero());
    }

    #[test]
    fn test_hash_ordering() {
        let h1 = Hash::compute(b"a");
        let h2 = Hash::compute(b"b");
        assert!(h1 != h2);
    }
}
