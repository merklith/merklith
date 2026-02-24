//! HTTP client for interacting with Merklith RPC.

use merklith_types::{Address, Hash, Transaction, U256};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

use crate::errors::{Result, SdkError};
use crate::types::*;

/// RPC request.
#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

/// RPC response.
#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    #[serde(default)]
    result: Option<T>,
    #[serde(default)]
    error: Option<RpcError>,
}

/// RPC error.
#[derive(Debug, Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

/// Merklith SDK client.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    url: String,
    chain_id: Option<u64>,
}

impl Client {
    /// Create a new client.
    pub fn new(url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        
        Self {
            http,
            url: url.into(),
            chain_id: None,
        }
    }

    /// Connect to RPC endpoint.
    pub async fn connect(url: impl Into<String>) -> Result<Self> {
        let client = Self::new(url);
        let chain_id = client.chain_id().await?;
        Ok(client.with_chain_id(chain_id))
    }

    /// Set chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Get chain ID.
    pub async fn chain_id(&self,
    ) -> Result<u64> {
        let hex: String = self.request("eth_chainId", json!([])).await?;
        parse_hex_u64(&hex)
    }

    /// Get current block number.
    pub async fn get_block_number(&self,
    ) -> Result<u64> {
        let hex: String = self.request("eth_blockNumber", json!([])).await?;
        parse_hex_u64(&hex)
    }

    /// Get balance.
    pub async fn get_balance(
        &self,
        address: &Address,
    ) -> Result<U256> {
        let addr_hex = format_address(address);
        let hex: String = self.request(
            "eth_getBalance",
            json!([addr_hex, "latest"]),
        ).await?;
        parse_hex_u256(&hex)
    }

    /// Get transaction count (nonce).
    pub async fn get_transaction_count(
        &self,
        address: &Address,
    ) -> Result<u64> {
        let addr_hex = format_address(address);
        let hex: String = self.request(
            "eth_getTransactionCount",
            json!([addr_hex, "latest"]),
        ).await?;
        parse_hex_u64(&hex)
    }

    /// Get gas price.
    pub async fn get_gas_price(&self,
    ) -> Result<U256> {
        let hex: String = self.request("eth_gasPrice", json!([])).await?;
        parse_hex_u256(&hex)
    }

    /// Get block by number.
    pub async fn get_block(
        &self,
        number: BlockId,
    ) -> Result<Option<serde_json::Value>> {
        self.request(
            "eth_getBlockByNumber",
            json!([number.to_string(), false]),
        ).await
    }

    /// Get transaction by hash.
    pub async fn get_transaction(
        &self,
        hash: &Hash,
    ) -> Result<Option<serde_json::Value>> {
        let hash_hex = format_hash(hash);
        self.request(
            "eth_getTransactionByHash",
            json!([hash_hex]),
        ).await
    }

    /// Get transaction receipt.
    pub async fn get_transaction_receipt(
        &self,
        hash: &Hash,
    ) -> Result<Option<TransactionReceipt>> {
        let hash_hex = format_hash(hash);
        let result: Option<serde_json::Value> = self.request(
            "eth_getTransactionReceipt",
            json!([hash_hex]),
        ).await?;

        result.map(parse_receipt).transpose()
    }

    /// Get code at address.
    pub async fn get_code(
        &self,
        address: &Address,
    ) -> Result<Vec<u8>> {
        let addr_hex = format_address(address);
        let hex: String = self.request(
            "eth_getCode",
            json!([addr_hex, "latest"]),
        ).await?;
        
        hex::decode(hex.trim_start_matches("0x"))
            .map_err(|e| SdkError::Serialization(e.to_string()))
    }

    /// Estimate gas.
    pub async fn estimate_gas(
        &self,
        tx: &Transaction,
    ) -> Result<u64> {
        let tx_json = transaction_to_json(tx);
        let hex: String = self.request(
            "eth_estimateGas",
            json!([tx_json]),
        ).await?;
        parse_hex_u64(&hex)
    }

    /// Call contract (read-only).
    pub async fn call(
        &self,
        tx: &Transaction,
        block: Option<BlockId>,
    ) -> Result<Vec<u8>> {
        let tx_json = transaction_to_json(tx);
        let block = block.unwrap_or(BlockId::Latest);
        
        let hex: String = self.request(
            "eth_call",
            json!([tx_json, block.to_string()]),
        ).await?;
        
        hex::decode(hex.trim_start_matches("0x"))
            .map_err(|e| SdkError::Serialization(e.to_string()))
    }

    /// Send raw transaction.
    pub async fn send_transaction(
        &self,
        _tx: &Transaction,
    ) -> Result<Hash> {
        // TODO: Implement proper transaction serialization
        // For now, return placeholder
        let tx_hex = "0x0".to_string();
        
        let hash_hex: String = self.request(
            "eth_sendRawTransaction",
            json!([tx_hex]),
        ).await?;
        
        parse_hash(&hash_hex)
    }

