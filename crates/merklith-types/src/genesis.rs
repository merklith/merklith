use crate::address::Address;
use crate::chain_config::ChainConfig;
use crate::hash::Hash;
use crate::signature::{BLSPublicKey, Ed25519PublicKey};
use crate::u256::U256;

/// Genesis block configuration.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GenesisConfig {
    /// Chain configuration
    pub chain_config: ChainConfig,
    /// Genesis timestamp
    pub timestamp: u64,
    /// Extra data in genesis block
    pub extra_data: Vec<u8>,
    /// Pre-funded accounts
    pub alloc: Vec<GenesisAlloc>,
    /// Initial validators
    pub validators: Vec<GenesisValidator>,
}

/// Genesis allocation entry
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisAlloc {
    pub address: Address,
    pub balance: U256,
    pub code: Option<Vec<u8>>,    // For system contracts
    pub storage: Option<Vec<(Hash, Vec<u8>)>>,
}

/// Genesis validator entry
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisValidator {
    pub address: Address,
    pub stake: U256,
    pub bls_public_key: BLSPublicKey,
    pub ed25519_public_key: Ed25519PublicKey,
}

impl GenesisConfig {
    /// Create a new genesis config with default chain config
    pub fn new(timestamp: u64) -> Self {
        Self {
            chain_config: ChainConfig::default(),
            timestamp,
            extra_data: Vec::new(),
            alloc: Vec::new(),
            validators: Vec::new(),
        }
    }

    /// Add an allocation
    pub fn add_alloc(&mut self, address: Address, balance: U256) {
        self.alloc.push(GenesisAlloc {
            address,
            balance,
            code: None,
            storage: None,
        });
    }

    /// Add a system contract
    pub fn add_system_contract(
        &mut self,
        address: Address,
        code: Vec<u8>,
        storage: Option<Vec<(Hash, Vec<u8>)>>,
    ) {
        self.alloc.push(GenesisAlloc {
            address,
            balance: U256::ZERO,
            code: Some(code),
            storage,
        });
    }

    /// Add a validator
    pub fn add_validator(
        &mut self,
        address: Address,
        stake: U256,
        bls_pk: BLSPublicKey,
        ed25519_pk: Ed25519PublicKey,
    ) {
        self.validators.push(GenesisValidator {
            address,
            stake,
            bls_public_key: bls_pk,
            ed25519_public_key: ed25519_pk,
        });
    }

    /// Get mainnet genesis config
    pub fn mainnet() -> Self {
        Self {
            chain_config: ChainConfig::mainnet(),
            timestamp: 1700000000, // Example timestamp
            extra_data: b"Merklith Mainnet".to_vec(),
            alloc: Vec::new(),
            validators: Vec::new(),
        }
    }

    /// Get testnet genesis config
    pub fn testnet() -> Self {
        Self {
            chain_config: ChainConfig::testnet(),
            timestamp: 1700000000,
            extra_data: b"Merklith Testnet".to_vec(),
            alloc: Vec::new(),
            validators: Vec::new(),
        }
    }

    /// Get devnet genesis config
    pub fn devnet() -> Self {
        Self {
            chain_config: ChainConfig::devnet(),
            timestamp: 1700000000,
            extra_data: b"Merklith Devnet".to_vec(),
            alloc: Vec::new(),
            validators: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_config_new() {
        let config = GenesisConfig::new(1700000000);
        assert_eq!(config.timestamp, 1700000000);
        assert!(config.alloc.is_empty());
        assert!(config.validators.is_empty());
    }

    #[test]
    fn test_genesis_add_alloc() {
        let mut config = GenesisConfig::new(0);
        let addr = Address::from_bytes([1u8; 20]);

        config.add_alloc(addr, U256::from(1000u64));

        assert_eq!(config.alloc.len(), 1);
        assert_eq!(config.alloc[0].address, addr);
        assert_eq!(config.alloc[0].balance, U256::from(1000u64));
    }

    #[test]
    fn test_genesis_add_system_contract() {
        let mut config = GenesisConfig::new(0);
        let addr = Address::from_bytes([0u8; 20]);
        let code = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic bytes

        config.add_system_contract(addr, code.clone(), None);

        assert_eq!(config.alloc.len(), 1);
        assert_eq!(config.alloc[0].code, Some(code));
    }

    #[test]
    fn test_genesis_presets() {
        let mainnet = GenesisConfig::mainnet();
        assert_eq!(mainnet.chain_config.chain_id, 1);

        let testnet = GenesisConfig::testnet();
        assert_eq!(testnet.chain_config.chain_id, 2);

        let devnet = GenesisConfig::devnet();
        assert_eq!(devnet.chain_config.chain_id, 1337);
    }
}
