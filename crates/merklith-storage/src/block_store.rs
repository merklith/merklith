//! Block Store - Persistent block storage

use crate::StorageError;
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

/// Block store for persistent block storage
pub struct BlockStore {
    path: PathBuf,
    blocks: Arc<RwLock<HashMap<u64, Vec<u8>>>>, // number -> raw data
    latest: Arc<RwLock<u64>>,
}

impl BlockStore {
    pub fn new(path: &std::path::Path) -> Result<Self, StorageError> {
        fs::create_dir_all(path).map_err(|e| StorageError::Io(e.to_string()))?;
        
        let store = Self {
            path: path.to_path_buf(),
            blocks: Arc::new(RwLock::new(HashMap::new())),
            latest: Arc::new(RwLock::new(0)),
        };
        
        store.load_from_disk()?;
        Ok(store)
    }
    
    /// Add a block to the store
    pub fn add_block(&self, number: u64, hash: [u8; 32], data: Vec<u8>) -> Result<(), StorageError> {
        // Store in memory
        self.blocks.write().insert(number, data.clone());
        
        // Update latest
        let mut latest = self.latest.write();
        if number > *latest {
            *latest = number;
        }
        
        // Persist to disk
        let block_file = self.path.join(format!("block_{:012}.bin", number));
        fs::write(&block_file, &data).map_err(|e| StorageError::Io(e.to_string()))?;
        
        // Also save hash file
        let hash_file = self.path.join(format!("block_{:012}.hash", number));
        fs::write(&hash_file, hash).map_err(|e| StorageError::Io(e.to_string()))?;
        
        // Update latest file
        let latest_file = self.path.join("latest");
        fs::write(&latest_file, number.to_string()).map_err(|e| StorageError::Io(e.to_string()))?;
        
        tracing::debug!("Block #{} persisted to disk", number);
        Ok(())
    }
    
    /// Get block by number
    pub fn get_block(&self, number: u64) -> Option<Vec<u8>> {
        // Try memory first
        if let Some(data) = self.blocks.read().get(&number) {
            return Some(data.clone());
        }
        
        // Try disk
        let block_file = self.path.join(format!("block_{:012}.bin", number));
        if block_file.exists() {
            if let Ok(data) = fs::read(&block_file) {
                self.blocks.write().insert(number, data.clone());
                return Some(data);
            }
        }
        
        None
    }
    
    /// Get latest block number
    pub fn latest_number(&self) -> u64 {
        *self.latest.read()
    }
    
    /// Get block count
    pub fn count(&self) -> usize {
        self.blocks.read().len()
    }
    
    /// Load blocks from disk
    fn load_from_disk(&self) -> Result<(), StorageError> {
        // Read latest
        let latest_file = self.path.join("latest");
        let latest = if latest_file.exists() {
            let content = fs::read_to_string(&latest_file)
                .map_err(|e| StorageError::Io(e.to_string()))?;
            content.trim().parse().unwrap_or(0)
        } else {
            0
        };
        
        *self.latest.write() = latest;
        tracing::info!("Latest block from disk: {}", latest);
        Ok(())
    }
}
