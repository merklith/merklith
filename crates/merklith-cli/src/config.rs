//! CLI configuration management.
//!
//! Handles RPC endpoints, default settings, etc.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default RPC endpoint
    pub rpc_url: String,
    /// Default chain ID
    pub chain_id: u64,
    /// Gas price (wei)
    pub gas_price: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Keystore directory
    pub keystore_dir: PathBuf,
    /// Default account
    pub default_account: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            chain_id: 1,
            gas_price: 1_000_000_000, // 1 gwei
            gas_limit: 100_000,
            keystore_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".merklith")
                .join("keystore"),
            default_account: None,
        }
    }
}

impl CliConfig {
    /// Load configuration from file.
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: CliConfig = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file.
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        
        // Create directory if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(config_path, contents)?;
        Ok(())
    }

    /// Get configuration file path.
    pub fn config_path() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".merklith").join("config.toml"))
    }

    /// Get keystore directory.
    pub fn keystore_path(&self) -> PathBuf {
        self.keystore_dir.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.rpc_url, "http://localhost:8545");
        assert_eq!(config.chain_id, 1);
    }
}
