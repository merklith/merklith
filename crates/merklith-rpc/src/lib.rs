//! Merklith RPC Server - Native merklith_* methods
//!
//! This implements the Merklith-specific RPC API with Ethereum compatibility

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use merklith_core::state_machine::State;

pub mod security;
pub use security::{SecurityManager, SecurityError, RateLimiter, ReplayProtection, InputValidator};

/// RPC configuration
#[derive(Debug, Clone)]
pub struct RpcServerConfig {
    pub http_addr: SocketAddr,
    pub http_port: u16,
    pub ws_addr: Option<SocketAddr>,
    pub cors: bool,
    pub max_body_size: u32,
    pub max_connections: u32,
    pub rate_limit: Option<u32>,
}

impl Default for RpcServerConfig {
    fn default() -> Self {
        Self {
            http_addr: "0.0.0.0:8545".parse().unwrap_or_else(|_| {
                std::net::SocketAddr::from(([0, 0, 0, 0], 8545))
            }),
            http_port: 8545,
            ws_addr: Some("0.0.0.0:8546".parse().unwrap_or_else(|_| {
                std::net::SocketAddr::from(([0, 0, 0, 0], 8546))
            })),
            cors: true,
            max_body_size: 10 * 1024 * 1024,
            max_connections: 100,
            rate_limit: None,
        }
    }
}

/// JSON-RPC Request
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Vec<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC Response
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC Error
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// RPC Server
pub struct RpcServer {
    config: RpcServerConfig,
    state: Arc<State>,
    chain_id: u64,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl RpcServer {
    pub fn new(config: RpcServerConfig, state: Arc<State>, chain_id: u64) -> Self {
        Self { config, state, chain_id, shutdown_tx: None }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        let addr = self.config.http_addr;
        let state = self.state.clone();
        let chain_id = self.chain_id;
        
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let server = hyper::Server::bind(&addr).serve(hyper::service::make_service_fn(move |_| {
            let state = state.clone();
            let chain_id = chain_id;
            async move {
                Ok::<_, hyper::Error>(hyper::service::service_fn(move |req| {
                    let state = state.clone();
                    let chain_id = chain_id;
                    async move {
                        handle_rpc_request(req, state, chain_id).await
                    }
                }))
            }
        }));

        let server = server.with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });

        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("RPC server error: {}", e);
            }
        });

        tracing::info!("Merklith RPC server listening on {}", addr);
        Ok(())
    }
}

async fn handle_rpc_request(
    req: hyper::Request<hyper::Body>,
    state: Arc<State>,
    chain_id: u64,
) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
    // Handle CORS preflight requests
    if req.method() == hyper::Method::OPTIONS {
        return Ok(hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
            .header("Access-Control-Max-Age", "86400")
            .body(hyper::Body::empty())
            .unwrap_or_else(|_| hyper::Response::new(hyper::Body::empty())));
    }

    if req.method() != hyper::Method::POST {
        // Build response safely without expect
        let response = hyper::Response::builder()
            .status(hyper::StatusCode::METHOD_NOT_ALLOWED)
            .header("Access-Control-Allow-Origin", "*")
            .body(hyper::Body::from("Only POST allowed"))
            .unwrap_or_else(|_| {
                // If even the fallback fails, return a minimal valid response
                hyper::Response::new(hyper::Body::from("Error"))
            });
        return Ok(response);
    }

    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let rpc_req: JsonRpcRequest = match serde_json::from_slice(&body_bytes) {
        Ok(r) => r,
        Err(e) => {
            // Build response safely without expect
            let response = hyper::Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .header("Access-Control-Allow-Origin", "*")
                .body(hyper::Body::from(format!("Invalid JSON: {}", e)))
                .unwrap_or_else(|_| {
                    // If even the fallback fails, return a minimal valid response
                    hyper::Response::new(hyper::Body::from("Invalid JSON"))
                });
            return Ok(response);
        }
    };

    let response = handle_method(&rpc_req, state, chain_id);

    let body = serde_json::to_string(&response).unwrap_or_default();
    Ok(hyper::Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .body(hyper::Body::from(body))
        .unwrap_or_else(|_| {
            hyper::Response::new(hyper::Body::from(
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":null}"#
            ))
        }))
}

