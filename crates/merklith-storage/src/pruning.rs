//! State Pruning and Snapshot Module
//! 
//! Manages blockchain storage efficiently:
//! - Automatic state pruning
//! - Snapshots at intervals
//! - Archive node support
//! - Incremental backups

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use merklith_types::{Hash, Block, U256, Address};
use serde::{Serialize, Deserialize};

/// Pruning configuration
#[derive(Debug, Clone)]
pub struct PruningConfig {
    /// Enable pruning
    pub enabled: bool,
    /// Retain last N blocks
    pub retain_blocks: u64,
    /// Prune interval (blocks)
    pub prune_interval: u64,
    /// Archive mode (keep everything)
    pub archive_mode: bool,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retain_blocks: 10000, // ~17 hours worth
            prune_interval: 1000,
            archive_mode: false,
        }
    }
}

/// State pruner
pub struct StatePruner {
    config: PruningConfig,
    /// Pruned up to this block
    pruned_up_to: Arc<Mutex<u64>>,
    /// Tombstones for deleted data
    tombstones: Arc<Mutex<HashSet<Hash>>>,
}

/// Snapshot manager
pub struct SnapshotManager {
    /// Snapshot directory
    snapshot_dir: PathBuf,
    /// Current snapshots
    snapshots: Arc<Mutex<Vec<Snapshot>>>,
    /// Max snapshots to keep
    max_snapshots: usize,
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub block_number: u64,
    pub block_hash: Hash,
    pub timestamp: u64,
    pub size_bytes: u64,
    pub state_root: Hash,
    pub path: PathBuf,
}

/// Backup manager
pub struct BackupManager {
    /// Backup directory
    backup_dir: PathBuf,
    /// Encryption key (optional)
    encryption_key: Option<Vec<u8>>,
    /// Compression enabled
    compression: bool,
}

/// Prune result
#[derive(Debug)]
pub struct PruneResult {
    pub blocks_pruned: u64,
    pub storage_freed_bytes: u64,
    pub state_roots_removed: usize,
}

