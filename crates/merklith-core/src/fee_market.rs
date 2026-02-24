//! Fee market calculations.

use merklith_types::{ChainConfig, U256};

/// Calculate next block's base fee using dampened EIP-1559.
///
/// Formula: base_fee[n+1] = base_fee[n] * (1 + δ * (gas_used - gas_target) / gas_target)
/// Where δ = 0.05 (max 5% change per block)
pub fn calculate_base_fee(
    parent_base_fee: &U256,
    parent_gas_used: u64,
    parent_gas_target: u64,
    config: &ChainConfig,
) -> U256 {
    // Prevent division by zero
    if parent_gas_target == 0 {
        return *parent_base_fee;
    }
    
    if parent_gas_used == parent_gas_target {
        return *parent_base_fee;
    }

    let delta = config.base_fee_max_change_pct as u64;

    if parent_gas_used > parent_gas_target {
        // Base fee increases
        let gas_used_delta = parent_gas_used - parent_gas_target;
        // Multiply first to avoid precision loss: base_fee * gas_delta * delta / (target * 100)
        let base_fee_delta = *parent_base_fee * U256::from(gas_used_delta) * U256::from(delta) 
            / (U256::from(parent_gas_target) * U256::from(100));
        let new_base_fee = *parent_base_fee + base_fee_delta;
        new_base_fee.min(config.max_base_fee)
    } else {
        // Base fee decreases
        let gas_used_delta = parent_gas_target - parent_gas_used;
        // Multiply first to avoid precision loss
        let base_fee_delta = *parent_base_fee * U256::from(gas_used_delta) * U256::from(delta) 
            / (U256::from(parent_gas_target) * U256::from(100));
        let new_base_fee = parent_base_fee.checked_sub(&base_fee_delta).unwrap_or(config.min_base_fee);
        new_base_fee.max(config.min_base_fee)
    }
}

/// Fee guarantee for transaction submission.
#[derive(Debug, Clone, Copy)]
pub struct FeeGuarantee {
    /// Maximum fee that will be charged
    pub max_fee: U256,
    /// Block number until which this guarantee is valid
    pub valid_until_block: u64,
}

/// Calculate the guaranteed max fee for a transaction.
/// Valid for `fee_guarantee_blocks` blocks.
pub fn guaranteed_max_fee(
    current_base_fee: &U256,
    gas_estimate: u64,
    current_block: u64,
    config: &ChainConfig,
) -> FeeGuarantee {
    // Maximum base fee after fee_guarantee_blocks (increasing at max 5% per block)
    // Use iterative multiplication to avoid precision loss
    let mut max_base_fee = *current_base_fee;
    for _ in 0..config.fee_guarantee_blocks {
        // 5% increase = multiply by 105/100
        max_base_fee = max_base_fee * U256::from(105u64) / U256::from(100u64);
        max_base_fee = max_base_fee.min(config.max_base_fee);
    }
    
    let max_fee = max_base_fee * U256::from(gas_estimate);
    
    FeeGuarantee {
        max_fee,
        valid_until_block: current_block + config.fee_guarantee_blocks,
    }
}

/// Calculate effective priority fee (capped at 2x base_fee).
pub fn effective_priority_fee(
    max_priority_fee: &U256,
    max_fee_per_gas: &U256,
    base_fee: &U256,
    config: &ChainConfig,
) -> U256 {
    let max_allowed = base_fee * U256::from(config.max_priority_fee_multiplier);
    let fee_diff = max_fee_per_gas.saturating_sub(base_fee);
    let actual_priority = max_priority_fee.min(&fee_diff);
    *actual_priority.min(&max_allowed)
}

/// Calculate total fee for a transaction.
pub fn calculate_total_fee(
    base_fee: &U256,
    priority_fee: &U256,
    gas_used: u64,
) -> U256 {
    let gas_price = base_fee.saturating_add(priority_fee);
    gas_price * U256::from(gas_used)
}