fn handle_method(req: &JsonRpcRequest, state: Arc<State>, chain_id: u64) -> JsonRpcResponse {
    match req.method.as_str() {
        // === Chain Info ===
        "merklith_chainId" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String(format!("0x{:x}", chain_id))),
            error: None,
            id: req.id.clone(),
        },
        
        "merklith_blockNumber" => {
            let block = state.block_number();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("0x{:x}", block))),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_getBalance" => {
            let addr_str = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let balance = if let Ok(addr) = parse_address(addr_str) {
                state.balance(&addr)
            } else {
                U256::ZERO
            };
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("{:x}", balance))),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_getNonce" => {
            let addr_str = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let nonce = if let Ok(addr) = parse_address(addr_str) {
                state.nonce(&addr)
            } else {
                0
            };
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("0x{:x}", nonce))),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_sendRawTransaction" => {
            // TODO: Real transaction parsing and execution
            let tx_hash = format!("0x{}", hex::encode(&rand::random::<[u8; 32]>()));
            tracing::info!("Transaction received: {}", tx_hash);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(tx_hash)),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_sendSignedTransaction" => {
            let from_str = req.params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let to_str = req.params.get(1).and_then(|v| v.as_str()).unwrap_or("");
            let amount_str = req.params.get(2).and_then(|v| v.as_str()).unwrap_or("0");
            let nonce_str = req.params.get(3).and_then(|v| v.as_str()).unwrap_or("0");
            let sig_str = req.params.get(4).and_then(|v| v.as_str()).unwrap_or("");
            let pubkey_str = req.params.get(5).and_then(|v| v.as_str()).unwrap_or("");
            
            match (parse_address(from_str), parse_address(to_str), parse_u256(amount_str), 
                   parse_u64(nonce_str), hex::decode(sig_str.strip_prefix("0x").unwrap_or(&sig_str)),
                   hex::decode(pubkey_str.strip_prefix("0x").unwrap_or(&pubkey_str))) {
                (Ok(from), Ok(to), Ok(amount), Ok(nonce), Ok(sig_bytes), Ok(pk_bytes)) 
                    if sig_bytes.len() == 64 && pk_bytes.len() == 32 => {
                    // Verify nonce
                    let expected_nonce = state.nonce(&from);
                    if nonce != expected_nonce {
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32001,
                                message: format!("Invalid nonce: expected {}, got {}", expected_nonce, nonce),
                            }),
                            id: req.id.clone(),
                        }
                    } else {
                        // Create and verify signature
                        use merklith_types::{Transaction, Ed25519Signature, Ed25519PublicKey};
                        use merklith_crypto::ed25519_verify;
                        
                        let tx = Transaction::new(
                            chain_id,
                            nonce,
                            Some(to),
                            amount,
                            21000,
                            U256::from(1_000_000_000u64),
                            U256::from(1_000_000u64),
                        );
                        
                        let signing_hash = tx.signing_hash();
                        let signature = match sig_bytes.as_slice().try_into() {
                            Ok(bytes) => Ed25519Signature::from_bytes(bytes),
                            Err(_) => {
                                return JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32602,
                                        message: "Invalid signature length".to_string(),
                                    }),
                                    id: req.id.clone(),
                                };
                            }
                        };
                        let public_key = match pk_bytes.as_slice().try_into() {
                            Ok(bytes) => Ed25519PublicKey::from_bytes(bytes),
                            Err(_) => {
                                return JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32602,
                                        message: "Invalid public key length".to_string(),
                                    }),
                                    id: req.id.clone(),
                                };
                            }
                        };
                        
                        // Verify signature
                        match ed25519_verify(&public_key, signing_hash.as_bytes(), &signature) {
                            Ok(_) => {
                                // Execute transfer
                                match state.transfer(&from, &to, amount) {
                                    Ok(tx_hash) => {
                                        let hash_hex = format!("0x{}", hex::encode(tx_hash.as_bytes()));
                                        JsonRpcResponse {
                                            jsonrpc: "2.0".to_string(),
                                            result: Some(Value::String(hash_hex)),
                                            error: None,
                                            id: req.id.clone(),
                                        }
                                    }
                                    Err(e) => JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError {
                                            code: -32000,
                                            message: e,
                                        }),
                                        id: req.id.clone(),
                                    }
                                }
                            }
                            Err(e) => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32002,
                                    message: format!("Invalid signature: {}", e),
                                }),
                                id: req.id.clone(),
                            }
                        }
                    }
                }
                _ => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params (need: from, to, amount, nonce, signature[64 bytes], pubkey[32 bytes])".to_string(),
                    }),
                    id: req.id.clone(),
                }
            }
        },
        
        "merklith_signAndSendTransaction" => {
            // SECURITY: This method is DISABLED to prevent private key exposure
            // Private keys should NEVER be sent over RPC or stored in logs
            // Use merklith_sendSignedTransaction with pre-signed transactions instead
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Method disabled for security: Use merklith_sendSignedTransaction with pre-signed transactions instead".to_string(),
                }),
                id: req.id.clone(),
            }
        },
        
        "merklith_transfer" => {
            // SECURITY WARNING: This method requires signature verification
            // For development: params = [from, to, amount, nonce, signature, pubkey]
            let from_str = req.params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let to_str = req.params.get(1).and_then(|v| v.as_str()).unwrap_or("");
            let amount_str = req.params.get(2).and_then(|v| v.as_str()).unwrap_or("0");
            let nonce_str = req.params.get(3).and_then(|v| v.as_str()).unwrap_or("");
            let sig_str = req.params.get(4).and_then(|v| v.as_str()).unwrap_or("");
            let pubkey_str = req.params.get(5).and_then(|v| v.as_str()).unwrap_or("");
            
            tracing::info!("Transfer request: from={}, to={}, amount={}", from_str, to_str, amount_str);
            
            // Signature verification is REQUIRED for security
            let has_signature = !nonce_str.is_empty() && !sig_str.is_empty() && !pubkey_str.is_empty();
            
            // Reject transfers without signature
            if !has_signature {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Signature required: params = [from, to, amount, nonce, signature, pubkey]".to_string(),
                    }),
                    id: req.id.clone(),
                };
            }
            
            match (parse_address(from_str), parse_address(to_str), parse_u256(amount_str)) {
                (Ok(from), Ok(to), Ok(amount)) => {
                    tracing::info!("Parsed addresses successfully");
                    
                    // Verify nonce and signature
                        match parse_u64(nonce_str) {
                            Ok(nonce) => {
                                let expected_nonce = state.nonce(&from);
                                if nonce != expected_nonce {
                                    return JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError {
                                            code: -32001,
                                            message: format!("Invalid nonce: expected {}, got {}", expected_nonce, nonce),
                                        }),
                                        id: req.id.clone(),
                                    };
                                }
                                
                                // Verify signature
                                use merklith_types::{Transaction, Ed25519Signature, Ed25519PublicKey};
                                use merklith_crypto::ed25519_verify;
                                
                                match (hex::decode(sig_str.strip_prefix("0x").unwrap_or(&sig_str)),
                                       hex::decode(pubkey_str.strip_prefix("0x").unwrap_or(&pubkey_str))) {
                                    (Ok(sig_bytes), Ok(pk_bytes)) if sig_bytes.len() == 64 && pk_bytes.len() == 32 => {
                                        let tx = Transaction::new(
                                            chain_id,
                                            nonce,
                                            Some(to),
                                            amount,
                                            21000,
                                            U256::from(1_000_000_000u64),
                                            U256::from(1_000_000u64),
                                        );
                                        
                                        let signing_hash = tx.signing_hash();
                                        let signature = match sig_bytes.as_slice().try_into() {
                                            Ok(bytes) => Ed25519Signature::from_bytes(bytes),
                                            Err(_) => {
                                                return JsonRpcResponse {
                                                    jsonrpc: "2.0".to_string(),
                                                    result: None,
                                                    error: Some(JsonRpcError {
                                                        code: -32602,
                                                        message: "Invalid signature length".to_string(),
                                                    }),
                                                    id: req.id.clone(),
                                                };
                                            }
                                        };
                                        let public_key = match pk_bytes.as_slice().try_into() {
                                            Ok(bytes) => Ed25519PublicKey::from_bytes(bytes),
                                            Err(_) => {
                                                return JsonRpcResponse {
                                                    jsonrpc: "2.0".to_string(),
                                                    result: None,
                                                    error: Some(JsonRpcError {
                                                        code: -32602,
                                                        message: "Invalid public key length".to_string(),
                                                    }),
                                                    id: req.id.clone(),
                                                };
                                            }
                                        };
                                        
                                        match ed25519_verify(&public_key, signing_hash.as_bytes(), &signature) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                return JsonRpcResponse {
                                                    jsonrpc: "2.0".to_string(),
                                                    result: None,
                                                    error: Some(JsonRpcError {
                                                        code: -32002,
                                                        message: format!("Invalid signature: {}", e),
                                                    }),
                                                    id: req.id.clone(),
                                                };
                                            }
                                        }
                                    }
                                    _ => {
                                        return JsonRpcResponse {
                                            jsonrpc: "2.0".to_string(),
                                            result: None,
                                            error: Some(JsonRpcError {
                                                code: -32602,
                                                message: "Invalid signature or public key format".to_string(),
                                            }),
                                            id: req.id.clone(),
                                        };
                                    }
                                }
                            }
                            Err(_) => {
                                return JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32602,
                                        message: "Invalid nonce format".to_string(),
                                    }),
                                    id: req.id.clone(),
                                };
                            }
                        }
                    
                    match state.transfer(&from, &to, amount) {
                        Ok(tx_hash) => {
                            let hash_hex = format!("0x{}", hex::encode(tx_hash.as_bytes()));
                            tracing::info!("Transfer successful: {}", hash_hex);
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(Value::String(hash_hex)),
                                error: None,
                                id: req.id.clone(),
                            }
                        }
                        Err(e) => {
                            tracing::error!("Transfer failed: {}", e);
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32000,
                                    message: e,
                                }),
                                id: req.id.clone(),
                            }
                        }
                    }
                }
                (from_err, to_err, amt_err) => {
                    tracing::error!("Parse failed: from={:?}, to={:?}, amount={:?}", from_err, to_err, amt_err);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Invalid params".to_string(),
                        }),
                        id: req.id.clone(),
                    }
                }
            }
        },
        
        "merklith_gasPrice" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x3b9aca00".to_string())), // 1 gwei in sparks
            error: None,
            id: req.id.clone(),
        },
        
        "merklith_estimateGas" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x5208".to_string())), // 21000
            error: None,
            id: req.id.clone(),
        },
        
        "merklith_version" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("merklith/0.1.0".to_string())),
            error: None,
            id: req.id.clone(),
        },
        
        "merklith_syncing" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::Bool(false)),
            error: None,
            id: req.id.clone(),
        },
        
        "merklith_createWallet" => {
            use merklith_crypto::Keypair;
            let keypair = Keypair::generate();
            let address = keypair.address();
            let private_key = hex::encode(keypair.to_bytes());
            let result = serde_json::json!({
                "address": format!("0x{}", hex::encode(address.as_bytes())),
                "privateKey": format!("0x{}", private_key)
            });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_getBlockByNumber" => {
            let block_num = req.params.first()
                .and_then(|v| v.as_str())
                .and_then(|s| if s == "latest" { Some(state.block_number()) } else { u64::from_str_radix(s.trim_start_matches("0x"), 16).ok() })
                .unwrap_or(state.block_number());
            
            match state.get_block(block_num) {
                Some(block) => {
                    let result = serde_json::json!({
                        "number": format!("0x{:x}", block.number),
                        "hash": format!("0x{}", hex::encode(block.hash)),
                        "parentHash": format!("0x{}", hex::encode(block.parent_hash)),
                        "nonce": "0x0000000000000000",
                        "transactions": [],
                        "gasLimit": "0x1c9c380",
                        "gasUsed": "0x0",
                        "timestamp": format!("0x{:x}", block.timestamp),
                    });
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(result),
                        error: None,
                        id: req.id.clone(),
                    }
                }
                None => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(Value::Null),
                    error: None,
                    id: req.id.clone(),
                }
            }
        },
        
        "merklith_getTransactionByHash" => {
            let tx_hash = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let result = serde_json::json!({
                "hash": tx_hash,
                "blockNumber": "0x1",
                "from": "0x742d35cc6634c0532925a3b844bc9e7595f0beb0",
                "to": "0x8ba1f109551bd432803012645ac136ddd64dba72",
                "value": "0xde0b6b3a7640000",
                "gas": "0x5208",
                "gasPrice": "0x3b9aca00",
                "status": "0x1"
            });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_accounts" => {
            let accounts: Vec<String> = state.all_accounts()
                .iter()
                .map(|(addr, _)| format!("0x{}", hex::encode(addr)))
                .collect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(accounts).unwrap()),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_getBlockInfo" => {
            let block_num = req.params.first()
                .and_then(|v| v.as_str())
                .and_then(|s| if s == "latest" { Some(state.block_number()) } else { u64::from_str_radix(s.trim_start_matches("0x"), 16).ok() })
                .unwrap_or(state.block_number());
            
            match state.get_block(block_num) {
                Some(block) => {
                    let result = serde_json::json!({
                        "number": format!("0x{:x}", block.number),
                        "hash": format!("0x{}", hex::encode(block.hash)),
                        "parentHash": format!("0x{}", hex::encode(block.parent_hash)),
                        "timestamp": format!("0x{:x}", block.timestamp),
                        "txCount": block.tx_count,
                    });
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(result),
                        error: None,
                        id: req.id.clone(),
                    }
                }
                None => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32001,
                        message: format!("Block {} not found", block_num),
                    }),
                    id: req.id.clone(),
                }
            }
        },
        
        "merklith_getCurrentBlockHash" => {
            let hash = state.block_hash();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("0x{}", hex::encode(hash.as_bytes())))),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_getBlockChain" => {
            let current = state.block_number();
            let from = req.params.get(0)
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = req.params.get(1)
                .and_then(|v| v.as_u64())
                .unwrap_or(10).min(100);
            
            let blocks: Vec<_> = (from..=current.min(from + count - 1))
                .filter_map(|n| state.get_block(n))
                .map(|b| serde_json::json!({
                    "number": format!("0x{:x}", b.number),
                    "hash": format!("0x{}", hex::encode(b.hash)),
                    "parentHash": format!("0x{}", hex::encode(b.parent_hash)),
                    "timestamp": format!("0x{:x}", b.timestamp),
                }))
                .collect();
            
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(blocks).unwrap()),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_getChainStats" => {
            let block_number = state.block_number();
            let block_hash = state.block_hash();
            
            let result = serde_json::json!({
                "chainId": format!("0x{:x}", chain_id),
                "blockNumber": format!("0x{:x}", block_number),
                "blockHash": format!("0x{}", hex::encode(block_hash.as_bytes())),
                "accounts": state.all_accounts().len(),
            });
            
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: req.id.clone(),
            }
        },
        
        "merklith_createAttestation" => {
            let private_key_str = req.params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let block_num_str = req.params.get(1).and_then(|v| v.as_str()).unwrap_or("0");
            
            let pk_hex = private_key_str.strip_prefix("0x").unwrap_or(private_key_str);
            
            match (hex::decode(pk_hex), parse_u64(block_num_str)) {
                (Ok(pk_bytes), Ok(block_num)) if pk_bytes.len() == 32 => {
                    match state.get_block(block_num) {
                        Some(block) => {
                            use merklith_crypto::Keypair;
                            use merklith_crypto::bls::BLSKeypair;
                            
                            let ed_keypair = Keypair::from_seed(&pk_bytes.as_slice().try_into().unwrap());
                            let attester = ed_keypair.address();
                            
                            let bls_seed: [u8; 32] = pk_bytes.as_slice().try_into().unwrap();
                            let bls_keypair = match BLSKeypair::from_bytes(&bls_seed) {
                                Ok(kp) => kp,
                                Err(e) => return JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32003,
                                        message: format!("BLS key error: {}", e),
                                    }),
                                    id: req.id.clone(),
                                }
                            };
                            
                            let mut msg = Vec::new();
                            msg.extend_from_slice(&block_num.to_le_bytes());
                            msg.extend_from_slice(&block.hash);
                            
                            let signature = bls_keypair.sign(&msg);
                            
                            let result = serde_json::json!({
                                "blockNumber": format!("0x{:x}", block_num),
                                "blockHash": format!("0x{}", hex::encode(block.hash)),
                                "attester": format!("0x{}", hex::encode(attester)),
                                "signature": format!("0x{}", hex::encode(signature.as_bytes())),
                            });
                            
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(result),
                                error: None,
                                id: req.id.clone(),
                            }
                        }
                        None => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32001,
                                message: format!("Block {} not found", block_num),
                            }),
                            id: req.id.clone(),
                        }
                    }
                }
                _ => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params (need: privateKey[32 bytes], blockNumber)".to_string(),
                    }),
                    id: req.id.clone(),
                }
            }
        },
        
        "merklith_deployContract" => {
            let from_str = req.params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let code_str = req.params.get(1).and_then(|v| v.as_str()).unwrap_or("");
            
            // Validate bytecode size (EIP-170 limit: 24KB)
            const MAX_BYTECODE_SIZE: usize = 24 * 1024;
            if code_str.len() > MAX_BYTECODE_SIZE * 2 + 2 { // +2 for "0x" prefix
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Bytecode exceeds maximum size of 24KB (EIP-170)".to_string(),
                    }),
                    id: req.id.clone(),
                };
            }
            
            let code = if code_str.starts_with("0x") {
                match hex::decode(&code_str[2..]) {
                    Ok(c) => c,
                    Err(_) => return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Invalid bytecode".to_string(),
                        }),
                        id: req.id.clone(),
                    }
                }
            } else {
                vec![]
            };
            
            match parse_address(from_str) {
                Ok(from) => {
                    match state.deploy_contract(&from, code) {
                        Ok(contract_addr) => {
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(Value::String(format!("0x{}", hex::encode(contract_addr)))),
                                error: None,
                                id: req.id.clone(),
                            }
                        }
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32000,
                                message: e,
                            }),
                            id: req.id.clone(),
                        }
                    }
                }
                Err(_) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid address".to_string(),
                    }),
                    id: req.id.clone(),
                }
            }
        },
        
        "merklith_getCode" => {
            let addr_str = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            match parse_address(addr_str) {
                Ok(addr) => {
                    let code = state.get_code(&addr);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(Value::String(format!("0x{}", hex::encode(&code)))),
                        error: None,
                        id: req.id.clone(),
                    }
                }
                Err(_) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid address".to_string(),
                    }),
                    id: req.id.clone(),
                }
            }
        },
        
        "merklith_getStorageAt" => {
            let addr_str = req.params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let key_str = req.params.get(1).and_then(|v| v.as_str()).unwrap_or("0x0000000000000000000000000000000000000000000000000000000000000000");
            
            match (parse_address(addr_str), parse_bytes32(key_str)) {
                (Ok(addr), Ok(key)) => {
                    let value = state.get_storage(&addr, key).unwrap_or([0u8; 32]);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(Value::String(format!("0x{}", hex::encode(value)))),
                        error: None,
                        id: req.id.clone(),
                    }
                }
                _ => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                    }),
                    id: req.id.clone(),
                }
            }
        },
        
        "merklith_call" => {
            let to_str = req.params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let data_str = req.params.get(1).and_then(|v| v.as_str()).unwrap_or("");
            
            // Validate call data size to prevent DoS (max 128KB)
            const MAX_CALL_DATA_SIZE: usize = 128 * 1024;
            if data_str.len() > MAX_CALL_DATA_SIZE * 2 + 2 { // +2 for "0x" prefix
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Call data exceeds maximum size of 128KB".to_string(),
                    }),
                    id: req.id.clone(),
                };
            }
            
            match parse_address(to_str) {
                Ok(to) => {
                    let code = state.get_code(&to);
                    let input = if data_str.starts_with("0x") {
                        hex::decode(&data_str[2..]).unwrap_or_default()
                    } else {
                        vec![]
                    };
                    
                    // Execute in VM
                    match execute_contract(&code, &input) {
                        Ok(result) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(Value::String(format!("0x{}", hex::encode(&result)))),
                            error: None,
                            id: req.id.clone(),
                        },
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32000,
                                message: e,
                            }),
                            id: req.id.clone(),
                        }
                    }
                }
                Err(_) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid address".to_string(),
                    }),
                    id: req.id.clone(),
                }
            }
        },
        
        // ============================================================
        // Ethereum Compatibility Aliases
        // These allow tools like MetaMask, web3.js, ethers.js to work
        // ============================================================

        // --- Chain/Node Info ---

        "eth_chainId" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String(format!("0x{:x}", chain_id))),
            error: None,
            id: req.id.clone(),
        },

        "eth_blockNumber" => {
            let block = state.block_number();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("0x{:x}", block))),
                error: None,
                id: req.id.clone(),
            }
        },

        "eth_gasPrice" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x3b9aca00".to_string())),
            error: None,
            id: req.id.clone(),
        },

        "eth_estimateGas" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x5208".to_string())),
            error: None,
            id: req.id.clone(),
        },

        "eth_syncing" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::Bool(false)),
            error: None,
            id: req.id.clone(),
        },

        "eth_mining" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::Bool(true)),
            error: None,
            id: req.id.clone(),
        },

        "eth_hashrate" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x0".to_string())),
            error: None,
            id: req.id.clone(),
        },

        "eth_protocolVersion" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x41".to_string())),
            error: None,
            id: req.id.clone(),
        },

        "eth_coinbase" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x0000000000000000000000000000000000000000".to_string())),
            error: None,
            id: req.id.clone(),
        },

        "eth_feeHistory" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({
                "baseFeePerGas": ["0x3b9aca00"],
                "gasUsedRatio": [0.0],
                "oldestBlock": "0x0",
                "reward": [["0x0"]]
            })),
            error: None,
            id: req.id.clone(),
        },

        "eth_maxPriorityFeePerGas" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x0".to_string())),
            error: None,
            id: req.id.clone(),
        },

        // --- Account Methods ---

        "eth_getBalance" => {
            // params: [address, block_tag] - block_tag ignored
            let addr_str = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let balance = if let Ok(addr) = parse_address(addr_str) {
                state.balance(&addr)
            } else {
                U256::ZERO
            };
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("{:x}", balance))),
                error: None,
                id: req.id.clone(),
            }
        },

        "eth_getTransactionCount" => {
            // params: [address, block_tag] - block_tag ignored
            let addr_str = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let nonce = if let Ok(addr) = parse_address(addr_str) {
                state.nonce(&addr)
            } else {
                0
            };
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("0x{:x}", nonce))),
                error: None,
                id: req.id.clone(),
            }
        },

        "eth_accounts" => {
            let accounts: Vec<String> = state.all_accounts()
                .iter()
                .map(|(addr, _)| format!("0x{}", hex::encode(addr)))
                .collect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(accounts).unwrap()),
                error: None,
                id: req.id.clone(),
            }
        },

        "eth_getCode" => {
            // params: [address, block_tag] - block_tag ignored
            let addr_str = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match parse_address(addr_str) {
                Ok(addr) => {
                    let code = state.get_code(&addr);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(Value::String(format!("0x{}", hex::encode(&code)))),
                        error: None,
                        id: req.id.clone(),
                    }
                }
                Err(_) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(Value::String("0x".to_string())),
                    error: None,
                    id: req.id.clone(),
                }
            }
        },

        "eth_getStorageAt" => {
            // params: [address, slot, block_tag] - block_tag ignored
            let addr_str = req.params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let key_str = req.params.get(1).and_then(|v| v.as_str()).unwrap_or("0x0000000000000000000000000000000000000000000000000000000000000000");
            match (parse_address(addr_str), parse_bytes32(key_str)) {
                (Ok(addr), Ok(key)) => {
                    let value = state.get_storage(&addr, key).unwrap_or([0u8; 32]);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(Value::String(format!("0x{}", hex::encode(value)))),
                        error: None,
                        id: req.id.clone(),
                    }
                }
                _ => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(Value::String(format!("0x{}", hex::encode([0u8; 32])))),
                    error: None,
                    id: req.id.clone(),
                }
            }
        },

        // --- Block Methods ---

        "eth_getBlockByNumber" => {
            // params: [block_number, full_transactions]
            let block_num = req.params.first()
                .and_then(|v| v.as_str())
                .and_then(|s| if s == "latest" || s == "pending" { Some(state.block_number()) }
                          else if s == "earliest" { Some(0) }
                          else { u64::from_str_radix(s.trim_start_matches("0x"), 16).ok() })
                .unwrap_or(state.block_number());

            match state.get_block(block_num) {
                Some(block) => {
                    let result = serde_json::json!({
                        "number": format!("0x{:x}", block.number),
                        "hash": format!("0x{}", hex::encode(block.hash)),
                        "parentHash": format!("0x{}", hex::encode(block.parent_hash)),
                        "nonce": "0x0000000000000000",
                        "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                        "logsBloom": format!("0x{}", "00".repeat(256)),
                        "transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                        "stateRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "miner": "0x0000000000000000000000000000000000000000",
                        "difficulty": "0x0",
                        "totalDifficulty": "0x0",
                        "extraData": "0x",
                        "size": "0x3e8",
                        "gasLimit": "0x1c9c380",
                        "gasUsed": "0x0",
                        "timestamp": format!("0x{:x}", block.timestamp),
                        "transactions": [],
                        "uncles": []
                    });
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(result),
                        error: None,
                        id: req.id.clone(),
                    }
                }
                None => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(Value::Null),
                    error: None,
                    id: req.id.clone(),
                }
            }
        },

        "eth_getBlockByHash" => {
            // params: [block_hash, full_transactions] - placeholder
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::Null),
                error: None,
                id: req.id.clone(),
            }
        },

        "eth_getBlockTransactionCountByHash" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x0".to_string())),
            error: None,
            id: req.id.clone(),
        },

        "eth_getBlockTransactionCountByNumber" => {
            let block_num = req.params.first()
                .and_then(|v| v.as_str())
                .and_then(|s| if s == "latest" { Some(state.block_number()) } else { u64::from_str_radix(s.trim_start_matches("0x"), 16).ok() })
                .unwrap_or(state.block_number());
            let tx_count = state.get_block(block_num).map(|b| b.tx_count).unwrap_or(0);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("0x{:x}", tx_count))),
                error: None,
                id: req.id.clone(),
            }
        },

        "eth_getUncleCountByBlockHash" | "eth_getUncleCountByBlockNumber" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x0".to_string())),
            error: None,
            id: req.id.clone(),
        },

        // --- Transaction Methods ---

        "eth_sendTransaction" => {
            // SECURITY: This method requires proper signature verification
            // Expected params: [{from, to, value, gas, gasPrice, data, nonce, signature, publicKey}]
            let tx_obj = req.params.first().unwrap_or(&Value::Null);
            let from_str = tx_obj.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to_str = tx_obj.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let value_str = tx_obj.get("value").and_then(|v| v.as_str()).unwrap_or("0x0");
            let nonce_str = tx_obj.get("nonce").and_then(|v| v.as_str()).unwrap_or("0x0");
            let sig_str = tx_obj.get("signature").and_then(|v| v.as_str()).unwrap_or("");
            let pubkey_str = tx_obj.get("publicKey").and_then(|v| v.as_str()).unwrap_or("");

            match (parse_address(from_str), parse_address(to_str), parse_u256(value_str), parse_u64(nonce_str)) {
                (Ok(from), Ok(to), Ok(amount), Ok(nonce)) => {
                    // Verify nonce
                    let expected_nonce = state.nonce(&from);
                    if nonce != expected_nonce {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32001,
                                message: format!("Invalid nonce: expected {}, got {}", expected_nonce, nonce),
                            }),
                            id: req.id.clone(),
                        };
                    }

                    // Signature is REQUIRED for security
                    if sig_str.is_empty() || pubkey_str.is_empty() {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Signature required: provide 'signature' and 'publicKey' in transaction object".to_string(),
                            }),
                            id: req.id.clone(),
                        };
                    }

                    // Verify signature
                        use merklith_types::{Transaction, Ed25519Signature, Ed25519PublicKey};
                        use merklith_crypto::ed25519_verify;

                        match (hex::decode(sig_str.strip_prefix("0x").unwrap_or(&sig_str)),
                               hex::decode(pubkey_str.strip_prefix("0x").unwrap_or(&pubkey_str))) {
                            (Ok(sig_bytes), Ok(pk_bytes)) if sig_bytes.len() == 64 && pk_bytes.len() == 32 => {
                                let tx = Transaction::new(
                                    chain_id,
                                    nonce,
                                    Some(to),
                                    amount,
                                    21000,
                                    U256::from(1_000_000_000u64),
                                    U256::from(1_000_000u64),
                                );

                                let signing_hash = tx.signing_hash();
                                let signature = match sig_bytes.as_slice().try_into() {
                                    Ok(bytes) => Ed25519Signature::from_bytes(bytes),
                                    Err(_) => {
                                        return JsonRpcResponse {
                                            jsonrpc: "2.0".to_string(),
                                            result: None,
                                            error: Some(JsonRpcError {
                                                code: -32602,
                                                message: "Invalid signature length".to_string(),
                                            }),
                                            id: req.id.clone(),
                                        };
                                    }
                                };
                                let public_key = match pk_bytes.as_slice().try_into() {
                                    Ok(bytes) => Ed25519PublicKey::from_bytes(bytes),
                                    Err(_) => {
                                        return JsonRpcResponse {
                                            jsonrpc: "2.0".to_string(),
                                            result: None,
                                            error: Some(JsonRpcError {
                                                code: -32602,
                                                message: "Invalid public key length".to_string(),
                                            }),
                                            id: req.id.clone(),
                                        };
                                    }
                                };

                                if let Err(e) = ed25519_verify(&public_key, signing_hash.as_bytes(), &signature) {
                                    return JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError {
                                            code: -32002,
                                            message: format!("Invalid signature: {}", e),
                                        }),
                                        id: req.id.clone(),
                                    };
                                }
                            }
                            _ => {
                                return JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32002,
                                        message: "Invalid signature or public key format".to_string(),
                                    }),
                                    id: req.id.clone(),
                                };
                            }
                        }

                    match state.transfer(&from, &to, amount) {
                        Ok(tx_hash) => {
                            let hash_hex = format!("0x{}", hex::encode(tx_hash.as_bytes()));
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(Value::String(hash_hex)),
                                error: None,
                                id: req.id.clone(),
                            }
                        }
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32000,
                                message: e,
                            }),
                            id: req.id.clone(),
                        }
                    }
                }
                _ => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                    }),
                    id: req.id.clone(),
                }
            }
        },

        "eth_sendRawTransaction" => {
            let tx_hash = format!("0x{}", hex::encode(&rand::random::<[u8; 32]>()));
            tracing::info!("eth_sendRawTransaction received: {}", tx_hash);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(tx_hash)),
                error: None,
                id: req.id.clone(),
            }
        },

        "eth_getTransactionByHash" => {
            let tx_hash = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let result = serde_json::json!({
                "hash": tx_hash,
                "nonce": "0x0",
                "blockHash": null,
                "blockNumber": null,
                "transactionIndex": "0x0",
                "from": "0x0000000000000000000000000000000000000000",
                "to": "0x0000000000000000000000000000000000000000",
                "value": "0x0",
                "gas": "0x5208",
                "gasPrice": "0x3b9aca00",
                "input": "0x"
            });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: req.id.clone(),
            }
        },

        "eth_getTransactionReceipt" => {
            let tx_hash = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let result = serde_json::json!({
                "transactionHash": tx_hash,
                "transactionIndex": "0x0",
                "blockHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "blockNumber": "0x1",
                "from": "0x0000000000000000000000000000000000000000",
                "to": "0x0000000000000000000000000000000000000000",
                "cumulativeGasUsed": "0x5208",
                "gasUsed": "0x5208",
                "contractAddress": null,
                "logs": [],
                "logsBloom": format!("0x{}", "00".repeat(256)),
                "status": "0x1"
            });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: req.id.clone(),
            }
        },

        // --- Contract Methods ---

        "eth_call" => {
            // params: [{to, data, ...}, block_tag]
            let tx_obj = req.params.first().unwrap_or(&Value::Null);
            let to_str = tx_obj.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let data_str = tx_obj.get("data")
                .or_else(|| tx_obj.get("input"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(to_str) {
                Ok(to) => {
                    let code = state.get_code(&to);
                    let input = if data_str.starts_with("0x") {
                        hex::decode(&data_str[2..]).unwrap_or_default()
                    } else {
                        vec![]
                    };
                    match execute_contract(&code, &input) {
                        Ok(result) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(Value::String(format!("0x{}", hex::encode(&result)))),
                            error: None,
                            id: req.id.clone(),
                        },
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32000,
                                message: e,
                            }),
                            id: req.id.clone(),
                        }
                    }
                }
                Err(_) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(Value::String("0x".to_string())),
                    error: None,
                    id: req.id.clone(),
                }
            }
        },

        // --- Web3/Net Methods ---

        "web3_clientVersion" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("merklith/0.1.0".to_string())),
            error: None,
            id: req.id.clone(),
        },

        "web3_sha3" => {
            let data_str = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let input = if data_str.starts_with("0x") {
                hex::decode(&data_str[2..]).unwrap_or_default()
            } else {
                data_str.as_bytes().to_vec()
            };
            let hash = merklith_crypto::hash::hash(&input);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(Value::String(format!("0x{}", hex::encode(hash.as_bytes())))),
                error: None,
                id: req.id.clone(),
            }
        },

        "net_version" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String(chain_id.to_string())),
            error: None,
            id: req.id.clone(),
        },

        "net_listening" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::Bool(true)),
            error: None,
            id: req.id.clone(),
        },

        "net_peerCount" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::String("0x0".to_string())),
            error: None,
            id: req.id.clone(),
        },

        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
            }),
            id: req.id.clone(),
        },
    }
}

