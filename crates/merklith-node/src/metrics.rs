//! Metrics collection and reporting.
//!
//! Uses Prometheus for metrics collection and exposition.

use prometheus::{Counter, Gauge, Histogram, Registry, Encoder, TextEncoder};
use std::sync::Arc;
use std::time::Instant;

/// Metrics collector.
pub struct Metrics {
    /// Prometheus registry
    registry: Registry,
    /// Current block number
    pub current_block: Gauge,
    /// Peer count
    pub peer_count: Gauge,
    /// Transaction pool size
    pub tx_pool_size: Gauge,
    /// Block production time
    pub block_production_time: Histogram,
    /// Total blocks produced
    pub blocks_produced: Counter,
    /// Total transactions processed
    pub transactions_processed: Counter,
    /// RPC requests
    pub rpc_requests: Counter,
    /// RPC request duration
    pub rpc_request_duration: Histogram,
}

impl Metrics {
    /// Create new metrics collector.
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let registry = Registry::new();

        let current_block = Gauge::new(
            "merklith_current_block",
            "Current block number",
        )?;
        registry.register(Box::new(current_block.clone()))?;

        let peer_count = Gauge::new(
            "merklith_peer_count",
            "Number of connected peers",
        )?;
        registry.register(Box::new(peer_count.clone()))?;

        let tx_pool_size = Gauge::new(
            "merklith_tx_pool_size",
            "Number of transactions in pool",
        )?;
        registry.register(Box::new(tx_pool_size.clone()))?;

        let block_production_time = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "merklith_block_production_seconds",
                "Time to produce a block",
            )
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0]),
        )?;
        registry.register(Box::new(block_production_time.clone()))?;

        let blocks_produced = Counter::new(
            "merklith_blocks_produced_total",
            "Total number of blocks produced",
        )?;
        registry.register(Box::new(blocks_produced.clone()))?;

        let transactions_processed = Counter::new(
            "merklith_transactions_total",
            "Total number of transactions processed",
        )?;
        registry.register(Box::new(transactions_processed.clone()))?;

        let rpc_requests = Counter::new(
            "merklith_rpc_requests_total",
            "Total number of RPC requests",
        )?;
        registry.register(Box::new(rpc_requests.clone()))?;

        let rpc_request_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "merklith_rpc_request_duration_seconds",
                "RPC request duration",
            )
            .buckets(vec![0.001, 0.01, 0.1, 0.5, 1.0]),
        )?;
        registry.register(Box::new(rpc_request_duration.clone()))?;

        Ok(Arc::new(Self {
            registry,
            current_block,
            peer_count,
            tx_pool_size,
            block_production_time,
            blocks_produced,
            transactions_processed,
            rpc_requests,
            rpc_request_duration,
        }))
    }

    /// Export metrics in Prometheus text format.
    pub fn export(&self,
    ) -> anyhow::Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Record block production.
    pub fn record_block(
        &self,
        block_number: u64,
        tx_count: usize,
        duration: std::time::Duration,
    ) {
        self.current_block.set(block_number as f64);
        self.blocks_produced.inc();
        self.transactions_processed.inc_by(tx_count as f64);
        self.block_production_time.observe(duration.as_secs_f64());
    }

    /// Record RPC request.
    pub fn record_rpc_request(
        &self,
        duration: std::time::Duration,
    ) {
        self.rpc_requests.inc();
        self.rpc_request_duration.observe(duration.as_secs_f64());
    }
}

/// Metrics server.
pub struct MetricsServer {
    addr: std::net::SocketAddr,
    metrics: Arc<Metrics>,
}

impl MetricsServer {
    /// Create new metrics server.
    pub fn new(
        addr: std::net::SocketAddr,
        metrics: Arc<Metrics>,
    ) -> Self {
        Self { addr, metrics }
    }

    /// Start the metrics server.
    pub async fn start(&self,
    ) -> anyhow::Result<()> {
        let metrics = self.metrics.clone();
        
        let app = axum::Router::new()
            .route("/metrics", axum::routing::get(move || {
                let metrics = metrics.clone();
                async move {
                    match metrics.export() {
                        Ok(output) => (axum::http::StatusCode::OK, output),
                        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Error".to_string()),
                    }
                }
            }));

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new().unwrap();
        assert_eq!(metrics.current_block.get(), 0.0);
    }

    #[test]
    fn test_metrics_export() {
        let metrics = Metrics::new().unwrap();
        metrics.current_block.set(100.0);
        
        let output = metrics.export().unwrap();
        assert!(output.contains("merklith_current_block"));
        assert!(output.contains("100"));
    }

    #[test]
    fn test_record_block() {
        let metrics = Metrics::new().unwrap();
        
        metrics.record_block(10, 5, std::time::Duration::from_secs(1));
        
        assert_eq!(metrics.current_block.get(), 10.0);
        assert_eq!(metrics.blocks_produced.get(), 1.0);
        assert_eq!(metrics.transactions_processed.get(), 5.0);
    }
}
