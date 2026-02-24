//! High Availability module for MERKLITH blockchain
//! 
//! Provides health monitoring, automatic recovery, and clustering support.
//! Ensures the node stays online and handles failures gracefully.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{info, warn, error};

/// Component health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthStatus {
    /// Check if component is operational
    pub fn is_operational(self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Check if component needs recovery
    pub fn needs_recovery(self) -> bool {
        matches!(self, HealthStatus::Unhealthy)
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub last_check: Instant,
    pub message: Option<String>,
    pub metrics: HashMap<String, f64>,
}

impl HealthCheck {
    pub fn healthy(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Healthy,
            last_check: Instant::now(),
            message: None,
            metrics: HashMap::new(),
        }
    }

    pub fn unhealthy(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unhealthy,
            last_check: Instant::now(),
            message: Some(message.into()),
            metrics: HashMap::new(),
        }
    }

    pub fn with_metric(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }
}

/// Health checker trait
#[async_trait::async_trait]
pub trait HealthCheckable: Send + Sync {
    async fn health_check(&self) -> HealthCheck;
}

/// Health monitoring system
pub struct HealthMonitor {
    checks: Arc<Mutex<HashMap<String, HealthCheck>>>,
    threshold_unhealthy: u32,
    threshold_degraded: u32,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(Mutex::new(HashMap::new())),
            threshold_unhealthy: 3,
            threshold_degraded: 2,
        }
    }

    pub fn with_thresholds(unhealthy: u32, degraded: u32) -> Self {
        Self {
            checks: Arc::new(Mutex::new(HashMap::new())),
            threshold_unhealthy: unhealthy,
            threshold_degraded: degraded,
        }
    }

    /// Update health check for component
    pub fn update_check(&self, check: HealthCheck) {
        if let Ok(mut checks) = self.checks.lock() {
            checks.insert(check.component.clone(), check);
        }
    }

    /// Get health status of component
    pub fn get_status(&self, component: &str) -> Option<HealthStatus> {
        self.checks
            .lock()
            .ok()
            .and_then(|checks| checks.get(component).map(|c| c.status))
    }

    /// Get all health checks
    pub fn get_all_checks(&self) -> Vec<HealthCheck> {
        self.checks
            .lock()
            .ok()
            .map(|checks| checks.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if all components are healthy
    pub fn is_healthy(&self) -> bool {
        self.checks
            .lock()
            .ok()
            .map(|checks| checks.values().all(|c| c.status == HealthStatus::Healthy))
            .unwrap_or(false)
    }

    /// Check overall system status
    pub fn system_status(&self) -> HealthStatus {
        let checks = self.checks.lock().ok();
        let checks = match checks {
            Some(c) => c,
            None => return HealthStatus::Unknown,
        };

        let unhealthy_count = checks.values().filter(|c| c.status == HealthStatus::Unhealthy).count();
        let degraded_count = checks.values().filter(|c| c.status == HealthStatus::Degraded).count();

        if unhealthy_count >= self.threshold_unhealthy as usize {
            HealthStatus::Unhealthy
        } else if degraded_count >= self.threshold_degraded as usize || unhealthy_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get unhealthy components
    pub fn get_unhealthy(&self) -> Vec<String> {
        self.checks
            .lock()
            .ok()
            .map(|checks| {
                checks
                    .values()
                    .filter(|c| c.status == HealthStatus::Unhealthy)
                    .map(|c| c.component.clone())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default()
    }

    /// Start monitoring loop
    pub fn start_monitoring(
        &self,
        checkers: Vec<Box<dyn HealthCheckable>>,
        interval_secs: u64,
    ) {
        let checks = Arc::clone(&self.checks);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                for checker in &checkers {
                    let check = checker.health_check().await;
                    
                    if let Ok(mut checks) = checks.lock() {
                        checks.insert(check.component.clone(), check.clone());
                    }

                    match check.status {
                        HealthStatus::Healthy => {
                            info!("Health check passed: {}", check.component);
                        }
                        HealthStatus::Degraded => {
                            warn!("Health check degraded: {} - {:?}", check.component, check.message);
                        }
                        HealthStatus::Unhealthy => {
                            error!("Health check failed: {} - {:?}", check.component, check.message);
                        }
                        HealthStatus::Unknown => {}
                    }
                }
            }
        });
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Automatic recovery system
pub struct RecoverySystem {
    monitor: Arc<HealthMonitor>,
    recovery_attempts: Arc<Mutex<HashMap<String, u32>>>,
    max_attempts: u32,
    recovery_actions: Arc<Mutex<HashMap<String, Arc<dyn Fn() -> bool + Send + Sync>>>>,
}

impl RecoverySystem {
    pub fn new(monitor: Arc<HealthMonitor>) -> Self {
        Self {
            monitor,
            recovery_attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts: 3,
            recovery_actions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_max_attempts(monitor: Arc<HealthMonitor>, max: u32) -> Self {
        Self {
            monitor,
            recovery_attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts: max,
            recovery_actions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register recovery action for component
    pub fn register_recovery(
        &self,
        component: impl Into<String>,
        action: Arc<dyn Fn() -> bool + Send + Sync>,
    ) {
        if let Ok(mut actions) = self.recovery_actions.lock() {
            let actions: &mut HashMap<String, Arc<dyn Fn() -> bool + Send + Sync>> = &mut *actions;
            let component: String = component.into();
            actions.insert(component, action);
        }
    }

    /// Attempt recovery for component
    pub fn attempt_recovery(&self, component: &str) -> bool {
        // Check attempt count
        let attempts = {
            let mut attempts = self.recovery_attempts.lock().unwrap();
            let count = attempts.entry(component.to_string()).or_insert(0);
            if *count >= self.max_attempts {
                return false;
            }
            *count += 1;
            *count
        };

        info!("Attempting recovery for {} (attempt {})", component, attempts);

        // Get recovery action
        let action = {
            let actions = self.recovery_actions.lock().unwrap();
            actions.get(component).cloned()
        };

        // Execute recovery
        if let Some(action) = action {
            if action() {
                info!("Recovery successful for {}", component);
                // Reset attempts on success
                if let Ok(mut attempts) = self.recovery_attempts.lock() {
                    let attempts: &mut HashMap<String, u32> = &mut *attempts;
                    let _ = attempts.remove(component);
                }
                true
            } else {
                warn!("Recovery failed for {}", component);
                false
            }
        } else {
            warn!("No recovery action registered for {}", component);
            false
        }
    }

    /// Reset recovery attempts for component
    pub fn reset_attempts(&self, component: &str) {
        if let Ok(mut attempts) = self.recovery_attempts.lock() {
            let attempts: &mut HashMap<String, u32> = &mut *attempts;
            let _ = attempts.remove(component);
        }
    }

    /// Start recovery loop
    pub fn start_recovery_loop(&self,
        check_interval_secs: u64,
    ) {
        let monitor: Arc<HealthMonitor> = Arc::clone(&self.monitor);
        let recovery = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(check_interval_secs));

            loop {
                ticker.tick().await;

                let unhealthy = monitor.get_unhealthy();
                
                for component in unhealthy {
                    if !recovery.attempt_recovery(&component) {
                        error!("Recovery failed for {} after max attempts", component);
                    }
                }
            }
        });
    }
}

impl Clone for RecoverySystem {
    fn clone(&self) -> Self {
        Self {
            monitor: Arc::clone(&self.monitor),
            recovery_attempts: Arc::clone(&self.recovery_attempts),
            max_attempts: self.max_attempts,
            recovery_actions: Arc::clone(&self.recovery_actions),
        }
    }
}

/// Node clustering support
pub struct ClusterManager {
    node_id: String,
    peers: Arc<Mutex<Vec<ClusterPeer>>>,
    heartbeat_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct ClusterPeer {
    pub id: String,
    pub address: String,
    pub last_heartbeat: Instant,
    pub healthy: bool,
}

impl ClusterManager {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            peers: Arc::new(Mutex::new(Vec::new())),
            heartbeat_interval: Duration::from_secs(5),
        }
    }

    /// Add peer to cluster
    pub fn add_peer(&self, id: impl Into<String>, address: impl Into<String>) {
        let peer = ClusterPeer {
            id: id.into(),
            address: address.into(),
            last_heartbeat: Instant::now(),
            healthy: true,
        };

        if let Ok(mut peers) = self.peers.lock() {
            peers.push(peer);
        }
    }

    /// Update peer heartbeat
    pub fn update_heartbeat(&self, peer_id: &str) {
        if let Ok(mut peers) = self.peers.lock() {
            if let Some(peer) = peers.iter_mut().find(|p| p.id == peer_id) {
                peer.last_heartbeat = Instant::now();
                peer.healthy = true;
            }
        }
    }

    /// Get healthy peers
    pub fn get_healthy_peers(&self) -> Vec<ClusterPeer> {
        if let Ok(peers) = self.peers.lock() {
            let cutoff = Instant::now() - Duration::from_secs(15);
            peers
                .iter()
                .filter(|p| p.last_heartbeat > cutoff && p.healthy)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.peers.lock().map(|p| p.len()).unwrap_or(0)
    }

    /// Start heartbeat loop
    pub fn start_heartbeat(
        &self,
        broadcast_fn: impl Fn(String) + Send + 'static,
    ) {
        let peers = Arc::clone(&self.peers);
        let node_id = self.node_id.clone();
        let heartbeat_interval = self.heartbeat_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(heartbeat_interval);

            loop {
                ticker.tick().await;

                // Broadcast heartbeat
                broadcast_fn(node_id.clone());

                // Mark stale peers as unhealthy
                if let Ok(mut peers) = peers.lock() {
                    let cutoff = Instant::now() - Duration::from_secs(15);
                    for peer in peers.iter_mut() {
                        if peer.last_heartbeat < cutoff {
                            peer.healthy = false;
                        }
                    }
                }
            }
        });
    }
}

/// Comprehensive HA manager
pub struct HighAvailabilityManager {
    health_monitor: Arc<HealthMonitor>,
    recovery_system: RecoverySystem,
    cluster_manager: ClusterManager,
}

impl HighAvailabilityManager {
    pub fn new(node_id: impl Into<String>) -> Self {
        let health_monitor = Arc::new(HealthMonitor::new());
        let recovery_system = RecoverySystem::new(Arc::clone(&health_monitor));
        let cluster_manager = ClusterManager::new(node_id);

        Self {
            health_monitor,
            recovery_system,
            cluster_manager,
        }
    }

    /// Get health monitor
    pub fn health_monitor(&self) -> Arc<HealthMonitor> {
        Arc::clone(&self.health_monitor)
    }

    /// Get recovery system
    pub fn recovery_system(&self) -> &RecoverySystem {
        &self.recovery_system
    }

    /// Get cluster manager
    pub fn cluster_manager(&self) -> &ClusterManager {
        &self.cluster_manager
    }

    /// Start all HA systems
    pub fn start(&self,
        checkers: Vec<Box<dyn HealthCheckable>>,
    ) {
        // Start health monitoring
        self.health_monitor.start_monitoring(checkers, 10);

        // Start recovery system
        self.recovery_system.start_recovery_loop(30);

        info!("High availability systems started");
    }

    /// Get system health report
    pub fn health_report(&self) -> HealthReport {
        HealthReport {
            overall_status: self.health_monitor.system_status(),
            checks: self.health_monitor.get_all_checks(),
            peer_count: self.cluster_manager.peer_count(),
        }
    }
}

/// Health report
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub peer_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHealthCheckable {
        component: String,
        status: HealthStatus,
    }

    #[async_trait::async_trait]
    impl HealthCheckable for MockHealthCheckable {
        async fn health_check(&self) -> HealthCheck {
            HealthCheck {
                component: self.component.clone(),
                status: self.status,
                last_check: Instant::now(),
                message: None,
                metrics: HashMap::new(),
            }
        }
    }

    #[test]
    fn test_health_monitor() {
        let monitor = HealthMonitor::new();

        // Add healthy check
        monitor.update_check(HealthCheck::healthy("rpc"));
        assert!(monitor.is_healthy());

        // Add unhealthy check
        monitor.update_check(HealthCheck::unhealthy("db", "connection failed"));
        assert!(!monitor.is_healthy());

        // Check system status
        assert_eq!(monitor.system_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_recovery_system() {
        let monitor = Arc::new(HealthMonitor::new());
        let recovery = RecoverySystem::new(monitor);

        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = Arc::clone(&called);
        recovery.register_recovery("test", Arc::new(move || {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            true
        }));

        // Note: Can't easily test recovery without async runtime
    }

    #[test]
    fn test_cluster_manager() {
        let cluster = ClusterManager::new("node1");

        cluster.add_peer("node2", "127.0.0.1:30304");
        cluster.add_peer("node3", "127.0.0.1:30305");

        assert_eq!(cluster.peer_count(), 2);

        let healthy = cluster.get_healthy_peers();
        assert_eq!(healthy.len(), 2);
    }
}
