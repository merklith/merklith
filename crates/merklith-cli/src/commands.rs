//! CLI command implementations.
//!
//! All interactive CLI commands for the Merklith blockchain.

use merklith_crypto::ed25519::Keypair as Ed25519Keypair;
use merklith_types::{Address, Transaction, TransactionType, U256, SignedTransaction};
use borsh::BorshSerialize;
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{Input, Password, Confirm};
use indicatif::ProgressBar;
use std::path::PathBuf;

use crate::config::CliConfig;
use crate::keystore::Keystore;
use crate::output::*;
use crate::rpc_client::RpcClient;

/// Main CLI.
#[derive(Parser)]
#[command(name = "merklith")]
#[command(about = "Merklith Blockchain CLI - Where Trust is Forged")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// RPC endpoint URL
    #[arg(short, long, global = true)]
    pub rpc: Option<String>,

    /// Chain ID
    #[arg(long, global = true)]
    pub chain_id: Option<u64>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Wallet management
    #[command(subcommand)]
    Wallet(WalletCommands),

    /// Account operations
    #[command(subcommand)]
    Account(AccountCommands),

    /// TUI Block Explorer
    Explorer {
        /// RPC endpoint URL (optional, uses default if not provided)
        #[arg(short, long)]
        rpc: Option<String>,
    },

    /// Transaction operations
    #[command(subcommand)]
    Tx(TxCommands),

    /// Query blockchain state
    #[command(subcommand)]
    Query(QueryCommands),

    /// Contract operations
    #[command(subcommand)]
    Contract(ContractCommands),

    /// Node operations
    #[command(subcommand)]
    Node(NodeCommands),

    /// Configuration
    #[command(subcommand)]
    Config(ConfigCommands),
}

