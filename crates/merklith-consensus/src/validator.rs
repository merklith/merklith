//! Validator state management.
//!
//! Tracks validator registrations, stakes, and lifecycle.

use merklith_types::{Address, U256};
use crate::poc::{ContributionMetrics, ValidatorInfo};
use crate::error::ConsensusError;

/// Validator lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorStatus {
    /// Validator is pending activation (waiting in queue)
    Pending,
    /// Validator is active and can participate in consensus
    Active,
    /// Validator is exiting (initiated exit)
    Exiting,
    /// Validator has exited and funds are withdrawable
    Withdrawable,
    /// Validator was slashed and ejected
    Slashed,
}

impl ValidatorStatus {
    /// Check if validator is active.
    pub fn is_active(&self) -> bool {
        matches!(self, ValidatorStatus::Active)
    }

    /// Check if validator can be slashed.
    pub fn can_be_slashed(&self) -> bool {
        matches!(self, ValidatorStatus::Active | ValidatorStatus::Exiting)
    }
}

/// On-chain validator record.
#[derive(Debug, Clone)]
pub struct Validator {
    /// Validator address
    pub address: Address,
    /// Withdrawal credentials (where rewards go)
    pub withdrawal_credentials: [u8; 32],
    /// Current stake
    pub stake: U256,
    /// Current status
    pub status: ValidatorStatus,
    /// Epoch when activated
    pub activation_epoch: u64,
    /// Epoch when exited (if applicable)
    pub exit_epoch: Option<u64>,
    /// Epoch when withdrawable
    pub withdrawable_epoch: Option<u64>,
    /// Whether validator has been slashed
    pub slashed: bool,
    /// Contribution metrics
    pub contribution: ContributionMetrics,
    /// Current epoch (for tracking)
    pub current_epoch: u64,
}

impl Validator {
    /// Create a new validator.
    pub fn new(
        address: Address,
        withdrawal_credentials: [u8; 32],
        stake: U256,
    ) -> Self {
        Self {
            address,
            withdrawal_credentials,
            stake,
            status: ValidatorStatus::Pending,
            activation_epoch: 0,
            exit_epoch: None,
            withdrawable_epoch: None,
            slashed: false,
            contribution: ContributionMetrics::new(),
            current_epoch: 0,
        }
    }

    /// Activate the validator.
    pub fn activate(&mut self, epoch: u64) {
        self.status = ValidatorStatus::Active;
        self.activation_epoch = epoch;
        self.current_epoch = epoch;
    }

    /// Initiate exit.
    pub fn initiate_exit(&mut self, epoch: u64) {
        if self.status == ValidatorStatus::Active {
            self.status = ValidatorStatus::Exiting;
            self.exit_epoch = Some(epoch);
        }
    }

    /// Mark as withdrawable.
    pub fn make_withdrawable(&mut self, epoch: u64) {
        self.status = ValidatorStatus::Withdrawable;
        self.withdrawable_epoch = Some(epoch);
    }

    /// Slash the validator.
    pub fn slash(&mut self, epoch: u64) {
        self.slashed = true;
        self.status = ValidatorStatus::Slashed;
        self.exit_epoch = Some(epoch);
        self.withdrawable_epoch = Some(epoch + 8192); // ~36 days
    }

    /// Get epochs active (0 if not yet activated).
    pub fn epochs_active(&self) -> u64 {
        if self.status == ValidatorStatus::Pending {
            0
        } else {
            self.current_epoch.saturating_sub(self.activation_epoch)
        }
    }

    /// Update contribution metrics.
    pub fn update_contribution(&mut self, metrics: ContributionMetrics) {
        self.contribution = metrics;
    }

    /// Convert to ValidatorInfo for PoC calculations.
    pub fn to_info(&self) -> ValidatorInfo {
        ValidatorInfo {
            address: self.address,
            stake: self.stake,
            contribution: self.contribution.clone(),
            epochs_active: self.epochs_active(),
        }
    }

    /// Check if validator is eligible for committee selection.
    pub fn is_eligible(&self) -> bool {
        self.status.is_active() && !self.slashed
    }
}

/// Validator set managing all validators.
#[derive(Debug)]
pub struct ValidatorSet {
    /// All validators by address
    validators: std::collections::HashMap<Address, Validator>,
    /// Current epoch
    current_epoch: u64,
}

impl ValidatorSet {
    /// Create a new empty validator set.
    pub fn new() -> Self {
        Self {
            validators: std::collections::HashMap::new(),
            current_epoch: 0,
        }
    }

    /// Register a new validator.
    pub fn register(
        &mut self,
        address: Address,
        withdrawal_credentials: [u8; 32],
        stake: U256,
    ) -> Result<&mut Validator, ConsensusError> {
        if self.validators.contains_key(&address) {
            return Err(ConsensusError::ValidatorAlreadyExists(
                address.to_string()
            ));
        }

        let validator = Validator::new(address, withdrawal_credentials, stake);
        self.validators.insert(address, validator);
        
        Ok(self.validators.get_mut(&address).unwrap())
    }

    /// Get a validator.
    pub fn get(&self, address: &Address) -> Option<&Validator> {
        self.validators.get(address)
    }

