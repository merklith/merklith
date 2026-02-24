//! WebSocket subscriptions for real-time updates.
//!
//! Supports subscriptions for:
//! - New blocks
//! - New pending transactions
//! - New logs
//! - Syncing status

use merklith_types::{Block, Hash, Transaction};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;

/// Subscription type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionType {
    /// New blocks
    NewHeads,
    /// New pending transactions
    NewPendingTransactions,
    /// New logs matching filter
    Logs,
    /// Syncing status
    Syncing,
}

impl SubscriptionType {
    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "newHeads" => Some(Self::NewHeads),
            "newPendingTransactions" => Some(Self::NewPendingTransactions),
            "logs" => Some(Self::Logs),
            "syncing" => Some(Self::Syncing),
            _ => None,
        }
    }

    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NewHeads => "newHeads",
            Self::NewPendingTransactions => "newPendingTransactions",
            Self::Logs => "logs",
            Self::Syncing => "syncing",
        }
    }
}

/// Subscription ID (hex string).
pub type SubscriptionId = String;

/// Subscription manager.
pub struct SubscriptionManager {
    /// Next subscription ID
    next_id: AtomicU64,
    /// Active subscriptions (id -> subscription)
    subscriptions: HashMap<SubscriptionId, Subscription>,
    /// Event broadcaster
    broadcaster: mpsc::Sender<SubscriptionEvent>,
}

/// Subscription metadata.
#[derive(Debug)]
pub struct Subscription {
    /// Subscription ID
    pub id: SubscriptionId,
    /// Type of subscription
    pub subscription_type: SubscriptionType,
    /// Filter for logs subscription (optional)
    pub filter: Option<LogFilter>,
    /// Client sender channel
    pub sender: mpsc::Sender<SubscriptionResult>,
}

/// Log filter for subscriptions.
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// Contract addresses to monitor
    pub addresses: Vec<String>,
    /// Topic filters
    pub topics: Vec<Option<String>>,
    /// From block
    pub from_block: Option<u64>,
    /// To block
    pub to_block: Option<u64>,
}

/// Events that can be broadcast to subscribers.
#[derive(Debug, Clone)]
pub enum SubscriptionEvent {
    /// New block mined
    NewBlock {
        hash: Hash,
        number: u64,
        parent_hash: Hash,
    },
    /// New transaction received
    NewTransaction {
        hash: Hash,
        from: Option<String>,
        to: Option<String>,
    },
    /// New log emitted
    NewLog {
        address: String,
        topics: Vec<String>,
        data: String,
        block_number: u64,
        transaction_hash: Hash,
        log_index: u64,
    },
    /// Syncing status changed
    SyncingStatus {
        syncing: bool,
        current_block: u64,
        highest_block: Option<u64>,
    },
}

/// Result sent to subscriber.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SubscriptionResult {
    /// New block header
    BlockHeader {
        subscription: SubscriptionId,
        result: BlockHeaderResult,
    },
    /// Transaction hash
    TransactionHash {
        subscription: SubscriptionId,
        result: String,
    },
    /// Log entry
    LogEntry {
        subscription: SubscriptionId,
        result: LogResult,
    },
    /// Syncing status
    Syncing {
        subscription: SubscriptionId,
        result: SyncingResult,
    },
}

/// Block header result.
#[derive(Debug, Clone, Serialize)]
pub struct BlockHeaderResult {
    pub parentHash: String,
    pub sha3Uncles: String,
    pub miner: String,
    pub stateRoot: String,
    pub transactionsRoot: String,
    pub receiptsRoot: String,
    pub logsBloom: String,
    pub difficulty: String,
    pub number: String,
    pub gasLimit: String,
    pub gasUsed: String,
    pub timestamp: String,
    pub extraData: String,
    pub mixHash: String,
    pub nonce: String,
    pub baseFeePerGas: Option<String>,
    pub hash: String,
}

/// Log result.
#[derive(Debug, Clone, Serialize)]
pub struct LogResult {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub blockNumber: String,
    pub transactionHash: String,
    pub transactionIndex: String,
    pub blockHash: String,
    pub logIndex: String,
    pub removed: bool,
}

