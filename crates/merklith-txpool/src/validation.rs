//! Transaction validation logic.
//!
//! Validates transactions before they enter the mempool.

use merklith_types::{Address, Transaction, ChainConfig, U256};
use merklith_core::state::AccountState;
use crate::error::PoolError;

/// Configuration for transaction validation.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum gas price (wei)
    pub min_gas_price: U256,
    /// Maximum gas limit per transaction
    pub max_gas_limit: u64,
    /// Maximum transaction size in bytes
    pub max_tx_size: usize,
    /// Chain ID
    pub chain_id: u64,
    /// Whether to require strict chain ID
    pub require_chain_id: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_gas_price: U256::from(1_000_000_000u64), // 1 gwei
            max_gas_limit: 30_000_000,
            max_tx_size: 128 * 1024, // 128 KB
            chain_id: 1,
            require_chain_id: true,
        }
    }
}

/// Context for transaction validation.
#[derive(Debug)]
pub struct ValidationContext<'a> {
    /// Current account state
    pub account_state: &'a AccountState,
    /// Current block number
    pub block_number: u64,
    /// Current block timestamp
    pub timestamp: u64,
    /// Base fee for this block
    pub base_fee: U256,
    /// Chain configuration
    pub chain_config: &'a ChainConfig,
}

impl<'a> ValidationContext<'a> {
    /// Create a new validation context.
    pub fn new(
        account_state: &'a AccountState,
        block_number: u64,
        timestamp: u64,
        base_fee: U256,
        chain_config: &'a ChainConfig,
    ) -> Self {
        Self {
            account_state,
            block_number,
            timestamp,
            base_fee,
            chain_config,
        }
    }
}

/// Transaction validation result.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Error message if invalid
    pub error: Option<PoolError>,
    /// Expected nonce for sender
    pub expected_nonce: u64,
    /// Sender balance
    pub sender_balance: U256,
}

impl ValidationResult {
    /// Create a successful result.
    pub fn success(expected_nonce: u64, sender_balance: U256) -> Self {
        Self {
            valid: true,
            error: None,
            expected_nonce,
            sender_balance,
        }
    }

    /// Create a failed result.
    pub fn failure(error: PoolError) -> Self {
        Self {
            valid: false,
            error: Some(error),
            expected_nonce: 0,
            sender_balance: U256::ZERO,
        }
    }
}

/// Validate a transaction.
///
/// # Arguments
/// - `tx`: The transaction to validate
/// - `config`: Validation configuration
/// - `context`: Current blockchain context
///
/// # Returns
/// Validation result indicating if the transaction is valid and why.
pub fn validate_transaction(
    tx: &Transaction,
    config: &ValidationConfig,
    context: &ValidationContext,
) -> ValidationResult {
    // Check transaction size
    if let Err(e) = validate_size(tx, config) {
        return ValidationResult::failure(e);
    }

    // Check gas limit
    if let Err(e) = validate_gas_limit(tx, config) {
        return ValidationResult::failure(e);
    }

    // Check gas price
    if let Err(e) = validate_gas_price(tx, config, context) {
        return ValidationResult::failure(e);
    }

    // Check chain ID
    if let Err(e) = validate_chain_id(tx, config) {
        return ValidationResult::failure(e);
    }

    // Get sender info from context
    let sender = match tx.sender() {
        Some(addr) => addr,
        None => {
            return ValidationResult::failure(PoolError::InvalidSignature(
                "Could not recover sender".to_string()
            ));
        }
    };

    let account = match context.account_state.get_account(&sender) {
        Some(acc) => acc,
        None => {
            // Account doesn't exist, check if it's a valid new account
            return ValidationResult::failure(PoolError::InsufficientBalance {
                have: 0,
                want: tx.value.as_u128() + (tx.gas_price * U256::from(tx.gas_limit)).as_u128(),
            });
        }
    };

    // Check nonce
    if let Err(e) = validate_nonce(tx, account.nonce) {
        return ValidationResult::failure(e);
    }

    // Check balance
    if let Err(e) = validate_balance(tx, account.balance) {
        return ValidationResult::failure(e);
    }

    ValidationResult::success(account.nonce, account.balance)
}

