//! Security module for MERKLITH blockchain
//! Provides rate limiting, input validation, and replay protection

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use merklith_types::{Address, Hash, SignedTransaction};

/// Rate limiter for RPC endpoints
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Check if request is allowed
    pub fn check_rate(&self, key: &str) -> Result<(), SecurityError> {
        let mut requests = self.requests.lock().map_err(|_| SecurityError::LockError)?;
        let now = Instant::now();
        
        // Get or create request history for this key
        let history = requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        history.retain(|&time| now.duration_since(time) < self.window);
        
        // Check if limit exceeded
        if history.len() >= self.max_requests {
            return Err(SecurityError::RateLimitExceeded);
        }
        
        // Record this request
        history.push(now);
        Ok(())
    }

    /// Check rate with IP
    pub fn check_ip_rate(&self, ip: &str) -> Result<(), SecurityError> {
        self.check_rate(&format!("ip:{}", ip))
    }

    /// Check rate with address
    pub fn check_address_rate(&self, address: &Address) -> Result<(), SecurityError> {
        self.check_rate(&format!("addr:{:x}", address))
    }
}

/// Transaction replay protection
pub struct ReplayProtection {
    seen_nonces: Arc<Mutex<HashMap<Address, u64>>>,
    seen_hashes: Arc<Mutex<HashMap<Hash, Instant>>>,
    hash_ttl: Duration,
}

impl ReplayProtection {
    pub fn new(hash_ttl_secs: u64) -> Self {
        Self {
            seen_nonces: Arc::new(Mutex::new(HashMap::new())),
            seen_hashes: Arc::new(Mutex::new(HashMap::new())),
            hash_ttl: Duration::from_secs(hash_ttl_secs),
        }
    }

    /// Check if transaction is a replay
    pub fn check_transaction(&self, tx: &SignedTransaction) -> Result<(), SecurityError> {
        let hash = tx.hash();
        let sender = tx.sender();
        
        // Check if we've seen this exact transaction hash
        {
            let mut seen_hashes = self.seen_hashes.lock().map_err(|_| SecurityError::LockError)?;
            let now = Instant::now();
            
            // Clean up old entries
            seen_hashes.retain(|_, time| now.duration_since(*time) < self.hash_ttl);
            
            if seen_hashes.contains_key(&hash) {
                return Err(SecurityError::ReplayTransaction);
            }
            
            // Record this hash
            seen_hashes.insert(hash, now);
        }
        
        // Check nonce sequence
        {
            let mut seen_nonces = self.seen_nonces.lock().map_err(|_| SecurityError::LockError)?;
            let last_nonce = seen_nonces.get(&sender).copied().unwrap_or(0);
            
            // Nonce must exactly match the expected value (prevents replay attacks)
            if tx.tx.nonce != last_nonce {
                return Err(SecurityError::InvalidNonce {
                    expected: last_nonce,
                    got: tx.tx.nonce,
                });
            }
            
            // Update last seen nonce
            seen_nonces.insert(sender, last_nonce + 1);
        }
        
        Ok(())
    }

    /// Clear old entries manually
    pub fn cleanup(&self) -> Result<(), SecurityError> {
        let mut seen_hashes = self.seen_hashes.lock().map_err(|_| SecurityError::LockError)?;
        let now = Instant::now();
        seen_hashes.retain(|_, time| now.duration_since(*time) < self.hash_ttl);
        Ok(())
    }
}

/// Input validator for security checks
pub struct InputValidator;

impl InputValidator {
    /// Validate Ethereum-style address
    pub fn validate_address(addr: &str) -> Result<(), SecurityError> {
        // Check format
        if !addr.starts_with("0x") {
            return Err(SecurityError::InvalidAddress);
        }
        
        let hex_part = &addr[2..];
        
        // Check length (20 bytes = 40 hex chars)
        if hex_part.len() != 40 {
            return Err(SecurityError::InvalidAddress);
        }
        
        // Check valid hex
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SecurityError::InvalidAddress);
        }
        
        Ok(())
    }

    /// Validate transaction value (prevent overflow)
    pub fn validate_value(value: &str) -> Result<(), SecurityError> {
        // Remove 0x prefix if present
        let value = value.trim_start_matches("0x");
        
        // Check valid hex
        if !value.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SecurityError::InvalidValue);
        }
        
        // Check length (U256 max is 32 bytes = 64 hex chars)
        if value.len() > 64 {
            return Err(SecurityError::ValueOverflow);
        }
        
        Ok(())
    }

    /// Validate gas limit
    pub fn validate_gas_limit(gas: u64) -> Result<(), SecurityError> {
        // Max gas per block
        const MAX_GAS_LIMIT: u64 = 30_000_000;
        
        if gas == 0 {
            return Err(SecurityError::InvalidGasLimit);
        }
        
        if gas > MAX_GAS_LIMIT {
            return Err(SecurityError::GasLimitTooHigh);
        }
        
        Ok(())
    }

    /// Validate gas price
    pub fn validate_gas_price(price: u64) -> Result<(), SecurityError> {
        // Max gas price (1000 gwei)
        const MAX_GAS_PRICE: u64 = 1_000_000_000_000;
        
        if price > MAX_GAS_PRICE {
            return Err(SecurityError::GasPriceTooHigh);
        }
        
        Ok(())
    }

    /// Validate chain ID
    pub fn validate_chain_id(chain_id: u64, expected: u64) -> Result<(), SecurityError> {
        if chain_id != expected {
            return Err(SecurityError::InvalidChainId {
                expected,
                got: chain_id,
            });
        }
        
        Ok(())
    }

    /// Sanitize string input (prevent injection attacks)
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|&c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            .take(256) // Max length
            .collect()
    }
}

