//! Merklith Rust SDK
//!
//! High-level SDK for interacting with the Merklith blockchain.
//!
//! # Example
//! ```rust,ignore
//! use merklith_sdk::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::connect("http://localhost:8545").await.unwrap();
//!     let block_number = client.get_block_number().await.unwrap();
//!     println!("Current block: {}", block_number);
//! }
//! ```

pub mod client;
pub mod contract;
pub mod errors;
pub mod events;
pub mod types;
pub mod wallet;

pub use client::Client;
pub use contract::Contract;
pub use errors::{SdkError, Result};
pub use types::*;
pub use wallet::Wallet;

/// Re-export merklith-types for convenience
pub use merklith_types::{Address, Hash, Transaction, U256};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_imports() {
        // Just ensure types are importable
        let _addr = Address::ZERO;
        let _hash = Hash::ZERO;
    }
}