/// Validate transaction size.
fn validate_size(tx: &Transaction, config: &ValidationConfig) -> Result<(), PoolError> {
    let size = tx.encode_size();
    if size > config.max_tx_size {
        return Err(PoolError::TransactionTooLarge {
            size,
            limit: config.max_tx_size,
        });
    }
    Ok(())
}

/// Validate gas limit.
fn validate_gas_limit(tx: &Transaction, config: &ValidationConfig) -> Result<(), PoolError> {
    if tx.gas_limit == 0 {
        return Err(PoolError::InvalidTransaction(
            "Gas limit cannot be zero".to_string()
        ));
    }

    if tx.gas_limit > config.max_gas_limit {
        return Err(PoolError::GasLimitExceeded {
            limit: config.max_gas_limit,
            got: tx.gas_limit,
        });
    }

    // Check if enough gas for intrinsic cost
    let intrinsic_gas = calculate_intrinsic_gas(tx);
    if tx.gas_limit < intrinsic_gas {
        return Err(PoolError::InvalidTransaction(
            format!("Gas limit {} below intrinsic cost {}", tx.gas_limit, intrinsic_gas)
        ));
    }

    Ok(())
}

/// Validate gas price.
fn validate_gas_price(
    tx: &Transaction,
    config: &ValidationConfig,
    context: &ValidationContext,
) -> Result<(), PoolError> {
    // Check minimum gas price
    if tx.gas_price < config.min_gas_price {
        return Err(PoolError::GasPriceTooLow {
            minimum: config.min_gas_price.as_u128(),
            got: tx.gas_price.as_u128(),
        });
    }

    // For EIP-1559 transactions, check effective gas price
    if let Some(max_fee) = tx.max_fee_per_gas {
        // Effective gas price = min(max_fee_per_gas, base_fee + max_priority_fee)
        let priority_fee = tx.max_priority_fee_per_gas.unwrap_or(U256::ZERO);
        let effective_gas_price = std::cmp::min(
            max_fee,
            context.base_fee + priority_fee,
        );

        if effective_gas_price < config.min_gas_price {
            return Err(PoolError::GasPriceTooLow {
                minimum: config.min_gas_price.as_u128(),
                got: effective_gas_price.as_u128(),
            });
        }
    }

    Ok(())
}

/// Validate chain ID.
fn validate_chain_id(tx: &Transaction, config: &ValidationConfig) -> Result<(), PoolError> {
    if config.require_chain_id {
        match tx.chain_id {
            Some(id) if id == config.chain_id => Ok(()),
            Some(id) => Err(PoolError::ChainIdMismatch {
                expected: config.chain_id,
                got: id,
            }),
            None => Err(PoolError::InvalidTransaction(
                "Chain ID required but not present".to_string()
            )),
        }
    } else {
        Ok(())
    }
}

/// Validate nonce.
fn validate_nonce(tx: &Transaction, expected_nonce: u64) -> Result<(), PoolError> {
    if tx.nonce < expected_nonce {
        return Err(PoolError::NonceTooLow {
            expected: expected_nonce,
            got: tx.nonce,
        });
    }

    // Allow nonces slightly higher (queued transactions)
    // Reject if too high (gap too large)
    let max_future_nonce = expected_nonce + 1000;
    if tx.nonce > max_future_nonce {
        return Err(PoolError::NonceTooHigh {
            expected: expected_nonce,
            got: tx.nonce,
        });
    }

    Ok(())
}

/// Validate balance.
fn validate_balance(tx: &Transaction, balance: U256) -> Result<(), PoolError> {
    // Calculate max cost: value + (gas_price * gas_limit)
    let max_cost = tx.value + (tx.gas_price * U256::from(tx.gas_limit));

    if balance < max_cost {
        return Err(PoolError::InsufficientBalance {
            have: balance.as_u128(),
            want: max_cost.as_u128(),
        });
    }

    Ok(())
}

/// Calculate intrinsic gas cost for a transaction.
fn calculate_intrinsic_gas(tx: &Transaction) -> u64 {
    // Base cost for transaction
    let mut gas = 21_000u64;

    // Add cost for data
    for byte in &tx.data {
        if *byte == 0 {
            gas += 4; // Zero byte
        } else {
            gas += 16; // Non-zero byte
        }
    }

    // Add cost for access list (if present)
    // This is a simplified version

    gas
}

