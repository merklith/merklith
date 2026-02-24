//! Integration tests for Merklith blockchain
//!
//! End-to-end tests that verify the full stack works together.

#[cfg(test)]
mod e2e_tests {
    use merklith_types::{Address, Transaction, TransactionType, U256};
    use merklith_crypto::ed25519::Ed25519Keypair;
    use merklith_core::state::AccountState;
    use merklith_storage::state_db::StateDB;
    use merklith_txpool::pool::TransactionPool;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// Setup test environment with fresh database
    fn setup_test_env() -> (TempDir, Arc<StateDB>, AccountState) {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(StateDB::new(temp_dir.path()).unwrap());
        let state = AccountState::new();
        (temp_dir, db, state)
    }

    #[test]
    fn test_end_to_end_transaction_flow() {
        // Setup
        let (_temp, db, mut state) = setup_test_env();
        
        // Create sender with balance
        let sender_keypair = Ed25519Keypair::generate();
        let sender = sender_keypair.address();
        state.create_account(sender, U256::from(1_000_000_000_000_000_000u128)); // 1 MERK
        
        // Create recipient
        let recipient = Address::from_bytes([1u8; 20]);
        
        // Create transaction
        let mut tx = Transaction {
            tx_type: TransactionType::Legacy,
            nonce: 0,
            gas_price: U256::from(1_000_000_000u64), // 1 gwei
            gas_limit: 21_000,
            to: Some(recipient),
            value: U256::from(100_000_000_000_000_000u128), // 0.1 MERK
            data: vec![],
            v: 0,
            r: U256::ZERO,
            s: U256::ZERO,
            chain_id: Some(1),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: None,
        };
        
        // Sign transaction
        let signature = sender_keypair.sign(&tx.hash().as_bytes()).unwrap();
        tx.v = signature.v;
        tx.r = signature.r;
        tx.s = signature.s;
        
        // Verify sender
        assert_eq!(tx.sender(), Some(sender));
        
        // Add to mempool
        let mut pool = TransactionPool::default();
        let validation_result = pool.add_transaction(tx, &merklith_txpool::validation::ValidationContext::new(
            &state, 1, 1000, U256::from(1_000_000_000u64), &merklith_types::ChainConfig::default()
        ));
        
        assert!(validation_result.is_ok());
        assert_eq!(pool.stats().total_count, 1);
        
        // Get pending transactions
        let pending = pool.get_pending(10);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].value, U256::from(100_000_000_000_000_000u128));
    }

    #[test]
    fn test_block_production_flow() {
        use merklith_core::block_builder::BlockBuilder;
        use merklith_types::BlockHeader;
        
        let (_temp, _db, mut state) = setup_test_env();
        
        // Create validator
        let validator_key = Ed25519Keypair::generate();
        let validator = validator_key.address();
        
        // Create block builder
        let mut builder = BlockBuilder::new(1, [0u8; 32], validator);
        
        // Add some transactions
        for i in 0..5 {
            let sender_key = Ed25519Keypair::generate();
            let sender = sender_key.address();
            state.create_account(sender, U256::from(1_000_000_000_000_000_000u128));
            
            let mut tx = Transaction {
                tx_type: TransactionType::Legacy,
                nonce: i as u64,
                gas_price: U256::from(1_000_000_000u64),
                gas_limit: 21_000,
                to: Some(Address::from_bytes([(i + 1) as u8; 20])),
                value: U256::from(100_000_000_000_000_000u128),
                data: vec![],
                v: 0,
                r: U256::ZERO,
                s: U256::ZERO,
                chain_id: Some(1),
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                access_list: None,
            };
            
            let sig = sender_key.sign(&tx.hash().as_bytes()).unwrap();
            tx.v = sig.v;
            tx.r = sig.r;
            tx.s = sig.s;
            
            builder.add_transaction(tx, 21_000, U256::from(1_000_000_000u64)).unwrap();
        }
        
        // Build block
        let block = builder.build().unwrap();
        
        assert_eq!(block.header.number, 1);
        assert_eq!(block.transactions.len(), 5);
        assert_eq!(block.header.validator, validator_key.public_bytes());
    }

    #[test]
    fn test_consensus_committee_selection() {
        use merklith_consensus::poc::{calculate_poc_score, ContributionMetrics, ValidatorInfo, PocConfig};
        use merklith_consensus::committee::{select_committee, CommitteeConfig};
        
        // Create validators
        let mut validators = vec![];
        for i in 1..=10 {
            let addr = Address::from_bytes([i as u8; 20]);
            let contribution = ContributionMetrics::new()
                .with_tx_count(100_000 * i as u64)
                .with_uptime(95.0 + i as f64);
            
            validators.push(
                ValidatorInfo::new(addr, U256::from(1_000_000_000_000_000_000u128 * i as u128))
                    .with_epochs_active(20)
                    .with_contribution(contribution)
            );
        }
        
        let poc_config = PocConfig::default();
        let committee_config = CommitteeConfig {
            target_size: 5,
            min_size: 3,
            max_size: 7,
            ..Default::default()
        };
        
        let seed = [1u8; 32];
        let committee = select_committee(
            1,
            &validators,
            seed,
            &poc_config,
            &committee_config,
        ).unwrap();
        
        assert!(committee.size() >= committee_config.min_size);
        assert!(committee.size() <= committee_config.target_size);
        
        // Check committee has valid members
        for member in &committee.members {
            assert!(member.poc_score > 0.0);
        }
    }

    #[test]
    fn test_governance_proposal_lifecycle() {
        use merklith_consensus::validator::{ValidatorSet, Validator, ValidatorStatus};
        use merklith_governance::proposal::{ProposalRegistry, Proposal, ProposalType, ProposalStatus, VoteSupport};
        
        // Setup validators
        let mut validators = ValidatorSet::new();
        let proposer = Address::from_bytes([1u8; 20]);
        
        validators.register(
            proposer,
            [0u8; 32],
            U256::from(1_000_000_000_000_000_000u128),
        ).unwrap().activate(0);
        
        // Create proposal
        let mut registry = ProposalRegistry::new();
        let proposal_id = registry.create_proposal(
            ProposalType::ParameterChange,
            proposer,
            "Increase block size".to_string(),
            "Increase max block size to 10MB".to_string(),
            100,
            U256::from(1_000_000_000u128), // total supply
        );
        
        // Get proposal
        let proposal = registry.get(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Pending);
        
        // Start voting
        let proposal = registry.get_mut(proposal_id).unwrap();
        proposal.start_voting(100).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Active);
        
        // Cast vote
        let proposal = registry.get_mut(proposal_id).unwrap();
        proposal.cast_vote(proposer, VoteSupport::For, U256::from(1000u128)).unwrap();
        
        assert_eq!(proposal.for_votes, U256::from(1000u128));
        assert!(proposal.has_voted(&proposer));
    }

    #[test]
    fn test_staking_and_rewards() {
        use system_contracts::staking::StakingContract;
        
        let mut staking = StakingContract::new();
        let validator = Address::from_bytes([1u8; 20]);
        let delegator = Address::from_bytes([2u8; 20]);
        
        // Register validator
        let stake = U256::from(100_000_000_000_000_000_000u128); // 100 MERK
        staking.register_validator(validator, stake, 1000).unwrap();
        
        // Delegate
        let delegation = U256::from(10_000_000_000_000_000_000u128); // 10 MERK
        staking.delegate(delegator, validator, delegation).unwrap();
        
        assert_eq!(staking.get_delegation(&delegator, &validator), delegation);
        
        let v = staking.get_validator(&validator).unwrap();
        assert_eq!(v.delegated, delegation);
        
        // Calculate rewards
        let rewards = staking.calculate_rewards(validator, 365).unwrap();
        assert!(rewards > U256::ZERO);
    }

    #[test]
    fn test_bridge_cross_chain_transfer() {
        use system_contracts::bridge::BridgeContract;
        
        let mut bridge = BridgeContract::new(2); // 2 signatures required
        
        // Setup
        bridge.add_chain(1, [0u8; 20]);
        
        let validator1 = Address::from_bytes([1u8; 20]);
        let validator2 = Address::from_bytes([2u8; 20]);
        bridge.add_validator(validator1);
        bridge.add_validator(validator2);
        
        // Initiate transfer
        let sender = Address::ZERO;
        let recipient = [3u8; 20];
        let amount = U256::from(1000u128);
        
        let transfer_id = bridge.initiate_transfer(sender, recipient, amount, 1).unwrap();
        
        // Sign
        bridge.sign_transfer(transfer_id, validator1, ([0u8; 32], [0u8; 32])).unwrap();
        bridge.sign_transfer(transfer_id, validator2, ([0u8; 32], [0u8; 32])).unwrap();
        
        // Complete
        bridge.complete_transfer(transfer_id).unwrap();
        
        assert!(bridge.is_completed(&transfer_id));
    }

    #[test]
    fn test_treasury_spending() {
        use system_contracts::treasury::TreasuryContract;
        
        let mut treasury = TreasuryContract::new();
        let authorized = Address::from_bytes([1u8; 20]);
        
        // Deposit
        treasury.deposit(U256::from(1_000_000u128));
        assert_eq!(treasury.balance(), U256::from(1_000_000u128));
        
        // Authorize spender
        treasury.authorize(authorized, U256::from(100_000u128));
        
        // Propose spending
        treasury.propose_spending(
            1,
            Address::from_bytes([2u8; 20]),
            U256::from(50_000u128),
            "Development grant".to_string(),
        ).unwrap();
        
        // Approve
        treasury.approve_spending(1, authorized).unwrap();
        
        // Execute
        treasury.execute_spending(1).unwrap();
        
        assert_eq!(treasury.balance(), U256::from(950_000u128));
    }
}