    /// Get a validator mutably.
    pub fn get_mut(&mut self, address: &Address) -> Option<&mut Validator> {
        self.validators.get_mut(address)
    }

    /// Get all validators.
    pub fn all(&self) -> Vec<&Validator> {
        self.validators.values().collect()
    }

    /// Get active validators.
    pub fn active(&self) -> Vec<&Validator> {
        self.validators
            .values()
            .filter(|v| v.status.is_active())
            .collect()
    }

    /// Get eligible validators (for committee selection).
    pub fn eligible(&self) -> Vec<&Validator> {
        self.validators
            .values()
            .filter(|v| v.is_eligible())
            .collect()
    }

    /// Get count of validators.
    pub fn count(&self) -> usize {
        self.validators.len()
    }

    /// Advance to next epoch.
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
        for validator in self.validators.values_mut() {
            validator.current_epoch = self.current_epoch;
        }
    }

    /// Get current epoch.
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Get total active stake.
    pub fn total_active_stake(&self) -> U256 {
        self.active()
            .iter()
            .map(|v| v.stake)
            .fold(U256::ZERO, |acc, s| acc + s)
    }

    /// Get validator info for committee selection.
    pub fn to_validator_info(&self) -> Vec<ValidatorInfo> {
        self.eligible()
            .into_iter()
            .map(|v| v.to_info())
            .collect()
    }
}

impl Default for ValidatorSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_lifecycle() {
        let mut validator = Validator::new(
            Address::ZERO,
            [1u8; 32],
            U256::from(1_000_000u128),
        );

        assert_eq!(validator.status, ValidatorStatus::Pending);
        assert!(!validator.is_eligible());

        // Activate
        validator.activate(10);
        assert_eq!(validator.status, ValidatorStatus::Active);
        assert!(validator.is_eligible());
        assert_eq!(validator.activation_epoch, 10);

        // Exit
        validator.initiate_exit(100);
        assert_eq!(validator.status, ValidatorStatus::Exiting);
        assert!(!validator.is_eligible());

        // Make withdrawable
        validator.make_withdrawable(200);
        assert_eq!(validator.status, ValidatorStatus::Withdrawable);
    }

    #[test]
    fn test_validator_slashing() {
        let mut validator = Validator::new(
            Address::ZERO,
            [1u8; 32],
            U256::from(1_000_000u128),
        );
        validator.activate(10);

        validator.slash(100);
        
        assert!(validator.slashed);
        assert_eq!(validator.status, ValidatorStatus::Slashed);
        assert!(!validator.can_be_slashed()); // Already slashed
    }

    #[test]
    fn test_validator_epochs_active() {
        let mut validator = Validator::new(
            Address::ZERO,
            [1u8; 32],
            U256::from(1_000_000u128),
        );

        assert_eq!(validator.epochs_active(), 0);

        validator.activate(10);
        validator.current_epoch = 50;
        
        assert_eq!(validator.epochs_active(), 40);
    }

    #[test]
    fn test_validator_set_registration() {
        let mut set = ValidatorSet::new();

        // Register first validator
        let addr1 = Address::from_bytes([1u8; 20]);
        let result = set.register(addr1, [1u8; 32], U256::from(1_000_000u128));
        assert!(result.is_ok());
        assert_eq!(set.count(), 1);

        // Try to register again (should fail)
        let result = set.register(addr1, [1u8; 32], U256::from(2_000_000u128));
        assert!(result.is_err());

        // Register second validator
        let addr2 = Address::from_bytes([2u8; 20]);
        set.register(addr2, [2u8; 32], U256::from(1_000_000u128)).unwrap();
        assert_eq!(set.count(), 2);
    }

    #[test]
    fn test_validator_set_active() {
        let mut set = ValidatorSet::new();

        let addr1 = Address::from_bytes([1u8; 20]);
        let addr2 = Address::from_bytes([2u8; 20]);

        set.register(addr1, [1u8; 32], U256::from(1_000_000u128)).unwrap();
        set.register(addr2, [2u8; 32], U256::from(1_000_000u128)).unwrap();

        // Initially none active
        assert_eq!(set.active().len(), 0);

        // Activate one
        set.get_mut(&addr1).unwrap().activate(0);
        
        assert_eq!(set.active().len(), 1);
        assert_eq!(set.eligible().len(), 1);
    }

    #[test]
    fn test_validator_set_advance_epoch() {
        let mut set = ValidatorSet::new();
        
        let addr = Address::from_bytes([1u8; 20]);
        set.register(addr, [1u8; 32], U256::from(1_000_000u128)).unwrap();
        
        set.advance_epoch();
        set.advance_epoch();
        
        assert_eq!(set.current_epoch(), 2);
        assert_eq!(set.get(&addr).unwrap().current_epoch, 2);
    }

    #[test]
    fn test_validator_status_checks() {
        assert!(ValidatorStatus::Active.is_active());
        assert!(!ValidatorStatus::Pending.is_active());
        assert!(!ValidatorStatus::Slashed.is_active());

        assert!(ValidatorStatus::Active.can_be_slashed());
        assert!(ValidatorStatus::Exiting.can_be_slashed());
        assert!(!ValidatorStatus::Pending.can_be_slashed());
        assert!(!ValidatorStatus::Slashed.can_be_slashed());
    }
}
