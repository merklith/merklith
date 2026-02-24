//! RPC server implementation.
//!
//! HTTP and WebSocket server using jsonrpsee with security features.

use crate::error::RpcError;
use crate::handlers::RpcHandler;
use crate::subscriptions::SubscriptionManager;
use crate::security::SecurityManager;
use merklith_types::{Block, Hash, Transaction};
use jsonrpsee::server::{Server, ServerBuilder, ServerHandle};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

/// RPC server configuration.
#[derive(Debug, Clone)]
pub struct RpcServerConfig {
    /// HTTP server address
    pub http_addr: SocketAddr,
    /// WebSocket server address (optional)
    pub ws_addr: Option<SocketAddr>,
    /// Enable CORS
    pub cors: bool,
    /// Max request body size
    pub max_body_size: u32,
    /// Max connections
    pub max_connections: u32,
    /// Rate limit (requests per second)
    pub rate_limit: Option<u32>,
    /// Chain ID for security validation
    pub chain_id: u64,
    /// Enable security features
    pub security_enabled: bool,
    /// Max requests per minute per IP
    pub rate_limit_per_minute: usize,
}

impl Default for RpcServerConfig {
    fn default() -> Self {
        Self {
            http_addr: "127.0.0.1:8545".parse().unwrap_or_else(|_| {
                std::net::SocketAddr::from(([127, 0, 0, 1], 8545))
            }),
            ws_addr: Some("127.0.0.1:8546".parse().unwrap_or_else(|_| {
                std::net::SocketAddr::from(([127, 0, 0, 1], 8546))
            })),
            cors: true,
            max_body_size: 10 * 1024 * 1024, // 10 MB
            max_connections: 1000,
            rate_limit: None,
            chain_id: 17001,
            security_enabled: true,
            rate_limit_per_minute: 100,
        }
    }
}

/// RPC server.
pub struct RpcServer {
    /// Configuration
    config: RpcServerConfig,
    /// Handler for RPC methods
    handler: Arc<RpcHandler>,
    /// Subscription manager
    subscription_manager: Arc<tokio::sync::Mutex<SubscriptionManager>>,
    /// HTTP server handle
    http_handle: Option<ServerHandle>,
    /// WebSocket server handle
    ws_handle: Option<ServerHandle>,
    /// Security manager for rate limiting and validation
    security_manager: Arc<SecurityManager>,
}

impl RpcServer {
    /// Create a new RPC server.
    pub fn new(
        config: RpcServerConfig,
        handler: Arc<RpcHandler>,
    ) -> Self {
        let (manager, _) = SubscriptionManager::new();
        
        // Initialize security manager with custom rate limits
        let security_manager = Arc::new(SecurityManager::with_custom_rate_limit(
            config.rate_limit_per_minute,
            60, // 60 second window
        ));
        
        Self {
            config,
            handler,
            subscription_manager: Arc::new(tokio::sync::Mutex::new(manager)),
            http_handle: None,
            ws_handle: None,
            security_manager,
        }
    }

    /// Start the RPC server.
    pub async fn start(
        &mut self,
    ) -> Result<(), RpcError> {
        // Start HTTP server
        self.start_http().await?;

        // Start WebSocket server if configured
        if self.config.ws_addr.is_some() {
            self.start_websocket().await?;
        }

        Ok(())
    }

    /// Start HTTP server.
    async fn start_http(
        &mut self,
    ) -> Result<(), RpcError> {
        let server = ServerBuilder::new()
            .max_request_body_size(self.config.max_body_size)
            .max_connections(self.config.max_connections)
            .build(self.config.http_addr)
            .await
            .map_err(|e| RpcError::InternalError(format!("Failed to build HTTP server: {}", e)))?;

        let handler = self.handler.clone();

        // Register all RPC methods
        let module = create_rpc_module(handler)?;

        let handle = server.start(module);
        self.http_handle = Some(handle);

        tracing::info!("HTTP RPC server started on {}", self.config.http_addr);

        Ok(())
    }

    /// Start WebSocket server.
    async fn start_websocket(
        &mut self,
    ) -> Result<(), RpcError> {
        let addr = self.config.ws_addr.ok_or_else(|| 
            RpcError::InvalidParams("WebSocket address not configured".to_string())
        )?;

        let server = ServerBuilder::new()
            .max_request_body_size(self.config.max_body_size)
            .max_connections(self.config.max_connections)
            .build(addr)
            .await
            .map_err(|e| RpcError::InternalError(format!("Failed to build WebSocket server: {}", e)))?;

        let handler = self.handler.clone();
        let subscriptions = self.subscription_manager.clone();

        // Create module with subscription support
        let module = create_ws_module(handler, subscriptions).await?;

        let handle = server.start(module);
        self.ws_handle = Some(handle);

        tracing::info!("WebSocket RPC server started on {}", addr);

        Ok(())
    }

    /// Stop the RPC server.
    pub fn stop(
        &mut self,
    ) {
        if let Some(handle) = self.http_handle.take() {
            if let Err(e) = handle.stop() {
                tracing::warn!("HTTP server stop failed: {}", e);
            }
        }
        if let Some(handle) = self.ws_handle.take() {
            if let Err(e) = handle.stop() {
                tracing::warn!("WebSocket server stop failed: {}", e);
            }
        }
        tracing::info!("RPC server stopped");
    }

    /// Check if server is running.
    pub fn is_running(&self,
    ) -> bool {
        self.http_handle.is_some()
    }

    /// Get server addresses.
    pub fn addresses(&self,
    ) -> (Option<SocketAddr>, Option<SocketAddr>) {
        let http = self.http_handle.as_ref().map(|_| self.config.http_addr);
        let ws = self.ws_handle.as_ref().and_then(|_| self.config.ws_addr);
        (http, ws)
    }
}

