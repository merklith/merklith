//! RPC client for CLI operations.
//!
//! HTTP client for making JSON-RPC calls to the Merklith node.

use merklith_types::{Address, Hash, Transaction, U256};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// RPC client.
#[derive(Debug, Clone)]
pub struct RpcClient {
    url: String,
    client: reqwest::Client,
}

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

impl RpcClient {
    /// Create a new RPC client.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Make an RPC call.
    pub async fn call<T: serde::de::DeserializeOwned + Default>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<T> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        let response = self.client
            .post(&self.url)
            .json(&request)
            .send()
            .await?;

        let rpc_response: RpcResponse<T> = response.json().await?;

        if let Some(error) = rpc_response.error {
            anyhow::bail!("RPC error {}: {}", error.code, error.message);
        }

        rpc_response.result
            .ok_or_else(|| anyhow::anyhow!("Empty result"))
    }

    // ============ Convenience Methods ============

    /// Get chain ID.
    pub async fn chain_id(&self,
    ) -> anyhow::Result<u64> {
        let hex: String = self.call("eth_chainId", json!([])).await?;
        parse_hex_u64(&hex)
    }

    /// Get current block number.
    pub async fn block_number(&self,
    ) -> anyhow::Result<u64> {
        let hex: String = self.call("eth_blockNumber", json!([])).await?;
        parse_hex_u64(&hex)
    }

    /// Get balance.
    pub async fn get_balance(
        &self,
        address: &Address,
    ) -> anyhow::Result<U256> {
        let addr_hex = format!("0x{}", hex::encode(address.as_bytes()));
        let hex: String = self.call(
            "eth_getBalance",
            json!([addr_hex, "latest"]),
        ).await?;
        parse_hex_u256(&hex)
    }

    /// Get transaction count (nonce).
    pub async fn get_transaction_count(
        &self,
        address: &Address,
    ) -> anyhow::Result<u64> {
        let addr_hex = format!("0x{}", hex::encode(address.as_bytes()));
        let hex: String = self.call(
            "eth_getTransactionCount",
            json!([addr_hex, "latest"]),
        ).await?;
        parse_hex_u64(&hex)
    }

    /// Get gas price.
    pub async fn gas_price(&self,
    ) -> anyhow::Result<U256> {
        let hex: String = self.call("eth_gasPrice", json!([])).await?;
        parse_hex_u256(&hex)
    }

    /// Send raw transaction.
    pub async fn send_raw_transaction(
        &self,
        raw_tx: &str,
    ) -> anyhow::Result<Hash> {
        let hash_hex: String = self.call(
            "eth_sendRawTransaction",
            json!([raw_tx]),
        ).await?;
        parse_hash(&hash_hex)
    }

    /// Get transaction receipt.
    pub async fn get_transaction_receipt(
        &self,
        hash: &Hash,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        let hash_hex = format!("0x{}", hex::encode(hash.as_bytes()));
        let result: Option<serde_json::Value> = self.call(
            "eth_getTransactionReceipt",
            json!([hash_hex]),
        ).await?;
        Ok(result)
    }

    /// Get block by number.
    pub async fn get_block_by_number(
        &self,
        number: u64,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        let result: Option<serde_json::Value> = self.call(
            "eth_getBlockByNumber",
            json!([format!("0x{:x}", number), false]),
        ).await?;
        Ok(result)
    }

    /// Get code at address.
    pub async fn get_code(
        &self,
        address: &Address,
    ) -> anyhow::Result<String> {
        let addr_hex = format!("0x{}", hex::encode(address.as_bytes()));
        let code: String = self.call(
            "eth_getCode",
            json!([addr_hex, "latest"]),
        ).await?;
        Ok(code)
    }

    /// Estimate gas.
    pub async fn estimate_gas(
        &self,
        tx: serde_json::Value,
    ) -> anyhow::Result<u64> {
        let hex: String = self.call("eth_estimateGas", json!([tx])).await?;
        parse_hex_u64(&hex)
    }

    /// Call contract.
    pub async fn call_contract(
        &self,
        tx: serde_json::Value,
    ) -> anyhow::Result<String> {
        let result: String = self.call("eth_call", json!([tx, "latest"])).await?;
        Ok(result)
    }

    /// Get node health.
    pub async fn health(&self,
    ) -> anyhow::Result<serde_json::Value> {
        self.call("merklith_health", json!([])).await
    }
}

/// Parse hex u64.
fn parse_hex_u64(hex: &str) -> anyhow::Result<u64> {
    let hex = hex.trim_start_matches("0x");
    u64::from_str_radix(hex, 16)
        .map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))
}

/// Parse hex U256.
fn parse_hex_u256(hex: &str) -> anyhow::Result<U256> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)?;
    let mut padded = [0u8; 32];
    padded[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(U256::from_be_bytes(padded))
}

/// Parse hex hash.
fn parse_hash(hex: &str) -> anyhow::Result<Hash> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)?;
    if bytes.len() != 32 {
        anyhow::bail!("Invalid hash length");
    }
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);
    Ok(Hash::from_bytes(hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_u64() {
        assert_eq!(parse_hex_u64("0x64").unwrap(), 100);
        assert_eq!(parse_hex_u64("0x0").unwrap(), 0);
    }

    #[test]
    fn test_parse_hex_u256() {
        let result = parse_hex_u256("0x64").unwrap();
        assert_eq!(result, U256::from(100u64));
    }
}
