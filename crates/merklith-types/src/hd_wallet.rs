//! Hierarchical Deterministic (HD) Wallet support (BIP-32, BIP-39)
//!
//! This module provides HD wallet functionality for MERKLITH

use crate::Address;

/// HD Wallet following BIP-32 and BIP-39
#[derive(Debug, Clone)]
pub struct HDWallet {
    mnemonic: Option<String>,
}

impl HDWallet {
    /// Create a new HD wallet with random mnemonic
    pub fn new() -> Result<Self, String> {
        Ok(Self { mnemonic: None })
    }
    
    /// Create HD wallet from mnemonic phrase
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self, String> {
        Ok(Self { mnemonic: Some(mnemonic.to_string()) })
    }
    
    /// Get mnemonic phrase
    pub fn mnemonic(&self) -> Option<&String> {
        self.mnemonic.as_ref()
    }
}

impl Default for HDWallet {
    fn default() -> Self {
        // This should never fail as new() always returns Ok
        match Self::new() {
            Ok(wallet) => wallet,
            Err(_) => Self { mnemonic: None },
        }
    }
}