use merklith_types::{Address, U256};
use std::str::FromStr;

fn parse_address(s: &str) -> Result<Address, ()> {
    Address::from_str(s).map_err(|_| ())
}

fn parse_u256(s: &str) -> Result<U256, ()> {
    if s.starts_with("0x") || s.starts_with("0X") {
        let hex_str = &s[2..];
        let hex_str = if hex_str.len() % 2 == 1 {
            format!("0{}", hex_str)
        } else {
            hex_str.to_string()
        };
        let bytes = hex::decode(&hex_str).map_err(|e| {
            tracing::error!("hex decode failed for '{}': {:?}", hex_str, e);
            ()
        })?;
        if bytes.len() > 32 {
            tracing::error!("bytes too long: {}", bytes.len());
            return Err(());
        }
        let mut padded = [0u8; 32];
        padded[32 - bytes.len()..].copy_from_slice(&bytes);
        Ok(U256::from_be_bytes(padded))
    } else {
        U256::from_str(s).map_err(|e| {
            tracing::error!("decimal parse failed for '{}': {:?}", s, e);
            ()
        })
    }
}

fn parse_u64(s: &str) -> Result<u64, ()> {
    if s.starts_with("0x") || s.starts_with("0X") {
        let hex_part = &s[2..];
        if hex_part.is_empty() {
            return Err(());
        }
        u64::from_str_radix(hex_part, 16).map_err(|_| ())
    } else {
        s.parse().map_err(|_| ())
    }
}