    /// Get logs.
    pub async fn get_logs(
        &self,
        filter: &Filter,
    ) -> Result<Vec<Log>> {
        let filter_json = filter_to_json(filter);
        let logs: Vec<serde_json::Value> = self.request(
            "eth_getLogs",
            json!([filter_json]),
        ).await?;
        
        logs.into_iter()
            .map(parse_log)
            .collect()
    }

    /// Wait for transaction receipt.
    pub async fn wait_for_transaction(
        &self,
        hash: &Hash,
        timeout: Duration,
    ) -> Result<TransactionReceipt> {
        let start = std::time::Instant::now();
        
        loop {
            if let Some(receipt) = self.get_transaction_receipt(hash).await? {
                return Ok(receipt);
            }
            
            if start.elapsed() > timeout {
                return Err(SdkError::Timeout("Transaction receipt timeout".to_string()));
            }
            
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// Make RPC request.
    async fn request<T: serde::de::DeserializeOwned + Default>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        let response_text = self.http
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .text()
            .await?;
        
        let response: RpcResponse<T> = serde_json::from_str(&response_text)
            .map_err(|e| SdkError::Serialization(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = response.error {
            return Err(SdkError::Rpc(format!(
                "{}: {}", error.code, error.message
            )));
        }

        response.result.ok_or_else(|| {
            SdkError::Rpc("Empty result".to_string())
        })
    }
}

/// Format address as hex.
fn format_address(addr: &Address) -> String {
    format!("0x{}", hex::encode(addr.as_bytes()))
}

/// Format hash as hex.
fn format_hash(hash: &Hash) -> String {
    format!("0x{}", hex::encode(hash.as_bytes()))
}

/// Parse hex u64.
fn parse_hex_u64(hex: &str) -> Result<u64> {
    let hex = hex.trim_start_matches("0x");
    u64::from_str_radix(hex, 16)
        .map_err(|e| SdkError::Serialization(e.to_string()))
}

/// Parse hex U256.
fn parse_hex_u256(hex: &str) -> Result<U256> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)
        .map_err(|e| SdkError::Serialization(e.to_string()))?;
    
    let mut padded = [0u8; 32];
    padded[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(U256::from_be_bytes(padded))
}

/// Parse hash.
fn parse_hash(hex: &str) -> Result<Hash> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)
        .map_err(|e| SdkError::Serialization(e.to_string()))?;
    
    if bytes.len() != 32 {
        return Err(SdkError::Serialization("Invalid hash length".to_string()));
    }
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);
    Ok(Hash::from_bytes(hash))
}

/// Convert transaction to JSON.
fn transaction_to_json(tx: &Transaction) -> serde_json::Value {
    json!({
        "to": tx.to.map(|a| format_address(&a)),
        "data": format!("0x{}", hex::encode(&tx.data)),
        "value": format!("0x{}", hex::encode(tx.value.to_be_bytes())),
        "gas": format!("0x{:x}", tx.gas_limit),
        "maxFeePerGas": format!("0x{}", hex::encode(tx.max_fee_per_gas.to_be_bytes())),
        "maxPriorityFeePerGas": format!("0x{}", hex::encode(tx.max_priority_fee_per_gas.to_be_bytes())),
        "nonce": format!("0x{:x}", tx.nonce),
        "chainId": format!("0x{:x}", tx.chain_id),
    })
}

/// Convert filter to JSON.
fn filter_to_json(filter: &Filter) -> serde_json::Value {
    let mut json = serde_json::Map::new();
    
    if let Some(from) = &filter.from_block {
        json.insert("fromBlock".to_string(), json!(from.to_string()));
    }
    
    if let Some(to) = &filter.to_block {
        json.insert("toBlock".to_string(), json!(to.to_string()));
    }
    
    if !filter.addresses.is_empty() {
        let addresses: Vec<String> = filter.addresses.iter()
            .map(format_address)
            .collect();
        json.insert("address".to_string(), json!(addresses));
    }
    
    json!(json)
}

/// Parse receipt.
fn parse_receipt(value: serde_json::Value) -> Result<TransactionReceipt> {
    // Simplified parsing
    Ok(TransactionReceipt {
        transaction_hash: [0u8; 32],
        transaction_index: 0,
        block_hash: [0u8; 32],
        block_number: 0,
        from: Address::ZERO,
        to: None,
        gas_used: 0,
        status: 1,
        logs: vec![],
    })
}

/// Parse log.
fn parse_log(value: serde_json::Value) -> Result<Log> {
    Ok(Log {
        address: Address::ZERO,
        topics: vec![],
        data: vec![],
        block_number: 0,
        transaction_hash: [0u8; 32],
        log_index: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new("http://localhost:8545");
        assert_eq!(client.url, "http://localhost:8545");
    }

    #[test]
    fn test_format_address() {
        let addr = Address::ZERO;
        let formatted = format_address(&addr);
        assert_eq!(formatted, "0x0000000000000000000000000000000000000000");
    }
}
