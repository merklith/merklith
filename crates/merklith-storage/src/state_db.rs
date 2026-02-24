//! State DB - Persistent account state

use crate::{Database, StorageError};
use merklith_types::{Address, U256};
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::str::FromStr;

/// Account state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccountState {
    pub balance: String,  // hex encoded
    pub nonce: u64,
    pub code_hash: Option<String>,
    pub storage_root: Option<String>,
}

/// State database for accounts
pub struct StateDB {
    db: Arc<Database>,
    accounts: Arc<RwLock<HashMap<[u8; 20], AccountState>>>,
    path: std::path::PathBuf,
}

impl StateDB {
    pub fn new(path: &Path) -> Result<Self, StorageError> {
        let db = Database::new(path)?;
        let state = Self {
            db: Arc::new(db),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            path: path.to_path_buf(),
        };
        state.load_accounts()?;
        Ok(state)
    }
    
    /// Get account balance
    pub fn balance(&self, address: &Address) -> U256 {
        let accounts = self.accounts.read();
        accounts.get(address.as_bytes()).map(|a| {
            U256::from_str(&a.balance).unwrap_or(U256::ZERO)
        }).unwrap_or(U256::ZERO)
    }
    
    /// Set account balance
    pub fn set_balance(&self, address: &Address, balance: U256) -> Result<(), StorageError> {
        let mut accounts = self.accounts.write();
        let account = accounts.entry(*address.as_bytes()).or_insert(AccountState {
            balance: "0x0".to_string(),
            nonce: 0,
            code_hash: None,
            storage_root: None,
        });
        account.balance = format!("0x{}", balance);
        drop(accounts);
        self.persist()
    }
    
    /// Get account nonce
    pub fn nonce(&self, address: &Address) -> u64 {
        let accounts = self.accounts.read();
        accounts.get(address.as_bytes()).map(|a| a.nonce).unwrap_or(0)
    }
    
    /// Set account nonce
    pub fn set_nonce(&self, address: &Address, nonce: u64) -> Result<(), StorageError> {
        let mut accounts = self.accounts.write();
        let account = accounts.entry(*address.as_bytes()).or_insert(AccountState {
            balance: "0x0".to_string(),
            nonce: 0,
            code_hash: None,
            storage_root: None,
        });
        account.nonce = nonce;
        drop(accounts);
        self.persist()
    }
    
    /// Transfer balance between accounts
    pub fn transfer(&self, from: &Address, to: &Address, amount: U256) -> Result<[u8; 32], StorageError> {
        let from_balance = self.balance(from);
        if from_balance < amount {
            return Err(StorageError::NotFound("Insufficient balance".to_string()));
        }
        
        let to_balance = self.balance(to);
        
        self.set_balance(from, from_balance - amount)?;
        self.set_balance(to, to_balance + amount)?;
        
        // Generate tx hash
        let mut hash_bytes = [0u8; 32];
        let nonce = self.nonce(from);
        hash_bytes[..8].copy_from_slice(&nonce.to_le_bytes());
        
        Ok(hash_bytes)
    }
    
    /// Persist accounts to disk
    fn persist(&self) -> Result<(), StorageError> {
        let accounts_file = self.path.join("accounts.json");
        let accounts = self.accounts.read();
        let json = serde_json::to_string_pretty(&*accounts)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        std::fs::write(&accounts_file, json).map_err(|e| StorageError::Io(e.to_string()))?;
        Ok(())
    }
    
    /// Load accounts from disk
    fn load_accounts(&self) -> Result<(), StorageError> {
        let accounts_file = self.path.join("accounts.json");
        if accounts_file.exists() {
            let content = std::fs::read_to_string(&accounts_file)
                .map_err(|e| StorageError::Io(e.to_string()))?;
            let loaded: HashMap<[u8; 20], AccountState> = serde_json::from_str(&content)
                .unwrap_or_default();
            *self.accounts.write() = loaded;
            tracing::info!("Loaded {} accounts from disk", self.accounts.read().len());
        }
        Ok(())
    }
    
    /// Get all accounts
    pub fn all_accounts(&self) -> Vec<([u8; 20], AccountState)> {
        self.accounts.read().iter().map(|(k, v)| (*k, v.clone())).collect()
    }
}
