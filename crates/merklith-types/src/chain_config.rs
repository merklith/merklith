use crate::u256::U256;

/// Chain-level configuration parameters.
/// These can be changed via governance (AIP).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainConfig {
    pub chain_id: u64,

    // Block parameters
    pub block_time_ms: u64,              // 2000 (2 seconds)
    pub gas_limit: u64,                   // 30_000_000
    pub gas_target: u64,                  // 15_000_000
    pub base_fee_max_change_pct: u8,      // 5 (5% per block)
    pub max_extra_data_bytes: usize,      // 32

    // Epoch & consensus
    pub epoch_length: u64,                // 1000 blocks
    pub checkpoint_interval: u64,         // 100 blocks
    pub committee_size: u32,              // 100
    pub proposer_timeout_ms: u64,         // 2000
    pub attestation_threshold_pct: u8,    // 67 (2/3 + 1)

    // Staking
    pub min_stake: U256,                  // 1000 MERK
    pub max_effective_stake: U256,        // 100_000 MERK
    pub unbonding_period_blocks: u64,     // ~21 days worth of blocks
    pub stake_log_base: u64,             // 1000 MERK (BASE_UNIT for logarithmic weight)

    // Fee market
    pub min_base_fee: U256,              // 1 Spark
    pub max_base_fee: U256,              // 10_000 Spark per gas
    pub max_priority_fee_multiplier: u8, // 2 (max 2x base_fee)
    pub fee_guarantee_blocks: u64,       // 10 blocks validity

    // Governance
    pub aip_deposit: U256,               // 10_000 MERK
    pub agp_deposit: U256,               // 1_000 MERK
    pub aep_deposit: U256,               // 100_000 MERK
    pub treasury_share_pct: u8,          // 20 (20% of block rewards)
    pub max_delegation_depth: u8,        // 5

    // Slashing
    pub double_sign_slash_pct: u8,       // 100
    pub downtime_slash_pct_per_day: u8,  // 1
    pub invalid_block_slash_pct: u8,     // 10
    pub censoring_slash_pct: u8,         // 50
    pub collusion_slash_pct: u8,         // 30
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self::mainnet()
    }
}

impl ChainConfig {
    /// Mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            chain_id: 1,
            block_time_ms: 2000,
            gas_limit: 30_000_000,
            gas_target: 15_000_000,
            base_fee_max_change_pct: 5,
            max_extra_data_bytes: 32,
            epoch_length: 1000,
            checkpoint_interval: 100,
            committee_size: 100,
            proposer_timeout_ms: 2000,
            attestation_threshold_pct: 67,
            min_stake: U256::from(1_000u64) * U256::MERK,
            max_effective_stake: U256::from(100_000u64) * U256::MERK,
            unbonding_period_blocks: 907200, // ~21 days at 2s blocks
            stake_log_base: 1_000,
            min_base_fee: U256::ONE,
            max_base_fee: U256::from(10_000u64),
            max_priority_fee_multiplier: 2,
            fee_guarantee_blocks: 10,
            aip_deposit: U256::from(10_000u64) * U256::MERK,
            agp_deposit: U256::from(1_000u64) * U256::MERK,
            aep_deposit: U256::from(100_000u64) * U256::MERK,
            treasury_share_pct: 20,
            max_delegation_depth: 5,
            double_sign_slash_pct: 100,
            downtime_slash_pct_per_day: 1,
            invalid_block_slash_pct: 10,
            censoring_slash_pct: 50,
            collusion_slash_pct: 30,
        }
    }

    /// Testnet "Forge" configuration
    pub fn testnet() -> Self {
        let mut config = Self::mainnet();
        config.chain_id = 2;
        config.min_stake = U256::from(1u64) * U256::MERK; // 1 MERK for testing
        config.epoch_length = 100; // Faster epochs for testing
        config.unbonding_period_blocks = 1000; // Faster unbonding
        config
    }

    /// Local development configuration
    pub fn devnet() -> Self {
        let mut config = Self::mainnet();
        config.chain_id = 1337;
        config.block_time_ms = 0; // Instant mining
        config.min_stake = U256::ZERO; // No minimum for dev
        config.epoch_length = 10; // Very fast epochs
        config.committee_size = 4; // Small committee
        config.unbonding_period_blocks = 10;
        config
    }

    /// Get block time as seconds
    pub fn block_time_seconds(&self) -> u64 {
        self.block_time_ms / 1000
    }

    /// Get the attestation threshold count for a given committee size
    pub fn attestation_threshold(&self, committee_size: u32) -> u32 {
        (committee_size * self.attestation_threshold_pct as u32 + 99) / 100
    }

    /// Check if chain ID is valid
    pub fn is_valid_chain_id(&self) -> bool {
        self.chain_id != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_config_mainnet() {
        let config = ChainConfig::mainnet();
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.block_time_ms, 2000);
        assert_eq!(config.gas_limit, 30_000_000);
        assert_eq!(config.committee_size, 100);
    }

    #[test]
    fn test_chain_config_testnet() {
        let config = ChainConfig::testnet();
        assert_eq!(config.chain_id, 2);
        assert_eq!(config.min_stake, U256::from(1u64) * U256::MERK);
        assert_eq!(config.epoch_length, 100);
    }

    #[test]
    fn test_chain_config_devnet() {
        let config = ChainConfig::devnet();
        assert_eq!(config.chain_id, 1337);
        assert_eq!(config.block_time_ms, 0);
        assert_eq!(config.committee_size, 4);
    }

    #[test]
    fn test_attestation_threshold() {
        let config = ChainConfig::mainnet();
        assert_eq!(config.attestation_threshold(100), 67);
        assert_eq!(config.attestation_threshold(10), 7);
    }

    #[test]
    fn test_block_time_seconds() {
        let config = ChainConfig::mainnet();
        assert_eq!(config.block_time_seconds(), 2);
    }

    #[test]
    fn test_valid_chain_id() {
        let mainnet = ChainConfig::mainnet();
        assert!(mainnet.is_valid_chain_id());

        let mut invalid = ChainConfig::mainnet();
        invalid.chain_id = 0;
        assert!(!invalid.is_valid_chain_id());
    }

    #[test]
    fn test_stake_values() {
        let config = ChainConfig::mainnet();
        assert_eq!(config.min_stake, U256::from(1_000u64) * U256::MERK);
        assert_eq!(config.max_effective_stake, U256::from(100_000u64) * U256::MERK);
    }

    #[test]
    fn test_governance_deposits() {
        let config = ChainConfig::mainnet();
        assert_eq!(config.aip_deposit, U256::from(10_000u64) * U256::MERK);
        assert_eq!(config.agp_deposit, U256::from(1_000u64) * U256::MERK);
        assert_eq!(config.aep_deposit, U256::from(100_000u64) * U256::MERK);
    }
}
