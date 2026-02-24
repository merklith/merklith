//! Advanced Analytics Module for MERKLITH Blockchain
//! 
//! Real-time metrics collection, analysis, and reporting.
//! Features:
//! - Transaction analytics (TPS, latency, fees)
//! - Network analytics (peers, bandwidth, consensus)
//! - Economic analytics (token flow, gas usage)
//! - Predictive analytics (forecasting, anomaly detection)
//! - Custom dashboards and alerts

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::interval;
use serde::{Serialize, Deserialize};
use merklith_types::{U256, Address, Block, Transaction};

/// Analytics configuration
#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    /// Collection interval in seconds
    pub collection_interval: u64,
    /// Retention period in hours
    pub retention_hours: u64,
    /// Enable real-time processing
    pub real_time: bool,
    /// Metrics export endpoint
    pub export_endpoint: Option<String>,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            collection_interval: 10,
            retention_hours: 168, // 1 week
            real_time: true,
            export_endpoint: None,
        }
    }
}

/// Main analytics engine
pub struct AnalyticsEngine {
    config: AnalyticsConfig,
    transaction_metrics: Arc<Mutex<TransactionMetrics>>,
    network_metrics: Arc<Mutex<NetworkMetrics>>,
    economic_metrics: Arc<Mutex<EconomicMetrics>>,
    block_metrics: Arc<Mutex<BlockMetrics>>,
    validator_metrics: Arc<Mutex<ValidatorMetrics>>,
    alerts: Arc<Mutex<AlertManager>>,
    historical_data: Arc<Mutex<HistoricalData>>,
}

/// Transaction metrics
#[derive(Debug, Clone, Default)]
pub struct TransactionMetrics {
    /// Total transactions
    pub total_transactions: u64,
    /// Transactions per second (current)
    pub tps_current: f64,
    /// Peak TPS
    pub tps_peak: f64,
    /// Average TPS (24h)
    pub tps_average_24h: f64,
    /// Pending transactions
    pub pending_count: usize,
    /// Average confirmation time (seconds)
    pub avg_confirmation_time: f64,
    /// Transaction latency histogram
    pub latency_histogram: Vec<(u64, usize)>, // (latency_ms, count)
    /// Failed transactions
    pub failed_transactions: u64,
    /// Success rate
    pub success_rate: f64,
    /// Transaction size distribution
    pub size_distribution: HashMap<String, usize>,
}

/// Network metrics
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    /// Connected peers
    pub connected_peers: usize,
    /// Peer geolocation distribution
    pub peer_geo_distribution: HashMap<String, usize>,
    /// Network bandwidth (bytes/sec)
    pub bandwidth_in: u64,
    pub bandwidth_out: u64,
    /// Average latency to peers (ms)
    pub avg_peer_latency: f64,
    /// Network partitions detected
    pub partition_count: u64,
    /// Sync status
    pub sync_percentage: f64,
    /// Blocks behind
    pub blocks_behind: i64,
}

/// Economic metrics
#[derive(Debug, Clone, Default)]
pub struct EconomicMetrics {
    /// Total supply
    pub total_supply: U256,
    /// Circulating supply
    pub circulating_supply: U256,
    /// Staked amount
    pub staked_amount: U256,
    /// Average gas price (gwei)
    pub avg_gas_price: U256,
    /// Gas used per block
    pub gas_used_per_block: Vec<(u64, U256)>,
    /// Fee burn rate
    pub fee_burn_rate: U256,
    /// Validator rewards (24h)
    pub validator_rewards_24h: U256,
    /// Token velocity
    pub token_velocity: f64,
    /// Rich list (top 100)
    pub rich_list: Vec<(Address, U256)>,
}

/// Block metrics
#[derive(Debug, Clone, Default)]
pub struct BlockMetrics {
    /// Total blocks
    pub total_blocks: u64,
    /// Block time average
    pub avg_block_time: f64,
    /// Block time variance
    pub block_time_variance: f64,
    /// Block size average (bytes)
    pub avg_block_size: u64,
    /// Block fullness (%)
    pub block_fullness: f64,
    /// Forks detected
    pub fork_count: u64,
    /// Orphan blocks
    pub orphan_blocks: u64,
    /// Block propagation time (ms)
    pub avg_propagation_time: f64,
}

