//! MERKLITH Benchmark - Performance testing tool

use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[derive(Parser)]
#[command(name = "merklith-benchmark")]
#[command(about = "MERKLITH blockchain performance benchmark")]
#[command(version)]
struct Args {
    /// RPC endpoint URL
    #[arg(short, long, default_value = "http://localhost:8545")]
    rpc: String,

    /// Number of transactions to send
    #[arg(short, long, default_value = "100")]
    transactions: usize,

    /// Number of concurrent senders
    #[arg(short, long, default_value = "10")]
    concurrency: usize,

    /// Test type
    #[arg(short, long, value_enum, default_value = "transfer")]
    test: TestType,

    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Duration limit in seconds
    #[arg(long)]
    duration: Option<u64>,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum TestType {
    /// Simple transfer benchmark
    Transfer,
    /// Contract deployment benchmark
    Deploy,
    /// Mixed workload
    Mixed,
    /// Stress test (maximum load)
    Stress,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    /// Human readable table
    Table,
    /// JSON output
    Json,
    /// CSV output
    Csv,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    println!("{}", "MERKLITH Performance Benchmark".bright_cyan().bold());
    println!("{}", "═══════════════════════════".bright_cyan());
    println!("RPC Endpoint: {}", args.rpc.bright_yellow());
    println!("Test Type: {:?}", args.test);
    println!("Transactions: {}", args.transactions);
    println!("Concurrency: {}", args.concurrency);
    println!();
    
    // Verify node is running
    if let Err(e) = verify_node(&args.rpc).await {
        eprintln!("{}", format!("Failed to connect to node: {}", e).bright_red());
        std::process::exit(1);
    }
    
    println!("{}", "Node connection verified".bright_green());
    println!();
    
    // Run benchmark based on test type
    let results = match args.test {
        TestType::Transfer => benchmark_transfers(&args).await?,
        TestType::Deploy => benchmark_deploy(&args).await?,
        TestType::Mixed => benchmark_mixed(&args).await?,
        TestType::Stress => benchmark_stress(&args).await?,
    };
    
    // Display results
    match args.format {
        OutputFormat::Table => display_table(&results),
        OutputFormat::Json => display_json(&results)?,
        OutputFormat::Csv => display_csv(&results)?,
    }
    
    // Recommendations
    println!("\n{}", "Recommendations".bright_green().bold());
    if results.tps < 10.0 {
        println!("⚠️  Low TPS detected. Consider:");
        println!("   - Increasing gas price");
        println!("   - Reducing concurrent transactions");
        println!("   - Checking node resources");
    } else if results.tps > 100.0 {
        println!("✅ Excellent performance!");
    } else {
        println!("✓ Acceptable performance");
    }
    
    Ok(())
}

#[derive(Debug)]
struct BenchmarkResults {
    test_type: String,
    total_transactions: usize,
    successful: usize,
    failed: usize,
    total_duration: Duration,
    min_latency: Duration,
    max_latency: Duration,
    avg_latency: Duration,
    tps: f64,
}

async fn benchmark_transfers(args: &Args) -> anyhow::Result<BenchmarkResults> {
    println!("{}", "Running Transfer Benchmark...".bright_yellow());
    
    let pb = ProgressBar::new(args.transactions as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );
    
    let start = Instant::now();
    let mut latencies: Vec<Duration> = Vec::new();
    let mut successful = 0;
    let mut failed = 0;
    
    // Create batches based on concurrency
    let batch_size = args.transactions / args.concurrency;
    let mut handles = Vec::new();
    
    for i in 0..args.concurrency {
        let rpc = args.rpc.clone();
        let pb = pb.clone();
        let count = if i == args.concurrency - 1 {
            args.transactions - (batch_size * i)
        } else {
            batch_size
        };
        
        let handle = tokio::spawn(async move {
            let mut local_success = 0;
            let mut local_latencies = Vec::new();
            
            for _ in 0..count {
                let tx_start = Instant::now();
                
                // Simulate transfer (in real implementation, would sign and send)
                match send_transfer(&rpc).await {
                    Ok(_) => {
                        local_success += 1;
                        local_latencies.push(tx_start.elapsed());
                    }
                    Err(_) => {
                        // Failed
                    }
                }
                
                pb.inc(1);
            }
            
            (local_success, local_latencies)
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        let (s, l) = handle.await?;
        successful += s;
        latencies.extend(l);
    }
    
    pb.finish_with_message("Done");
    
    let total_duration = start.elapsed();
    failed = args.transactions - successful;
    
    // Calculate statistics
    let avg_latency = if !latencies.is_empty() {
        Duration::from_nanos(
            (latencies.iter().map(|d| d.as_nanos() as u64).sum::<u64>() / latencies.len() as u64)
        )
    } else {
        Duration::from_secs(0)
    };
    
    let min_latency = latencies.iter().min().copied().unwrap_or(Duration::from_secs(0));
    let max_latency = latencies.iter().max().copied().unwrap_or(Duration::from_secs(0));
    
    let tps = successful as f64 / total_duration.as_secs_f64();
    
    Ok(BenchmarkResults {
        test_type: "Transfer".to_string(),
        total_transactions: args.transactions,
        successful,
        failed,
        total_duration,
        min_latency,
        max_latency,
        avg_latency,
        tps,
    })
}

async fn benchmark_deploy(_args: &Args) -> anyhow::Result<BenchmarkResults> {
    // Placeholder implementation
    Ok(BenchmarkResults {
        test_type: "Deploy".to_string(),
        total_transactions: 0,
        successful: 0,
        failed: 0,
        total_duration: Duration::from_secs(0),
        min_latency: Duration::from_secs(0),
        max_latency: Duration::from_secs(0),
        avg_latency: Duration::from_secs(0),
        tps: 0.0,
    })
}

async fn benchmark_mixed(_args: &Args) -> anyhow::Result<BenchmarkResults> {
    // Placeholder implementation
    Ok(BenchmarkResults {
        test_type: "Mixed".to_string(),
        total_transactions: 0,
        successful: 0,
        failed: 0,
        total_duration: Duration::from_secs(0),
        min_latency: Duration::from_secs(0),
        max_latency: Duration::from_secs(0),
        avg_latency: Duration::from_secs(0),
        tps: 0.0,
    })
}

async fn benchmark_stress(args: &Args) -> anyhow::Result<BenchmarkResults> {
    println!("{}", "Running Stress Test...".bright_red().bold());
    println!("⚠️  This will send maximum load to the node");
    
    // Run for specified duration or default 30s
    let duration = args.duration.unwrap_or(30);
    
    let start = Instant::now();
    let mut successful = 0;
    let mut failed = 0;
    
    let deadline = start + Duration::from_secs(duration);
    
    while Instant::now() < deadline {
        match timeout(
            Duration::from_millis(100),
            send_transfer(&args.rpc)
        ).await {
            Ok(Ok(_)) => successful += 1,
            Ok(Err(_)) => failed += 1,
            Err(_) => failed += 1, // Timeout
        }
    }
    
    let total_duration = start.elapsed();
    let total = successful + failed;
    let tps = total as f64 / total_duration.as_secs_f64();
    
    Ok(BenchmarkResults {
        test_type: "Stress".to_string(),
        total_transactions: total,
        successful,
        failed,
        total_duration,
        min_latency: Duration::from_secs(0),
        max_latency: Duration::from_secs(0),
        avg_latency: Duration::from_secs(0),
        tps,
    })
}

async fn send_transfer(rpc_url: &str) -> anyhow::Result<()> {
    // Placeholder - would actually send a transaction
    // For now, just check if node is alive
    let client = reqwest::Client::new();
    let response = client
        .post(rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("RPC call failed"))
    }
}

async fn verify_node(rpc_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_chainId",
            "params": [],
            "id": 1
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Node not responding"))
    }
}

fn display_table(results: &BenchmarkResults) {
    println!("\n{}", "Benchmark Results".bright_green().bold());
    println!("{}", "═".repeat(50).bright_green());
    
    println!("{:<25} {}", "Test Type:", results.test_type);
    println!("{:<25} {}", "Total Transactions:", results.total_transactions);
    println!("{:<25} {} {}", "Successful:", results.successful.to_string().bright_green(), format!("({:.1}%)", results.successful as f64 / results.total_transactions as f64 * 100.0));
    println!("{:<25} {} {}", "Failed:", results.failed.to_string().bright_red(), format!("({:.1}%)", results.failed as f64 / results.total_transactions as f64 * 100.0));
    
    println!("\n{}", "Performance".bright_green().bold());
    println!("{:<25} {:.2} TPS", "Throughput:", results.tps);
    println!("{:<25} {:?}", "Total Duration:", results.total_duration);
    
    if results.test_type != "Stress" {
        println!("\n{}", "Latency".bright_green().bold());
        println!("{:<25} {:?}", "Minimum:", results.min_latency);
        println!("{:<25} {:?}", "Maximum:", results.max_latency);
        println!("{:<25} {:?}", "Average:", results.avg_latency);
    }
}

fn display_json(results: &BenchmarkResults) -> anyhow::Result<()> {
    let json = serde_json::json!({
        "test_type": results.test_type,
        "total_transactions": results.total_transactions,
        "successful": results.successful,
        "failed": results.failed,
        "success_rate": results.successful as f64 / results.total_transactions as f64,
        "tps": results.tps,
        "total_duration_ms": results.total_duration.as_millis(),
        "latency_ms": {
            "min": results.min_latency.as_millis(),
            "max": results.max_latency.as_millis(),
            "avg": results.avg_latency.as_millis(),
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn display_csv(results: &BenchmarkResults) -> anyhow::Result<()> {
    println!("test_type,total,successful,failed,tps,duration_ms");
    println!("{},{},{},{},{:.2},{}",
        results.test_type,
        results.total_transactions,
        results.successful,
        results.failed,
        results.tps,
        results.total_duration.as_millis()
    );
    Ok(())
}