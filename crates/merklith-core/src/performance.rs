//! Performance optimization module for MERKLITH blockchain
//! 
//! Provides caching, memory pools, and async optimizations.

use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use lru::LruCache;

/// Generic LRU cache wrapper with TTL support
pub struct TimedCache<K, V> {
    cache: Mutex<LruCache<K, CacheEntry<V>>>,
    ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
}

impl<K: Eq + Hash, V: Clone> TimedCache<K, V> {
    /// Create cache with capacity and TTL
    pub fn new(capacity: usize, ttl_secs: u64) -> Self {
        let cap = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1).unwrap());
        Self {
            cache: Mutex::new(LruCache::new(cap)),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Get value from cache
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.lock().ok()?;
        
        if let Some(entry) = cache.get(key) {
            if entry.inserted_at.elapsed() < self.ttl {
                return Some(entry.value.clone());
            }
            // Entry expired, remove it
            cache.pop(key);
        }
        
        None
    }

    /// Insert value into cache
    pub fn put(&self, key: K, value: V) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(key, CacheEntry {
                value,
                inserted_at: Instant::now(),
            });
        }
    }

    /// Clear all entries
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.lock().map(|c| c.len()).unwrap_or(0)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Block cache for fast lookups
pub type BlockCache = TimedCache<u64, merklith_types::Block>;

/// Transaction cache
pub type TransactionCache = TimedCache<merklith_types::Hash, merklith_types::Transaction>;

/// State cache for account balances
pub type StateCache = TimedCache<merklith_types::Address, merklith_types::Account>;

/// Performance metrics collector
pub struct PerformanceMetrics {
    metrics: Mutex<HashMap<String, MetricValue>>,
}

#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timing(Duration),
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            metrics: Mutex::new(HashMap::new()),
        }
    }

    /// Increment counter
    pub fn increment(&self, name: impl Into<String>) {
        let name = name.into();
        if let Ok(mut metrics) = self.metrics.lock() {
            let entry = metrics.entry(name).or_insert(MetricValue::Counter(0));
            if let MetricValue::Counter(count) = entry {
                *count += 1;
            }
        }
    }

    /// Set gauge value
    pub fn gauge(&self, name: impl Into<String>, value: f64) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.insert(name.into(), MetricValue::Gauge(value));
        }
    }

    /// Record timing
    pub fn timing(&self, name: impl Into<String>, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.insert(name.into(), MetricValue::Timing(duration));
        }
    }

    /// Record value in histogram
    pub fn histogram(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        if let Ok(mut metrics) = self.metrics.lock() {
            let entry = metrics.entry(name).or_insert(MetricValue::Histogram(vec![]));
            if let MetricValue::Histogram(values) = entry {
                values.push(value);
                // Keep only last 1000 values
                if values.len() > 1000 {
                    values.remove(0);
                }
            }
        }
    }

    /// Get metric
    pub fn get(&self, name: &str) -> Option<MetricValue> {
        self.metrics.lock().ok()?.get(name).cloned()
    }

    /// Get all metrics
    pub fn get_all(&self) -> HashMap<String, MetricValue> {
        self.metrics.lock().ok().map(|m| m.clone()).unwrap_or_default()
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory pool for reusable buffers
pub struct BufferPool {
    pool: Mutex<Vec<Vec<u8>>>,
    buffer_size: usize,
    max_pool_size: usize,
}

impl BufferPool {
    /// Create buffer pool
    pub fn new(buffer_size: usize, max_pool_size: usize) -> Self {
        Self {
            pool: Mutex::new(Vec::with_capacity(max_pool_size)),
            buffer_size,
            max_pool_size,
        }
    }

    /// Get buffer from pool
    pub fn acquire(&self) -> Vec<u8> {
        if let Ok(mut pool) = self.pool.lock() {
            if let Some(buffer) = pool.pop() {
                return buffer;
            }
        }
        
        // Create new buffer if pool is empty
        Vec::with_capacity(self.buffer_size)
    }

    /// Return buffer to pool
    pub fn release(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_pool_size {
                pool.push(buffer);
            }
        }
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.pool.lock().map(|p| p.len()).unwrap_or(0)
    }
}

/// Batch processor for efficient bulk operations
pub struct BatchProcessor<T> {
    items: Mutex<Vec<T>>,
    batch_size: usize,
    timeout: Duration,
}

impl<T: Send + 'static> BatchProcessor<T> {
    /// Create batch processor
    pub fn new(batch_size: usize, timeout_ms: u64) -> Self {
        Self {
            items: Mutex::new(Vec::with_capacity(batch_size)),
            batch_size,
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    /// Add item to batch
    pub fn push(&self, item: T) -> bool {
        if let Ok(mut items) = self.items.lock() {
            items.push(item);
            items.len() >= self.batch_size
        } else {
            false
        }
    }

    /// Get current batch and clear
    pub fn take_batch(&self) -> Vec<T> {
        if let Ok(mut items) = self.items.lock() {
            std::mem::take(&mut *items)
        } else {
            Vec::new()
        }
    }

    /// Get batch size
    pub fn len(&self) -> usize {
        self.items.lock().map(|i| i.len()).unwrap_or(0)
    }

    /// Start batch processing loop
    pub fn start_processing(&self,
        process_fn: impl Fn(Vec<T>) + Send + 'static,
    ) {
        let batch = Arc::new(self.clone());
        let timeout = self.timeout;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(timeout);

            loop {
                interval.tick().await;

                let items = batch.take_batch();
                if !items.is_empty() {
                    process_fn(items);
                }
            }
        });
    }
}

