//! Blockchain Audit System
//!
//! Provides complete audit trail for:
//! - All transactions
//! - Block production
//! - Validator actions
//! - State changes
//! - Security events
//!
//! Features:
//! - Immutable audit log
//! - Tamper detection with hashes
//! - Efficient querying
//! - Export capabilities

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use sha3::{Sha3_256, Digest};

/// Audit event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    // Transaction events
    TransactionSubmitted,
    TransactionValidated,
    TransactionExecuted,
    TransactionFailed,
    TransactionReverted,
    
    // Block events
    BlockProposed,
    BlockValidated,
    BlockFinalized,
    BlockRejected,
    
    // Validator events
    ValidatorRegistered,
    ValidatorStaked,
    ValidatorUnstaked,
    ValidatorSlashed,
    ValidatorRewarded,
    ValidatorJailed,
    ValidatorUnjailed,
    
    // Consensus events
    VoteCast,
    ProposalSubmitted,
    FinalityReached,
    ForkDetected,
    
    // State events
    StateChanged,
    BalanceUpdated,
    ContractDeployed,
    ContractCalled,
    
    // Security events
    RateLimitTriggered,
    SuspiciousActivity,
    AccessDenied,
}

/// Audit event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Complete audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    pub event_type: AuditEventType,
    /// Timestamp (Unix seconds)
    pub timestamp: u64,
    /// Block number (if applicable)
    pub block_number: Option<u64>,
    /// Transaction hash (if applicable)
    pub tx_hash: Option<String>,
    /// Actor address (validator, user, etc.)
    pub actor: String,
    /// Event description
    pub description: String,
    /// Additional data (JSON)
    pub data: HashMap<String, serde_json::Value>,
    /// Severity level
    pub severity: AuditSeverity,
    /// Previous event hash (for chain integrity)
    pub prev_hash: String,
    /// This event's hash
    pub hash: String,
}

impl AuditEvent {
    /// Create new audit event
    pub fn new(
        event_type: AuditEventType,
        actor: String,
        description: String,
        severity: AuditSeverity,
    ) -> Self {
        let timestamp = current_timestamp();
        let id = format!("{}-{}", timestamp, generate_nonce());
        
        let mut event = Self {
            id,
            event_type,
            timestamp,
            block_number: None,
            tx_hash: None,
            actor,
            description,
            data: HashMap::new(),
            severity,
            prev_hash: String::new(),
            hash: String::new(),
        };
        
        event.hash = event.calculate_hash();
        event
    }
    
    /// Calculate event hash
    fn calculate_hash(&self) -> String {
        let mut hasher = Sha3_256::new();
        
        let data = format!(
            "{}:{}:{}:{}:{}:{}",
            self.id,
            self.timestamp,
            format!("{:?}", self.event_type),
            self.actor,
            self.description,
            self.prev_hash
        );
        
        hasher.update(data.as_bytes());
        
        // Include data hash
        if !self.data.is_empty() {
            let data_json = serde_json::to_string(&self.data).unwrap_or_default();
            hasher.update(data_json.as_bytes());
        }
        
        format!("0x{:x}", hasher.finalize())
    }
    
    /// Verify event integrity
    pub fn verify(&self) -> bool {
        self.hash == self.calculate_hash()
    }
    
    /// Set block number
    pub fn with_block(mut self, block_number: u64) -> Self {
        self.block_number = Some(block_number);
        self.hash = self.calculate_hash();
        self
    }
    
    /// Set transaction hash
    pub fn with_tx(mut self, tx_hash: String) -> Self {
        self.tx_hash = Some(tx_hash);
        self.hash = self.calculate_hash();
        self
    }
    
    /// Add data field
    pub fn with_data(mut self, key: &str, value: serde_json::Value) -> Self {
        self.data.insert(key.to_string(), value);
        self.hash = self.calculate_hash();
        self
    }
    
    /// Set previous hash (chain link)
    pub fn with_prev_hash(mut self, prev_hash: String) -> Self {
        self.prev_hash = prev_hash;
        self.hash = self.calculate_hash();
        self
    }
}

