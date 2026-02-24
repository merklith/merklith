//! Merklith Node - Full node implementation.
//!
//! This crate provides the full node binary that ties together
//! all other crates to run a complete Merklith blockchain node.

pub mod node;
pub mod config;
pub mod metrics;
pub mod telemetry;

use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error};

/// Command-line arguments.
#[derive(Parser, Debug)]
#[command(name = "merklith-node")]
#[command(about = "Merklith Node - Where Trust is Forged")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    /// Config file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Data directory
    #[arg(short, long, default_value = "./data")]
    data_dir: PathBuf,

    /// RPC HTTP port
    #[arg(long, default_value = "8545")]
    rpc_port: u16,

    /// P2P port
    #[arg(long, default_value = "30303")]
    p2p_port: u16,

    /// Enable validator mode
    #[arg(long)]
    validator: bool,

    /// Chain ID
    #[arg(long, default_value = "1337")]
    chain_id: u64,

    /// Bootstrap peers (comma-separated, e.g. "127.0.0.1:30303,10.0.0.1:30303")
    #[arg(long)]
    bootstrap: Option<String>,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Enable metrics
    #[arg(long)]
    metrics: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize telemetry
    telemetry::init_telemetry(&args.log_level,
        false, // pretty format for CLI
    )?;

    print_banner();

    // Load or create config
    let mut config = if let Some(config_path) = &args.config {
        info!("Loading configuration from: {:?}", config_path);
        config::NodeConfig::from_file(config_path)?
    } else {
        info!("Using default configuration");
        config::NodeConfig::default()
    };

    // Override with CLI args
    config.data_dir = args.data_dir;
    // Bind to 0.0.0.0 for Docker/external access, not 127.0.0.1
    config.rpc.http_addr = format!("0.0.0.0:{}", args.rpc_port).parse()?;
    config.rpc.http_enabled = true;
    config.network.p2p_port = args.p2p_port;
    config.consensus.validator = args.validator;
    config.consensus.chain_id = args.chain_id;
    config.metrics.enabled = args.metrics;
    
    // Parse bootstrap peers
    if let Some(bootstrap) = &args.bootstrap {
        config.network.bootstrap_nodes = bootstrap
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Validate config
    config.validate()?;

    info!("Configuration:");
    info!("  RPC http_enabled: {}", config.rpc.http_enabled);
    info!("  RPC ws_enabled: {}", config.rpc.ws_enabled);
    info!("  Name: {}", config.name);
    info!("  Data dir: {:?}", config.data_dir);
    info!("  Chain ID: {}", config.consensus.chain_id);
    info!("  RPC port: {}", config.rpc.http_addr.port());
    info!("  P2P port: {}", config.network.p2p_port);
    info!("  Validator mode: {}", config.consensus.validator);

    // Create and start node
    let (mut node, _shutdown) = node::MerklithNode::new(config).await?;
    
    if let Err(e) = node.start().await {
        error!("Failed to start node: {}", e);
        return Err(e);
    }

    // Run node (blocks until shutdown)
    if let Err(e) = node.run().await {
        error!("Node error: {}", e);
        return Err(e);
    }

    info!("Merklith node shutdown complete");
    Ok(())
}

/// Print startup banner.
fn print_banner() {
    println!();
    println!(r#"    _                _ _           _   _           _      _       _   _      _    _"#);
    println!(r#"   / \   _ __  _ __ | (_) ___ __ _| |_(_)_ __   __| |    | |     | \ | | ___| | _(_) ___ _ __   ___ _ __"#);
    println!(r#"  / _ \ | '_ \| '_ \| | |/ __/ _` | __| | '_ \ / _` |    | |     |  \| |/ _ \ |/ / |/ _ \ '_ \ / _ \ '__|"#);
    println!(r#" / ___ \| | | | | | | | | (_| (_| | |_| | | | | (_| |    | |___  | |\  |  __/   <| |  __/ | | |  __/ |"#);
    println!(r#"/_/   \_\_| |_|_| |_|_|_|\___\__,_|\__|_|_| |_|\__,_|    |_____| |_| \_|\___|_|\_\_|\___|_| |_|\___|_|"#);
    println!();
    println!("                    Where Trust is Forged");
    println!("                    Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args() {
        let args = Args::parse_from([
            "merklith-node",
            "--rpc-port", "8545",
            "--chain-id", "42",
        ]);
        
        assert_eq!(args.rpc_port, 8545);
        assert_eq!(args.chain_id, 42);
        assert!(!args.validator);
    }
}
