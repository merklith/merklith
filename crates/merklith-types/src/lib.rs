//! Merklith Types - Core type definitions for the Merklith blockchain.
//!
//! This crate provides the fundamental types used throughout the Merklith blockchain:
//! - Addresses (20-byte, Bech32m encoded)
//! - Hashes (32-byte, blake3 digests)  
//! - U256 (256-bit unsigned integer)
//! - Blocks, Transactions, Receipts
//! - Accounts, Signatures
//! - Genesis and Chain configuration

pub mod address;
pub mod hash;
pub mod u256;
pub mod block;
pub mod transaction;
pub mod receipt;
pub mod account;
pub mod signature;
pub mod genesis;
pub mod chain_config;
pub mod error;
// TODO: hd_wallet requires ed25519-dalek, sha2, hmac, pbkdf2, rand as direct deps
// pub mod hd_wallet;

#[cfg(any(feature = "serde", feature = "borsh"))]
mod serialization;

pub use address::Address;
pub use hash::Hash;
pub use u256::U256;
pub use block::{Block, BlockHeader};
pub use transaction::{Transaction, SignedTransaction, AccessListEntry, TransactionType};
pub use receipt::{TransactionReceipt, Log};
pub use account::{Account, AccountType};
pub use signature::{Ed25519Signature, Ed25519PublicKey, BLSSignature, BLSPublicKey};
pub use genesis::{GenesisConfig, GenesisAlloc, GenesisValidator};
pub use chain_config::ChainConfig;
pub use error::TypesError;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        Address, Hash, U256, Block, BlockHeader,
        Transaction, SignedTransaction, AccessListEntry, TransactionType,
        TransactionReceipt, Log,
        Account, AccountType,
        Ed25519Signature, Ed25519PublicKey,
        BLSSignature, BLSPublicKey,
        ChainConfig, GenesisConfig, TypesError,
    };
}
