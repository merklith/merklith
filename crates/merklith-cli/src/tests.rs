//! CLI tests.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CliConfig;
    use std::path::PathBuf;

    #[test]
    fn test_cli_config_default() {
        let config = CliConfig::default();
        assert_eq!(config.rpc_url, "http://localhost:8545");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.gas_price, 1_000_000_000);
        assert_eq!(config.gas_limit, 100_000);
    }

    #[test]
    fn test_config_save_and_load() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Create and save config
        let config = CliConfig {
            rpc_url: "http://test:8545".to_string(),
            chain_id: 42,
            gas_price: 2_000_000_000,
            gas_limit: 200_000,
            keystore_dir: temp_dir.path().join("keystore"),
            default_account: Some("test_account".to_string()),
        };
        
        // Test toml serialization
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("http://test:8545"));
        assert!(toml_str.contains("42"));
    }
}

use crate::keystore::Keystore;
use merklith_types::Address;
use merklith_crypto::ed25519::Keypair;

#[cfg(test)]
mod keystore_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_keystore_creation() {
        let temp_dir = TempDir::new().unwrap();
        let keystore = Keystore::new(temp_dir.path().to_path_buf()).unwrap();
        
        assert!(keystore.list_wallets().is_empty());
    }

    #[test]
    fn test_keystore_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut keystore = Keystore::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Generate a test keypair
        let keypair = Keypair::generate();
        let address = keypair.address();
        let private_key = keypair.to_bytes();
        
        // Save wallet
        keystore.save_wallet("test_wallet", address, &private_key, "password123", true).unwrap();
        
        // Verify wallet was saved
        assert_eq!(keystore.list_wallets().len(), 1);
        assert!(keystore.has_wallet(&address));
        
        // Load wallet
        let loaded_key = keystore.load_wallet(&address, "password123").unwrap();
        assert_eq!(loaded_key, private_key);
    }

    #[test]
    fn test_keystore_default_wallet() {
        let temp_dir = TempDir::new().unwrap();
        let mut keystore = Keystore::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Create two wallets
        let keypair1 = Keypair::generate();
        let keypair2 = Keypair::generate();
        
        keystore.save_wallet("wallet1", keypair1.address(), &keypair1.to_bytes(), "pass", true).unwrap();
        keystore.save_wallet("wallet2", keypair2.address(), &keypair2.to_bytes(), "pass", false).unwrap();
        
        // Check default
        let default = keystore.get_default().unwrap();
        assert_eq!(default.address, keypair1.address());
        assert!(default.is_default);
    }

    #[test]
    fn test_keystore_remove_wallet() {
        let temp_dir = TempDir::new().unwrap();
        let mut keystore = Keystore::new(temp_dir.path().to_path_buf()).unwrap();
        
        let keypair = Keypair::generate();
        let address = keypair.address();
        
        keystore.save_wallet("test", address, &keypair.to_bytes(), "pass", false).unwrap();
        assert!(keystore.has_wallet(&address));
        
        keystore.remove_wallet(&address).unwrap();
        assert!(!keystore.has_wallet(&address));
    }

    #[test]
    fn test_keystore_wrong_password() {
        let temp_dir = TempDir::new().unwrap();
        let mut keystore = Keystore::new(temp_dir.path().to_path_buf()).unwrap();
        
        let keypair = Keypair::generate();
        let address = keypair.address();
        
        keystore.save_wallet("test", address, &keypair.to_bytes(), "correct", false).unwrap();
        
        // Try to load with wrong password
        let result = keystore.load_wallet(&address, "wrong");
        assert!(result.is_err());
    }
}