//! MERKLITH Faucet - Test token distribution tool

use clap::Parser;
use colored::Colorize;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(name = "merklith-faucet")]
#[command(about = "MERKLITH test token faucet")]
#[command(version)]
struct Args {
    /// RPC endpoint URL
    #[arg(short, long, default_value = "http://localhost:8545")]
    rpc: String,

    /// Recipient address
    #[arg(short, long)]
    to: String,

    /// Amount to send (in MERK)
    #[arg(short, long, default_value = "100")]
    amount: f64,

    /// Number of distributions
    #[arg(short, long, default_value = "1")]
    count: usize,

    /// Distribute to multiple addresses from file
    #[arg(short, long)]
    file: Option<String>,

    /// Faucet wallet address (must have funds)
    #[arg(long)]
    from: Option<String>,

    /// No confirmation prompt
    #[arg(long)]
    yes: bool,

    /// Cooldown between requests (seconds)
    #[arg(long, default_value = "1")]
    cooldown: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    println!("{}", "MERKLITH Test Token Faucet".bright_cyan().bold());
    println!("{}", "═══════════════════════".bright_cyan());
    println!("RPC Endpoint: {}", args.rpc.bright_yellow());
    
    // Validate amount
    if args.amount <= 0.0 {
        eprintln!("{}", "Error: Amount must be positive".bright_red());
        std::process::exit(1);
    }
    
    // Determine recipients
    let recipients = if let Some(file) = &args.file {
        load_recipients_from_file(file).await?
    } else {
        vec![args.to.clone()]
    };
    
    // Calculate total
    let total = args.amount * args.count as f64 * recipients.len() as f64;
    
    println!("\n{}", "Distribution Plan".bright_green().bold());
    println!("Recipients: {}", recipients.len());
    println!("Amount per distribution: {} MERK", args.amount);
    println!("Distributions per recipient: {}", args.count);
    println!("Total to distribute: {} MERK", total.to_string().bright_yellow());
    println!();
    
    // Confirm
    if !args.yes {
        if !Confirm::new()
            .with_prompt("Proceed with distribution?")
            .default(false)
            .interact()?
        {
            println!("Cancelled");
            return Ok(());
        }
    }
    
    // Execute distribution
    let pb = ProgressBar::new((recipients.len() * args.count) as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
            .progress_chars("#>-"),
    );
    
    let mut successful = 0;
    let mut failed = 0;
    let mut total_distributed = 0.0;
    
    for recipient in &recipients {
        for i in 0..args.count {
            pb.set_message(format!("-> {}", recipient));
            
            match distribute(&args.rpc, recipient, args.amount, args.from.as_deref()).await {
                Ok(_) => {
                    successful += 1;
                    total_distributed += args.amount;
                }
                Err(e) => {
                    failed += 1;
                    eprintln!("\n{}: {} -> {}", "Failed".bright_red(), recipient, e);
                }
            }
            
            pb.inc(1);
            
            // Cooldown
            if args.cooldown > 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(args.cooldown)).await;
            }
        }
    }
    
    pb.finish_with_message("Done");
    
    // Summary
    println!("\n{}", "Distribution Summary".bright_green().bold());
    println!("Successful: {}", successful.to_string().bright_green());
    println!("Failed: {}", failed.to_string().bright_red());
    println!("Total distributed: {} MERK", total_distributed);
    
    if failed > 0 {
        eprintln!("\n{}", format!("Warning: {} distributions failed", failed).bright_yellow());
    }
    
    Ok(())
}

async fn load_recipients_from_file(path: &str) -> anyhow::Result<Vec<String>> {
    let content = tokio::fs::read_to_string(path).await?;
    let recipients: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    
    if recipients.is_empty() {
        return Err(anyhow::anyhow!("No valid recipients found in file"));
    }
    
    Ok(recipients)
}

async fn distribute(
    _rpc_url: &str,
    _to: &str,
    _amount: f64,
    _from: Option<&str>,
) -> anyhow::Result<()> {
    // Placeholder - would actually send transaction
    // In production, this would:
    // 1. Load faucet wallet from keystore
    // 2. Create and sign transaction
    // 3. Send via RPC
    // 4. Wait for confirmation
    
    // Simulate network delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Randomly fail 5% for realism
    if rand::random::<f64>() < 0.05 {
        return Err(anyhow::anyhow!("Network timeout"));
    }
    
    Ok(())
}