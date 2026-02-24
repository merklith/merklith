//! Node configuration.
//!
//! Handles loading and validation of node configuration from
//! config files and command-line arguments.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Node configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node name
    pub name: String,
    /// Data directory
    pub data_dir: PathBuf,
    /// Network configuration
    pub network: NetworkConfig,
    /// RPC configuration
    pub rpc: RpcConfig,
    /// Consensus configuration
    pub consensus: ConsensusConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            name: "merklith-node".to_string(),
            data_dir: PathBuf::from("./data"),
            network: NetworkConfig::default(),
            rpc: RpcConfig::default(),
            consensus: ConsensusConfig::default(),
            storage: StorageConfig::default(),
            metrics: MetricsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl NodeConfig {
    /// Load configuration from file.
    /// Path is validated to prevent directory traversal attacks.
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        // Validate path to prevent directory traversal
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            anyhow::bail!("Invalid path: directory traversal detected");
        }
        
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", path.display(), e))?;
        let config: NodeConfig = toml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file '{}': {}", path.display(), e))?;
        Ok(config)
    }

    /// Save configuration to file.
    /// Path is validated to prevent directory traversal attacks.
    pub fn to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        // Validate path to prevent directory traversal
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            anyhow::bail!("Invalid path: directory traversal detected");
        }
        
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)
            .map_err(|e| anyhow::anyhow!("Failed to write config file '{}': {}", path.display(), e))?;
        Ok(())
    }

    /// Validate configuration.
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate network config
        if self.network.p2p_port == 0 {
            anyhow::bail!("P2P port cannot be 0");
        }

        // Validate RPC config
        if self.rpc.http_enabled && self.rpc.http_port == 0 {
            anyhow::bail!("RPC HTTP port cannot be 0");
        }

        Ok(())
    }
}

/// Network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable P2P networking
    pub enabled: bool,
    /// P2P listen port
    pub p2p_port: u16,
    /// P2P listen address
    pub p2p_addr: String,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    /// Maximum peers
    pub max_peers: usize,
    /// Enable UPnP
    pub upnp: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            p2p_port: 30303,
            p2p_addr: "0.0.0.0".to_string(),
            bootstrap_nodes: vec![],
            max_peers: 50,
            upnp: false,
        }
    }
}

/// RPC configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// Enable HTTP RPC
    pub http_enabled: bool,
    /// HTTP RPC address
    pub http_addr: SocketAddr,
    /// HTTP RPC port
    pub http_port: u16,
    /// Enable WebSocket RPC
    pub ws_enabled: bool,
    /// WebSocket RPC address
    pub ws_addr: SocketAddr,
    /// Enable CORS
    pub cors: bool,
    /// Maximum request body size (MB)
    pub max_body_size: usize,
    /// Rate limit (requests per second)
    pub rate_limit: Option<u32>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            http_enabled: true,
            http_addr: "127.0.0.1:8545".parse().unwrap_or_else(|_| {
                // Fallback to a safe default if parsing fails (should never happen)
                std::net::SocketAddr::from(([127, 0, 0, 1], 8545))
            }),
            http_port: 8545,
            ws_enabled: true,
            ws_addr: "127.0.0.1:8546".parse().unwrap_or_else(|_| {
                // Fallback to a safe default if parsing fails (should never happen)
                std::net::SocketAddr::from(([127, 0, 0, 1], 8546))
            }),
            cors: true,
            max_body_size: 10,
            rate_limit: None,
        }
    }
}

/// Consensus configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Chain ID
    pub chain_id: u64,
    /// Block time in seconds (adaptive based on network activity)
    pub block_time: u64,
    /// Enable validator mode
    pub validator: bool,
    /// Validator key file
    pub validator_key: Option<PathBuf>,
    /// Minimum stake (in MERK)
    pub min_stake: u64,
    /// Max consecutive empty blocks before increasing block time
    pub max_empty_blocks: Option<u32>,
    /// Timeout for empty blocks (seconds) - produce heartbeat block after this
    pub empty_block_timeout: Option<u64>,
    /// Finality threshold (number of blocks to consider final)
    pub finality_threshold: Option<u32>,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            chain_id: 1337, // Devnet
            block_time: 12, // 12 seconds - Bitcoin/Ethereum arası optimal
            validator: false,
            validator_key: None,
            min_stake: 0, // Devnet: no minimum
            max_empty_blocks: Some(2), // Skip 2 empty blocks max
            empty_block_timeout: Some(60), // 60s timeout for heartbeat
            finality_threshold: Some(1), // PoC: single block finality
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path
    pub db_path: PathBuf,
    /// Cache size (MB)
    pub cache_size: usize,
    /// Enable compression
    pub compression: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("./data/db"),
            cache_size: 512,
            compression: true,
        }
    }
}

/// Metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics
    pub enabled: bool,
    /// Metrics server address
    pub addr: SocketAddr,
    /// Metrics export interval (seconds)
    pub interval: u64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            addr: "127.0.0.1:9090".parse().unwrap_or_else(|_| {
                std::net::SocketAddr::from(([127, 0, 0, 1], 9090))
            }),
            interval: 60,
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log to file
    pub log_file: Option<PathBuf>,
    /// Log format (json|pretty)
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_file: None,
            format: "pretty".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert_eq!(config.name, "merklith-node");
        assert!(config.network.enabled);
        assert!(config.rpc.http_enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = NodeConfig::default();
        assert!(config.validate().is_ok());

        // Invalid port
        config.network.p2p_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = NodeConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        
        assert!(toml_str.contains("name"));
        assert!(toml_str.contains("merklith-node"));
    }
}
