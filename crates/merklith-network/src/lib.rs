//! Network - Real P2P networking with TCP

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Duration;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Network error
#[derive(Debug, Clone)]
pub enum NetworkError {
    Io(String),
    ConnectionFailed(String),
    SendFailed(String),
    ParseError(String),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::Io(s) => write!(f, "IO: {}", s),
            NetworkError::ConnectionFailed(s) => write!(f, "Connection: {}", s),
            NetworkError::SendFailed(s) => write!(f, "Send: {}", s),
            NetworkError::ParseError(s) => write!(f, "Parse: {}", s),
        }
    }
}

impl std::error::Error for NetworkError {}

/// P2P Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    /// Handshake from new peer
    Handshake { node_id: String, listen_port: u16 },
    /// New block announcement
    NewBlock { number: u64, hash: Vec<u8>, parent_hash: Vec<u8> },
    /// New transaction announcement
    NewTransaction { hash: Vec<u8> },
    /// Request blocks
    GetBlocks { from: u64, count: u64 },
    /// Block response
    Blocks { blocks: Vec<BlockData> },
    /// Ping
    Ping,
    /// Pong
    Pong,
}

/// Block data for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub number: u64,
    pub hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
    pub transactions: Vec<u8>,
}

/// Network event
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PeerConnected { peer_id: String, address: String },
    PeerDisconnected { peer_id: String },
    NewBlock { hash: merklith_types::Hash, number: u64, parent_hash: [u8; 32] },
    NewTransaction { hash: merklith_types::Hash },
    MessageReceived { from: String, data: Vec<u8> },
    SyncProgress { current: u64, target: u64 },
}

/// Network command
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    Connect { address: String },
    BroadcastBlock { number: u64, hash: [u8; 32], parent_hash: [u8; 32] },
    BroadcastTransaction { hash: [u8; 32] },
    Shutdown,
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub local_id: String,
    pub listen_addr: String,
    pub listen_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
}

impl NetworkConfig {
    pub fn new(local_id: String) -> Self {
        Self {
            local_id,
            listen_addr: "0.0.0.0".to_string(),
            listen_port: 30303,
            bootstrap_peers: vec![],
            max_peers: 50,
        }
    }
    
    pub fn with_port(mut self, port: u16) -> Self {
        self.listen_port = port;
        self.listen_addr = format!("0.0.0.0:{}", port);
        self
    }
    
    pub fn with_bootstrap(mut self, peers: Vec<String>) -> Self {
        self.bootstrap_peers = peers;
        self
    }
}

/// Connected peer info
#[derive(Debug, Clone)]
struct Peer {
    _id: String,
    address: String,
    _port: u16,
}

/// Real P2P network node
pub struct NetworkNode {
    local_id: String,
    listen_addr: String,
    listen_port: u16,
    event_tx: mpsc::Sender<NetworkEvent>,
    cmd_rx: mpsc::Receiver<NetworkCommand>,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    running: Arc<RwLock<bool>>,
    pending_connections: Vec<String>,
}

impl NetworkNode {
    pub fn new(config: NetworkConfig, event_tx: mpsc::Sender<NetworkEvent>) -> (Self, mpsc::Sender<NetworkCommand>) {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        
        let node = Self {
            local_id: config.local_id,
            listen_addr: format!("{}:{}", config.listen_addr, config.listen_port),
            listen_port: config.listen_port,
            event_tx,
            cmd_rx,
            peers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            pending_connections: config.bootstrap_peers,
        };
        
        (node, cmd_tx)
    }
    
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        *self.running.write() = true;
        
