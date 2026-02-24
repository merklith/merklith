//! Enterprise Security Module
//!
//! Provides:
//! - Rate limiting with token bucket algorithm
//! - DDoS protection
//! - Spam detection
//! - Malformed transaction filtering
//! - IP reputation system
//! - Audit logging for all security events

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

// Security configuration
const DEFAULT_RATE_LIMIT: u32 = 100; // requests per minute
const DEFAULT_BURST_SIZE: u32 = 20; // burst allowance
const BLOCK_DURATION_SECONDS: u64 = 3600; // 1 hour
const MAX_FAILED_ATTEMPTS: u32 = 10;
const SPAM_DETECTION_WINDOW: Duration = Duration::from_secs(60);
const SUSPICIOUS_PATTERN_THRESHOLD: u32 = 5;

/// Security event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityEventType {
    RateLimitExceeded,
    InvalidRequest,
    MalformedTransaction,
    SpamDetected,
    DDoSSuspected,
    InvalidSignature,
    ReplayAttack,
    BlockProductionTimeout,
    SuspiciousActivity,
    IpBlocked,
    IpUnblocked,
}

/// Security event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: u64,
    pub event_type: SecurityEventType,
    pub source: String,
    pub details: String,
    pub severity: Severity,
    pub action_taken: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// IP reputation tracking
#[derive(Debug, Clone)]
struct IpReputation {
    ip: IpAddr,
    request_count: u32,
    failed_attempts: u32,
    last_request: Instant,
    blocked_until: Option<Instant>,
    suspicious_patterns: u32,
    reputation_score: i32, // -100 to 100
}

impl IpReputation {
    fn new(ip: IpAddr) -> Self {
        Self {
            ip,
            request_count: 0,
            failed_attempts: 0,
            last_request: Instant::now(),
            blocked_until: None,
            suspicious_patterns: 0,
            reputation_score: 0,
        }
    }
    
    fn is_blocked(&self) -> bool {
        if let Some(blocked_until) = self.blocked_until {
            Instant::now() < blocked_until
        } else {
            false
        }
    }
    
    fn block_for(&mut self, duration: Duration) {
        self.blocked_until = Some(Instant::now() + duration);
        self.reputation_score -= 20;
    }
    
    fn record_request(&mut self) {
        self.request_count += 1;
        self.last_request = Instant::now();
        
        // Decay reputation slightly on successful requests
        if self.reputation_score < 100 {
            self.reputation_score += 1;
        }
    }
    
    fn record_failure(&mut self) {
        self.failed_attempts += 1;
        self.reputation_score -= 5;
        
        if self.failed_attempts >= MAX_FAILED_ATTEMPTS {
            self.block_for(Duration::from_secs(BLOCK_DURATION_SECONDS));
        }
    }
    
