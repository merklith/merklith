//! Output formatting utilities.
//!
//! Pretty printing for CLI commands.

use merklith_types::{Address, Hash, U256};
use colored::Colorize;
use tabled::{Table, Tabled};

/// Format address for display.
pub fn format_address(addr: &Address) -> String {
    let s = format!("0x{}", hex::encode(addr.as_bytes()));
    format_address_short(&s)
}

/// Format address (short version).
pub fn format_address_short(addr: &str) -> String {
    if addr.len() > 12 {
        format!("{}...{}", &addr[..10], &addr[addr.len()-8..])
    } else {
        addr.to_string()
    }
}

/// Format hash for display.
pub fn format_hash(hash: &Hash) -> String {
    format!("0x{}", hex::encode(hash.as_bytes()))
}

/// Format U256 as MERK.
pub fn format_merk(value: &U256) -> String {
    // Convert from wei to MERK (18 decimals)
    let wei = value.as_u128();
    let merk = wei as f64 / 1_000_000_000_000_000_000.0;
    
    if merk >= 1.0 {
        format!("{:.4} MERK", merk)
    } else if merk >= 0.001 {
        format!("{:.6} MERK", merk)
    } else {
        format!("{} wei", wei)
    }
}

/// Format wei to human readable.
pub fn format_wei(wei: u128) -> String {
    if wei >= 1_000_000_000_000_000_000 {
        let merk = wei as f64 / 1_000_000_000_000_000_000.0;
        format!("{:.4} MERK", merk)
    } else if wei >= 1_000_000_000_000_000 {
        let millimerc = wei as f64 / 1_000_000_000_000_000.0;
        format!("{:.2} mMERK", millimerc)
    } else if wei >= 1_000_000_000 {
        let gwei = wei as f64 / 1_000_000_000.0;
        format!("{:.2} Gwei", gwei)
    } else {
        format!("{} wei", wei)
    }
}

/// Print success message.
pub fn print_success(msg: &str) {
    println!("{}", format!("✓ {}", msg).green());
}

/// Print error message.
pub fn print_error(msg: &str) {
    eprintln!("{}", format!("✗ {}", msg).red());
}

/// Print warning message.
pub fn print_warning(msg: &str) {
    println!("{}", format!("⚠ {}", msg).yellow());
}

/// Print info message.
pub fn print_info(msg: &str) {
    println!("{}", format!("ℹ {}", msg).blue());
}

/// Print transaction receipt.
pub fn print_transaction_receipt(receipt: serde_json::Value) {
    println!("{}", "Transaction Receipt".bold());
    println!("{}", "=".repeat(50));
    
    if let Some(hash) = receipt.get("transactionHash").and_then(|v| v.as_str()) {
        println!("Transaction Hash: {}", hash.bright_cyan());
    }
    
    if let Some(block) = receipt.get("blockNumber").and_then(|v| v.as_str()) {
        println!("Block Number:     {}", block.bright_green());
    }
    
    if let Some(status) = receipt.get("status").and_then(|v| v.as_str()) {
        let status_str = if status == "0x1" {
            "Success".green()
        } else {
            "Failed".red()
        };
        println!("Status:           {}", status_str);
    }
    
    if let Some(gas) = receipt.get("gasUsed").and_then(|v| v.as_str()) {
        let gas_used = u64::from_str_radix(&gas[2..], 16).unwrap_or(0);
        println!("Gas Used:         {}", gas_used.to_string().bright_yellow());
    }
}

/// Print balance table.
pub fn print_balance_table(balances: &[(String, U256)]) {
    #[derive(Tabled)]
    struct BalanceRow {
        address: String,
        balance: String,
    }

    let rows: Vec<BalanceRow> = balances
        .iter()
        .map(|(addr, bal)| BalanceRow {
            address: format_address_short(addr),
            balance: format_merk(bal),
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
}

/// Print block info.
pub fn print_block_info(block: &serde_json::Value) {
    println!("{}", "Block Information".bold());
    println!("{}", "=".repeat(50));
    
    if let Some(number) = block.get("number").and_then(|v| v.as_str()) {
        println!("Number:       {}", number.bright_green());
    }
    
    if let Some(hash) = block.get("hash").and_then(|v| v.as_str()) {
        println!("Hash:         {}", hash.bright_cyan());
    }
    
    if let Some(parent) = block.get("parentHash").and_then(|v| v.as_str()) {
        println!("Parent Hash:  {}", format_address_short(parent));
    }
    
    if let Some(timestamp) = block.get("timestamp").and_then(|v| v.as_str()) {
        let ts = u64::from_str_radix(&timestamp[2..], 16).unwrap_or(0);
        println!("Timestamp:    {}", ts.to_string().bright_yellow());
    }
    
    if let Some(txs) = block.get("transactions").and_then(|v| v.as_array()) {
        println!("Transactions: {}", txs.len().to_string().bright_magenta());
    }
}

/// Print network info.
pub fn print_network_info(chain_id: u64, block_number: u64, gas_price: U256) {
    println!("{}", "Network Information".bold());
    println!("{}", "=".repeat(50));
    println!("Chain ID:      {}", chain_id.to_string().bright_green());
    println!("Block Number:  {}", block_number.to_string().bright_cyan());
    println!("Gas Price:     {}", format_merk(&gas_price).bright_yellow());
}

/// Create a progress bar.
pub fn create_progress_bar(len: u64) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new(len);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_address() {
        let addr = Address::from_bytes([0u8; 20]);
        let formatted = format_address(&addr);
        assert!(formatted.contains("0x0000"));
    }

    #[test]
    fn test_format_wei() {
        assert_eq!(format_wei(1_000_000_000_000_000_000), "1.0000 MERK");
        assert_eq!(format_wei(1_000_000_000), "1.00 Gwei");
        assert_eq!(format_wei(500), "500 wei");
    }

    #[test]
    fn test_format_merk() {
        let val = U256::from(1_000_000_000_000_000_000u64);
        assert_eq!(format_merk(&val), "1.0000 MERK");
    }
}
