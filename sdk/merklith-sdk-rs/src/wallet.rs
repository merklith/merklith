//! Wallet management for SDK.

use merklith_crypto::ed25519::Keypair as Ed25519Keypair;
use merklith_types::{Address, Transaction, SignedTransaction};
use crate::errors::{Result, SdkError};

/// Wallet for signing transactions.
#[derive(Debug)]
pub struct Wallet {
    keypair: Ed25519Keypair,
    address: Address,
}

impl Wallet {
    /// Create a new random wallet.
    pub fn new() -> Self {
        let keypair = Ed25519Keypair::generate();
        let address = keypair.address();
        Self { keypair, address }
    }

    /// Load wallet from private key bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let keypair = Ed25519Keypair::from_seed(bytes);
        let address = keypair.address();
        Ok(Self { keypair, address })
    }

    /// Load wallet from hex string.
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches("0x");
        let bytes = hex::decode(hex)
            .map_err(|e| SdkError::Wallet(format!("Invalid hex: {}", e)))?;
        
        if bytes.len() != 32 {
            return Err(SdkError::Wallet("Invalid key length".to_string()));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);
        
        Self::from_bytes(&key_bytes)
    }

    /// Get the wallet address.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Sign a transaction.
    pub fn sign_transaction(
        &self,
        tx: Transaction,
    ) -> Result<SignedTransaction> {
        // Sign the transaction hash
        let signature = self.keypair.sign(tx.signing_hash().as_bytes());
        let public_key = self.keypair.public_key();
        
        // Create signed transaction
        Ok(SignedTransaction::new(tx, signature, public_key))
    }

    /// Sign a message.
    pub fn sign_message(
        &self,
        message: &[u8],
    ) -> Result<Vec<u8>> {
        let signature = self.keypair.sign(message);
        
        // Serialize signature (Ed25519 signatures are 64 bytes)
        Ok(signature.as_bytes().to_vec())
    }

    /// Get private key bytes (careful!).
    pub fn private_key(&self) -> [u8; 32] {
        self.keypair.to_bytes()
    }

    /// Export to hex string.
    pub fn export_hex(&self,
    ) -> String {
        format!("0x{}", hex::encode(self.private_key()))
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(!wallet.address().is_zero());
    }

    #[test]
    fn test_wallet_from_hex() {
        let wallet = Wallet::new();
        let exported = wallet.export_hex();
        
        // Import back
        let imported = Wallet::from_hex(&exported).unwrap();
        assert_eq!(wallet.address(), imported.address());
    }

    #[test]
    fn test_wallet_sign_message() {
        let wallet = Wallet::new();
        let message = b"Hello, Merklith!";
        
        let signature = wallet.sign_message(message).unwrap();
        assert!(!signature.is_empty());
    }
}
