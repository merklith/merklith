//! Merklith CLI - Command-line interface for Merklith blockchain.
//!
//! This crate provides the CLI tool for interacting with the Merklith network.

pub mod commands;
pub mod rpc_client;
pub mod output;
pub mod config;
pub mod keystore;
pub mod explorer;
pub mod tests;

use clap::Parser;
use colored::Colorize;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Print banner
    print_banner();

    // Parse CLI arguments
    let cli = commands::Cli::parse();

    // Execute command
    let cmd = cli.command;
    let _rpc = cli.rpc; // Extract before dropping cli
    if let Err(e) = commands::execute(cmd, _rpc).await {
        eprintln!("{}", format!("Error: {}", e).red());
        std::process::exit(1);
    }

    Ok(())
}

fn print_banner() {
    println!();
    println!(r#"    _                _ _           _   _           _      _       _   _      _    _"#);
    println!(r#"   / \   _ __  _ __ | (_) ___ __ _| |_(_)_ __   __| |    | |     | \ | | ___| | _(_) ___ _ __   ___ _ __"#);
    println!(r#"  / _ \ | '_ \| '_ \| | |/ __/ _` | __| | '_ \ / _` |    | |     |  \| |/ _ \ |/ / |/ _ \ '_ \ / _ \ '__|"#);
    println!(r#" / ___ \| | | | | | | | | (_| (_| | |_| | | | | (_| |    | |___  | |\  |  __/   <| |  __/ | | |  __/ |"#);
    println!(r#"/_/   \_\_| |_|_| |_|_|_|\___\__,_|\__|_|_| |_|\__,_|    |_____| |_| \_|\___|_|\_\_|\___|_| |_|\___|_|"#);
    println!();
    println!("                    {}", "Where Trust is Forged".bright_cyan().italic());
    println!();
}