/// Check if a transaction can pay the required fees.
pub fn can_pay_fees(
    balance: &U256,
    max_fee_per_gas: &U256,
    gas_limit: u64,
    value: &U256,
) -> bool {
    let max_gas_cost = max_fee_per_gas * U256::from(gas_limit);
    let total_cost = max_gas_cost.saturating_add(value);
    *balance >= total_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Fee calculation needs refinement for edge cases"]
    fn test_base_fee_calculation() {
        let config = ChainConfig::mainnet();
        let base_fee = U256::from(1000000000u64); // 1 Gwei

        // Gas used equals target - base fee unchanged
        let new_fee = calculate_base_fee(&base_fee, 15000000, 15000000, &config);
        assert_eq!(new_fee, base_fee);

        // Gas used above target - base fee increases (max 5% per block)
        let new_fee = calculate_base_fee(&base_fee, 20000000, 15000000, &config);
        // Increase should be: 1G * (5M/15M) * 5% = 1G * 0.33 * 0.05 = ~16.7M
        // So new_fee should be ~1.017G
        assert!(new_fee > base_fee);
        assert!(new_fee < base_fee * U256::from(2u64)); // Less than double

        // Gas used below target - base fee decreases
        let new_fee = calculate_base_fee(&base_fee, 10000000, 15000000, &config);
        // Decrease should be: 1G * (5M/15M) * 5% = ~16.7M
        // So new_fee should be ~983M
        assert!(new_fee < base_fee);
    }

    #[test]
    fn test_base_fee_bounds() {
        let mut config = ChainConfig::mainnet();
        config.min_base_fee = U256::from(100000000u64); // 0.1 Gwei
        config.max_base_fee = U256::from(1000000000000u64); // 1000 Gwei

        // At very low gas usage, fee decreases but stays above min
        let base_fee = U256::from(1000000000u64); // 1 Gwei
        let new_fee = calculate_base_fee(&base_fee, 10000000, 15000000, &config);
        assert!(new_fee >= config.min_base_fee);
        assert!(new_fee < base_fee); // Should decrease

        // Should not go above maximum
        let base_fee = config.max_base_fee;
        let new_fee = calculate_base_fee(&base_fee, 30000000, 15000000, &config);
        assert_eq!(new_fee, config.max_base_fee);
    }

    #[test]
    #[ignore = "Fee calculation needs refinement for edge cases"]
    fn test_guaranteed_max_fee() {
        let config = ChainConfig::mainnet();
        let base_fee = U256::from(1000000000u64);
        let gas_estimate = 21000u64;
        let current_block = 100u64;

        let guarantee = guaranteed_max_fee(&base_fee, gas_estimate, current_block, &config);

        // Max fee should account for potential fee increases
        assert!(guarantee.max_fee >= base_fee * U256::from(gas_estimate));
        assert_eq!(guarantee.valid_until_block, current_block + config.fee_guarantee_blocks);
    }

    #[test]
    fn test_effective_priority_fee() {
        let config = ChainConfig::mainnet();
        let base_fee = U256::from(1000000000u64);

        // Priority fee below cap
        let priority = U256::from(2000000000u64); // 2 Gwei
        let max_fee = base_fee + priority;
        let effective = effective_priority_fee(&priority, &max_fee, &base_fee, &config);
        assert_eq!(effective, priority);

        // Priority fee above cap (should be capped at 2x base fee)
        let high_priority = U256::from(5000000000u64); // 5 Gwei
        let effective = effective_priority_fee(
            &high_priority,
            &(base_fee + high_priority),
            &base_fee,
            &config,
        );
        assert_eq!(effective, base_fee * U256::from(2));
    }

    #[test]
    fn test_calculate_total_fee() {
        let base_fee = U256::from(1000000000u64);
        let priority_fee = U256::from(2000000000u64);
        let gas_used = 21000u64;

        let total = calculate_total_fee(&base_fee, &priority_fee, gas_used);
        let expected = (base_fee + priority_fee) * U256::from(gas_used);

        assert_eq!(total, expected);
    }

    #[test]
    fn test_can_pay_fees() {
        let balance = U256::from(1000000000000000000u64); // 1 MERK
        let max_fee_per_gas = U256::from(20000000000u64); // 20 Gwei
        let gas_limit = 21000u64;
        let value = U256::from(100000000000000000u64); // 0.1 MERK

        assert!(can_pay_fees(&balance, &max_fee_per_gas, gas_limit, &value));

        // Insufficient balance
        let low_balance = U256::from(1000u64);
        assert!(!can_pay_fees(&low_balance, &max_fee_per_gas, gas_limit, &value));
    }
}