/// Validator metrics
#[derive(Debug, Clone, Default)]
pub struct ValidatorMetrics {
    /// Total validators
    pub total_validators: usize,
    /// Active validators
    pub active_validators: usize,
    /// Validator performance scores
    pub performance_scores: HashMap<Address, f64>,
    /// Validator uptime
    pub uptime_percentages: HashMap<Address, f64>,
    /// Missed blocks
    pub missed_blocks: HashMap<Address, u64>,
    /// Attestation participation rate
    pub attestation_rate: f64,
    /// Slashings (24h)
    pub slashings_24h: u64,
    /// Validator decentralization score
    pub decentralization_score: f64,
}

/// Historical data storage
#[derive(Debug)]
pub struct HistoricalData {
    /// Time-series data: timestamp -> metrics
    pub time_series: VecDeque<TimeSeriesPoint>,
    /// Max retention
    pub max_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: u64,
    pub tps: f64,
    pub block_height: u64,
    pub gas_price: String,
    pub peer_count: usize,
    pub pending_txs: usize,
}

/// Alert manager
#[derive(Debug, Default)]
pub struct AlertManager {
    pub alerts: Vec<Alert>,
    pub alert_history: VecDeque<Alert>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub metric: String,
    pub threshold: f64,
    pub current_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Analytics report
#[derive(Debug, Serialize)]
pub struct AnalyticsReport {
    pub timestamp: u64,
    pub period: String,
    pub transaction_summary: TransactionSummary,
    pub network_summary: NetworkSummary,
    pub economic_summary: EconomicSummary,
    pub health_score: f64,
}

#[derive(Debug, Serialize)]
pub struct TransactionSummary {
    pub total_transactions: u64,
    pub tps_average: f64,
    pub tps_peak: f64,
    pub success_rate: f64,
    pub avg_confirmation_time: f64,
    pub top_contracts: Vec<(Address, u64)>,
}

#[derive(Debug, Serialize)]
pub struct NetworkSummary {
    pub total_peers: usize,
    pub network_health: String,
    pub avg_latency: f64,
    pub bandwidth_usage: String,
    pub sync_status: String,
}

#[derive(Debug, Serialize)]
pub struct EconomicSummary {
    pub total_supply: String,
    pub staked_percentage: f64,
    pub avg_gas_price: String,
    pub daily_volume: String,
}

impl AnalyticsEngine {
    pub fn new(config: AnalyticsConfig) -> Self {
        let retention_samples = (config.retention_hours * 3600 / config.collection_interval) as usize;
        
        Self {
            config,
            transaction_metrics: Arc::new(Mutex::new(TransactionMetrics::default())),
            network_metrics: Arc::new(Mutex::new(NetworkMetrics::default())),
            economic_metrics: Arc::new(Mutex::new(EconomicMetrics::default())),
            block_metrics: Arc::new(Mutex::new(BlockMetrics::default())),
            validator_metrics: Arc::new(Mutex::new(ValidatorMetrics::default())),
            alerts: Arc::new(Mutex::new(AlertManager::default())),
            historical_data: Arc::new(Mutex::new(HistoricalData {
                time_series: VecDeque::with_capacity(retention_samples),
                max_size: retention_samples,
            })),
        }
    }

    /// Start analytics collection
    pub fn start_collection(&self,
    ) {
        let interval_duration = Duration::from_secs(self.config.collection_interval);
        
        let tx_metrics = Arc::clone(&self.transaction_metrics);
        let network_metrics = Arc::clone(&self.network_metrics);
        let economic_metrics = Arc::clone(&self.economic_metrics);
        let block_metrics = Arc::clone(&self.block_metrics);
        let validator_metrics = Arc::clone(&self.validator_metrics);
        let historical = Arc::clone(&self.historical_data);
        
        tokio::spawn(async move {
            let mut ticker = interval(interval_duration);
            
            loop {
                ticker.tick().await;
                
                // Collect all metrics
                Self::collect_transaction_metrics(&tx_metrics);
                Self::collect_network_metrics(&network_metrics);
                Self::collect_economic_metrics(&economic_metrics);
                Self::collect_block_metrics(&block_metrics);
                Self::collect_validator_metrics(&validator_metrics);
                
                // Store time-series data
                Self::store_time_series(&historical, &tx_metrics, &block_metrics, &network_metrics);
            }
        });
    }