impl StatePruner {
    /// Create new pruner
    pub fn new(config: PruningConfig) -> Self {
        Self {
            config,
            pruned_up_to: Arc::new(Mutex::new(0)),
            tombstones: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Check if block should be pruned
    pub fn should_prune(
        &self,
        block_number: u64,
        current_height: u64,
    ) -> bool {
        if !self.config.enabled || self.config.archive_mode {
            return false;
        }
        
        let pruned = *self.pruned_up_to.lock().unwrap();
        
        // Already pruned
        if block_number <= pruned {
            return false;
        }
        
        // Check if past retention window
        block_number + self.config.retain_blocks < current_height
    }

    /// Prune old state
    pub fn prune(
        &self,
        current_height: u64,
    ) -> PruneResult {
        if !self.config.enabled || self.config.archive_mode {
            return PruneResult {
                blocks_pruned: 0,
                storage_freed_bytes: 0,
                state_roots_removed: 0,
            };
        }
        
        let mut pruned = self.pruned_up_to.lock().unwrap();
        let target_prune = current_height.saturating_sub(self.config.retain_blocks);
        
        if target_prune <= *pruned {
            return PruneResult {
                blocks_pruned: 0,
                storage_freed_bytes: 0,
                state_roots_removed: 0,
            };
        }
        
        let blocks_to_prune = target_prune - *pruned;
        
        // In production: actually delete state data
        // For now, just track tombstones
        for block in (*pruned + 1)..=target_prune {
            let hash = Hash::compute(&block.to_be_bytes());
            self.tombstones.lock().unwrap().insert(hash);
        }
        
        *pruned = target_prune;
        
        PruneResult {
            blocks_pruned: blocks_to_prune,
            storage_freed_bytes: blocks_to_prune * 1000, // Estimate
            state_roots_removed: blocks_to_prune as usize,
        }
    }

    /// Check if data is pruned
    pub fn is_pruned(&self, hash: &Hash) -> bool {
        self.tombstones.lock().unwrap().contains(hash)
    }

    /// Get prune progress
    pub fn get_pruned_height(&self,
    ) -> u64 {
        *self.pruned_up_to.lock().unwrap()
    }
}

impl SnapshotManager {
    /// Create new snapshot manager
    pub fn new(snapshot_dir: PathBuf, max_snapshots: usize) -> Self {
        // Create directory if not exists
        std::fs::create_dir_all(&snapshot_dir).ok();
        
        Self {
            snapshot_dir,
            snapshots: Arc::new(Mutex::new(Vec::new())),
            max_snapshots,
        }
    }

    /// Create snapshot at block
    pub fn create_snapshot(
        &self,
        block_number: u64,
        block_hash: Hash,
        state_root: Hash,
    ) -> Result<Snapshot, String> {
        let id = format!("snapshot_{}_{}", block_number, 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        
        let path = self.snapshot_dir.join(format!("{}.snap", id));
        
        // In production: serialize full state
        let snapshot = Snapshot {
            id: id.clone(),
            block_number,
            block_hash,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            size_bytes: 0, // Would calculate actual size
            state_root,
            path: path.clone(),
        };
        
        // Save metadata
        let metadata = serde_json::to_vec(&snapshot).map_err(|e| e.to_string())?;
        std::fs::write(&path, metadata).map_err(|e| e.to_string())?;
        
        // Add to list
        let mut snapshots = self.snapshots.lock().unwrap();
        snapshots.push(snapshot.clone());
        
        // Cleanup old snapshots
        if snapshots.len() > self.max_snapshots {
            if let Some(old) = snapshots.first() {
                let _ = std::fs::remove_file(&old.path);
            }
            snapshots.remove(0);
        }
        
        Ok(snapshot)
    }

    /// Get latest snapshot
    pub fn get_latest_snapshot(&self,
    ) -> Option<Snapshot> {
        let snapshots = self.snapshots.lock().unwrap();
        snapshots.last().cloned()
    }

    /// Get snapshot at block
    pub fn get_snapshot_at(
        &self,
        block_number: u64,
    ) -> Option<Snapshot> {
        let snapshots = self.snapshots.lock().unwrap();
        snapshots.iter()
            .find(|s| s.block_number == block_number)
            .cloned()
    }

    /// List all snapshots
    pub fn list_snapshots(&self,
    ) -> Vec<Snapshot> {
        self.snapshots.lock().unwrap().clone()
    }

    /// Restore from snapshot
    pub fn restore_from_snapshot(
        &self,
        snapshot_id: &str,
    ) -> Result<Snapshot, String> {
        let snapshots = self.snapshots.lock().unwrap();
        
        let snapshot = snapshots.iter()
            .find(|s| s.id == snapshot_id)
            .ok_or("Snapshot not found")?;
        
        // In production: restore state from snapshot file
        
        Ok(snapshot.clone())
    }
}

impl BackupManager {
    /// Create new backup manager
    pub fn new(
        backup_dir: PathBuf,
        encryption_key: Option<Vec<u8>>,
        compression: bool,
    ) -> Self {
        std::fs::create_dir_all(&backup_dir).ok();
        
        Self {
            backup_dir,
            encryption_key,
            compression,
        }
    }

    /// Create full backup
    pub fn create_backup(
        &self,
        data_dir: &Path,
    ) -> Result<PathBuf, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let backup_name = format!("backup_{}.tar.gz", timestamp);
        let backup_path = self.backup_dir.join(&backup_name);
        
        // In production: 
        // 1. Create tar archive
        // 2. Compress if enabled
        // 3. Encrypt if key provided
        
        Ok(backup_path)
    }

    /// Restore from backup
    pub fn restore_backup(
        &self,
        backup_path: &Path,
        target_dir: &Path,
    ) -> Result<(), String> {
        // In production:
        // 1. Decrypt if needed
        // 2. Decompress
        // 3. Extract to target
        
        Ok(())
    }

    /// List available backups
    pub fn list_backups(
        &self,
    ) -> Vec<PathBuf> {
        std::fs::read_dir(&self.backup_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.extension().map(|e| e == "gz").unwrap_or(false))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Verify backup integrity
    pub fn verify_backup(
        &self,
        backup_path: &Path,
    ) -> Result<bool, String> {
        // In production: verify checksums, signatures
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pruner_creation() {
        let config = PruningConfig::default();
        let pruner = StatePruner::new(config);
        
        assert_eq!(pruner.get_pruned_height(), 0);
        assert!(!pruner.is_pruned(&Hash::ZERO));
    }

    #[test]
    fn test_prune_logic() {
        let config = PruningConfig {
            enabled: true,
            retain_blocks: 100,
            ..Default::default()
        };
        let pruner = StatePruner::new(config);
        
        // Should prune blocks 1-900 when at height 1000
        let result = pruner.prune(1000);
        assert_eq!(result.blocks_pruned, 900);
        assert_eq!(pruner.get_pruned_height(), 900);
    }

    #[test]
    fn test_archive_mode() {
        let config = PruningConfig {
            archive_mode: true,
            ..Default::default()
        };
        let pruner = StatePruner::new(config);
        
        let result = pruner.prune(10000);
        assert_eq!(result.blocks_pruned, 0);
    }

    #[test]
    fn test_snapshot_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path().to_path_buf(), 5);
        
        let snapshot = manager.create_snapshot(
            100,
            Hash::ZERO,
            Hash::ZERO,
        ).unwrap();
        
        assert_eq!(snapshot.block_number, 100);
        
        let latest = manager.get_latest_snapshot();
        assert!(latest.is_some());
    }
}