/// Check if a transaction can replace an existing one.
///
/// Same nonce, higher gas price required.
pub fn can_replace(existing: &Transaction, new: &Transaction) -> Result<(), PoolError> {
    if existing.sender() != new.sender() {
        return Err(PoolError::InvalidTransaction(
            "Cannot replace transaction from different sender".to_string()
        ));
    }

    if existing.nonce != new.nonce {
        return Err(PoolError::InvalidTransaction(
            "Cannot replace transaction with different nonce".to_string()
        ));
    }

    // New transaction must have at least 10% higher gas price
    let min_required = existing.gas_price + (existing.gas_price / U256::from(10));
    if new.gas_price < min_required {
        return Err(PoolError::ReplacementUnderpriced {
            required: min_required.as_u128(),
            got: new.gas_price.as_u128(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use merklith_types::{TransactionType, U256};

    fn create_test_tx() -> Transaction {
        Transaction {
            tx_type: TransactionType::Legacy,
            nonce: 0,
            gas_price: U256::from(10_000_000_000u64), // 10 gwei
            gas_limit: 100_000,
            to: Some(Address::from_bytes([1u8; 20])),
            value: U256::from(1000u64),
            data: vec![1, 2, 3],
            v: 0,
            r: U256::ZERO,
            s: U256::ZERO,
            chain_id: Some(1),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: None,
        }
    }

    #[test]
    fn test_validate_size() {
        let tx = create_test_tx();
        let config = ValidationConfig::default();

        // Should pass with default limits
        assert!(validate_size(&tx, &config).is_ok());

        // Should fail with very small limit
        let small_config = ValidationConfig {
            max_tx_size: 10,
            ..Default::default()
        };
        assert!(validate_size(&tx, &small_config).is_err());
    }

    #[test]
    fn test_validate_gas_limit() {
        let mut tx = create_test_tx();
        let config = ValidationConfig::default();

        // Valid gas limit
        assert!(validate_gas_limit(&tx, &config).is_ok());

        // Zero gas limit
        tx.gas_limit = 0;
        assert!(validate_gas_limit(&tx, &config).is_err());

        // Too high gas limit
        tx.gas_limit = 100_000_000;
        assert!(validate_gas_limit(&tx, &config).is_err());
    }

    #[test]
    fn test_validate_gas_price() {
        let mut tx = create_test_tx();
        let config = ValidationConfig::default();
        let context = ValidationContext::new(
            &AccountState::new(),
            100,
            1000,
            U256::from(1_000_000_000u64),
            &ChainConfig::default(),
        );

        // Valid gas price (10 gwei > 1 gwei min)
        assert!(validate_gas_price(&tx, &config, &context).is_ok());

        // Too low gas price
        tx.gas_price = U256::from(100);
        assert!(validate_gas_price(&tx, &config, &context).is_err());
    }

    #[test]
    fn test_validate_nonce() {
        // Nonce matches expected
        assert!(validate_nonce(&create_test_tx(), 0).is_ok());

        // Nonce too low
        let result = validate_nonce(&create_test_tx(), 5);
        assert!(matches!(result, Err(PoolError::NonceTooLow { .. })));

        // Nonce too high
        let mut tx = create_test_tx();
        tx.nonce = 2000;
        let result = validate_nonce(&tx, 0);
        assert!(matches!(result, Err(PoolError::NonceTooHigh { .. })));
    }

    #[test]
    fn test_calculate_intrinsic_gas() {
        let tx = create_test_tx();
        let gas = calculate_intrinsic_gas(&tx);

        // Base 21_000 + 3 bytes * 16 = 21_048
        assert_eq!(gas, 21_048);
    }

    #[test]
    fn test_can_replace() {
        let existing = create_test_tx();
        let mut new = create_test_tx();

        // Same gas price - cannot replace
        assert!(can_replace(&existing, &new).is_err());

        // 5% higher - still cannot replace (need 10%)
        new.gas_price = existing.gas_price + (existing.gas_price / U256::from(20));
        assert!(can_replace(&existing, &new).is_err());

        // 10% higher - can replace
        new.gas_price = existing.gas_price + (existing.gas_price / U256::from(10));
        assert!(can_replace(&existing, &new).is_ok());
    }
}
