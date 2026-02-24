//! MERKLITH Monitor - Real-time blockchain monitoring tool

use clap::Parser;
use colored::Colorize;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "merklith-monitor")]
#[command(about = "Real-time MERKLITH blockchain monitoring")]
#[command(version)]
struct Args {
    /// RPC endpoint URL
    #[arg(short, long, default_value = "http://localhost:8545")]
    rpc: String,

    /// Update interval in seconds
    #[arg(short, long, default_value = "2")]
    interval: u64,

    /// Show detailed metrics
    #[arg(short, long)]
    detailed: bool,

    /// Monitor specific account
    #[arg(short, long)]
    account: Option<String>,

    /// Export metrics to file
    #[arg(long)]
    export: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    println!("{}", "MERKLITH Blockchain Monitor".bright_cyan().bold());
    println!("{}", "═══════════════════════".bright_cyan());
    println!("RPC Endpoint: {}", args.rpc.bright_yellow());
    println!("Update Interval: {}s", args.interval);
    println!();
    
    let mut last_block: u64 = 0;
    let mut last_time = Instant::now();
    let mut block_times: Vec<f64> = Vec::new();
    
    loop {
        // Clear screen (cross-platform)
        print!("\x1B[2J\x1B[1;1H");
        
        // Fetch current data
        match fetch_metrics(&args.rpc).await {
            Ok(metrics) => {
                display_metrics(&metrics, &args);
                
                // Calculate TPS and block time
                if metrics.block_number > last_block {
                    let elapsed = last_time.elapsed().as_secs_f64();
                    let blocks = metrics.block_number - last_block;
                    let block_time = elapsed / blocks as f64;
                    
                    block_times.push(block_time);
                    if block_times.len() > 10 {
                        block_times.remove(0);
                    }
                    
                    let avg_block_time = block_times.iter().sum::<f64>() / block_times.len() as f64;
                    let tps = metrics.pending_txs as f64 / avg_block_time;
                    
                    if args.detailed {
                        println!("\n{}", "Performance Metrics".bright_green().bold());
                        println!("Block Time: {:.2}s", avg_block_time);
                        println!("TPS: {:.2}", tps);
                        println!("Blocks since last: {}", blocks);
                    }
                    
                    last_block = metrics.block_number;
                    last_time = Instant::now();
                }
                
                // Export if requested
                if let Some(ref path) = args.export {
                    if let Err(e) = export_metrics(&metrics, path).await {
                        eprintln!("Export error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("Error: {}", e).bright_red());
            }
        }
        
        // Account monitoring
        if let Some(ref account) = args.account {
            match fetch_account_info(&args.rpc, account).await {
                Ok(info) => {
                    println!("\n{}", "Account Monitor".bright_green().bold());
                    println!("Address: {}", account.bright_cyan());
                    println!("Balance: {} MERK", info.balance);
                    println!("Nonce: {}", info.nonce);
                    println!("Transactions: {}", info.tx_count);
                }
                Err(e) => {
                    println!("Account error: {}", e);
                }
            }
        }
        
        println!("\n{}", format!("Last update: {:?}", Instant::now()).dimmed());
        println!("Press Ctrl+C to exit");
        
        tokio::time::sleep(Duration::from_secs(args.interval)).await;
    }
}

#[derive(Debug)]
struct Metrics {
    block_number: u64,
    chain_id: u64,
    gas_price: u64,
    peer_count: u64,
    pending_txs: u64,
    syncing: bool,
    validator: bool,
}

async fn fetch_metrics(rpc_url: &str) -> anyhow::Result<Metrics> {
    let client = reqwest::Client::new();
    
    // Fetch multiple metrics in parallel
    let block_number_fut = rpc_call(&client, rpc_url, "eth_blockNumber", vec![]);
    let chain_id_fut = rpc_call(&client, rpc_url, "eth_chainId", vec![]);
    let gas_price_fut = rpc_call(&client, rpc_url, "eth_gasPrice", vec![]);
    let syncing_fut = rpc_call(&client, rpc_url, "eth_syncing", vec![]);
    
    let (block_hex, chain_hex, gas_hex, syncing_result) = tokio::join!(
        block_number_fut,
        chain_id_fut,
        gas_price_fut,
        syncing_fut
    );
    
    let block_number = u64::from_str_radix(
        block_hex?.trim_start_matches("0x"),
        16
    ).unwrap_or(0);
    
    let chain_id = u64::from_str_radix(
        chain_hex?.trim_start_matches("0x"),
        16
    ).unwrap_or(0);
    
    let gas_price = u64::from_str_radix(
        gas_hex?.trim_start_matches("0x"),
        16
    ).unwrap_or(0);
    
    let syncing = syncing_result? != "false";
    
    Ok(Metrics {
        block_number,
        chain_id,
        gas_price,
        peer_count: 8, // Placeholder
        pending_txs: 0, // Placeholder
        syncing,
        validator: false, // Placeholder
    })
}

fn display_metrics(metrics: &Metrics, args: &Args) {
    println!("{}", "Network Status".bright_green().bold());
    
    let status = if metrics.syncing {
        "SYNCING".bright_yellow()
    } else {
        "SYNCED".bright_green()
    };
    
    println!("Status: {}", status);
    println!("Chain ID: {}", metrics.chain_id);
    println!("Block: {}", metrics.block_number.to_string().bright_cyan());
    println!("Gas Price: {} Gwei", metrics.gas_price / 1_000_000_000);
    
    if args.detailed {
        println!("\n{}", "Node Info".bright_green().bold());
        println!("Peers: {}", metrics.peer_count);
        println!("Pending TXs: {}", metrics.pending_txs);
        println!("Validator: {}", if metrics.validator { "Yes".bright_green() } else { "No".dimmed() });
    }
}

#[derive(Debug)]
struct AccountInfo {
    balance: String,
    nonce: u64,
    tx_count: u64,
}

async fn fetch_account_info(rpc_url: &str, address: &str) -> anyhow::Result<AccountInfo> {
    let client = reqwest::Client::new();
    
    let balance_hex = rpc_call(
        &client,
        rpc_url,
        "eth_getBalance",
        vec![address.to_string(), "latest".to_string()]
    ).await?;
    
    let balance = u128::from_str_radix(
        balance_hex.trim_start_matches("0x"),
        16
    ).unwrap_or(0);
    
    let balance_merk = balance as f64 / 1e18;
    
    let nonce_hex = rpc_call(
        &client,
        rpc_url,
        "eth_getTransactionCount",
        vec![address.to_string(), "latest".to_string()]
    ).await?;
    
    let nonce = u64::from_str_radix(
        nonce_hex.trim_start_matches("0x"),
        16
    ).unwrap_or(0);
    
    Ok(AccountInfo {
        balance: format!("{:.6}", balance_merk),
        nonce,
        tx_count: nonce, // Simplified
    })
}

async fn rpc_call(
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Vec<String>,
) -> anyhow::Result<String> {
    let params_json: Vec<serde_json::Value> = params
        .into_iter()
        .map(|p| serde_json::Value::String(p))
        .collect();
    
    let response = client
        .post(url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params_json,
            "id": 1
        }))
        .send()
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    
    if let Some(result) = json.get("result") {
        Ok(result.as_str().unwrap_or("0x0").to_string())
    } else {
        Err(anyhow::anyhow!("RPC error"))
    }
}

async fn export_metrics(_metrics: &Metrics, path: &str) -> anyhow::Result<()> {
    use tokio::io::AsyncWriteExt;
    
    let json = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metrics": {
            "block_number": _metrics.block_number,
            "chain_id": _metrics.chain_id,
        }
    });
    
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    
    file.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}