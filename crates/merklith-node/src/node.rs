//! Full node implementation.

use merklith_core::state_machine::State;
use merklith_network::{NetworkNode, NetworkEvent, NetworkCommand, NetworkConfig};
use merklith_rpc::{RpcServer, RpcServerConfig};
use merklith_storage::state_db::StateDB;
use merklith_txpool::pool::TransactionPool;
use merklith_types::U256;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, Duration};
use tracing::{info, warn};

use crate::config::NodeConfig;

/// Node state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// Initializing
    Initializing,
    /// Starting up
    Starting,
    /// Running normally
    Running,
    /// Syncing with network
    Syncing,
    /// Shutting down
    ShuttingDown,
    /// Stopped
    Stopped,
}

impl NodeState {
    /// Check if node is active.
    pub fn is_active(&self) -> bool {
        matches!(self, NodeState::Running | NodeState::Syncing)
    }
}

/// The Merklith full node.
pub struct MerklithNode {
    /// Node configuration
    pub config: NodeConfig,
    /// Current state
    pub node_state: Arc<RwLock<NodeState>>,
    /// Blockchain state (real state machine)
    pub chain_state: Arc<State>,
    /// Transaction pool
    pub tx_pool: Arc<Mutex<TransactionPool>>,
    /// Network node
    pub network: Option<NetworkNode>,
    /// RPC server
    pub rpc_server: Option<RpcServer>,
    /// Network command sender
    pub network_cmd: Option<mpsc::Sender<NetworkCommand>>,
    /// Shutdown signal
    pub shutdown: mpsc::Receiver<()>,
}

impl MerklithNode {
    /// Create a new node.
    pub async fn new(
        config: NodeConfig,
    ) -> anyhow::Result<(Self, mpsc::Sender<()>)> {
        info!("Initializing Merklith node: {}", config.name);

        // Initialize state database
        let state_db = Arc::new(StateDB::new(&config.storage.db_path)?);

        // Initialize transaction pool
        let tx_pool_config = merklith_txpool::pool::PoolConfig::default();
        let tx_pool = Arc::new(Mutex::new(TransactionPool::new(tx_pool_config)));

        // Initialize blockchain state (real state machine) with proper data directory
        let state_path = config.data_dir.join("state");
        let chain_state = Arc::new(State::with_path(state_path));

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        let node = Self {
            config,
            node_state: Arc::new(RwLock::new(NodeState::Initializing)),
            chain_state,
            tx_pool,
            network: None,
            rpc_server: None,
            network_cmd: None,
            shutdown: shutdown_rx,
        };

        Ok((node, shutdown_tx))
    }

    /// Start the node.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting Merklith node (Chain ID: {})", self.config.consensus.chain_id);
        
        *self.node_state.write().await = NodeState::Starting;

        // Start network if enabled
        if self.config.network.enabled {
            self.start_network().await?;
        }

        // Start RPC server
        if self.config.rpc.http_enabled || self.config.rpc.ws_enabled {
            self.start_rpc().await?;
        }

        // Start block production loop (with network broadcast)
        let network_cmd = self.network_cmd.clone();
        self.start_block_production(network_cmd).await;

        *self.node_state.write().await = NodeState::Running;
        info!("Merklith node started successfully");