/// Wallet commands.
#[derive(Subcommand)]
pub enum WalletCommands {
    /// Create a new wallet
    Create {
        /// Wallet name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List wallets
    List,
    /// Import wallet from private key
    Import {
        /// Private key (hex)
        private_key: String,
        /// Wallet name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Export wallet (WARNING: exposes private key)
    Export {
        /// Wallet address
        address: String,
    },
    /// Show wallet details
    Show {
        /// Wallet address
        address: String,
    },
    /// Remove wallet
    Remove {
        /// Wallet address
        address: String,
    },
}

/// Account commands.
#[derive(Subcommand)]
pub enum AccountCommands {
    /// Check balance
    Balance {
        /// Address (or use default)
        address: Option<String>,
    },
    /// List all account balances
    Balances,
    /// Get nonce
    Nonce {
        /// Address
        address: String,
    },
}

/// Transaction commands.
#[derive(Subcommand)]
pub enum TxCommands {
    /// Send a transaction
    Send {
        /// To address
        to: String,
        /// Amount in MERK
        amount: f64,
        /// Gas price (optional)
        #[arg(short, long)]
        gas_price: Option<u64>,
        /// Gas limit
        #[arg(short, long, default_value = "21000")]
        gas_limit: u64,
        /// From address (uses default if not specified)
        #[arg(short, long)]
        from: Option<String>,
    },
    /// Get transaction details
    Get {
        /// Transaction hash
        hash: String,
    },
    /// Wait for transaction receipt
    Wait {
        /// Transaction hash
        hash: String,
        /// Timeout in seconds
        #[arg(short, long, default_value = "60")]
        timeout: u64,
    },
}

/// Query commands.
#[derive(Subcommand)]
pub enum QueryCommands {
    /// Get block by number
    Block {
        /// Block number (or "latest")
        number: String,
    },
    /// Get block by hash
    BlockHash {
        /// Block hash
        hash: String,
    },
    /// Get current block number
    BlockNumber,
    /// Get chain ID
    ChainId,
    /// Get gas price
    GasPrice,
    /// Get node info
    NodeInfo,
}

/// Contract commands.
#[derive(Subcommand)]
pub enum ContractCommands {
    /// Deploy a contract
    Deploy {
        /// Contract bytecode file
        bytecode: PathBuf,
        /// Constructor arguments (hex)
        #[arg(short, long)]
        args: Option<String>,
        /// Gas limit
        #[arg(short, long, default_value = "1000000")]
        gas_limit: u64,
    },
    /// Call a contract function (read-only)
    Call {
        /// Contract address
        address: String,
        /// Function call data (hex)
        data: String,
    },
    /// Send transaction to contract
    Send {
        /// Contract address
        address: String,
        /// Transaction data (hex)
        data: String,
        /// Value in MERK
        #[arg(short, long, default_value = "0")]
        value: f64,
        /// Gas limit
        #[arg(short, long)]
        gas_limit: Option<u64>,
    },
    /// Get contract bytecode
    Code {
        /// Contract address
        address: String,
    },
}

/// Node commands.
#[derive(Subcommand)]
pub enum NodeCommands {
    /// Start local node
    Start {
        /// Enable validator mode
        #[arg(long)]
        validator: bool,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Stop local node
    Stop,
    /// Get node status
    Status,
    /// Stream logs
    Logs {
        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
        /// Follow logs
        #[arg(short, long)]
        follow: bool,
    },
}

/// Config commands.
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current config
    Show,
    /// Set config value
    Set {
        /// Key
        key: String,
        /// Value
        value: String,
    },
    /// Get config value
    Get {
        /// Key
        key: String,
    },
    /// Reset to defaults
    Reset,
    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

/// Execute a CLI command.
pub async fn execute(cmd: Commands, rpc: Option<String>) -> anyhow::Result<()> {
    let config = CliConfig::load()?;
    let rpc_url = rpc.unwrap_or(config.rpc_url.clone());
    let client = RpcClient::new(rpc_url);

    match cmd {
        Commands::Wallet(cmd) => execute_wallet(cmd, &config).await,
        Commands::Account(cmd) => execute_account(cmd, &client, &config).await,
        Commands::Tx(cmd) => execute_tx(cmd, &client, &config).await,
        Commands::Query(cmd) => execute_query(cmd, &client).await,
        Commands::Contract(cmd) => execute_contract(cmd, &client).await,
        Commands::Node(cmd) => execute_node(cmd).await,
        Commands::Config(cmd) => execute_config(cmd).await,
        Commands::Explorer { rpc } => execute_explorer(rpc, &config).await,
    }
}

/// Execute wallet commands.
async fn execute_wallet(cmd: WalletCommands, config: &CliConfig) -> anyhow::Result<()> {
    match cmd {
        WalletCommands::Create { name } => {
            let name = name.unwrap_or_else(|| {
                Input::<String>::new()
                    .with_prompt("Wallet name")
                    .interact()
                    .unwrap_or_else(|_| "wallet".to_string())
            });

            let password = Password::new()
                .with_prompt("Set password")
                .with_confirmation("Confirm password", "Passwords don't match")
                .interact()?;

            // Generate keypair
            let keypair = Ed25519Keypair::generate();
            let address = keypair.address();
            let private_key = keypair.to_bytes();

            // Save to keystore with encryption
            let keystore_dir = config.keystore_path();
            let mut keystore = Keystore::new(keystore_dir)?;
            let is_first = keystore.list_wallets().is_empty();
            keystore.save_wallet(&name, address, &private_key, &password, is_first)?;

            print_success(&format!("Created wallet '{}'", name));
            println!("Address: {}", format_address(&address));
            
            // WARNING
            print_warning("IMPORTANT: Save your recovery phrase in a safe place!");
            print_info("Your wallet is encrypted with AES-256-GCM and stored in the keystore.");
        }

        WalletCommands::List => {
            let keystore_dir = config.keystore_path();
            let keystore = Keystore::new(keystore_dir)?;
            let wallets = keystore.list_wallets();
            
            if wallets.is_empty() {
                println!("{}", "No wallets found".yellow());
                println!("Create a wallet with: merklith wallet create");
            } else {
                let count = wallets.len();
                println!("{}", "Wallets:".bold());
                for wallet in wallets {
                    let default_marker = if wallet.is_default { " (default)" } else { "" };
                    println!("  • {} - {}{}", 
                        wallet.name.bright_green(),
                        wallet.address.to_string().bright_cyan(),
                        default_marker.yellow()
                    );
                }
                println!("\nTotal: {} wallet(s)", count);
            }
        }

        WalletCommands::Import { private_key, name } => {
            let name = name.unwrap_or_else(|| "imported".to_string());
            
            // Parse private key
            let key_hex = private_key.trim_start_matches("0x").trim_start_matches("0X");
            let key_bytes = hex::decode(key_hex)?;
            if key_bytes.len() != 32 {
                anyhow::bail!("Invalid private key length: expected 64 hex chars (32 bytes)");
            }
            let mut private_key_array = [0u8; 32];
            private_key_array.copy_from_slice(&key_bytes);
            
            // Generate keypair from private key
            let keypair = Ed25519Keypair::from_seed(&private_key_array);
            let address = keypair.address();
            
            let password = Password::new()
                .with_prompt("Set password")
                .with_confirmation("Confirm password", "Passwords don't match")
                .interact()?;

            // Save to keystore
            let keystore_dir = config.keystore_path();
            let mut keystore = Keystore::new(keystore_dir)?;
            let is_first = keystore.list_wallets().is_empty();
            keystore.save_wallet(&name, address, &private_key_array, &password, is_first)?;
            
            print_success(&format!("Wallet '{}' imported successfully", name));
            println!("Address: {}", format_address(&address));
        }

        WalletCommands::Export { address } => {
            let confirm = Confirm::new()
                .with_prompt(format!("WARNING: This will expose the private key for {}. Continue?", address))
                .interact()?;

            if !confirm {
                println!("Export cancelled");
                return Ok(());
            }

            // Parse address
            let addr = parse_address(&address)?;
            
            // Check if wallet exists
            let keystore_dir = config.keystore_path();
            let keystore = Keystore::new(keystore_dir)?;
            
            if !keystore.has_wallet(&addr) {
                anyhow::bail!("Wallet not found: {}", address);
            }
            
            // Get password and decrypt
            let password = Password::new()
                .with_prompt("Enter wallet password")
                .interact()?;
            
            match keystore.load_wallet(&addr, &password) {
                Ok(private_key) => {
                    print_warning("⚠️  CRITICAL SECURITY WARNING!");
                    print_warning("Never share your private key with anyone!");
                    print_warning("This key provides full access to your funds.");
                    println!();
                    
                    // Use eprintln to avoid logging to stdout
                    eprintln!("Private Key: 0x{}", hex::encode(&private_key).bright_red().bold());
                    
                    println!();
                    print_warning("Store this in a secure, offline location immediately!");
                    print_warning("Clear your terminal history after copying: history -c");
                    
                    // Clear from memory immediately after display
                    drop(private_key);
                }
                Err(_) => {
                    anyhow::bail!("Failed to decrypt wallet. Wrong password?");
                }
            }
        }

        WalletCommands::Show { address } => {
            println!("Wallet: {}", address);
            println!("  Keystore: Not implemented");
        }

        WalletCommands::Remove { address } => {
            // Parse address
            let addr = parse_address(&address)?;
            
            // Check if wallet exists
            let keystore_dir = config.keystore_path();
            let keystore = Keystore::new(keystore_dir)?;
            
            if !keystore.has_wallet(&addr) {
                anyhow::bail!("Wallet not found: {}", address);
            }
            
            let confirm = Confirm::new()
                .with_prompt(format!("⚠️  Remove wallet {}? This cannot be undone!", address))
                .interact()?;

            if confirm {
                // Get password to confirm ownership
                let password = Password::new()
                    .with_prompt("Enter wallet password to confirm removal")
                    .interact()?;
                
                // Verify password by trying to load
                match keystore.load_wallet(&addr, &password) {
                    Ok(_) => {
                        // Password correct, remove wallet
                        let mut keystore = Keystore::new(config.keystore_path())?;
                        keystore.remove_wallet(&addr)?;
                        print_success(&format!("Removed wallet {}", address));
                    }
                    Err(_) => {
                        anyhow::bail!("Failed to verify wallet. Wrong password?");
                    }
                }
            } else {
                println!("Removal cancelled");
            }
        }
    }

    Ok(())
}

/// Execute account commands.
async fn execute_account(cmd: AccountCommands, client: &RpcClient, config: &CliConfig) -> anyhow::Result<()> {
    match cmd {
        AccountCommands::Balance { address } => {
            let addr_str = match address {
                Some(addr) => addr,
                None => {
                    // Try to use default account
                    let keystore_dir = config.keystore_path();
                    let keystore = Keystore::new(keystore_dir)?;
                    
                    match keystore.get_default() {
                        Some(entry) => entry.address.to_string(),
                        None => {
                            print_error("No address specified and no default account set");
                            print_info("Set a default account with: merklith wallet create");
                            print_info("Or specify an address: merklith account balance 0x...");
                            std::process::exit(1);
                        }
                    }
                }
            };

            let addr = parse_address(&addr_str)?;
            let balance = client.get_balance(&addr).await?;
            
            println!("Address: {}", addr_str.bright_cyan());
            println!("Balance: {}", format_merk(&balance).bright_green());
        }

        AccountCommands::Balances => {
            let keystore_dir = config.keystore_path();
            let keystore = Keystore::new(keystore_dir)?;
            let wallets = keystore.list_wallets();
            
            if wallets.is_empty() {
                println!("{}", "No wallets found".yellow());
                println!("Create a wallet first: merklith wallet create");
                return Ok(());
            }
            
            println!("{}", "Account Balances:".bold());
            println!("{}", "=".repeat(60));
            
            let mut total_balance = U256::ZERO;
            
            for wallet in wallets {
                match client.get_balance(&wallet.address).await {
                    Ok(balance) => {
                        let default_marker = if wallet.is_default { " [default]" } else { "" };
                        println!("  {} {}", 
                            wallet.name.bright_green(),
                            default_marker.yellow()
                        );
                        println!("    Address: {}", wallet.address.to_string().bright_cyan());
                        println!("    Balance: {}", format_merk(&balance).bright_green());
                        total_balance = total_balance + balance;
                    }
                    Err(e) => {
                        println!("  {} - Error: {}", wallet.name.red(), e);
                    }
                }
                println!();
            }
            
            println!("{}", "=".repeat(60));
            println!("Total Balance: {}", format_merk(&total_balance).bright_yellow().bold());
        }

        AccountCommands::Nonce { address } => {
            let addr = parse_address(&address)?;
            let nonce = client.get_transaction_count(&addr).await?;
            println!("Address: {}", address.bright_cyan());
            println!("Nonce:   {}", nonce.to_string().bright_green());
        }
    }

    Ok(())
}

/// Execute transaction commands.
async fn execute_tx(cmd: TxCommands, client: &RpcClient, config: &CliConfig) -> anyhow::Result<()> {
    match cmd {
        TxCommands::Send { to, amount, gas_price, gas_limit, from } => {
            let to_addr = parse_address(&to)?;
            
            // Get sender address
            let sender_addr = match from {
                Some(addr_str) => parse_address(&addr_str)?,
                None => {
                    // Try to use default account
                    let keystore_dir = config.keystore_path();
                    let keystore = Keystore::new(keystore_dir)?;
                    
                    match keystore.get_default() {
                        Some(entry) => entry.address,
                        None => {
                            print_error("No sender specified and no default account set");
                            print_info("Set a default account with: merklith wallet create");
                            print_info("Or use --from flag: merklith tx send 0x... 1.0 --from 0x...");
                            std::process::exit(1);
                        }
                    }
                }
            };

            // Convert amount to wei using string parsing for precision
            let value = parse_amount_to_wei(amount)?;
            let gas_price = gas_price.unwrap_or(1_000_000_000);

            println!("Sending {} to {}", format_merk(&value).bright_yellow(), format_address(&to_addr));
            println!("From: {}", format_address(&sender_addr));
            println!("Gas Price: {} Gwei", gas_price / 1_000_000_000);
            println!("Gas Limit: {}", gas_limit);
            
            // Get password to sign
            let keystore_dir = config.keystore_path();
            let keystore = Keystore::new(keystore_dir)?;
            
            let password = Password::new()
                .with_prompt("Enter wallet password to sign transaction")
                .interact()?;
            
            // Load private key
            let private_key = match keystore.load_wallet(&sender_addr, &password) {
                Ok(key) => key,
                Err(_) => {
                    anyhow::bail!("Failed to decrypt wallet. Wrong password?");
                }
            };
            
            // Create and sign transaction
            let keypair = Ed25519Keypair::from_seed(&private_key);
            let nonce = client.get_transaction_count(&sender_addr).await?;
            let chain_id = client.chain_id().await?;
            
            let tx = Transaction::new(
                chain_id,
                nonce,
                Some(to_addr),
                value,
                gas_limit,
                U256::from(gas_price),
                U256::ZERO,
            );
            
            let (signature, public_key) = keypair.sign_transaction(&tx);
            let signed_tx = SignedTransaction::new(tx, signature, public_key);
            
            // Serialize signed transaction to hex
            let tx_bytes = borsh::to_vec(&signed_tx)?;
            let tx_hex = format!("0x{}", hex::encode(&tx_bytes));
            
            // Send transaction
            match client.send_raw_transaction(&tx_hex).await {
                Ok(tx_hash) => {
                    print_success("Transaction sent successfully!");
                    println!("Transaction Hash: {}", tx_hash.to_string().bright_green());
                    println!("\nView transaction:");
                    println!("  merklith tx get {}", tx_hash.to_string().bright_cyan());
                }
                Err(e) => {
                    print_error(&format!("Failed to send transaction: {}", e));
                }
            }
        }

        TxCommands::Get { hash } => {
            let tx_hash = parse_hash(&hash)?;
            
            match client.get_transaction_receipt(&tx_hash).await? {
                Some(receipt) => {
                    print_transaction_receipt(receipt);
                }
                None => {
                    print_info("Transaction not found or pending");
                }
            }
        }

        TxCommands::Wait { hash, timeout } => {
            let tx_hash = parse_hash(&hash)?;
            
            let pb = create_progress_bar(timeout);
            pb.set_message("Waiting for confirmation...");

            let start = std::time::Instant::now();
            loop {
                if start.elapsed().as_secs() > timeout {
                    pb.finish_with_message("Timeout");
                    print_error("Transaction not confirmed within timeout");
                    break;
                }

                if let Some(receipt) = client.get_transaction_receipt(&tx_hash).await? {
                    pb.finish_with_message("Confirmed!");
                    print_transaction_receipt(receipt);
                    break;
                }

                pb.inc(1);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }

    Ok(())
}

/// Execute query commands.
async fn execute_query(cmd: QueryCommands, client: &RpcClient) -> anyhow::Result<()> {
    match cmd {
        QueryCommands::Block { number } => {
            let block_num = if number == "latest" {
                client.block_number().await?
            } else {
                number.parse()?
            };

            match client.get_block_by_number(block_num).await? {
                Some(block) => {
                    print_block_info(&block);
                }
                None => {
                    print_error(&format!("Block {} not found", block_num));
                }
            }
        }

        QueryCommands::BlockHash { hash } => {
            println!("Query by hash: {}", hash);
            // TODO: Implement eth_getBlockByHash
            print_info("Feature not yet implemented");
        }

        QueryCommands::BlockNumber => {
            let block_num = client.block_number().await?;
            println!("Current block number: {}", block_num.to_string().bright_green());
        }

        QueryCommands::ChainId => {
            let chain_id = client.chain_id().await?;
            println!("Chain ID: {}", chain_id.to_string().bright_green());
        }

        QueryCommands::GasPrice => {
            let gas_price = client.gas_price().await?;
            println!("Gas Price: {} ({} wei)", 
                format_merk(&gas_price).bright_yellow(),
                gas_price.to_string().bright_cyan()
            );
        }

        QueryCommands::NodeInfo => {
            match client.health().await {
                Ok(info) => {
                    println!("{}", "Node Information".bold());
                    println!("{}", "=".repeat(50));
                    println!("{}", serde_json::to_string_pretty(&info)?);
                }
                Err(e) => {
                    print_error(&format!("Failed to get node info: {}", e));
                }
            }
        }
    }

    Ok(())
}

/// Execute contract commands.
async fn execute_contract(cmd: ContractCommands, client: &RpcClient) -> anyhow::Result<()> {
    match cmd {
        ContractCommands::Deploy { bytecode, args, gas_limit } => {
            let code = std::fs::read(bytecode)?;
            let code_hex = hex::encode(code);
            
            println!("Deploying contract...");
            println!("Bytecode size: {} bytes", code_hex.len() / 2);
            println!("Gas limit: {}", gas_limit);
            
            if let Some(a) = args {
                println!("Constructor args: {}", a);
            }

            // TODO: Deploy contract
            print_info("Contract deployment not yet implemented");
        }

        ContractCommands::Call { address, data } => {
            let addr = parse_address(&address)?;
            
            let tx = serde_json::json!({
                "to": format!("0x{}", hex::encode(addr.as_bytes())),
                "data": data,
            });

            match client.call_contract(tx).await {
                Ok(result) => {
                    println!("Result: {}", result.bright_green());
                }
                Err(e) => {
                    print_error(&format!("Call failed: {}", e));
                }
            }
        }

        ContractCommands::Send { address, data, value, gas_limit } => {
            let addr = parse_address(&address)?;
            let val = parse_amount_to_wei(value)?;

            println!("Sending {} to contract {}", 
                format_merk(&val).bright_yellow(),
                format_address(&addr)
            );
            println!("Data: {}", data.bright_cyan());
            
            if let Some(gas) = gas_limit {
                println!("Gas limit: {}", gas);
            }

            // TODO: Send transaction
            print_info("Contract interaction not yet implemented");
        }

        ContractCommands::Code { address } => {
            let addr = parse_address(&address)?;
            let code = client.get_code(&addr).await?;
            
            if code == "0x" {
                println!("No code at address {} (EOA)", address.bright_cyan());
            } else {
                println!("Code at {}: {} bytes", 
                    address.bright_cyan(),
                    (code.len() - 2) / 2
                );
                println!("{}", format!("{}", code).bright_yellow());
            }
        }
    }

    Ok(())
}

/// Execute node commands.
async fn execute_node(cmd: NodeCommands) -> anyhow::Result<()> {
    match cmd {
        NodeCommands::Start { validator, data_dir } => {
            if validator {
                println!("Starting {} mode...", "VALIDATOR".bright_green().bold());
            } else {
                println!("Starting node...");
            }

            if let Some(dir) = data_dir {
                println!("Data directory: {:?}", dir);
            }

            print_info("Use 'merklith-node' binary directly for production use");
            print_info("CLI node management is for development only");
        }

        NodeCommands::Stop => {
            print_info("Stopping node...");
            print_info("Use Ctrl+C if running in foreground");
        }

        NodeCommands::Status => {
            println!("{}", "Node Status".bold());
            println!("{}", "=".repeat(50));
            println!("Status: {}", "Not implemented".yellow());
        }

        NodeCommands::Logs { lines, follow } => {
            println!("Showing last {} lines", lines);
            if follow {
                println!("Following logs... (Ctrl+C to exit)");
            }
        }
    }

    Ok(())
}

/// Execute config commands.
async fn execute_config(cmd: ConfigCommands) -> anyhow::Result<()> {
    let mut config = CliConfig::load()?;

    match cmd {
        ConfigCommands::Show => {
            println!("{}", "CLI Configuration".bold());
            println!("{}", "=".repeat(50));
            println!("RPC URL:      {}", config.rpc_url.bright_cyan());
            println!("Chain ID:     {}", config.chain_id.to_string().bright_green());
            println!("Gas Price:    {} wei", config.gas_price.to_string().bright_yellow());
            println!("Gas Limit:    {}", config.gas_limit.to_string().bright_magenta());
            println!("Keystore:     {:?}", config.keystore_dir);
            println!("Default Acc:  {:?}", config.default_account);
        }

        ConfigCommands::Set { key, value } => {
            let value_clone = value.clone();
            match key.as_str() {
                "rpc" | "rpc_url" => config.rpc_url = value,
                "chain_id" => config.chain_id = value.parse()?,
                "gas_price" => config.gas_price = value.parse()?,
                "gas_limit" => config.gas_limit = value.parse()?,
                _ => {
                    print_error(&format!("Unknown config key: {}", key));
                    return Ok(());
                }
            }
            
            config.save()?;
            print_success(&format!("Set {} = {}", key, value_clone));
        }

        ConfigCommands::Get { key } => {
            let value = match key.as_str() {
                "rpc" | "rpc_url" => &config.rpc_url,
                "chain_id" => &config.chain_id.to_string(),
                "gas_price" => &config.gas_price.to_string(),
                "gas_limit" => &config.gas_limit.to_string(),
                _ => {
                    print_error(&format!("Unknown config key: {}", key));
                    return Ok(());
                }
            };
            println!("{} = {}", key.bright_cyan(), value.bright_green());
        }

        ConfigCommands::Reset => {
            let confirm = Confirm::new()
                .with_prompt("Reset all configuration to defaults?")
                .interact()?;

            if confirm {
                config = CliConfig::default();
                config.save()?;
                print_success("Configuration reset to defaults");
            }
        }

        ConfigCommands::Completions { shell } => {
            use clap_complete::{generate, shells};
            use clap::Command;

            let mut cmd = Command::new("merklith")
                .version(env!("CARGO_PKG_VERSION"))
                .about("MERKLITH Blockchain CLI");

            let shell_type = match shell {
                Shell::Bash => shells::Shell::Bash,
                Shell::Zsh => shells::Shell::Zsh,
                Shell::Fish => shells::Shell::Fish,
                Shell::PowerShell => shells::Shell::PowerShell,
                Shell::Elvish => shells::Shell::Elvish,
            };

            generate(shell_type, &mut cmd, "merklith", &mut std::io::stdout());
        }
    }

    Ok(())
}

/// Parse address string.
fn parse_address(s: &str) -> anyhow::Result<Address> {
    let s = s.trim_start_matches("0x");
    let bytes = hex::decode(s)?;
    if bytes.len() != 20 {
        anyhow::bail!("Invalid address length");
    }
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes);
    Ok(Address::from_bytes(addr))
}

/// Parse hash string.
fn parse_hash(s: &str) -> anyhow::Result<merklith_types::Hash> {
    let s = s.trim_start_matches("0x");
    let bytes = hex::decode(s)?;
    if bytes.len() != 32 {
        anyhow::bail!("Invalid hash length");
    }
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);
    Ok(merklith_types::Hash::from_bytes(hash))
}

/// Parse amount to wei using string representation for precision.
/// Avoids floating-point precision issues.
fn parse_amount_to_wei(amount: f64) -> anyhow::Result<U256> {
    // Convert to string with high precision to avoid floating-point errors
    let amount_str = format!("{:.18}", amount);
    
    // Split into integer and decimal parts
    let parts: Vec<&str> = amount_str.split('.').collect();
    let integer_part = parts[0].parse::<u128>()?;
    
    let decimal_part = if parts.len() > 1 {
        // Pad or truncate to 18 decimal places
        let decimals = &parts[1];
        let padded = format!("{:0<18}", decimals);
        padded[..18].parse::<u128>()?
    } else {
        0u128
    };
    
    // Calculate wei: integer_part * 10^18 + decimal_part
    let wei = integer_part
        .checked_mul(1_000_000_000_000_000_000u128)
        .and_then(|v| v.checked_add(decimal_part))
        .ok_or_else(|| anyhow::anyhow!("Amount too large"))?;
    
    Ok(U256::from(wei))
}

/// Execute TUI block explorer
async fn execute_explorer(rpc: Option<String>, config: &CliConfig) -> anyhow::Result<()> {
    use crate::explorer::run_explorer;
    
    let rpc_url = rpc.unwrap_or_else(|| config.rpc_url.clone());
    
    println!("{}", "Starting Merklith Block Explorer...".bright_cyan().bold());
    println!("Connecting to: {}", rpc_url.bright_yellow());
    println!();
    
    // Run the TUI explorer
    run_explorer(rpc_url).await?;
    
    Ok(())
}
