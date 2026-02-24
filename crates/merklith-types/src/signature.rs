use crate::error::TypesError;
use std::fmt;

/// Ed25519 signature (64 bytes) — used for transaction signing.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ed25519Signature([u8; 64]);

impl Ed25519Signature {
    pub const LEN: usize = 64;

    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, TypesError> {
        if slice.len() != 64 {
            return Err(TypesError::InvalidSignatureLength {
                expected: 64,
                actual: slice.len(),
            });
        }
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl Default for Ed25519Signature {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl fmt::Debug for Ed25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ed25519Signature(0x{}...)", &hex::encode(&self.0[..8]))
    }
}

impl fmt::LowerHex for Ed25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

/// Ed25519 public key (32 bytes).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Ed25519PublicKey([u8; 32]);

impl Ed25519PublicKey {
    pub const LEN: usize = 32;

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, TypesError> {
        if slice.len() != 32 {
            return Err(TypesError::InvalidPublicKeyLength {
                expected: 32,
                actual: slice.len(),
            });
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Derive address from this public key
    pub fn to_address(&self) -> crate::address::Address {
        crate::address::Address::from_public_key(&self.0)
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl fmt::Debug for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ed25519PublicKey(0x{}...)", &hex::encode(&self.0[..8]))
    }
}

impl fmt::LowerHex for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

/// BLS12-381 signature (96 bytes) — used for committee attestations.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct BLSSignature(Vec<u8>); // 96 bytes

impl BLSSignature {
    pub const LEN: usize = 96;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TypesError> {
        if bytes.len() != 96 {
            return Err(TypesError::InvalidSignatureLength {
                expected: 96,
                actual: bytes.len(),
            });
        }
        Ok(Self(bytes.to_vec()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl fmt::Debug for BLSSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BLSSignature(0x{}...)", &hex::encode(&self.0[..8]))
    }
}

impl fmt::LowerHex for BLSSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

/// BLS12-381 public key (48 bytes).
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct BLSPublicKey(Vec<u8>); // 48 bytes

impl BLSPublicKey {
    pub const LEN: usize = 48;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TypesError> {
        if bytes.len() != 48 {
            return Err(TypesError::InvalidPublicKeyLength {
                expected: 48,
                actual: bytes.len(),
            });
        }
        Ok(Self(bytes.to_vec()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl fmt::Debug for BLSPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BLSPublicKey(0x{}...)", &hex::encode(&self.0[..8]))
    }
}

impl fmt::LowerHex for BLSPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_signature() {
        let sig = Ed25519Signature::from_bytes([1u8; 64]);
        assert_eq!(sig.as_bytes(), &[1u8; 64]);
        assert!(!sig.is_zero());

        let zero = Ed25519Signature::default();
        assert!(zero.is_zero());
    }

    #[test]
    fn test_ed25519_public_key() {
        let pk = Ed25519PublicKey::from_bytes([1u8; 32]);
        assert_eq!(pk.as_bytes(), &[1u8; 32]);

        // Address derivation
        let addr = pk.to_address();
        assert!(!addr.is_zero());
    }

    #[test]
    fn test_bls_signature() {
        let sig = BLSSignature::from_bytes(&[1u8; 96]).unwrap();
        assert_eq!(sig.as_bytes(), &[1u8; 96]);

        // Wrong length
        assert!(BLSSignature::from_bytes(&[1u8; 95]).is_err());
    }

    #[test]
    fn test_bls_public_key() {
        let pk = BLSPublicKey::from_bytes(&[1u8; 48]).unwrap();
        assert_eq!(pk.as_bytes(), &[1u8; 48]);

        // Wrong length
        assert!(BLSPublicKey::from_bytes(&[1u8; 47]).is_err());
    }
}