        Ok(())
    }

    /// Start the network layer.
    async fn start_network(
        &mut self,
    ) -> anyhow::Result<()> {
        info!("Starting P2P network...");

        let (event_tx, mut event_rx) = mpsc::channel(100);
        
        let p2p_port = self.config.network.p2p_port;
        let bootstrap_peers = self.config.network.bootstrap_nodes.clone();
        
        let network_config = merklith_network::NetworkConfig::new(
            format!("node_{}", rand::random::<u64>())
        ).with_port(p2p_port)
         .with_bootstrap(bootstrap_peers);

        let (network, cmd_sender) = NetworkNode::new(network_config, event_tx);
        self.network = Some(network);
        self.network_cmd = Some(cmd_sender.clone());
        
        // Clone for event handler
        let chain_state = self.chain_state.clone();

        // Spawn network event handler
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    NetworkEvent::PeerConnected { peer_id, address } => {
                        info!("✅ Peer connected: {} at {:?}", peer_id, address);
                    }
                    NetworkEvent::PeerDisconnected { peer_id } => {
                        info!("❌ Peer disconnected: {}", peer_id);
                    }
                    NetworkEvent::NewBlock { hash, number, parent_hash } => {
                        let hash_bytes: [u8; 32] = *hash.as_bytes();
                        
                        // Check if we already have this block
                        if chain_state.has_block(&hash_bytes) {
                            tracing::debug!("Already have block #{}", number);
                            continue;
                        }
                        
                        // Get current block number
                        let current = chain_state.block_number();
                        
                        if number == current + 1 {
                            // Try to add the block (verifies parent hash)
                            if chain_state.add_block(number, hash_bytes, parent_hash) {
                                info!("📥 Synced block #{} from peer", number);
                            } else {
                                tracing::warn!("Failed to add block #{} (invalid parent)", number);
                            }
                        } else if number > current + 1 {
                            info!("📥 Received block #{} but we're at #{} (need catch-up)", number, current);
                        } else {
                            tracing::debug!("Ignoring old block #{} (we have #{})", number, current);
                        }
                    }
                    NetworkEvent::NewTransaction { hash } => {
                        tracing::debug!("📝 Received transaction: {}", hex::encode(hash));
                    }
                    NetworkEvent::SyncProgress { current, target } => {
                        info!("🔄 Syncing: {} / {} blocks", current, target);
                    }
                    _ => {}
                }
            }
        });

        // Start the network
        if let Some(mut network) = self.network.take() {
            tokio::spawn(async move {
                if let Err(e) = network.start().await {
                    warn!("Network error: {}", e);
                }
            });
        }

        info!("P2P network started");
        Ok(())
    }

    /// Start the RPC server.
    async fn start_rpc(
        &mut self,
    ) -> anyhow::Result<()> {
        info!("Starting RPC server...");

        let rpc_config = RpcServerConfig {
            http_addr: self.config.rpc.http_addr,
            http_port: self.config.rpc.http_addr.port(),
            ws_addr: if self.config.rpc.ws_enabled {
                Some(self.config.rpc.ws_addr)
            } else {
                None
            },
            cors: self.config.rpc.cors,
            max_body_size: self.config.rpc.max_body_size as u32 * 1024 * 1024,
            max_connections: 1000,
            rate_limit: self.config.rpc.rate_limit,
        };

        let mut rpc_server = RpcServer::new(
            rpc_config, 
            self.chain_state.clone(),
            self.config.consensus.chain_id,
        );
        
        rpc_server.start().await?;

        self.rpc_server = Some(rpc_server);
        info!("RPC server started on {:?}", self.config.rpc.http_addr);
        Ok(())
    }

    /// Start block production with economic incentives.
    /// 
    /// Strategy:
    /// 1. Transaction varsa: Hemen block üret (12 saniyede bir max)
    /// 2. Transaction yoksa: Saatte 1 block üret (heartbeat)
    /// 3. Block reward: Validator'a ödül (2 MERK base + fees + bonus)
    /// 
    /// Bu sayede:
    /// - Ağ verimli çalışır (boş block spam'i yok)
    /// - Validator'lar ödüllendirilir
    /// - Zincir ilerler (saatte 1 block garanti)
    async fn start_block_production(
        &self,
        network_cmd: Option<mpsc::Sender<NetworkCommand>>,
    ) {
        // Only produce blocks if this node is a validator
        if !self.config.consensus.validator {
            info!("Node is not a validator, skipping block production");
            return;
        }
        
        // Time constants
        const MIN_BLOCK_TIME: u64 = 12;           // Min 12 saniye (hızlı ama spam değil)
        const HEARTBEAT_INTERVAL: u64 = 3600;      // Saatte 1 block (60*60)
        const MAX_EMPTY_SKIP: u32 = 5;             // 5 boş block atla max
        
        let node_state = self.node_state.clone();
        let chain_state = self.chain_state.clone();
        let tx_pool = self.tx_pool.clone();
        let validator_address = self.config.consensus.validator_key.as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|hex_str| hex::decode(hex_str.trim()).ok())
            .and_then(|bytes| {
                if bytes.len() == 20 {
                    let mut addr = [0u8; 20];
                    addr.copy_from_slice(&bytes);
                    Some(merklith_types::Address::from_bytes(addr))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                // Default validator address for devnet
                merklith_types::Address::from_bytes([0xABu8; 20])
            });

        tokio::spawn(async move {
            let mut last_block_time = std::time::Instant::now();
            let mut empty_count = 0u32;
            let mut last_heartbeat = std::time::Instant::now();
            
            loop {
                // Wait minimum block time
                let elapsed = last_block_time.elapsed().as_secs();
                if elapsed < MIN_BLOCK_TIME {
                    tokio::time::sleep(Duration::from_secs(MIN_BLOCK_TIME - elapsed)).await;
                }
                
                // Check if we're still running
                if !node_state.read().await.is_active() {
                    break;
                }

                // Check transaction pool
                let pool = tx_pool.lock().await;
                let pending_txs = pool.get_pending(1000);
                let tx_count = pending_txs.len();
                drop(pool);
                
                // Decision: Block üretmeli miyiz?
                let should_produce = if tx_count > 0 {
                    // Transaction varsa: Hemen üret (ama MIN_BLOCK_TIME kadar beklemiş olmalı)
                    true
                } else {
                    // Transaction yoksa: Saatte 1 block (heartbeat)
                    let time_since_heartbeat = last_heartbeat.elapsed().as_secs();
                    if time_since_heartbeat >= HEARTBEAT_INTERVAL {
                        empty_count += 1;
                        true // Saat doldu, heartbeat block üret
                    } else {
                        // Henüz saat dolmadı, boş block üretme
                        empty_count += 1;
                        if empty_count <= MAX_EMPTY_SKIP {
                            // İlk 5 boş block'u atla (loglama yok)
                            continue;
                        }
                        // 5'ten sonra her 10'da bir logla
                        if empty_count % 10 == 0 {
                            tracing::debug!(
                                "Waiting for transactions or heartbeat... ({} empty, {}s until heartbeat)",
                                empty_count,
                                HEARTBEAT_INTERVAL - time_since_heartbeat
                            );
                        }
                        continue;
                    }
                };
                
                if !should_produce {
                    continue;
                }
                
                // Reset counters
                last_block_time = std::time::Instant::now();
                if tx_count == 0 {
                    last_heartbeat = std::time::Instant::now();
                }
                empty_count = 0;

                // Get parent hash
                let parent_hash = *chain_state.block_hash().as_bytes();
                
                // Produce block with reward
                let is_heartbeat = tx_count == 0;
                match chain_state.produce_block(&validator_address, pending_txs, is_heartbeat) {
                    Ok(result) => {
                        let reward_merk = result.validator_reward / U256::from(1_000_000_000_000_000_000u128);
                        
                        if tx_count > 0 {
                            info!(
                                "✓ Block #{}: {} txs | Reward: {} MERK | Hash: {}",
                                result.block_number,
                                result.transactions_count,
                                reward_merk,
                                hex::encode(&result.block_hash[..8])
                            );
                        } else {
                            info!(
                                "~ Heartbeat #{}: Empty | Security reward: {} MERK | Next in ~1h",
                                result.block_number,
                                reward_merk
                            );
                        }
                        
                        // Broadcast to network
                        if let Some(cmd) = &network_cmd {
                            let _ = cmd.send(NetworkCommand::BroadcastBlock {
                                number: result.block_number,
                                hash: result.block_hash,
                                parent_hash,
                            }).await;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Block production failed: {:?}", e);
                    }
                }
            }
        });
    }

    /// Run the node (main loop).
    pub async fn run(
        &mut self,
    ) -> anyhow::Result<()> {
        info!("Node is running. Press Ctrl+C to shutdown.");

        // Wait for shutdown signal
        tokio::select! {
            _ = self.shutdown.recv() => {
                info!("Shutdown signal received");
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C received");
            }
        }

        self.shutdown().await;
        Ok(())
    }

    /// Graceful shutdown.
    pub async fn shutdown(
        &mut self,
    ) {
        info!("Shutting down Merklith node...");
        *self.node_state.write().await = NodeState::ShuttingDown;

        // Stop RPC server
        if let Some(rpc) = self.rpc_server.take() {
            info!("Stopping RPC server...");
            drop(rpc);
        }

        // Stop network
        if let Some(cmd) = &self.network_cmd {
            let _ = cmd.send(NetworkCommand::Shutdown).await;
        }

        if let Some(mut network) = self.network.take() {
            info!("Stopping network...");
            network.shutdown();
        }

        *self.node_state.write().await = NodeState::Stopped;
        info!("Merklith node stopped");
    }

    /// Get current block number.
    pub async fn current_block(&self) -> u64 {
        self.chain_state.block_number()
    }

    /// Get node state.
    pub async fn state(&self) -> NodeState {
        *self.node_state.read().await
    }

    /// Check if node is healthy.
    pub async fn is_healthy(&self) -> bool {
        matches!(self.state().await, NodeState::Running | NodeState::Syncing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_creation() {
        let config = NodeConfig::default();
        let (node, _shutdown) = MerklithNode::new(config).await.unwrap();
        
        assert!(matches!(*node.node_state.read().await, NodeState::Initializing));
    }

    #[tokio::test]
    async fn test_node_state() {
        let config = NodeConfig::default();
        let (mut node, _shutdown) = MerklithNode::new(config).await.unwrap();

        assert!(!node.is_healthy().await);
        
        *node.node_state.write().await = NodeState::Running;
        assert!(node.is_healthy().await);
    }

    #[test]
    fn test_node_state_is_active() {
        assert!(NodeState::Running.is_active());
        assert!(NodeState::Syncing.is_active());
        assert!(!NodeState::Stopped.is_active());
        assert!(!NodeState::Initializing.is_active());
    }
}
