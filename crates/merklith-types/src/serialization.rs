//! Serialization implementations for merklith-types
//! 
//! This module provides serde and borsh implementations for all types.

use crate::*;

// Serde implementations
#[cfg(feature = "serde")]
mod serde_impls {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    // U256
    impl Serialize for U256 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for U256 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            U256::from_str(&s).map_err(serde::de::Error::custom)
        }
    }

    // Hash
    impl Serialize for Hash {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Hash {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Hash::from_str(&s).map_err(serde::de::Error::custom)
        }
    }

    // Address
    impl Serialize for Address {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Address {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Address::from_str(&s).map_err(serde::de::Error::custom)
        }
    }

    // Ed25519Signature
    impl Serialize for Ed25519Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            format!("0x{}", hex::encode(self.as_bytes())).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Ed25519Signature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            let s = if s.starts_with("0x") { &s[2..] } else { &s };
            let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
            Ed25519Signature::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }

    // Ed25519PublicKey
    impl Serialize for Ed25519PublicKey {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            format!("0x{}", hex::encode(self.as_bytes())).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Ed25519PublicKey {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            let s = if s.starts_with("0x") { &s[2..] } else { &s };
            let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
            Ed25519PublicKey::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }

    // BLSSignature
    impl Serialize for BLSSignature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            format!("0x{}", hex::encode(self.as_bytes())).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for BLSSignature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            let s = if s.starts_with("0x") { &s[2..] } else { &s };
            let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
            BLSSignature::from_bytes(&bytes).map_err(serde::de::Error::custom)
        }
    }

    // BLSPublicKey
    impl Serialize for BLSPublicKey {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            format!("0x{}", hex::encode(self.as_bytes())).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for BLSPublicKey {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            let s = if s.starts_with("0x") { &s[2..] } else { &s };
            let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
            BLSPublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

// Borsh implementations
#[cfg(feature = "borsh")]
mod borsh_impls {
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};

    // U256 - stored as little-endian bytes
    impl BorshSerialize for U256 {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            writer.write_all(&self.to_le_bytes())
        }
    }

    impl BorshDeserialize for U256 {
        fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            let mut bytes = [0u8; 32];
            reader.read_exact(&mut bytes)?;
            Ok(U256::from_le_bytes(bytes))
        }
    }

    // Hash - stored as raw bytes
    impl BorshSerialize for Hash {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            writer.write_all(self.as_bytes())
        }
    }

    impl BorshDeserialize for Hash {
        fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            let mut bytes = [0u8; 32];
            reader.read_exact(&mut bytes)?;
            Ok(Hash::from_bytes(bytes))
        }
    }

    // Address - stored as raw bytes
    impl BorshSerialize for Address {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            writer.write_all(self.as_bytes())
        }
    }

    impl BorshDeserialize for Address {
        fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            let mut bytes = [0u8; 20];
            reader.read_exact(&mut bytes)?;
            Ok(Address::from_bytes(bytes))
        }
    }

    // Ed25519Signature
    impl BorshSerialize for Ed25519Signature {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            writer.write_all(self.as_bytes())
        }
    }

    impl BorshDeserialize for Ed25519Signature {
        fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            let mut bytes = [0u8; 64];
            reader.read_exact(&mut bytes)?;
            Ok(Ed25519Signature::from_bytes(bytes))
        }
    }

    // Ed25519PublicKey
    impl BorshSerialize for Ed25519PublicKey {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            writer.write_all(self.as_bytes())
        }
    }

    impl BorshDeserialize for Ed25519PublicKey {
        fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            let mut bytes = [0u8; 32];
            reader.read_exact(&mut bytes)?;
            Ok(Ed25519PublicKey::from_bytes(bytes))
        }
    }

    // BLSSignature
    impl BorshSerialize for BLSSignature {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            BorshSerialize::serialize(&(self.as_bytes().len() as u32), writer)?;
            writer.write_all(self.as_bytes())
        }
    }

    impl BorshDeserialize for BLSSignature {
        fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            let len = u32::deserialize_reader(reader)? as usize;
            if len != 96 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid BLS signature length: {}", len),
                ));
            }
            let mut bytes = vec![0u8; len];
            reader.read_exact(&mut bytes)?;
            BLSSignature::from_bytes(&bytes).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })
        }
    }

    // BLSPublicKey
    impl BorshSerialize for BLSPublicKey {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            BorshSerialize::serialize(&(self.as_bytes().len() as u32), writer)?;
            writer.write_all(self.as_bytes())
        }
    }

    impl BorshDeserialize for BLSPublicKey {
        fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            let len = u32::deserialize_reader(reader)? as usize;
            if len != 48 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid BLS public key length: {}", len),
                ));
            }
            let mut bytes = vec![0u8; len];
            reader.read_exact(&mut bytes)?;
            BLSPublicKey::from_bytes(&bytes).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn test_u256_serde_roundtrip() {
        let original = U256::from(12345u64);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: U256 = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    #[cfg(feature = "borsh")]
    fn test_u256_borsh_roundtrip() {
        let original = U256::from(12345u64);
        let encoded = borsh::to_vec(&original).unwrap();
        let deserialized: U256 = borsh::from_slice(&encoded).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_hash_serde_roundtrip() {
        let original = Hash::compute(b"test");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Hash = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    #[cfg(feature = "borsh")]
    fn test_hash_borsh_roundtrip() {
        let original = Hash::compute(b"test");
        let encoded = borsh::to_vec(&original).unwrap();
        let deserialized: Hash = borsh::from_slice(&encoded).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_address_serde_roundtrip() {
        let original = Address::from_bytes([1u8; 20]);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    #[cfg(feature = "borsh")]
    fn test_address_borsh_roundtrip() {
        let original = Address::from_bytes([1u8; 20]);
        let encoded = borsh::to_vec(&original).unwrap();
        let deserialized: Address = borsh::from_slice(&encoded).unwrap();
        assert_eq!(original, deserialized);
    }
}