    fn record_suspicious(&mut self) {
        self.suspicious_patterns += 1;
        self.reputation_score -= 10;
        
        if self.suspicious_patterns >= SUSPICIOUS_PATTERN_THRESHOLD {
            self.block_for(Duration::from_secs(BLOCK_DURATION_SECONDS * 2));
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    rate: f64,      // tokens per second
    capacity: f64,  // max tokens
}

impl TokenBucket {
    fn new(rate: u32, capacity: u32) -> Self {
        Self {
            tokens: capacity as f64,
            last_update: Instant::now(),
            rate: rate as f64 / 60.0, // convert per minute to per second
            capacity: capacity as f64,
        }
    }
    
    fn try_consume(&mut self, tokens: u32) -> bool {
        self.add_tokens();
        
        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }
    
    fn add_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;
        
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
    }
}

/// Transaction spam detection
#[derive(Debug)]
struct TransactionPattern {
    from: String,
    count: u32,
    first_seen: Instant,
    last_seen: Instant,
    total_value: u128,
}

/// Enterprise Security Manager
pub struct SecurityManager {
    /// IP rate limiters
    rate_limiters: Arc<Mutex<HashMap<IpAddr, TokenBucket>>>,
    /// IP reputation database
    ip_reputation: Arc<Mutex<HashMap<IpAddr, IpReputation>>>,
    /// Transaction pattern detection
    tx_patterns: Arc<Mutex<HashMap<String, TransactionPattern>>>,
    /// Security event log
    event_log: Arc<Mutex<Vec<SecurityEvent>>>,
    /// Rate limit configuration
    rate_limit: u32,
    /// Burst size
    burst_size: u32,
    /// Whitelisted IPs (never block)
    whitelist: Arc<Mutex<HashSet<IpAddr>>>,
    /// Blacklisted IPs (always block)
    blacklist: Arc<Mutex<HashSet<IpAddr>>>,
}

use std::collections::HashSet;

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            rate_limiters: Arc::new(Mutex::new(HashMap::new())),
            ip_reputation: Arc::new(Mutex::new(HashMap::new())),
            tx_patterns: Arc::new(Mutex::new(HashMap::new())),
            event_log: Arc::new(Mutex::new(Vec::new())),
            rate_limit: DEFAULT_RATE_LIMIT,
            burst_size: DEFAULT_BURST_SIZE,
            whitelist: Arc::new(Mutex::new(HashSet::new())),
            blacklist: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    /// Create with custom rate limits
    pub fn with_rate_limit(rate_limit: u32, burst_size: u32) -> Self {
        Self {
            rate_limiters: Arc::new(Mutex::new(HashMap::new())),
            ip_reputation: Arc::new(Mutex::new(HashMap::new())),
            tx_patterns: Arc::new(Mutex::new(HashMap::new())),
            event_log: Arc::new(Mutex::new(Vec::new())),
            rate_limit,
            burst_size,
            whitelist: Arc::new(Mutex::new(HashSet::new())),
            blacklist: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    /// Check if request is allowed
    pub fn check_request(
        &self,
        ip: IpAddr,
        request_size: usize,
    ) -> Result<(), SecurityError> {
        // Check whitelist
        if self.whitelist.lock().unwrap().contains(&ip) {
            return Ok(());
        }
        
        // Check blacklist
        if self.blacklist.lock().unwrap().contains(&ip) {
            self.log_event(
                SecurityEventType::IpBlocked,
                ip.to_string(),
                "IP is blacklisted".to_string(),
                Severity::High,
                "Request rejected".to_string(),
            );
            return Err(SecurityError::IpBlacklisted(ip));
        }
        
        // Check IP reputation
        let mut reputation = self.ip_reputation.lock().unwrap();
        let rep = reputation.entry(ip).or_insert_with(|| IpReputation::new(ip));
        
        if rep.is_blocked() {
            self.log_event(
                SecurityEventType::IpBlocked,
                ip.to_string(),
                "IP temporarily blocked due to suspicious activity".to_string(),
                Severity::Medium,
                "Request rejected".to_string(),
            );
            return Err(SecurityError::IpBlocked(ip));
        }
        
        // Check rate limit
        let mut limiters = self.rate_limiters.lock().unwrap();
        let bucket = limiters.entry(ip).or_insert_with(|| {
            TokenBucket::new(self.rate_limit, self.burst_size)
        });
        
        if !bucket.try_consume(1) {
            rep.record_failure();
            
            self.log_event(
                SecurityEventType::RateLimitExceeded,
                ip.to_string(),
                format!("Rate limit exceeded: {} req/min", self.rate_limit),
                Severity::Medium,
                "Request throttled".to_string(),
            );
            
            return Err(SecurityError::RateLimitExceeded);
        }
        
        // Check request size (prevent large payload attacks)
        if request_size > 10 * 1024 * 1024 { // 10MB limit
            rep.record_suspicious();
            
            self.log_event(
                SecurityEventType::SuspiciousActivity,
                ip.to_string(),
                format!("Large request: {} bytes", request_size),
                Severity::Medium,
                "Request rejected".to_string(),
            );
            
            return Err(SecurityError::PayloadTooLarge(request_size));
        }
        
        rep.record_request();
        
        Ok(())
    }
    
    /// Validate transaction for spam/abuse
    pub fn validate_transaction(
        &self,
        from: &str,
        to: &str,
        value: u128,
        data: &[u8],
    ) -> Result<(), SecurityError> {
        // Check for spam patterns
        let mut patterns = self.tx_patterns.lock().unwrap();
        let pattern = patterns.entry(from.to_string()).or_insert(TransactionPattern {
            from: from.to_string(),
            count: 0,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            total_value: 0,
        });
        
        let now = Instant::now();
        
        // Reset if window passed
        if now.duration_since(pattern.first_seen) > SPAM_DETECTION_WINDOW {
            pattern.count = 0;
            pattern.first_seen = now;
            pattern.total_value = 0;
        }
        
        pattern.count += 1;
        pattern.last_seen = now;
        pattern.total_value += value;
        
        // Spam detection: >100 txs per minute from same address
        if pattern.count > 100 {
            self.log_event(
                SecurityEventType::SpamDetected,
                from.to_string(),
                format!("High transaction frequency: {} txs/min", pattern.count),
                Severity::High,
                "Transaction rejected".to_string(),
            );
            
            return Err(SecurityError::SpamDetected(from.to_string()));
        }
        
        // Check for zero-value spam
        if value == 0 && data.len() > 10000 {
            self.log_event(
                SecurityEventType::SpamDetected,
                from.to_string(),
                "Zero-value transaction with large data".to_string(),
                Severity::Medium,
                "Transaction rejected".to_string(),
            );
            
            return Err(SecurityError::LikelySpam);
        }
        
        // Check for dust spam (very small values)
        if value > 0 && value < 1000 && pattern.count > 50 {
            self.log_event(
                SecurityEventType::SpamDetected,
                from.to_string(),
                "Dust spam attack suspected".to_string(),
                Severity::Medium,
                "Transaction rate limited".to_string(),
            );
            
            return Err(SecurityError::SpamDetected(from.to_string()));
        }
        
        Ok(())
    }
    
    /// Record failed authentication attempt
    pub fn record_auth_failure(
        &self,
        ip: IpAddr,
        reason: &str,
    ) {
        let mut reputation = self.ip_reputation.lock().unwrap();
        let rep = reputation.entry(ip).or_insert_with(|| IpReputation::new(ip));
        
        rep.record_failure();
        
        self.log_event(
            SecurityEventType::InvalidSignature,
            ip.to_string(),
            reason.to_string(),
            Severity::Medium,
            "Auth failure recorded".to_string(),
        );
    }
    
    /// Detect DDoS patterns
    pub fn check_ddos(
        &self,
        ip: IpAddr,
    ) -> Result<(), SecurityError> {
        let reputation = self.ip_reputation.lock().unwrap();
        
        if let Some(rep) = reputation.get(&ip) {
            // Check for DDoS indicators
            if rep.request_count > 10000 || rep.reputation_score < -50 {
                drop(reputation);
                
                // Block the IP
                let mut rep_mut = self.ip_reputation.lock().unwrap();
                if let Some(r) = rep_mut.get_mut(&ip) {
                    r.block_for(Duration::from_secs(BLOCK_DURATION_SECONDS * 24)); // 24 hours
                }
                
                self.log_event(
                    SecurityEventType::DDoSSuspected,
                    ip.to_string(),
                    format!(
                        "DDoS detected: {} requests, reputation: {}",
                        rep.request_count,
                        rep.reputation_score
                    ),
                    Severity::Critical,
                    "IP blocked for 24 hours".to_string(),
                );
                
                return Err(SecurityError::DDoSDetected(ip));
            }
        }
        
        Ok(())
    }
    
    /// Add IP to whitelist
    pub fn whitelist_ip(&self, ip: IpAddr) {
        self.whitelist.lock().unwrap().insert(ip);
        
        // Remove from blacklist if present
        self.blacklist.lock().unwrap().remove(&ip);
        
        self.log_event(
            SecurityEventType::IpUnblocked,
            ip.to_string(),
            "IP added to whitelist".to_string(),
            Severity::Low,
            "Whitelisted".to_string(),
        );
    }
    
    /// Add IP to blacklist
    pub fn blacklist_ip(&self, ip: IpAddr, duration: Duration) {
        self.blacklist.lock().unwrap().insert(ip);
        self.whitelist.lock().unwrap().remove(&ip);
        
        let mut reputation = self.ip_reputation.lock().unwrap();
        let rep = reputation.entry(ip).or_insert_with(|| IpReputation::new(ip));
        rep.block_for(duration);
        
        self.log_event(
            SecurityEventType::IpBlocked,
            ip.to_string(),
            format!("IP manually blacklisted for {:?}", duration),
            Severity::High,
            "Blacklisted".to_string(),
        );
    }
    
    /// Log security event
    fn log_event(
        &self,
        event_type: SecurityEventType,
        source: String,
        details: String,
        severity: Severity,
        action_taken: String,
    ) {
        let event = SecurityEvent {
            timestamp: current_timestamp(),
            event_type,
            source,
            details,
            severity,
            action_taken,
        };
        
        let mut log = self.event_log.lock().unwrap();
        log.push(event);
        
        // Keep only last 10000 events
        if log.len() > 10000 {
            log.remove(0);
        }
        
        // Also log to tracing
        match severity {
            Severity::Critical => tracing::error!("Security: {:?}", event),
            Severity::High => tracing::warn!("Security: {:?}", event),
            Severity::Medium => tracing::info!("Security: {:?}", event),
            Severity::Low => tracing::debug!("Security: {:?}", event),
        }
    }
    
    /// Get security events
    pub fn get_events(
        &self,
        limit: usize,
    ) -> Vec<SecurityEvent> {
        let log = self.event_log.lock().unwrap();
        log.iter().rev().take(limit).cloned().collect()
    }
    
    /// Get IP reputation
    pub fn get_ip_reputation(
        &self,
        ip: IpAddr,
    ) -> Option<(i32, bool)> {
        self.ip_reputation
            .lock()
            .unwrap()
            .get(&ip)
            .map(|rep| (rep.reputation_score, rep.is_blocked()))
    }
    
    /// Get stats
    pub fn get_stats(&self) -> SecurityStats {
        SecurityStats {
            total_ips_tracked: self.ip_reputation.lock().unwrap().len(),
            blocked_ips: self
                .ip_reputation
                .lock()
                .unwrap()
                .values()
                .filter(|r| r.is_blocked())
                .count(),
            total_events: self.event_log.lock().unwrap().len(),
            whitelisted_ips: self.whitelist.lock().unwrap().len(),
            blacklisted_ips: self.blacklist.lock().unwrap().len(),
        }
    }
    
    /// Clean up old entries (call periodically)
    pub fn cleanup(&self) {
        let mut reputation = self.ip_reputation.lock().unwrap();
        let now = Instant::now();
        
        // Remove entries older than 24 hours that aren't blocked
        reputation.retain(|_, rep| {
            !rep.is_blocked() && now.duration_since(rep.last_request) < Duration::from_secs(86400)
        });
        
        let mut limiters = self.rate_limiters.lock().unwrap();
        limiters.clear(); // Reset rate limiters periodically
        
        let mut patterns = self.tx_patterns.lock().unwrap();
        let now = Instant::now();
        patterns.retain(|_, pattern| {
            now.duration_since(pattern.last_seen) < SPAM_DETECTION_WINDOW
        });
    }
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub total_ips_tracked: usize,
    pub blocked_ips: usize,
    pub total_events: usize,
    pub whitelisted_ips: usize,
    pub blacklisted_ips: usize,
}

/// Security errors
#[derive(Debug, Clone)]
pub enum SecurityError {
    RateLimitExceeded,
    IpBlocked(IpAddr),
    IpBlacklisted(IpAddr),
    PayloadTooLarge(usize),
    SpamDetected(String),
    LikelySpam,
    DDoSDetected(IpAddr),
    InvalidSignature,
    ReplayAttack,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            SecurityError::IpBlocked(ip) => write!(f, "IP blocked: {}", ip),
            SecurityError::IpBlacklisted(ip) => write!(f, "IP blacklisted: {}", ip),
            SecurityError::PayloadTooLarge(size) => write!(f, "Payload too large: {} bytes", size),
            SecurityError::SpamDetected(addr) => write!(f, "Spam detected from: {}", addr),
            SecurityError::LikelySpam => write!(f, "Likely spam transaction"),
            SecurityError::DDoSDetected(ip) => write!(f, "DDoS detected from: {}", ip),
            SecurityError::InvalidSignature => write!(f, "Invalid signature"),
            SecurityError::ReplayAttack => write!(f, "Replay attack detected"),
        }
    }
}

impl std::error::Error for SecurityError {}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_rate_limiting() {
        let manager = SecurityManager::with_rate_limit(10, 5);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // Should allow burst
        for _ in 0..5 {
            assert!(manager.check_request(ip, 1000).is_ok());
        }
        
        // Should rate limit after burst
        assert!(manager.check_request(ip, 1000).is_err());
    }
    
    #[test]
    fn test_spam_detection() {
        let manager = SecurityManager::new();
        let from = "0x1234567890abcdef";
        
        // Normal transactions should pass
        for i in 0..50 {
            assert!(manager.validate_transaction(from, "0xrecipient", i as u128, &[]).is_ok());
        }
        
        // Should detect spam after threshold
        // This would require more iterations to trigger
    }
}
