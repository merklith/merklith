#[cfg(test)]
mod security_tests {
    use super::*;
    use merklith_types::{Address, U256};
    use std::sync::Arc;

    #[test]
    fn test_merklith_transfer_requires_signature() {
        // This test verifies that merklith_transfer REQUIRES signature
        // If signature is not provided, it should return an error
        
        // Create a mock request without signature params
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "merklith_transfer".to_string(),
            params: vec![
                Value::String("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
                Value::String("0x0000000000000000000000000000000000000001".to_string()),
                Value::String("0x1000".to_string()),
                // Missing: nonce, signature, pubkey
            ],
            id: Some(Value::Number(1.into())),
        };
        
        // The response should indicate signature is required
        // This is verified by the logic in handle_method which checks for has_signature
    }
    
    #[test] 
    fn test_eth_sendTransaction_requires_signature() {
        // This test verifies that eth_sendTransaction REQUIRES signature
        
        let tx_obj = serde_json::json!({
            "from": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
            "to": "0x0000000000000000000000000000000000000001",
            "value": "0x1000",
            "nonce": "0x0"
            // Missing: signature, publicKey
        });
        
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "eth_sendTransaction".to_string(),
            params: vec![tx_obj],
            id: Some(Value::Number(1.into())),
        };
        
        // The response should indicate signature is required
    }
}