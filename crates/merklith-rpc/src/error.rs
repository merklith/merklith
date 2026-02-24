//! RPC error types and responses.

use jsonrpsee::types::error::ErrorObjectOwned;
use serde::Serialize;
use thiserror::Error;

/// RPC error codes matching Ethereum JSON-RPC spec.
pub mod error_codes {
    /// Parse error
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid request
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Server error (custom)
    pub const SERVER_ERROR: i32 = -32000;
    /// Invalid input
    pub const INVALID_INPUT: i32 = -32000;
    /// Resource not found
    pub const RESOURCE_NOT_FOUND: i32 = -32001;
    /// Resource unavailable
    pub const RESOURCE_UNAVAILABLE: i32 = -32002;
    /// Transaction rejected
    pub const TRANSACTION_REJECTED: i32 = -32003;
    /// Method not supported
    pub const METHOD_NOT_SUPPORTED: i32 = -32004;
    /// Limit exceeded
    pub const LIMIT_EXCEEDED: i32 = -32005;
    /// JSON-RPC version not supported
    pub const JSONRPC_VERSION_NOT_SUPPORTED: i32 = -32006;
    /// Client is not connected
    pub const CLIENT_NOT_CONNECTED: i32 = -32007;
}

/// RPC errors.
#[derive(Debug, Error, Clone)]
pub enum RpcError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid params: {0}")]
    InvalidParams(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),

    #[error("Transaction rejected: {0}")]
    TransactionRejected(String),

    #[error("Method not supported: {0}")]
    MethodNotSupported(String),

    #[error("Limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Custom error: {code}, {message}")]
    Custom { code: i32, message: String },
}

impl RpcError {
    /// Get the error code.
    pub fn code(&self) -> i32 {
        match self {
            RpcError::ParseError(_) => error_codes::PARSE_ERROR,
            RpcError::InvalidRequest(_) => error_codes::INVALID_REQUEST,
            RpcError::MethodNotFound(_) => error_codes::METHOD_NOT_FOUND,
            RpcError::InvalidParams(_) => error_codes::INVALID_PARAMS,
            RpcError::InternalError(_) => error_codes::INTERNAL_ERROR,
            RpcError::ResourceNotFound(_) => error_codes::RESOURCE_NOT_FOUND,
            RpcError::ResourceUnavailable(_) => error_codes::RESOURCE_UNAVAILABLE,
            RpcError::TransactionRejected(_) => error_codes::TRANSACTION_REJECTED,
            RpcError::MethodNotSupported(_) => error_codes::METHOD_NOT_SUPPORTED,
            RpcError::LimitExceeded(_) => error_codes::LIMIT_EXCEEDED,
            RpcError::ExecutionError(_) => error_codes::SERVER_ERROR,
            RpcError::StateError(_) => error_codes::SERVER_ERROR,
            RpcError::Custom { code, .. } => *code,
        }
    }

    /// Convert to JSON-RPC error object.
    pub fn to_error_object(&self) -> ErrorObjectOwned {
        ErrorObjectOwned::owned(
            self.code(),
            self.to_string(),
            None::<()>,
        )
    }
}

impl From<RpcError> for ErrorObjectOwned {
    fn from(err: RpcError) -> Self {
        err.to_error_object()
    }
}

/// RPC response wrapper.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum RpcResponse<T> {
    /// Success response
    Success {
        jsonrpc: String,
        result: T,
        id: serde_json::Value,
    },
    /// Error response
    Error {
        jsonrpc: String,
        error: RpcErrorDetails,
        id: serde_json::Value,
    },
}

/// Error details for RPC responses.
#[derive(Debug, Clone, Serialize)]
pub struct RpcErrorDetails {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcErrorDetails {
    /// Create new error details.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Add data to error.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// RPC request structure.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// RPC batch request.
pub type RpcBatchRequest = Vec<RpcRequest>;

/// Standard RPC result type.
pub type RpcResult<T> = Result<T, RpcError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
    }

    #[test]
    fn test_rpc_error_code() {
        let err = RpcError::ParseError("test".to_string());
        assert_eq!(err.code(), -32700);

        let err = RpcError::MethodNotFound("eth_unknown".to_string());
        assert_eq!(err.code(), -32601);

        let err = RpcError::InternalError("db error".to_string());
        assert_eq!(err.code(), -32603);
    }

    #[test]
    fn test_rpc_error_details() {
        let details = RpcErrorDetails::new(-32000, "Custom error");
        assert_eq!(details.code, -32000);
        assert_eq!(details.message, "Custom error");
        assert!(details.data.is_none());

        let details = details.with_data(serde_json::json!({"extra": "info"}));
        assert!(details.data.is_some());
    }

    #[test]
    fn test_rpc_request_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": ["0x1234", "latest"],
            "id": 1
        }"#;

        let req: RpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "eth_getBalance");
        assert_eq!(req.id, serde_json::json!(1));
    }

    #[test]
    fn test_rpc_response_serialization() {
        let response = RpcResponse::Success {
            jsonrpc: "2.0".to_string(),
            result: "0x1234",
            id: serde_json::json!(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("2.0"));
        assert!(json.contains("0x1234"));
        assert!(json.contains("result"));
    }
}
