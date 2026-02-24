//! Keystore for encrypted wallet storage.
//!
//! Wallets are encrypted with AES-256-GCM using a key derived from the password
//! with Argon2id.

use merklith_crypto::keystore::{encrypt_keystore, decrypt_keystore};
use merklith_types::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Keystore entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreEntry {
    /// Wallet name
    pub name: String,
    /// Wallet address
    pub address: Address,
    /// Creation timestamp
    pub created_at: u64,
    /// Whether this is the default wallet
    pub is_default: bool,
}

/// In-memory keystore
#[derive(Debug, Default)]
pub struct Keystore {
    /// Keystore directory
    dir: PathBuf,
    /// Loaded entries
    entries: HashMap<Address, KeystoreEntry>,
}

impl Keystore {
    /// Create or load keystore from directory
    pub fn new(dir: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(&dir)?;
        
        let mut keystore = Self {
            dir,
            entries: HashMap::new(),
        };
        
        keystore.load_entries()?;
        Ok(keystore)
    }
    
    /// Load all keystore entries from disk
    fn load_entries(&mut self) -> anyhow::Result<()> {
        let index_path = self.dir.join("index.json");
        if index_path.exists() {
            let contents = fs::read_to_string(&index_path)?;
            let entries: Vec<KeystoreEntry> = serde_json::from_str(&contents)?;
            for entry in entries {
                self.entries.insert(entry.address, entry);
            }
        }
        Ok(())
    }
    
    /// Save index to disk
    fn save_index(&self) -> anyhow::Result<()> {
        let index_path = self.dir.join("index.json");
        let entries: Vec<&KeystoreEntry> = self.entries.values().collect();
        let contents = serde_json::to_string_pretty(&entries)?;
        fs::write(index_path, contents)?;
        Ok(())
    }
    
    /// Save encrypted wallet to keystore
    pub fn save_wallet(
        &mut self,
        name: &str,
        address: Address,
        private_key: &[u8; 32],
        password: &str,
        is_default: bool,
    ) -> anyhow::Result<()> {
        // Save to file using merklith-crypto
        let wallet_file = self.dir.join(format!("{}.json", hex::encode(address.as_bytes())));
        encrypt_keystore(private_key, password, &wallet_file)?;
        
        // Update index
        let entry = KeystoreEntry {
            name: name.to_string(),
            address,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            is_default,
        };
        
        // If this is default, unset others
        if is_default {
            for e in self.entries.values_mut() {
                e.is_default = false;
            }
        }
        
        self.entries.insert(address, entry);
        self.save_index()?;
        
        Ok(())
    }
    
    /// Load and decrypt wallet from keystore
    pub fn load_wallet(
        &self,
        address: &Address,
        password: &str,
    ) -> anyhow::Result<[u8; 32]> {
        let wallet_file = self.dir.join(format!("{}.json", hex::encode(address.as_bytes())));
        let private_key = decrypt_keystore(&wallet_file, password)?;
        Ok(private_key)
    }
    
    /// Remove wallet from keystore
    pub fn remove_wallet(&mut self, address: &Address) -> anyhow::Result<()> {
        if let Some(_) = self.entries.remove(address) {
            let wallet_file = self.dir.join(format!("{}.json", hex::encode(address.as_bytes())));
            if wallet_file.exists() {
                fs::remove_file(&wallet_file)?;
            }
            self.save_index()?;
        }
        Ok(())
    }
    
    /// List all wallets
    pub fn list_wallets(&self) -> Vec<&KeystoreEntry> {
        self.entries.values().collect()
    }
    
    /// Get wallet by address
    pub fn get_wallet(&self, address: &Address) -> Option<&KeystoreEntry> {
        self.entries.get(address)
    }
    
    /// Get default wallet
    pub fn get_default(&self) -> Option<&KeystoreEntry> {
        self.entries.values().find(|e| e.is_default)
    }
    
    /// Set default wallet
    pub fn set_default(&mut self, address: &Address) -> anyhow::Result<()> {
        // First check if wallet exists
        if !self.entries.contains_key(address) {
            anyhow::bail!("Wallet not found");
        }
        
        // Reset all to false
        for e in self.entries.values_mut() {
            e.is_default = false;
        }
        
        // Set target to true
        if let Some(entry) = self.entries.get_mut(address) {
            entry.is_default = true;
            self.save_index()?;
        }
        
        Ok(())
    }
    
    /// Check if wallet exists
    pub fn has_wallet(&self, address: &Address) -> bool {
        self.entries.contains_key(address)
    }
}