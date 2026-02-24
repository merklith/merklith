use crate::error::TypesError;
use std::fmt;
use std::str::FromStr;

/// 20-byte account address derived from ed25519 public key.
/// Display format: Bech32m with "merk" human-readable prefix.
///
/// # Derivation
/// `address = blake3(ed25519_pubkey)[0..20]`
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Address([u8; 20]);

impl Address {
    pub const ZERO: Self = Self([0u8; 20]);
    pub const LEN: usize = 20;

    /// Bech32m human-readable prefix
    pub const BECH32_HRP: &'static str = "merk";

    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Create from a byte slice
    pub fn from_slice(slice: &[u8]) -> Result<Self, TypesError> {
        if slice.len() != 20 {
            return Err(TypesError::InvalidAddressLength(slice.len()));
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Derive address from ed25519 public key bytes (32 bytes).
    /// Uses blake3 hash, takes first 20 bytes.
    pub fn from_public_key(pubkey: &[u8; 32]) -> Self {
        let hash = blake3::hash(pubkey);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash.as_bytes()[..20]);
        Self(addr)
    }

    /// Check if this is the zero address
    pub fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }

    /// Check if this is a system contract address (all zeros except last 4 bytes)
    pub fn is_system(&self) -> bool {
        self.0[..16].iter().all(|&b| b == 0) && !self.is_zero()
    }

    /// Convert to hex string without 0x prefix
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Encode as Bech32m with "merk" prefix
        let hrp = bech32::Hrp::parse_unchecked(Self::BECH32_HRP);
        match bech32::encode::<bech32::Bech32m>(hrp, &self.0) {
            Ok(encoded) => write!(f, "{}", encoded),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address(0x{})", hex::encode(self.0))
    }
}

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl fmt::UpperHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode_upper(self.0))
    }
}

impl FromStr for Address {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Support both Bech32m ("merk1...") and hex ("0x...")
        if s.starts_with("merk1") {
            let (hrp, data) = bech32::decode(s).map_err(|e| {
                TypesError::Bech32Error(e.to_string())
            })?;

            let expected_hrp = bech32::Hrp::parse_unchecked(Self::BECH32_HRP);
            if hrp != expected_hrp {
                return Err(TypesError::InvalidAddressFormat(format!(
                    "Invalid HRP: expected '{}', got '{}'",
                    Self::BECH32_HRP,
                    hrp
                )));
            }

            let data_len = data.len();
            let bytes: [u8; 20] = data.try_into().map_err(|_| {
                TypesError::InvalidAddressLength(data_len)
            })?;

            Ok(Self::from_bytes(bytes))
        } else if s.starts_with("0x") || s.starts_with("0X") {
            let bytes = hex::decode(&s[2..])?;
            Self::from_slice(&bytes)
        } else {
            Err(TypesError::InvalidAddressFormat(s.to_string()))
        }
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_zero() {
        assert_eq!(Address::ZERO.as_bytes(), &[0u8; 20]);
        assert!(Address::ZERO.is_zero());
    }

    #[test]
    fn test_address_from_bytes() {
        let bytes = [1u8; 20];
        let addr = Address::from_bytes(bytes);
        assert_eq!(addr.as_bytes(), &bytes);
    }

    #[test]
    fn test_address_from_public_key() {
        let pubkey = [42u8; 32];
        let addr = Address::from_public_key(&pubkey);
        assert!(!addr.is_zero());

        // Deterministic
        let addr2 = Address::from_public_key(&pubkey);
        assert_eq!(addr, addr2);

        // Different pubkey = different address
        let pubkey2 = [43u8; 32];
        let addr3 = Address::from_public_key(&pubkey2);
        assert_ne!(addr, addr3);
    }

    #[test]
    fn test_address_bech32m_roundtrip() {
        let bytes: [u8; 20] = (0..20).map(|i| i as u8).collect::<Vec<_>>().try_into().unwrap();
        let addr = Address::from_bytes(bytes);

        // Encode
        let encoded = addr.to_string();
        assert!(encoded.starts_with("merk1"));

        // Decode
        let decoded: Address = encoded.parse().unwrap();
        assert_eq!(addr, decoded);
    }

    #[test]
    fn test_address_hex_roundtrip() {
        let bytes = [0xabu8; 20];
        let addr = Address::from_bytes(bytes);

        let hex = format!("{:x}", addr);
        let parsed: Address = hex.parse().unwrap();
        assert_eq!(addr, parsed);
    }

    #[test]
    fn test_address_from_str_invalid() {
        // Invalid Bech32m
        assert!(Address::from_str("invalid").is_err());

        // Wrong HRP
        assert!(Address::from_str("xyz1...").is_err());

        // Too short
        assert!(Address::from_str("0x1234").is_err());
    }

    #[test]
    fn test_address_is_system() {
        // Zero is not system
        assert!(!Address::ZERO.is_system());

        // System address (16 zeros + 4 bytes)
        let mut bytes = [0u8; 20];
        bytes[16] = 1;
        let system_addr = Address::from_bytes(bytes);
        assert!(system_addr.is_system());

        // Non-system address
        let mut bytes = [0u8; 20];
        bytes[0] = 1;
        let normal_addr = Address::from_bytes(bytes);
        assert!(!normal_addr.is_system());
    }

    #[test]
    fn test_address_ordering() {
        let addr1 = Address::from_bytes([0u8; 20]);
        let addr2 = Address::from_bytes([1u8; 20]);
        assert!(addr1 < addr2);
        assert!(addr2 > addr1);
    }
}
