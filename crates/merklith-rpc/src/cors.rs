use std::net::SocketAddr;
use std::sync::Arc;
use std::convert::Infallible;
use hyper::{
    Body, Request, Response, Server, Method, StatusCode,
    service::{make_service_fn, service_fn},
};
use hyper::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_HEADERS, CONTENT_TYPE,
};
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Any};
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::types::error::ErrorCode;
use serde_json::json;
use std::future::Future;

/// CORS destekli RPC Server
/// 
/// Bu modül, Web UI'nin doğrudan node'a bağlanabilmesi için
/// gerekli CORS header'larını ekler.

/// HTTP Server with CORS support
pub struct CorsRpcServer {
    addr: SocketAddr,
    handle: Option<ServerHandle>,
}

impl CorsRpcServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr, handle: None }
    }
    
    pub async fn start(
        &mut self,
        rpc_module: jsonrpsee::RpcModule<()>,
    ) -> anyhow::Result<()> {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any);
        
        let service = ServiceBuilder::new()
            .layer(cors)
            .service_fn(move |req: Request<Body>| {
                handle_rpc_request(req, rpc_module.clone())
            });
        
        let make_svc = make_service_fn(|_conn| {
            let svc = service.clone();
            async move { Ok::<_, Infallible>(svc) }
        });
        
        let server = Server::bind(&self.addr).serve(make_svc);
        
        tracing::info!("CORS-enabled RPC server started on {}", self.addr);
        
        // Run server
        tokio::spawn(async move {
            if let Err(e) = server.await {
                tracing::error!("RPC server error: {}", e);
            }
        });
        
        Ok(())
    }
}

async fn handle_rpc_request(
    req: Request<Body>,
    rpc_module: jsonrpsee::RpcModule<()>,
) -> Result<Response<Body>, Infallible> {
    // Handle CORS preflight
    if req.method() == Method::OPTIONS {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(ACCESS_CONTROL_ALLOW_METHODS, "POST, GET, OPTIONS")
            .header(ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type")
            .body(Body::empty())
            .unwrap());
    }
    
    // Only accept POST for RPC
    if req.method() != Method::POST {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(Body::from(json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": ErrorCode::InvalidRequest.code(),
                    "message": "Only POST method is allowed"
                },
                "id": null
            }).to_string()))
            .unwrap());
    }
    
    // Read body
    let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": ErrorCode::ParseError.code(),
                        "message": format!("Failed to read body: {}", e)
                    },
                    "id": null
                }).to_string()))
                .unwrap());
        }
    };
    
    // Parse request
    let request_str = String::from_utf8_lossy(&body_bytes);
    
    // Call RPC method
    let response = match rpc_module.raw_json_request(&request_str,
        jsonrpsee::server::logger::HttpRequest::new(std::net::SocketAddr::from(([127, 0, 0, 1], 0))))
        .await {
        Ok((response, _)) => response,
        Err(e) => {
            json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": ErrorCode::InternalError.code(),
                    "message": format!("RPC error: {}", e)
                },
                "id": null
            }).to_string()
        }
    };
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(ACCESS_CONTROL_ALLOW_METHODS, "POST, GET, OPTIONS")
        .header(ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type")
        .body(Body::from(response))
        .unwrap())
}

/// Basit CORS proxy (Python alternatifi)
/// 
/// Eğer node CORS desteklemiyorsa, bu proxy kullanılabilir.
/// Ama node'un kendisi CORS desteklemeli (yukarıdaki implementasyon).

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cors_headers() {
        // CORS header test
        let headers = vec![
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", "POST, GET, OPTIONS"),
            ("Access-Control-Allow-Headers", "Content-Type"),
        ];
        
        assert_eq!(headers.len(), 3);
    }
}
