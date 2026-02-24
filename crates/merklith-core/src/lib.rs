//! Merklith Core - Core blockchain logic.

pub mod chain;
pub mod error;
pub mod fee_market;
pub mod block_builder;
pub mod state;
pub mod state_machine;
pub mod high_availability;
pub mod performance;

pub use chain::Chain;
pub use error::CoreError;
pub use fee_market::{calculate_base_fee, guaranteed_max_fee, effective_priority_fee, FeeGuarantee};
pub use block_builder::{BlockBuilder, BuilderError};
pub use state_machine::{State, Account};
pub use high_availability::{
    HighAvailabilityManager, HealthMonitor, HealthStatus, HealthCheck,
    RecoverySystem, ClusterManager
};
pub use performance::{
    OptimizationManager, PerformanceMetrics, BlockCache, TransactionCache, StateCache,
    BufferPool, BatchProcessor, CacheStats
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_integration() {
        // Create a genesis block
        let genesis_header = merklith_types::BlockHeader::new(
            merklith_types::Hash::ZERO,
            0,
            0,
            30000000,
            merklith_types::Address::ZERO,
        );
        let genesis = merklith_types::Block::new(genesis_header, vec![]);

        // Create chain
        let chain = Chain::new(genesis.clone());
        assert_eq!(chain.head(), genesis.hash());

        // Test fee calculation
        let config = merklith_types::ChainConfig::mainnet();
        let base_fee = merklith_types::U256::from(1000000000u64);
        let new_fee = fee_market::calculate_base_fee(
            &base_fee, 15000000, 15000000, &config
        );
        assert_eq!(new_fee, base_fee);
    }
}