impl<T> Clone for BatchProcessor<T> {
    fn clone(&self) -> Self {
        Self {
            items: Mutex::new(Vec::with_capacity(self.batch_size)),
            batch_size: self.batch_size,
            timeout: self.timeout,
        }
    }
}

/// Connection pool for async clients
pub struct AsyncConnectionPool<T> {
    connections: Mutex<Vec<T>>,
    max_size: usize,
}

impl<T> AsyncConnectionPool<T> {
    /// Create connection pool
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Mutex::new(Vec::with_capacity(max_size)),
            max_size,
        }
    }

    /// Add connection to pool
    pub fn add(&self, conn: T) -> Result<(), T> {
        if let Ok(mut connections) = self.connections.lock() {
            if connections.len() < self.max_size {
                connections.push(conn);
                return Ok(());
            }
        }
        Err(conn)
    }

    /// Get connection from pool
    pub fn get(&self) -> Option<T> {
        self.connections.lock().ok()?.pop()
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.connections.lock().map(|c| c.len()).unwrap_or(0)
    }
}

/// Optimization manager
pub struct OptimizationManager {
    /// Block cache
    pub block_cache: Arc<BlockCache>,
    /// Transaction cache
    pub tx_cache: Arc<TransactionCache>,
    /// State cache
    pub state_cache: Arc<StateCache>,
    /// Performance metrics
    pub metrics: Arc<PerformanceMetrics>,
    /// Buffer pool
    pub buffer_pool: Arc<BufferPool>,
}

impl OptimizationManager {
    /// Create optimization manager with default settings
    pub fn new() -> Self {
        Self {
            block_cache: Arc::new(TimedCache::new(1000, 300)), // 1000 blocks, 5 min TTL
            tx_cache: Arc::new(TimedCache::new(10000, 300)),   // 10000 txs, 5 min TTL
            state_cache: Arc::new(TimedCache::new(10000, 60)), // 10000 accounts, 1 min TTL
            metrics: Arc::new(PerformanceMetrics::new()),
            buffer_pool: Arc::new(BufferPool::new(4096, 100)),
        }
    }

    /// Get block from cache or fetch
    pub fn get_block<F>(
        &self,
        number: u64,
        fetch_fn: F,
    ) -> Option<merklith_types::Block>
    where
        F: FnOnce(u64) -> Option<merklith_types::Block>,
    {
        // Try cache first
        if let Some(block) = self.block_cache.get(&number) {
            self.metrics.increment("cache.block.hit");
            return Some(block);
        }

        self.metrics.increment("cache.block.miss");

        // Fetch and cache
        if let Some(block) = fetch_fn(number) {
            self.block_cache.put(number, block.clone());
            Some(block)
        } else {
            None
        }
    }

    /// Get transaction from cache
    pub fn get_transaction(&self,
        hash: &merklith_types::Hash,
    ) -> Option<merklith_types::Transaction> {
        self.tx_cache.get(hash)
    }

    /// Cache transaction
    pub fn cache_transaction(&self,
        hash: merklith_types::Hash,
        tx: merklith_types::Transaction,
    ) {
        self.tx_cache.put(hash, tx);
    }

    /// Get account from cache
    pub fn get_account(&self,
        address: &merklith_types::Address,
    ) -> Option<merklith_types::Account> {
        self.state_cache.get(address)
    }

    /// Cache account
    pub fn cache_account(&self,
        address: merklith_types::Address,
        account: merklith_types::Account,
    ) {
        self.state_cache.put(address, account);
    }

    /// Record RPC request metrics
    pub fn record_rpc_request(&self,
        method: &str,
        duration: Duration,
    ) {
        self.metrics.increment(format!("rpc.{}.count", method));
        self.metrics.histogram(format!("rpc.{}.duration", method), duration.as_millis() as f64);
    }

    /// Get cache stats
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            block_cache_size: self.block_cache.len(),
            tx_cache_size: self.tx_cache.len(),
            state_cache_size: self.state_cache.len(),
            buffer_pool_size: self.buffer_pool.size(),
        }
    }

    /// Get performance metrics
    pub fn performance_report(&self) -> HashMap<String, MetricValue> {
        self.metrics.get_all()
    }
}

impl Default for OptimizationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub block_cache_size: usize,
    pub tx_cache_size: usize,
    pub state_cache_size: usize,
    pub buffer_pool_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timed_cache() {
        let cache = TimedCache::new(100, 1);
        
        cache.put(1, "value1".to_string());
        assert_eq!(cache.get(&1), Some("value1".to_string()));
        
        // Wait for TTL to expire
        std::thread::sleep(Duration::from_secs(2));
        assert_eq!(cache.get(&1), None);
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new();
        
        metrics.increment("requests");
        metrics.increment("requests");
        
        if let Some(MetricValue::Counter(count)) = metrics.get("requests") {
            assert_eq!(count, 2);
        } else {
            panic!("Expected counter");
        }
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(1024, 5);
        
        let buffer = pool.acquire();
        assert!(buffer.capacity() >= 1024);
        
        pool.release(buffer);
        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_optimization_manager() {
        let manager = OptimizationManager::new();
        
        let stats = manager.cache_stats();
        assert_eq!(stats.block_cache_size, 0);
        
        // Test cache
        manager.cache_transaction(
            merklith_types::Hash::ZERO,
            merklith_types::Transaction::default(),
        );
        
        let stats = manager.cache_stats();
        assert_eq!(stats.tx_cache_size, 1);
    }
}
