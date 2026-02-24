//! JSON-RPC method handlers.

use merklith_types::{Address, U256};
use merklith_txpool::pool::TransactionPool;
use merklith_storage::state_db::StateDB;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum RpcError {
    InternalError(String),
    InvalidParams(String),
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcError::InternalError(e) => write!(f, "Internal error: {}", e),
            RpcError::InvalidParams(e) => write!(f, "Invalid params: {}", e),
        }
    }
}

impl std::error::Error for RpcError {}

pub struct RpcHandler {
    pub state_db: Arc<StateDB>,
    pub tx_pool: Arc<Mutex<TransactionPool>>,
    pub chain_id: u64,
    pub current_block: Arc<AtomicU64>,
    pub gas_price: U256,
}

impl RpcHandler {
    pub fn new(
        state_db: Arc<StateDB>,
        tx_pool: Arc<Mutex<TransactionPool>>,
        chain_id: u64,
    ) -> Self {
        Self {
            state_db,
            tx_pool,
            chain_id,
            current_block: Arc::new(AtomicU64::new(0)),
            gas_price: U256::from(1_000_000_000u64),
        }
    }
}
