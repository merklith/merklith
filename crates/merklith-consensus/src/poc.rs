//! Proof of Contribution (PoC) scoring mechanism.
//!
//! The PoC score combines stake, contribution (transactions, uptime, code),
//! and age (anti-Sybil) to determine validator selection probability.

use merklith_types::{Address, U256};

/// Configuration for PoC score calculation.
#[derive(Debug, Clone, Copy)]
pub struct PocConfig {
    /// Stake weight in score calculation (0.0 to 1.0)
    pub stake_weight: f64,
    /// Contribution weight in score calculation (0.0 to 1.0)
    pub contribution_weight: f64,
    /// Age weight in score calculation (0.0 to 1.0)
    pub age_weight: f64,
    /// Minimum stake required to be a validator
    pub min_stake: U256,
    /// Maximum effective stake (beyond this doesn't increase score)
    pub max_effective_stake: U256,
    /// Epochs before validator is considered "aged"
    pub age_threshold: u64,
    /// Maximum score multiplier from contribution
    pub max_contribution_multiplier: f64,
}

impl Default for PocConfig {
    fn default() -> Self {
        Self {
            stake_weight: 0.5,
            contribution_weight: 0.3,
            age_weight: 0.2,
            min_stake: U256::from(32_000_000_000_000_000_000u128), // 32 MERK in wei
            max_effective_stake: U256::from(320_000_000_000_000_000_000u128), // 320 MERK
            age_threshold: 10,
            max_contribution_multiplier: 2.0,
        }
    }
}

/// Contribution metrics for a validator.
#[derive(Debug, Clone, Default)]
pub struct ContributionMetrics {
    /// Number of transactions processed
    pub tx_count: u64,
    /// Total gas provided
    pub gas_provided: u64,
    /// Blocks proposed
    pub blocks_proposed: u64,
    /// Attestations included
    pub attestations: u64,
    /// Contracts deployed
    pub contracts_deployed: u64,
    /// Uptime percentage (0-10000, where 10000 = 100%)
    pub uptime_bps: u16,
    /// Software version (higher is better)
    pub software_version: u32,
    /// Epochs since joining
    pub epochs_active: u64,
}

impl ContributionMetrics {
    /// Create new metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set transaction count.
    pub fn with_tx_count(mut self, count: u64) -> Self {
        self.tx_count = count;
        self
    }

    /// Set gas provided.
    pub fn with_gas_provided(mut self, gas: u64) -> Self {
        self.gas_provided = gas;
        self
    }

    /// Set blocks proposed.
    pub fn with_blocks_proposed(mut self, blocks: u64) -> Self {
        self.blocks_proposed = blocks;
        self
    }

    /// Set uptime (percentage as 0-100).
    pub fn with_uptime(mut self, percentage: f64) -> Self {
        self.uptime_bps = (percentage * 100.0) as u16;
        self
    }

    /// Calculate raw contribution score.
    /// 
    /// Higher is better. Based on:
    /// - Transaction throughput (40%)
    /// - Gas efficiency (20%)
    /// - Block production (20%)
    /// - Uptime (20%)
    pub fn calculate_score(&self) -> f64 {
        // Normalize each component (0-1 scale)
        let tx_score = (self.tx_count as f64 / 1_000_000.0).min(1.0) * 0.4;
        let gas_score = (self.gas_provided as f64 / 1_000_000_000_000.0).min(1.0) * 0.2;
        let block_score = (self.blocks_proposed as f64 / 10000.0).min(1.0) * 0.2;
        let uptime_score = (self.uptime_bps as f64 / 10000.0) * 0.2;

        tx_score + gas_score + block_score + uptime_score
    }
}

/// Calculate the stake weight for a validator.
/// 
/// Uses a square root function to prevent stake concentration.
/// Returns a value between 0.0 and 1.0.
pub fn calculate_stake_weight(stake: U256, max_effective_stake: U256) -> f64 {
    let stake_f64 = stake.as_u128() as f64;
    let max_f64 = max_effective_stake.as_u128() as f64;
    
    let effective_stake = stake_f64.min(max_f64);
    let sqrt_stake = effective_stake.sqrt();
    let sqrt_max = max_f64.sqrt();
    
    sqrt_stake / sqrt_max
}

/// Calculate the age weight for a validator.
/// 
/// Linear increase from 0.0 to 1.0 over `age_threshold` epochs.
pub fn calculate_age_weight(epochs_active: u64, age_threshold: u64) -> f64 {
    if age_threshold == 0 {
        return 1.0;
    }
    
    (epochs_active as f64 / age_threshold as f64).min(1.0)
}