/// Syncing status result.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SyncingResult {
    /// Not syncing
    Bool(bool),
    /// Syncing with progress
    Progress {
        startingBlock: String,
        currentBlock: String,
        highestBlock: String,
    },
}

impl SubscriptionManager {
    /// Create new subscription manager.
    pub fn new() -> (Self, mpsc::Receiver<SubscriptionEvent>) {
        let (tx, rx) = mpsc::channel(1000);
        
        let manager = Self {
            next_id: AtomicU64::new(1),
            subscriptions: HashMap::new(),
            broadcaster: tx,
        };

        (manager, rx)
    }

    /// Subscribe to events.
    pub fn subscribe(
        &mut self,
        subscription_type: SubscriptionType,
        filter: Option<LogFilter>,
        sender: mpsc::Sender<SubscriptionResult>,
    ) -> SubscriptionId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let id_hex = format!("0x{:x}", id);

        let subscription = Subscription {
            id: id_hex.clone(),
            subscription_type,
            filter,
            sender,
        };

        self.subscriptions.insert(id_hex.clone(), subscription);
        id_hex
    }

    /// Unsubscribe from events.
    pub fn unsubscribe(
        &mut self,
        id: &SubscriptionId,
    ) -> bool {
        self.subscriptions.remove(id).is_some()
    }

    /// Get subscription by ID.
    pub fn get_subscription(
        &self,
        id: &SubscriptionId,
    ) -> Option<&Subscription> {
        self.subscriptions.get(id)
    }

    /// Broadcast event to all matching subscribers.
    pub async fn broadcast(
        &self,
        event: &SubscriptionEvent,
    ) {
        for subscription in self.subscriptions.values() {
            if Self::should_send(subscription, event) {
                let result = Self::create_result(&subscription.id, event
                );
                
                // Send to subscriber (ignore errors)
                let _ = subscription.sender.send(result).await;
            }
        }
    }

    /// Check if subscription should receive this event.
    fn should_send(
        subscription: &Subscription,
        event: &SubscriptionEvent,
    ) -> bool {
        match (subscription.subscription_type, event) {
            (SubscriptionType::NewHeads, SubscriptionEvent::NewBlock { .. }) => true,
            (SubscriptionType::NewPendingTransactions, SubscriptionEvent::NewTransaction { .. }) => true,
            (SubscriptionType::Logs, SubscriptionEvent::NewLog { address, .. }) => {
                // Check filter
                if let Some(filter) = &subscription.filter {
                    filter.addresses.is_empty() || filter.addresses.contains(address)
                } else {
                    true
                }
            }
            (SubscriptionType::Syncing, SubscriptionEvent::SyncingStatus { .. }) => true,
            _ => false,
        }
    }

    /// Create result object for event.
    fn create_result(
        subscription_id: &SubscriptionId,
        event: &SubscriptionEvent,
    ) -> SubscriptionResult {
        match event {
            SubscriptionEvent::NewBlock { hash, number, parent_hash } => {
                SubscriptionResult::BlockHeader {
                    subscription: subscription_id.clone(),
                    result: BlockHeaderResult {
                        parentHash: format!("0x{}", hex::encode(parent_hash.as_bytes())),
                        sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347".to_string(),
                        miner: "0x0000000000000000000000000000000000000000".to_string(),
                        stateRoot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                        transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421".to_string(),
                        receiptsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421".to_string(),
                        logsBloom: "0x".to_string() + &"0".repeat(512),
                        difficulty: "0x0".to_string(),
                        number: format!("0x{:x}", number),
                        gasLimit: "0x1c9c380".to_string(),
                        gasUsed: "0x0".to_string(),
                        timestamp: format!("0x{:x}", std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()),
                        extraData: "0x".to_string(),
                        mixHash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                        nonce: "0x0000000000000000".to_string(),
                        baseFeePerGas: Some("0x0".to_string()),
                        hash: format!("0x{}", hex::encode(hash.as_bytes())),
                    },
                }
            }
            SubscriptionEvent::NewTransaction { hash, .. } => {
                SubscriptionResult::TransactionHash {
                    subscription: subscription_id.clone(),
                    result: format!("0x{}", hex::encode(hash.as_bytes())),
                }
            }
            SubscriptionEvent::NewLog { address, topics, data, block_number, transaction_hash, log_index } => {
                SubscriptionResult::LogEntry {
                    subscription: subscription_id.clone(),
                    result: LogResult {
                        address: address.clone(),
                        topics: topics.clone(),
                        data: data.clone(),
                        blockNumber: format!("0x{:x}", block_number),
                        transactionHash: format!("0x{}", hex::encode(transaction_hash.as_bytes())),
                        transactionIndex: "0x0".to_string(),
                        blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                        logIndex: format!("0x{:x}", log_index),
                        removed: false,
                    },
                }
            }
            SubscriptionEvent::SyncingStatus { syncing, current_block, highest_block } => {
                let result = if *syncing {
                    SyncingResult::Progress {
                        startingBlock: "0x0".to_string(),
                        currentBlock: format!("0x{:x}", current_block),
                        highestBlock: format!("0x{:x}", highest_block.unwrap_or(*current_block)),
                    }
                } else {
                    SyncingResult::Bool(false)
                };

                SubscriptionResult::Syncing {
                    subscription: subscription_id.clone(),
                    result,
                }
            }
        }
    }

    /// Get broadcaster channel.
    pub fn broadcaster(&self,
    ) -> mpsc::Sender<SubscriptionEvent> {
        self.broadcaster.clone()
    }

    /// Get number of active subscriptions.
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        let (manager, _) = Self::new();
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscription_manager() {
        let (mut manager, _rx) = SubscriptionManager::new();
        let (tx, mut rx) = mpsc::channel(10);

        // Subscribe to new blocks
        let id = manager.subscribe(SubscriptionType::NewHeads, None, tx);
        assert!(id.starts_with("0x"));
        assert_eq!(manager.subscription_count(), 1);

        // Broadcast a block
        let event = SubscriptionEvent::NewBlock {
            hash: merklith_types::Hash::ZERO,
            number: 100,
            parent_hash: merklith_types::Hash::ZERO,
        };

        manager.broadcast(&event).await;

        // Should receive the result
        let result = rx.recv().await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let (mut manager, _rx) = SubscriptionManager::new();
        let (tx, _rx) = mpsc::channel(10);

        let id = manager.subscribe(SubscriptionType::NewHeads, None, tx);
        assert_eq!(manager.subscription_count(), 1);

        let removed = manager.unsubscribe(&id);
        assert!(removed);
        assert_eq!(manager.subscription_count(), 0);

        // Unsubscribe again should return false
        let removed = manager.unsubscribe(&id);
        assert!(!removed);
    }

    #[test]
    fn test_subscription_type_parsing() {
        assert_eq!(
            SubscriptionType::from_str("newHeads"),
            Some(SubscriptionType::NewHeads)
        );
        assert_eq!(
            SubscriptionType::from_str("newPendingTransactions"),
            Some(SubscriptionType::NewPendingTransactions)
        );
        assert_eq!(
            SubscriptionType::from_str("logs"),
            Some(SubscriptionType::Logs)
        );
        assert_eq!(
            SubscriptionType::from_str("syncing"),
            Some(SubscriptionType::Syncing)
        );
        assert_eq!(SubscriptionType::from_str("unknown"), None);
    }

    #[test]
    fn test_should_send() {
        let sub = Subscription {
            id: "0x1".to_string(),
            subscription_type: SubscriptionType::NewHeads,
            filter: None,
            sender: mpsc::channel(1).0,
        };

        let event = SubscriptionEvent::NewBlock {
            hash: merklith_types::Hash::ZERO,
            number: 100,
            parent_hash: merklith_types::Hash::ZERO,
        };

        assert!(SubscriptionManager::should_send(&sub, &event));
    }
}