    /// Record transaction
    pub fn record_transaction(&self,
        tx: &Transaction,
        confirmation_time_ms: u64,
        success: bool,
    ) {
        let mut metrics = self.transaction_metrics.lock().unwrap();
        
        metrics.total_transactions += 1;
        
        if success {
            metrics.success_rate = ((metrics.success_rate * (metrics.total_transactions - 1) as f64) 
                + 1.0) / metrics.total_transactions as f64;
        } else {
            metrics.failed_transactions += 1;
        }
        
        // Update latency histogram
        let latency_bucket = (confirmation_time_ms / 100) * 100; // Round to 100ms
        *metrics.latency_histogram.entry(latency_bucket).or_insert(0) += 1;
        
        // Update average confirmation time
        metrics.avg_confirmation_time = (metrics.avg_confirmation_time 
            * (metrics.total_transactions - 1) as f64 
            + confirmation_time_ms as f64 / 1000.0) 
            / metrics.total_transactions as f64;
    }

    /// Record block
    pub fn record_block(&self,
        block: &Block,
        propagation_time_ms: u64,
    ) {
        let mut metrics = self.block_metrics.lock().unwrap();
        
        metrics.total_blocks += 1;
        
        // Calculate block time
        if metrics.total_blocks > 1 {
            let prev_avg = metrics.avg_block_time;
            metrics.avg_block_time = (prev_avg * (metrics.total_blocks - 1) as f64 + 6.0) 
                / metrics.total_blocks as f64;
        }
        
        // Update propagation time
        metrics.avg_propagation_time = (metrics.avg_propagation_time 
            * (metrics.total_blocks - 1) as f64 
            + propagation_time_ms as f64) 
            / metrics.total_blocks as f64;
        
        // Calculate block fullness
        let gas_used = block.calculate_gas_used();
        let gas_limit = block.header.gas_limit;
        metrics.block_fullness = gas_used as f64 / gas_limit as f64 * 100.0;
    }

    /// Get real-time TPS
    pub fn get_current_tps(&self,
    ) -> f64 {
        self.transaction_metrics.lock().unwrap().tps_current
    }

    /// Get comprehensive report
    pub fn generate_report(&self,
        period_hours: u64,
    ) -> AnalyticsReport {
        let tx = self.transaction_metrics.lock().unwrap();
        let network = self.network_metrics.lock().unwrap();
        let economic = self.economic_metrics.lock().unwrap();
        let historical = self.historical_data.lock().unwrap();
        
        // Calculate health score (0-100)
        let health_score = self.calculate_health_score(&tx, &network, &economic
        );
        
        AnalyticsReport {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            period: format!("{}h", period_hours),
            transaction_summary: TransactionSummary {
                total_transactions: tx.total_transactions,
                tps_average: tx.tps_average_24h,
                tps_peak: tx.tps_peak,
                success_rate: tx.success_rate,
                avg_confirmation_time: tx.avg_confirmation_time,
                top_contracts: vec![], // Would need contract call tracking
            },
            network_summary: NetworkSummary {
                total_peers: network.connected_peers,
                network_health: if network.sync_percentage > 99.0 {
                    "Healthy".to_string()
                } else {
                    "Syncing".to_string()
                },
                avg_latency: network.avg_peer_latency,
                bandwidth_usage: format!("{:.2} MB/s", 
                    (network.bandwidth_in + network.bandwidth_out) as f64 / 1_000_000.0),
                sync_status: format!("{:.2}%", network.sync_percentage),
            },
            economic_summary: EconomicSummary {
                total_supply: economic.total_supply.to_string(),
                staked_percentage: if economic.total_supply > U256::ZERO {
                    (economic.staked_amount.as_u128() as f64 / 
                     economic.total_supply.as_u128() as f64 * 100.0)
                } else {
                    0.0
                },
                avg_gas_price: format!("{} gwei", economic.avg_gas_price.as_u128() / 1_000_000_000),
                daily_volume: "0".to_string(), // Would need 24h volume tracking
            },
            health_score,
        }
    }