/// Audit trail for the blockchain
pub struct AuditTrail {
    /// All events in chronological order
    events: Arc<Mutex<Vec<AuditEvent>>>,
    /// Events indexed by block number
    events_by_block: Arc<Mutex<HashMap<u64, Vec<String>>>,
    /// Events indexed by transaction hash
    events_by_tx: Arc<Mutex<HashMap<String, Vec<String>>>,
    /// Events indexed by actor
    events_by_actor: Arc<Mutex<HashMap<String, Vec<String>>>,
    /// Last event hash (for chain integrity)
    last_hash: Arc<Mutex<String>>,
    /// Event counters
    counters: Arc<Mutex<HashMap<AuditEventType, u64>>>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            events_by_block: Arc::new(Mutex::new(HashMap::new())),
            events_by_tx: Arc::new(Mutex::new(HashMap::new())),
            events_by_actor: Arc::new(Mutex::new(HashMap::new())),
            last_hash: Arc::new(Mutex::new(String::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Record an event
    pub fn record(&self, mut event: AuditEvent) -> Result<(), AuditError> {
        // Link to previous event
        let last = self.last_hash.lock().map_err(|_| AuditError::LockError)?;
        event.prev_hash = last.clone();
        event.hash = event.calculate_hash();
        drop(last);
        
        // Update last hash
        let mut last = self.last_hash.lock().map_err(|_| AuditError::LockError)?;
        *last = event.hash.clone();
        drop(last);
        
        // Add to main log
        let mut events = self.events.lock().map_err(|_| AuditError::LockError)?;
        let event_id = event.id.clone();
        events.push(event.clone());
        drop(events);
        
        // Index by block
        if let Some(block_num) = event.block_number {
            let mut by_block = self.events_by_block.lock().map_err(|_| AuditError::LockError)?;
            by_block
                .entry(block_num)
                .or_insert_with(Vec::new)
                .push(event_id.clone());
        }
        
        // Index by transaction
        if let Some(tx_hash) = &event.tx_hash {
            let mut by_tx = self.events_by_tx.lock().map_err(|_| AuditError::LockError)?;
            by_tx
                .entry(tx_hash.clone())
                .or_insert_with(Vec::new)
                .push(event_id.clone());
        }
        
        // Index by actor
        let mut by_actor = self.events_by_actor.lock().map_err(|_| AuditError::LockError)?;
        by_actor
            .entry(event.actor.clone())
            .or_insert_with(Vec::new)
            .push(event_id.clone());
        
        // Update counters
        let mut counters = self.counters.lock().map_err(|_| AuditError::LockError)?;
        *counters.entry(event.event_type).or_insert(0) += 1;
        
        // Log critical events immediately
        if matches!(event.severity, AuditSeverity::Critical) {
            tracing::error!(
                "CRITICAL AUDIT EVENT: {:?} - {} - {}",
                event.event_type,
                event.actor,
                event.description
            );
        }
        
        Ok(())
    }
    
    /// Get all events
    pub fn get_all_events(&self,
        limit: Option<usize>,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        
        let mut result: Vec<AuditEvent> = events.clone();
        
        if let Some(lim) = limit {
            result.truncate(lim);
        }
        
        Ok(result)
    }
    
    /// Get events by block
    pub fn get_events_by_block(
        &self,
        block_number: u64,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let by_block = self.events_by_block.lock().map_err(|_| AuditError::LockError)?;
        let event_ids = by_block.get(&block_number).cloned().unwrap_or_default();
        drop(by_block);
        
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        let result: Vec<AuditEvent> = events
            .iter()
            .filter(|e| event_ids.contains(&e.id))
            .cloned()
            .collect();
        
        Ok(result)
    }
    
    /// Get events by transaction
    pub fn get_events_by_tx(
        &self,
        tx_hash: &str,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let by_tx = self.events_by_tx.lock().map_err(|_| AuditError::LockError)?;
        let event_ids = by_tx.get(tx_hash).cloned().unwrap_or_default();
        drop(by_tx);
        
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        let result: Vec<AuditEvent> = events
            .iter()
            .filter(|e| event_ids.contains(&e.id))
            .cloned()
            .collect();
        
        Ok(result)
    }
    
    /// Get events by actor
    pub fn get_events_by_actor(
        &self,
        actor: &str,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let by_actor = self.events_by_actor.lock().map_err(|_| AuditError::LockError)?;
        let event_ids = by_actor.get(actor).cloned().unwrap_or_default();
        drop(by_actor);
        
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        let result: Vec<AuditEvent> = events
            .iter()
            .filter(|e| event_ids.contains(&e.id))
            .cloned()
            .collect();
        
        Ok(result)
    }
    
    /// Get events by type
    pub fn get_events_by_type(
        &self,
        event_type: AuditEventType,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        let result: Vec<AuditEvent> = events
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect();
        
        Ok(result)
    }
    
    /// Get event by ID
    pub fn get_event_by_id(
        &self,
        id: &str,
    ) -> Result<Option<AuditEvent>, AuditError> {
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        Ok(events.iter().find(|e| e.id == id).cloned())
    }
    
    /// Verify entire audit chain integrity
    pub fn verify_integrity(&self) -> Result<AuditIntegrityReport, AuditError> {
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        
        let mut broken_links = Vec::new();
        let mut invalid_hashes = Vec::new();
        let mut prev_hash = String::new();
        
        for event in events.iter() {
            // Check hash integrity
            if !event.verify() {
                invalid_hashes.push(event.id.clone());
            }
            
            // Check chain link
            if event.prev_hash != prev_hash {
                broken_links.push(event.id.clone());
            }
            
            prev_hash = event.hash.clone();
        }
        
        Ok(AuditIntegrityReport {
            total_events: events.len(),
            valid: invalid_hashes.is_empty() && broken_links.is_empty(),
            broken_links,
            invalid_hashes,
        })
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> Result<AuditStats, AuditError> {
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        let counters = self.counters.lock().map_err(|_| AuditError::LockError)?;
        
        let total_events = events.len();
        
        let events_by_severity: HashMap<AuditSeverity, usize> = events
            .iter()
            .fold(HashMap::new(), |mut acc, e| {
                *acc.entry(e.severity).or_insert(0) += 1;
                acc
            });
        
        Ok(AuditStats {
            total_events,
            events_by_type: counters.clone(),
            events_by_severity,
            unique_actors: self.events_by_actor.lock().map_err(|_| AuditError::LockError)?.len(),
            latest_event_timestamp: events.last().map(|e| e.timestamp),
        })
    }
    
    /// Export to JSON
    pub fn export_json(
        &self,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<String, AuditError> {
        let events = self.events.lock().map_err(|_| AuditError::LockError)?;
        
        let filtered: Vec<&AuditEvent> = events
            .iter()
            .filter(|e| {
                if let Some(start) = start_time {
                    if e.timestamp < start { return false; }
                }
                if let Some(end) = end_time {
                    if e.timestamp > end { return false; }
                }
                true
            })
            .collect();
        
        serde_json::to_string_pretty(&filtered).map_err(|e| AuditError::SerializationError(e.to_string()))
    }
    
    /// Trim old events (keep last N)
    pub fn trim(&self, keep_last: usize) -> Result<usize, AuditError> {
        let mut events = self.events.lock().map_err(|_| AuditError::LockError)?;
        
        if events.len() <= keep_last {
            return Ok(0);
        }
        
        let to_remove = events.len() - keep_last;
        let removed_ids: Vec<String> = events[..to_remove].iter().map(|e| e.id.clone()).collect();
        events.drain(..to_remove);
        drop(events);
        
        // Clean up indexes
        let mut by_block = self.events_by_block.lock().map_err(|_| AuditError::LockError)?;
        for ids in by_block.values_mut() {
            ids.retain(|id| !removed_ids.contains(id));
        }
        by_block.retain(|_, ids| !ids.is_empty());
        
        let mut by_tx = self.events_by_tx.lock().map_err(|_| AuditError::LockError)?;
        for ids in by_tx.values_mut() {
            ids.retain(|id| !removed_ids.contains(id));
        }
        by_tx.retain(|_, ids| !ids.is_empty());
        
        let mut by_actor = self.events_by_actor.lock().map_err(|_| AuditError::LockError)?;
        for ids in by_actor.values_mut() {
            ids.retain(|id| !removed_ids.contains(id));
        }
        by_actor.retain(|_, ids| !ids.is_empty());
        
        Ok(to_remove)
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit integrity report
#[derive(Debug, Clone)]
pub struct AuditIntegrityReport {
    pub total_events: usize,
    pub valid: bool,
    pub broken_links: Vec<String>,
    pub invalid_hashes: Vec<String>,
}

/// Audit statistics
#[derive(Debug, Clone)]
pub struct AuditStats {
    pub total_events: usize,
    pub events_by_type: HashMap<AuditEventType, u64>,
    pub events_by_severity: HashMap<AuditSeverity, usize>,
    pub unique_actors: usize,
    pub latest_event_timestamp: Option<u64>,
}

/// Audit errors
#[derive(Debug, Clone)]
pub enum AuditError {
    LockError,
    SerializationError(String),
    InvalidEvent,
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::LockError => write!(f, "Lock poisoned"),
            AuditError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            AuditError::InvalidEvent => write!(f, "Invalid audit event"),
        }
    }
}

impl std::error::Error for AuditError {}

/// Helper functions
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generate_nonce() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Convenience macros for recording audit events
#[macro_export]
macro_rules! audit_tx {
    ($audit:expr, $tx_hash:expr, $actor:expr, $desc:expr, $severity:expr) => {
        $audit.record(
            AuditEvent::new(
                AuditEventType::TransactionExecuted,
                $actor.to_string(),
                $desc.to_string(),
                $severity,
            )
            .with_tx($tx_hash.to_string())
        )
    };
}

#[macro_export]
macro_rules! audit_block {
    ($audit:expr, $block_num:expr, $actor:expr, $desc:expr, $severity:expr) => {
        $audit.record(
            AuditEvent::new(
                AuditEventType::BlockFinalized,
                $actor.to_string(),
                $desc.to_string(),
                $severity,
            )
            .with_block($block_num)
        )
    };
}

#[macro_export]
macro_rules! audit_validator {
    ($audit:expr, $event_type:expr, $validator:expr, $desc:expr) => {
        $audit.record(
            AuditEvent::new(
                $event_type,
                $validator.to_string(),
                $desc.to_string(),
                AuditSeverity::Info,
            )
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::TransactionSubmitted,
            "0x123".to_string(),
            "Test transaction".to_string(),
            AuditSeverity::Info,
        );
        
        assert!(event.verify());
        assert!(!event.id.is_empty());
    }
    
    #[test]
    fn test_audit_chain_integrity() {
        let audit = AuditTrail::new();
        
        // Create chain of events
        let event1 = AuditEvent::new(
            AuditEventType::TransactionSubmitted,
            "0x123".to_string(),
            "Tx 1".to_string(),
            AuditSeverity::Info,
        );
        audit.record(event1).unwrap();
        
        let event2 = AuditEvent::new(
            AuditEventType::BlockProposed,
            "0xvalidator".to_string(),
            "Block 1".to_string(),
            AuditSeverity::Info,
        );
        audit.record(event2).unwrap();
        
        let report = audit.verify_integrity().unwrap();
        assert!(report.valid);
    }
    
    #[test]
    fn test_query_by_actor() {
        let audit = AuditTrail::new();
        
        audit.record(AuditEvent::new(
            AuditEventType::TransactionSubmitted,
            "0xuser1".to_string(),
            "Tx 1".to_string(),
            AuditSeverity::Info,
        )).unwrap();
        
        audit.record(AuditEvent::new(
            AuditEventType::TransactionSubmitted,
            "0xuser2".to_string(),
            "Tx 2".to_string(),
            AuditSeverity::Info,
        )).unwrap();
        
        let events = audit.get_events_by_actor("0xuser1").unwrap();
        assert_eq!(events.len(), 1);
    }
}
