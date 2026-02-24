//! Merklith Storage - Persistent storage with JSON files

pub mod state_db;
pub mod block_store;

use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use parking_lot::RwLock;

/// Storage error
#[derive(Debug, Clone)]
pub enum StorageError {
    Io(String),
    Serialization(String),
    NotFound(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(s) => write!(f, "IO error: {}", s),
            StorageError::Serialization(s) => write!(f, "Serialization error: {}", s),
            StorageError::NotFound(s) => write!(f, "Not found: {}", s),
        }
    }
}

impl std::error::Error for StorageError {}

/// Database - Simple JSON file-based storage
pub struct Database {
    path: PathBuf,
    data: Arc<RwLock<serde_json::Value>>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, StorageError> {
        fs::create_dir_all(path).map_err(|e| StorageError::Io(e.to_string()))?;
        
        let data_file = path.join("data.json");
        let data = if data_file.exists() {
            let content = fs::read_to_string(&data_file)
                .map_err(|e| StorageError::Io(e.to_string()))?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        
        Ok(Self {
            path: path.to_path_buf(),
            data: Arc::new(RwLock::new(data)),
        })
    }
    
    pub fn get(&self, column: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let data = self.data.read();
        let key_hex = hex::encode(key);
        
        if let Some(columns) = data.get(column) {
            if let Some(value) = columns.get(&key_hex) {
                if let Some(str_val) = value.as_str() {
                    return Ok(Some(hex::decode(str_val).map_err(|e| StorageError::Serialization(e.to_string()))?));
                }
            }
        }
        Ok(None)
    }
    
    pub fn put(&self, column: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let key_hex = hex::encode(key);
        let value_hex = hex::encode(value);
        
        // Clone data for persistence (to avoid holding lock during I/O)
        let data_to_persist = {
            let mut data = self.data.write();
            
            if let Some(columns) = data.get_mut(column) {
                if let Some(obj) = columns.as_object_mut() {
                    obj.insert(key_hex.clone(), serde_json::json!(value_hex));
                }
            } else {
                let mut map = serde_json::Map::new();
                map.insert(key_hex, serde_json::json!(value_hex));
                if let Some(root) = data.as_object_mut() {
                    root.insert(column.to_string(), serde_json::json!(map));
                } else {
                    *data = serde_json::json!({column: map});
                }
            }
            
            data.clone()
        }; // Lock released here
        
        self.persist(&data_to_persist)?;
        Ok(())
    }
    
    pub fn delete(&self, column: &str, key: &[u8]) -> Result<(), StorageError> {
        let key_hex = hex::encode(key);
        
        // Clone data for persistence
        let data_to_persist = {
            let mut data = self.data.write();
            
            if let Some(columns) = data.get_mut(column) {
                if let Some(obj) = columns.as_object_mut() {
                    obj.remove(&key_hex);
                }
            }
            
            data.clone()
        }; // Lock released here
        
        self.persist(&data_to_persist)?;
        Ok(())
    }
    
    fn persist(&self, data: &serde_json::Value) -> Result<(), StorageError> {
        let data_file = self.path.join("data.json");
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(&data_file, content).map_err(|e| StorageError::Io(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let _db = Database::new(temp_dir.path()).unwrap();
        
        // Database directory should be created successfully
        assert!(temp_dir.path().exists());
        // data.json is only created when data is written
    }

    #[test]
    fn test_database_put_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path()).unwrap();
        
        let column = "test_column";
        let key = b"test_key";
        let value = b"test_value";
        
        // Put value
        db.put(column, key, value).unwrap();
        
        // Get value
        let retrieved = db.get(column, key).unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));
    }

    #[test]
    fn test_database_get_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path()).unwrap();
        
        let column = "test_column";
        let key = b"nonexistent_key";
        
        // Get non-existent value
        let retrieved = db.get(column, key).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_database_delete() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path()).unwrap();
        
        let column = "test_column";
        let key = b"test_key";
        let value = b"test_value";
        
        // Put value
        db.put(column, key, value).unwrap();
        
        // Verify it exists
        assert!(db.get(column, key).unwrap().is_some());
        
        // Delete value
        db.delete(column, key).unwrap();
        
        // Verify it's gone
        let retrieved = db.get(column, key).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_database_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let column = "test_column";
        let key = b"test_key";
        let value = b"test_value";
        
        // Create database and put value
        {
            let db = Database::new(temp_dir.path()).unwrap();
            db.put(column, key, value).unwrap();
        }
        
        // Create new database instance (should load from disk)
        {
            let db = Database::new(temp_dir.path()).unwrap();
            let retrieved = db.get(column, key).unwrap();
            assert_eq!(retrieved, Some(value.to_vec()));
        }
    }

    #[test]
    fn test_database_multiple_columns() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path()).unwrap();
        
        // Put values in different columns
        db.put("column1", b"key1", b"value1").unwrap();
        db.put("column2", b"key2", b"value2").unwrap();
        
        // Retrieve values
        assert_eq!(db.get("column1", b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get("column2", b"key2").unwrap(), Some(b"value2".to_vec()));
        
        // Ensure values are in correct columns
        assert_eq!(db.get("column1", b"key2").unwrap(), None);
        assert_eq!(db.get("column2", b"key1").unwrap(), None);
    }

    #[test]
    fn test_database_overwrite_value() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path()).unwrap();
        
        let column = "test_column";
        let key = b"test_key";
        
        // Put initial value
        db.put(column, key, b"initial_value").unwrap();
        
        // Overwrite with new value
        db.put(column, key, b"new_value").unwrap();
        
        // Verify new value
        let retrieved = db.get(column, key).unwrap();
        assert_eq!(retrieved, Some(b"new_value".to_vec()));
    }

    #[test]
    fn test_storage_error_display() {
        let io_error = StorageError::Io("test io error".to_string());
        assert!(format!("{}", io_error).contains("IO error"));
        
        let serialization_error = StorageError::Serialization("test serialization error".to_string());
        assert!(format!("{}", serialization_error).contains("Serialization error"));
        
        let not_found_error = StorageError::NotFound("test not found".to_string());
        assert!(format!("{}", not_found_error).contains("Not found"));
    }
}