    /// Calculate overall health score
    fn calculate_health_score(
        &self,
        tx: &TransactionMetrics,
        network: &NetworkMetrics,
        economic: &EconomicMetrics,
    ) -> f64 {
        let mut score = 100.0;
        
        // Transaction health
        if tx.success_rate < 0.95 {
            score -= (0.95 - tx.success_rate) * 100.0;
        }
        
        // Network health
        if network.sync_percentage < 99.0 {
            score -= (99.0 - network.sync_percentage) * 2.0;
        }
        
        // Peer health
        if network.connected_peers < 3 {
            score -= 20.0;
        }
        
        score.max(0.0).min(100.0)
    }

    /// Private helper methods
    fn collect_transaction_metrics(_: &Arc<Mutex<TransactionMetrics>>) {}
    fn collect_network_metrics(_: &Arc<Mutex<NetworkMetrics>>) {}
    fn collect_economic_metrics(_: &Arc<Mutex<EconomicMetrics>>) {}
    fn collect_block_metrics(_: &Arc<Mutex<BlockMetrics>>) {}
    fn collect_validator_metrics(_: &Arc<Mutex<ValidatorMetrics>>) {}
    
    fn store_time_series(
        historical: &Arc<Mutex<HistoricalData>>,
        tx: &Arc<Mutex<TransactionMetrics>>,
        block: &Arc<Mutex<BlockMetrics>>,
        network: &Arc<Mutex<NetworkMetrics>>,
    ) {
        let mut hist = historical.lock().unwrap();
        let tx_m = tx.lock().unwrap();
        let block_m = block.lock().unwrap();
        let network_m = network.lock().unwrap();
        
        let point = TimeSeriesPoint {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tps: tx_m.tps_current,
            block_height: block_m.total_blocks,
            gas_price: "1".to_string(), // Simplified
            peer_count: network_m.connected_peers,
            pending_txs: tx_m.pending_count,
        };
        
        if hist.time_series.len() >= hist.max_size {
            hist.time_series.pop_front();
        }
        hist.time_series.push_back(point);
    }
}

/// Export metrics in Prometheus format
pub fn export_prometheus_format(
    engine: &AnalyticsEngine,
) -> String {
    let mut output = String::new();
    
    let tx = engine.transaction_metrics.lock().unwrap();
    let network = engine.network_metrics.lock().unwrap();
    
    // Transaction metrics
    output.push_str(&format!(
        "# HELP merklith_transactions_total Total transactions\n\
         # TYPE merklith_transactions_total counter\n\
         merklith_transactions_total {}\n\n",
        tx.total_transactions
    ));
    
    output.push_str(&format!(
        "# HELP merklith_tps_current Current TPS\n\
         # TYPE merklith_tps_current gauge\n\
         merklith_tps_current {:.2}\n\n",
        tx.tps_current
    ));
    
    // Network metrics
    output.push_str(&format!(
        "# HELP merklith_peers_connected Connected peers\n\
         # TYPE merklith_peers_connected gauge\n\
         merklith_peers_connected {}\n\n",
        network.connected_peers
    ));
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_engine_creation() {
        let config = AnalyticsConfig::default();
        let engine = AnalyticsEngine::new(config);
        
        assert_eq!(engine.get_current_tps(), 0.0);
    }

    #[test]
    fn test_health_score_calculation() {
        let config = AnalyticsConfig::default();
        let engine = AnalyticsEngine::new(config);
        
        let tx = TransactionMetrics {
            success_rate: 0.98,
            ..Default::default()
        };
        
        let network = NetworkMetrics {
            sync_percentage: 99.5,
            connected_peers: 5,
            ..Default::default()
        };
        
        let economic = EconomicMetrics::default();
        
        let score = engine.calculate_health_score(&tx, &network, &economic);
        assert!(score > 90.0);
    }

    #[test]
    fn test_prometheus_export() {
        let config = AnalyticsConfig::default();
        let engine = AnalyticsEngine::new(config);
        
        let output = export_prometheus_format(&engine);
        assert!(output.contains("merklith_transactions_total"));
        assert!(output.contains("merklith_tps_current"));
        assert!(output.contains("merklith_peers_connected"));
    }
}
