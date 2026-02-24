use axum::{
    routing::{get, post},
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;

mod handlers;
mod models;
mod websocket;
mod cache;

use handlers::*;
use models::*;
use cache::Cache;

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<Cache>,
    pub rpc_url: String,
    pub db_pool: Option<Arc<RwLock<sqlx::PgPool>>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let rpc_url = std::env::var("MERKLITH_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8545".to_string());
    
    // Initialize cache
    let cache = Arc::new(Cache::new());
    
    let state = AppState {
        cache,
        rpc_url,
        db_pool: None,
    };
    
    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Blocks
        .route("/api/blocks", get(get_blocks))
        .route("/api/blocks/latest", get(get_latest_block))
        .route("/api/blocks/:number", get(get_block_by_number))
        .route("/api/blocks/hash/:hash", get(get_block_by_hash))
        
        // Transactions
        .route("/api/transactions", get(get_transactions))
        .route("/api/transactions/:hash", get(get_transaction))
        .route("/api/transactions/pending", get(get_pending_transactions))
        
        // Accounts
        .route("/api/accounts/:address", get(get_account))
        .route("/api/accounts/:address/transactions", get(get_account_transactions))
        .route("/api/accounts/:address/balance", get(get_account_balance))
        
        // Validators
        .route("/api/validators", get(get_validators))
        .route("/api/validators/:address", get(get_validator))
        .route("/api/validators/stats", get(get_validator_stats))
        
        // Network
        .route("/api/network/stats", get(get_network_stats))
        .route("/api/network/peers", get(get_peers))
        .route("/api/network/syncing", get(get_sync_status))
        
        // Search
        .route("/api/search", get(search))
        
        // WebSocket
        .route("/ws", get(websocket::handle_socket))
        
        // State
        .with_state(state)
        
        // CORS
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3002));
    tracing::info!("Web API server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}