/// Security-related errors
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityError {
    RateLimitExceeded,
    ReplayTransaction,
    InvalidNonce { expected: u64, got: u64 },
    InvalidAddress,
    InvalidValue,
    ValueOverflow,
    InvalidGasLimit,
    GasLimitTooHigh,
    GasPriceTooHigh,
    InvalidChainId { expected: u64, got: u64 },
    LockError,
    InvalidInput(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            SecurityError::ReplayTransaction => write!(f, "Transaction replay detected"),
            SecurityError::InvalidNonce { expected, got } => {
                write!(f, "Invalid nonce: expected {}, got {}", expected, got)
            }
            SecurityError::InvalidAddress => write!(f, "Invalid address format"),
            SecurityError::InvalidValue => write!(f, "Invalid value format"),
            SecurityError::ValueOverflow => write!(f, "Value overflow"),
            SecurityError::InvalidGasLimit => write!(f, "Invalid gas limit"),
            SecurityError::GasLimitTooHigh => write!(f, "Gas limit too high"),
            SecurityError::GasPriceTooHigh => write!(f, "Gas price too high"),
            SecurityError::InvalidChainId { expected, got } => {
                write!(f, "Invalid chain ID: expected {}, got {}", expected, got)
            }
            SecurityError::LockError => write!(f, "Lock acquisition failed"),
            SecurityError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Comprehensive security manager
pub struct SecurityManager {
    rate_limiter: RateLimiter,
    replay_protection: ReplayProtection,
    chain_id: u64,
}

impl SecurityManager {
    pub fn new(chain_id: u64) -> Self {
        Self {
            rate_limiter: RateLimiter::new(100, 60), // 100 requests per minute
            replay_protection: ReplayProtection::new(3600), // 1 hour TTL
            chain_id,
        }
    }

    pub fn with_custom_rate_limit(max_requests: usize, window_secs: u64) -> Self {
        Self {
            rate_limiter: RateLimiter::new(max_requests, window_secs),
            replay_protection: ReplayProtection::new(3600),
            chain_id: 17001,
        }
    }

    /// Validate incoming transaction
    pub fn validate_transaction(
        &self, tx: &SignedTransaction
    ) -> Result<(), SecurityError> {
        let sender = tx.sender();

        // Check rate limit for sender
        self.rate_limiter.check_address_rate(&sender)?;

        // Check replay protection
        self.replay_protection.check_transaction(tx)?;

        // Validate chain ID
        InputValidator::validate_chain_id(tx.tx.chain_id, self.chain_id)?;

        // Validate gas limit
        InputValidator::validate_gas_limit(tx.tx.gas_limit)?;

        // Validate max fee per gas (use first limb as u64)
        InputValidator::validate_gas_price(tx.tx.max_fee_per_gas.as_limbs()[0])?;

        Ok(())
    }

    /// Check RPC rate limit
    pub fn check_rpc_rate(&self, ip: &str) -> Result<(), SecurityError> {
        self.rate_limiter.check_ip_rate(ip)
    }

    /// Cleanup old entries
    pub fn cleanup(&self) -> Result<(), SecurityError> {
        self.replay_protection.cleanup()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 60);
        
        // Should allow 3 requests
        assert!(limiter.check_rate("test").is_ok());
        assert!(limiter.check_rate("test").is_ok());
        assert!(limiter.check_rate("test").is_ok());
        
        // 4th should fail
        assert!(matches!(limiter.check_rate("test"), Err(SecurityError::RateLimitExceeded)));
        
        // Different key should work
        assert!(limiter.check_rate("test2").is_ok());
    }

    #[test]
    fn test_input_validator_address() {
        assert!(InputValidator::validate_address("0x1234567890123456789012345678901234567890").is_ok());
        assert!(InputValidator::validate_address("1234567890123456789012345678901234567890").is_err()); // No 0x
        assert!(InputValidator::validate_address("0x123").is_err()); // Too short
        assert!(InputValidator::validate_address("0xGGGG").is_err()); // Invalid hex
    }

    #[test]
    fn test_input_validator_gas() {
        assert!(InputValidator::validate_gas_limit(21000).is_ok());
        assert!(InputValidator::validate_gas_limit(0).is_err());
        assert!(InputValidator::validate_gas_limit(31_000_000).is_err()); // Too high
    }

    #[test]
    fn test_security_manager() {
        let manager = SecurityManager::new(17001);
        
        // Test chain ID validation
        assert!(InputValidator::validate_chain_id(17001, 17001).is_ok());
        assert!(InputValidator::validate_chain_id(1, 17001).is_err());
    }
}