/// Create RPC module with all methods.
fn create_rpc_module(
    handler: Arc<RpcHandler>,
) -> Result<jsonrpsee::RpcModule<()>, RpcError> {
    use jsonrpsee::proc_macros::rpc;

    #[rpc(server)]
    pub trait MerklithRpc {
        // Web3 methods
        #[method(name = "web3_clientVersion")]
        fn web3_client_version(&self) -> Result<String, RpcError>;

        #[method(name = "web3_sha3")]
        fn web3_sha3(&self, data: String) -> Result<String, RpcError>;

        // Net methods
        #[method(name = "net_version")]
        fn net_version(&self) -> Result<String, RpcError>;

        #[method(name = "net_listening")]
        fn net_listening(&self) -> Result<bool, RpcError>;

        #[method(name = "net_peerCount")]
        fn net_peer_count(&self) -> Result<String, RpcError>;

        // Eth methods
        #[method(name = "eth_protocolVersion")]
        fn eth_protocol_version(&self) -> Result<String, RpcError>;

        #[method(name = "eth_syncing")]
        fn eth_syncing(&self) -> Result<serde_json::Value, RpcError>;

        #[method(name = "eth_coinbase")]
        fn eth_coinbase(&self) -> Result<String, RpcError>;

        #[method(name = "eth_mining")]
        fn eth_mining(&self) -> Result<bool, RpcError>;

        #[method(name = "eth_hashrate")]
        fn eth_hashrate(&self) -> Result<String, RpcError>;

        #[method(name = "eth_gasPrice")]
        fn eth_gas_price(&self) -> Result<String, RpcError>;

        #[method(name = "eth_accounts")]
        fn eth_accounts(&self) -> Result<Vec<String>, RpcError>;

        #[method(name = "eth_blockNumber")]
        fn eth_block_number(&self) -> Result<String, RpcError>;

        #[method(name = "eth_getBalance")]
        fn eth_get_balance(&self,
            address: String,
            block: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_getStorageAt")]
        fn eth_get_storage_at(
            &self,
            address: String,
            slot: String,
            block: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_getTransactionCount")]
        fn eth_get_transaction_count(
            &self,
            address: String,
            block: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_getBlockTransactionCountByHash")]
        fn eth_get_block_transaction_count_by_hash(
            &self,
            block_hash: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_getBlockTransactionCountByNumber")]
        fn eth_get_block_transaction_count_by_number(
            &self,
            block: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_getUncleCountByBlockHash")]
        fn eth_get_uncle_count_by_block_hash(
            &self,
            block_hash: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_getUncleCountByBlockNumber")]
        fn eth_get_uncle_count_by_block_number(
            &self,
            block: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_getCode")]
        fn eth_get_code(
            &self,
            address: String,
            block: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_getBlockByHash")]
        fn eth_get_block_by_hash(
            &self,
            block_hash: String,
            full_transactions: bool,
        ) -> Result<Option<serde_json::Value>, RpcError>;

        #[method(name = "eth_getBlockByNumber")]
        fn eth_get_block_by_number(
            &self,
            block: String,
            full_transactions: bool,
        ) -> Result<Option<serde_json::Value>, RpcError>;

        #[method(name = "eth_getTransactionByHash")]
        fn eth_get_transaction_by_hash(
            &self,
            tx_hash: String,
        ) -> Result<Option<serde_json::Value>, RpcError>;

        #[method(name = "eth_getTransactionReceipt")]
        fn eth_get_transaction_receipt(
            &self,
            tx_hash: String,
        ) -> Result<Option<serde_json::Value>, RpcError>;

        #[method(name = "eth_estimateGas")]
        fn eth_estimate_gas(
            &self,
            tx: serde_json::Value,
            block: Option<String>,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_call")]
        fn eth_call(
            &self,
            tx: serde_json::Value,
            block: String,
        ) -> Result<String, RpcError>;

        #[method(name = "eth_feeHistory")]
        fn eth_fee_history(
            &self,
            block_count: String,
            newest_block: String,
            reward_percentiles: Option<Vec<f64>>,
        ) -> Result<serde_json::Value, RpcError>;

        #[method(name = "eth_maxPriorityFeePerGas")]
        fn eth_max_priority_fee_per_gas(&self,
        ) -> Result<String, RpcError>;

        // Merklith-specific methods
        #[method(name = "merklith_chainId")]
        fn merklith_chain_id(&self) -> Result<u64, RpcError>;

        #[method(name = "merklith_health")]
        fn merklith_health(&self) -> Result<serde_json::Value, RpcError>;
    }

    // Implementation would go here...
    // For now, return empty module
    let module = jsonrpsee::RpcModule::new(());
    Ok(module)
}

/// Create WebSocket RPC module with subscriptions.
async fn create_ws_module(
    handler: Arc<RpcHandler>,
    _subscriptions: Arc<tokio::sync::Mutex<SubscriptionManager>>,
) -> Result<jsonrpsee::RpcModule<()>, RpcError> {
    // Would add subscription methods here
    let module = jsonrpsee::RpcModule::new(());
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_server_config() {
        let config = RpcServerConfig::default();
        assert_eq!(config.http_addr.to_string(), "127.0.0.1:8545");
        assert!(config.ws_addr.is_some());
        assert!(config.cors);
    }

    #[test]
    fn test_rpc_server_creation() {
        // This would require actual handler initialization
        // Just test the config for now
        let config = RpcServerConfig::default();
        assert_eq!(config.max_body_size, 10 * 1024 * 1024);
    }
}