/// Calculate the PoC (Proof of Contribution) score.
/// 
/// Combines stake, contribution, and age into a single score.
/// Returns a value between 0.0 and 1.0.
/// 
/// Formula: score = stake_weight * stake_component + 
///                  contribution_weight * contribution_component +
///                  age_weight * age_component
pub fn calculate_poc_score(
    stake: U256,
    contribution: &ContributionMetrics,
    epochs_active: u64,
    config: &PocConfig,
) -> Result<f64, super::error::ConsensusError> {
    // Check minimum stake
    if stake < config.min_stake {
        return Err(super::error::ConsensusError::InsufficientStake(
            format!("Stake {} below minimum {}", stake, config.min_stake)
        ));
    }

    // Calculate components
    let stake_component = calculate_stake_weight(stake, config.max_effective_stake);
    let contribution_component = contribution.calculate_score();
    let age_component = calculate_age_weight(epochs_active, config.age_threshold);

    // Combine weighted components
    let score = config.stake_weight * stake_component +
                config.contribution_weight * contribution_component +
                config.age_weight * age_component;

    Ok(score.min(1.0))
}

/// Validator information for PoC calculations.
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub address: Address,
    pub stake: U256,
    pub contribution: ContributionMetrics,
    pub epochs_active: u64,
}

impl ValidatorInfo {
    /// Create new validator info.
    pub fn new(address: Address, stake: U256) -> Self {
        Self {
            address,
            stake,
            contribution: ContributionMetrics::new(),
            epochs_active: 0,
        }
    }

    /// Set contribution metrics.
    pub fn with_contribution(mut self, contribution: ContributionMetrics) -> Self {
        self.contribution = contribution;
        self
    }

    /// Set epochs active.
    pub fn with_epochs_active(mut self, epochs: u64) -> Self {
        self.epochs_active = epochs;
        self
    }

    /// Calculate PoC score.
    pub fn calculate_score(&self, config: &PocConfig) -> Result<f64, super::error::ConsensusError> {
        calculate_poc_score(self.stake, &self.contribution, self.epochs_active, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contribution_metrics() {
        let metrics = ContributionMetrics::new()
            .with_tx_count(500_000)
            .with_gas_provided(100_000_000_000)
            .with_blocks_proposed(1000)
            .with_uptime(99.5);

        let score = metrics.calculate_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_stake_weight_calculation() {
        let max = U256::from(1_000_000u128);
        
        // Half of max stake
        let half = U256::from(500_000u128);
        let weight_half = calculate_stake_weight(half, max);
        assert!(weight_half > 0.0);
        assert!(weight_half < 1.0);
        
        // Max stake
        let weight_max = calculate_stake_weight(max, max);
        assert!((weight_max - 1.0).abs() < 0.001);
        
        // Beyond max (should be capped)
        let double = U256::from(2_000_000u128);
        let weight_double = calculate_stake_weight(double, max);
        assert!((weight_double - weight_max).abs() < 0.001);
    }

    #[test]
    fn test_age_weight() {
        let threshold = 10;
        
        // New validator
        assert_eq!(calculate_age_weight(0, threshold), 0.0);
        
        // Halfway there
        assert_eq!(calculate_age_weight(5, threshold), 0.5);
        
        // Aged validator
        assert_eq!(calculate_age_weight(10, threshold), 1.0);
        
        // Very old (should be capped)
        assert_eq!(calculate_age_weight(100, threshold), 1.0);
    }

    #[test]
    fn test_poc_score() {
        let config = PocConfig::default();
        let stake = U256::from(100_000_000_000_000_000_000u128); // 100 MERK
        let contribution = ContributionMetrics::new()
            .with_tx_count(100_000)
            .with_uptime(95.0);
        
        let score = calculate_poc_score(stake, &contribution, 20, &config).unwrap();
        
        // Score should be reasonable
        assert!(score > 0.0);
        assert!(score <= 1.0);
        
        // Higher contribution = higher score
        let better_contribution = ContributionMetrics::new()
            .with_tx_count(500_000)
            .with_uptime(99.0);
        let better_score = calculate_poc_score(stake, &better_contribution, 20, &config).unwrap();
        
        assert!(better_score > score);
    }

    #[test]
    fn test_poc_score_insufficient_stake() {
        let config = PocConfig::default();
        let low_stake = U256::from(1_000_000u128); // Way below minimum
        let contribution = ContributionMetrics::new();
        
        let result = calculate_poc_score(low_stake, &contribution, 20, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validator_info() {
        let validator = ValidatorInfo::new(Address::ZERO, U256::from(1_000_000u128))
            .with_epochs_active(15)
            .with_contribution(ContributionMetrics::new().with_uptime(98.0));
        
        assert_eq!(validator.epochs_active, 15);
        assert_eq!(validator.contribution.uptime_bps, 9800);
    }

    #[test]
    fn test_poc_config_default() {
        let config = PocConfig::default();
        
        // Weights should sum to 1.0
        let total = config.stake_weight + config.contribution_weight + config.age_weight;
        assert!((total - 1.0).abs() < 0.001);
        
        assert!(config.min_stake > U256::ZERO);
        assert!(config.max_effective_stake > config.min_stake);
    }
}