fn parse_bytes32(s: &str) -> Result<[u8; 32], ()> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 64 {
        return Err(());
    }
    let bytes = hex::decode(s).map_err(|_| ())?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

fn execute_contract(code: &[u8], input: &[u8]) -> Result<Vec<u8>, String> {
    use merklith_vm::{MerklithVM, ExecutionContext};
    use bytes::Bytes;
    
    let vm = MerklithVM::new()
        .map_err(|e| format!("Failed to create VM: {}", e))?;
    
    let ctx = ExecutionContext::new_call(
        merklith_types::Address::ZERO,
        merklith_types::Address::ZERO,
        merklith_types::Address::ZERO,
        1_000_000,
        Bytes::copy_from_slice(input),
    );
    
    let ctx = ExecutionContext {
        code: Bytes::copy_from_slice(code),
        ..ctx
    };
    
    match vm.execute(ctx) {
        Ok(result) if result.success => Ok(result.data.to_vec()),
        Ok(result) => Err(format!("Contract execution failed")),
        Err(e) => Err(format!("VM execution error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::{Address, U256};

    #[test]
    fn test_rpc_config_default() {
        let config = RpcServerConfig::default();
        assert_eq!(config.http_port, 8545);
        assert!(config.cors);
        assert_eq!(config.max_body_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_parse_address_valid() {
        // Create a valid 20-byte hex address
        let addr_str = "0x1234567890123456789012345678901234567890";
        let result = parse_address(addr_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_address_invalid() {
        let addr_str = "invalid";
        let result = parse_address(addr_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u256_hex() {
        let result = parse_u256("0xFF").unwrap();
        assert_eq!(result, U256::from(255u64));
    }

    #[test]
    fn test_parse_u256_decimal() {
        let result = parse_u256("1000").unwrap();
        assert_eq!(result, U256::from(1000u64));
    }

    #[test]
    fn test_parse_u256_odd_hex() {
        // Should handle odd-length hex strings
        let result = parse_u256("0xF").unwrap();
        assert_eq!(result, U256::from(15u64));
    }

    #[test]
    fn test_parse_u256_invalid() {
        let result = parse_u256("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u64_hex() {
        let result = parse_u64("0xFF").unwrap();
        assert_eq!(result, 255u64);
    }

    #[test]
    fn test_parse_u64_decimal() {
        let result = parse_u64("1000").unwrap();
        assert_eq!(result, 1000u64);
    }

    #[test]
    fn test_parse_u64_invalid() {
        let result = parse_u64("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bytes32_valid() {
        let result = parse_bytes32("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_parse_bytes32_invalid_length() {
        let result = parse_bytes32("0x1234");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_rpc_request_creation() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "eth_chainId".to_string(),
            params: vec![],
            id: Some(serde_json::json!(1)),
        };
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "eth_chainId");
    }

    #[test]
    fn test_json_rpc_response_creation() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!("0x1")),
            error: None,
            id: Some(serde_json::json!(1)),
        };
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_error_creation() {
        let error = JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
        };
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }
}