        // Start TCP listener in background
        let listen_addr = self.listen_addr.clone();
        let local_id = self.local_id.clone();
        let peers = self.peers.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            if let Ok(addr) = listen_addr.parse::<std::net::SocketAddr>() {
                if let Ok(listener) = TcpListener::bind(addr).await {
                    tracing::info!("P2P listening on {}", listen_addr);
                    
                    while *running.read() {
                        tokio::select! {
                            accept_result = listener.accept() => {
                                match accept_result {
                                    Ok((stream, addr)) => {
                                        let peer_id = format!("peer_{}", rand::random::<u32>());
                                        
                                        // Send handshake
                                        let _handshake = P2PMessage::Handshake {
                                            node_id: local_id.clone(),
                                            listen_port: 30303,
                                        };
                                        
                                        peers.write().insert(peer_id.clone(), Peer {
                                            _id: peer_id.clone(),
                                            address: addr.to_string(),
                                            _port: addr.port(),
                                        });
                                        
                                        let _ = event_tx.send(NetworkEvent::PeerConnected {
                                            peer_id,
                                            address: addr.to_string(),
                                        }).await;
                                        
                                        tracing::info!("Peer connected from {}", addr);
                                        
                                        // Handle incoming messages from this peer
                                        Self::handle_peer_stream(stream, event_tx.clone(), running.clone());
                                    }
                                    Err(e) => {
                                        tracing::debug!("Accept error: {}", e);
                                    }
                                }
                            }
                            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                        }
                    }
                }
            }
        });
        
        // Connect to bootstrap peers
        let bootstrap: Vec<String> = self.pending_connections.drain(..).collect();
        for peer_addr in bootstrap {
            if let Err(e) = self.connect(&peer_addr).await {
                tracing::debug!("Failed to connect to bootstrap peer {}: {}", peer_addr, e);
            }
        }
        
        // Start command handler
        self.start_command_handler();
        
        tracing::info!("Network node {} started", self.local_id);
        Ok(())
    }
    
    /// Start command handler loop
    fn start_command_handler(&mut self) {
        let peers = self.peers.clone();
        let running = self.running.clone();
        let mut cmd_rx = std::mem::replace(&mut self.cmd_rx, mpsc::channel(1).1);
        
        tokio::spawn(async move {
            while *running.read() {
                tokio::select! {
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            NetworkCommand::BroadcastBlock { number, hash, parent_hash } => {
                                let msg = P2PMessage::NewBlock {
                                    number,
                                    hash: hash.to_vec(),
                                    parent_hash: parent_hash.to_vec(),
                                };
                                
                                if let Ok(data) = bincode::serialize(&msg) {
                                    let peers_list: Vec<_> = peers.read().iter()
                                        .map(|(k, v)| (k.clone(), v.address.clone()))
                                        .collect();
                                    
                                    for (peer_id, peer_addr) in peers_list {
                                        if let Ok(mut stream) = TcpStream::connect(&peer_addr).await {
                                            if stream.write_all(&data).await.is_ok() {
                                                tracing::debug!("Sent block #{} to peer {}", number, peer_id);
                                            }
                                        }
                                    }
                                    
                                    if !peers.read().is_empty() {
                                        tracing::info!("Broadcast block #{} to {} peers", number, peers.read().len());
                                    }
                                }
                            }
                            NetworkCommand::BroadcastTransaction { hash } => {
                                let msg = P2PMessage::NewTransaction { hash: hash.to_vec() };
                                
                                if let Ok(data) = bincode::serialize(&msg) {
                                    let peers_list: Vec<_> = peers.read().iter()
                                        .map(|(k, v)| (k.clone(), v.address.clone()))
                                        .collect();
                                    
                                    for (peer_id, peer_addr) in peers_list {
                                        if let Ok(mut stream) = TcpStream::connect(&peer_addr).await {
                                            if stream.write_all(&data).await.is_ok() {
                                                tracing::debug!("Sent tx to peer {}", peer_id);
                                            }
                                        }
                                    }
                                }
                            }
                            NetworkCommand::Connect { address } => {
                                if let Ok(_stream) = TcpStream::connect(&address).await {
                                    let peer_id = format!("peer_{}", rand::random::<u32>());
                                    peers.write().insert(peer_id.clone(), Peer {
                                        _id: peer_id.clone(),
                                        address: address.clone(),
                                        _port: 30303,
                                    });
                                    tracing::info!("Connected to peer at {}", address);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                }
            }
        });
    }
    
    fn handle_peer_stream(
        mut stream: TcpStream,
        event_tx: mpsc::Sender<NetworkEvent>,
        running: Arc<RwLock<bool>>,
    ) {
        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            
            while *running.read() {
                tokio::select! {
                    read_result = stream.read(&mut buf) => {
                        match read_result {
                            Ok(0) => break, // Connection closed
                            Ok(n) => {
                                if let Ok(msg) = bincode::deserialize::<P2PMessage>(&buf[..n]) {
                                    match msg {
                                        P2PMessage::NewBlock { number, hash, parent_hash } => {
                                            if hash.len() == 32 && parent_hash.len() == 32 {
                                                let mut h = [0u8; 32];
                                                let mut ph = [0u8; 32];
                                                h.copy_from_slice(&hash);
                                                ph.copy_from_slice(&parent_hash);
                                                let _ = event_tx.send(NetworkEvent::NewBlock {
                                                    hash: merklith_types::Hash::from_bytes(h),
                                                    number,
                                                    parent_hash: ph,
                                                }).await;
                                                tracing::debug!("Received block #{} from peer", number);
                                            }
                                        }
                                        P2PMessage::NewTransaction { hash } => {
                                            if hash.len() == 32 {
                                                let mut h = [0u8; 32];
                                                h.copy_from_slice(&hash);
                                                let _ = event_tx.send(NetworkEvent::NewTransaction {
                                                    hash: merklith_types::Hash::from_bytes(h),
                                                }).await;
                                            }
                                        }
                                        P2PMessage::Ping => {
                                            let pong = P2PMessage::Pong;
                                            if let Ok(data) = bincode::serialize(&pong) {
                                                let _ = stream.write_all(&data).await;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(30)) => {
                        // Send ping to keep connection alive
                        let ping = P2PMessage::Ping;
                        if let Ok(data) = bincode::serialize(&ping) {
                            let _ = stream.write_all(&data).await;
                        }
                    }
                }
            }
        });
    }
    
    pub async fn connect(&mut self, addr: &str) -> Result<(), NetworkError> {
        let stream = TcpStream::connect(addr).await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        
        let peer_id = format!("peer_{}", rand::random::<u32>());
        
        // Send handshake
        let handshake = P2PMessage::Handshake {
            node_id: self.local_id.clone(),
            listen_port: self.listen_port,
        };
        
        let data = bincode::serialize(&handshake)
            .map_err(|e| NetworkError::ParseError(e.to_string()))?;
        
        let mut stream_clone = stream;
        stream_clone.write_all(&data).await
            .map_err(|e| NetworkError::SendFailed(e.to_string()))?;
        
        // Add to peers
        self.peers.write().insert(peer_id.clone(), Peer {
            _id: peer_id.clone(),
            address: addr.to_string(),
            _port: addr.parse().map(|a: std::net::SocketAddr| a.port()).unwrap_or(30303),
        });
        
        let _ = self.event_tx.send(NetworkEvent::PeerConnected {
            peer_id: peer_id.clone(),
            address: addr.to_string(),
        }).await;
        
        tracing::info!("Connected to peer at {}", addr);
        Ok(())
    }
    
    pub async fn broadcast_block(&self, number: u64, hash: [u8; 32], parent_hash: [u8; 32]) {
        let msg = P2PMessage::NewBlock {
            number,
            hash: hash.to_vec(),
            parent_hash: parent_hash.to_vec(),
        };
        
        if let Ok(data) = bincode::serialize(&msg) {
            let peers = self.peers.read();
            for (peer_id, peer) in peers.iter() {
                // Try to send to each peer
                if let Ok(stream) = TcpStream::connect(&peer.address).await {
                    let mut s = stream;
                    if s.write_all(&data).await.is_ok() {
                        tracing::debug!("Sent block #{} to peer {}", number, peer_id);
                    }
                }
            }
        }
    }
    
    pub fn shutdown(&mut self) {
        *self.running.write() = false;
        tracing::info!("Network node {} shutdown", self.local_id);
    }
    
    pub fn connected_peers(&self) -> usize {
        self.peers.read().len()
    }
    
    pub fn local_id(&self) -> &str {
        &self.local_id
    }
    
    pub fn get_peers(&self) -> Vec<String> {
        self.peers.read().keys().cloned().collect()
    }
}

// Compatibility stubs
pub mod behaviour {
    pub use super::{NetworkConfig, P2PMessage};
}

pub mod gossip {
    #[derive(Debug, Clone)]
    pub struct GossipConfig;
    impl Default for GossipConfig { fn default() -> Self { Self } }
}

pub mod discovery {
    #[derive(Debug, Clone)]
    pub struct DiscoveryConfig;
}

pub mod peer_manager {
    #[derive(Debug, Clone)]
    pub struct PeerManagerConfig;
    impl Default for PeerManagerConfig { fn default() -> Self { Self } }
}

pub mod sync {
    #[derive(Debug, Clone)]
    pub struct SyncConfig;
    impl Default for SyncConfig { fn default() -> Self { Self } }
}
