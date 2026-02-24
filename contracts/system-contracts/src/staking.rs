//! Staking Contract
//!
//! Manages validator stakes, rewards, and delegations.

use merklith_types::{Address, U256};
use std::collections::HashMap;

/// Staking state.
#[derive(Debug)]
pub struct StakingContract {
    /// Validator stakes
    validators: HashMap<Address, Validator>,
    /// Delegations (delegator -> validator -> amount)
    delegations: HashMap<Address, HashMap<Address, U256>>,
    /// Total staked
    total_staked: U256,
    /// Reward rate per epoch (basis points)
    reward_rate_bps: u16,
    /// Minimum stake
    min_stake: U256,
    /// Unbonding period (epochs)
    unbonding_period: u64,
}

/// Validator info.
#[derive(Debug, Clone)]
pub struct Validator {
    /// Validator address
    pub address: Address,
    /// Self-stake
    pub self_stake: U256,
    /// Total delegated stake
    pub delegated: U256,
    /// Commission rate (basis points)
    pub commission_bps: u16,
    /// Whether active
    pub active: bool,
    /// Unbonding amount
    pub unbonding: U256,
    /// Epoch when unbonding completes
    pub unbonding_epoch: u64,
}

impl StakingContract {
    /// Create new staking contract.
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            delegations: HashMap::new(),
            total_staked: U256::ZERO,
            reward_rate_bps: 500, // 5% annual
            min_stake: U256::from(32_000_000_000_000_000_000u128), // 32 MERK
            unbonding_period: 14, // 14 epochs (~1 day)
        }
    }

    /// Register as validator.
    pub fn register_validator(
        &mut self,
        address: Address,
        stake: U256,
        commission_bps: u16,
    ) -> Result<(), String> {
        if stake < self.min_stake {
            return Err("Stake below minimum".to_string());
        }

        if self.validators.contains_key(&address) {
            return Err("Already a validator".to_string());
        }

        let validator = Validator {
            address,
            self_stake: stake,
            delegated: U256::ZERO,
            commission_bps,
            active: true,
            unbonding: U256::ZERO,
            unbonding_epoch: 0,
        };

        self.validators.insert(address, validator);
        self.total_staked += stake;

        Ok(())
    }

    /// Stake more (self-stake).
    pub fn stake(
        &mut self,
        validator: Address,
        amount: U256,
    ) -> Result<(), String> {
        let v = self.validators
            .get_mut(&validator)
            .ok_or("Not a validator")?;
        
        v.self_stake += amount;
        self.total_staked += amount;

        Ok(())
    }

    /// Delegate to validator.
    pub fn delegate(
        &mut self,
        delegator: Address,
        validator: Address,
        amount: U256,
    ) -> Result<(), String> {
        let v = self.validators
            .get_mut(&validator)
            .ok_or("Not a validator")?;

        v.delegated += amount;
        self.total_staked += amount;

        self.delegations
            .entry(delegator)
            .or_default()
            .entry(validator)
            .and_modify(|a| *a += amount)
            .or_insert(amount);

        Ok(())
    }

    /// Unbond stake.
    pub fn unbond(
        &mut self,
        validator: Address,
        amount: U256,
        current_epoch: u64,
    ) -> Result<(), String> {
        let v = self.validators
            .get_mut(&validator)
            .ok_or("Not a validator")?;

        if v.self_stake < amount {
            return Err("Insufficient stake".to_string());
        }

        v.self_stake -= amount;
        v.unbonding += amount;
        v.unbonding_epoch = current_epoch + self.unbonding_period;

        Ok(())
    }

    /// Withdraw unbonded stake.
    pub fn withdraw(
        &mut self,
        validator: Address,
        current_epoch: u64,
    ) -> Result<U256, String> {
        let v = self.validators
            .get_mut(&validator)
            .ok_or("Not a validator")?;

        if current_epoch < v.unbonding_epoch {
            return Err("Unbonding period not complete".to_string());
        }

        let amount = v.unbonding;
        v.unbonding = U256::ZERO;
        v.unbonding_epoch = 0;

        Ok(amount)
    }

    /// Calculate rewards.
    pub fn calculate_rewards(
        &self,
        validator: Address,
        epochs: u64,
    ) -> Result<U256, String> {
        let v = self.validators
            .get(&validator)
            .ok_or("Not a validator")?;

        let total = v.self_stake + v.delegated;
        let rate = U256::from(self.reward_rate_bps);
        let epochs = U256::from(epochs);
        
        // Simple calculation: stake * rate * epochs / 10000 / 365
        let annual_rewards = total * rate / U256::from(10000);
        let period_rewards = annual_rewards * epochs / U256::from(365);

        Ok(period_rewards)
    }

    /// Get validator info.
    pub fn get_validator(&self,
        address: &Address,
    ) -> Option<&Validator> {
        self.validators.get(address)
    }

    /// Get total staked.
    pub fn total_staked(&self) -> U256 {
        self.total_staked
    }

    /// Get delegation.
    pub fn get_delegation(
        &self,
        delegator: &Address,
        validator: &Address,
    ) -> U256 {
        self.delegations
            .get(delegator)
            .and_then(|d| d.get(validator))
            .copied()
            .unwrap_or(U256::ZERO)
    }
}

impl Default for StakingContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_validator() {
        let mut staking = StakingContract::new();
        let addr = Address::ZERO;
        
        let stake = U256::from(100_000_000_000_000_000_000u128); // 100 MERK
        assert!(staking.register_validator(addr, stake, 1000).is_ok());
        
        assert_eq!(staking.total_staked(), stake);
    }

    #[test]
    fn test_stake_below_minimum() {
        let mut staking = StakingContract::new();
        let addr = Address::ZERO;
        
        let stake = U256::from(1_000_000_000u128); // 1 Gwei - too small
        assert!(staking.register_validator(addr, stake, 1000).is_err());
    }

    #[test]
    fn test_delegate() {
        let mut staking = StakingContract::new();
        let validator = Address::from_bytes([1u8; 20]);
        let delegator = Address::from_bytes([2u8; 20]);
        
        // Use minimum stake (32 MERK)
        let min_stake = U256::from(32_000_000_000_000_000_000u128);
        staking.register_validator(validator, min_stake, 1000).unwrap();
        
        let amount = U256::from(10_000_000_000_000_000_000u128); // 10 MERK
        assert!(staking.delegate(delegator, validator, amount).is_ok());
        
        assert_eq!(staking.get_delegation(&delegator, &validator), amount);
    }

    #[test]
    fn test_unbond() {
        let mut staking = StakingContract::new();
        let validator = Address::ZERO;
        
        // Use minimum stake (32 MERK)
        let stake = U256::from(32_000_000_000_000_000_000u128);
        staking.register_validator(validator, stake, 1000).unwrap();
        
        let unbond_amount = U256::from(10_000_000_000_000_000_000u128); // 10 MERK
        assert!(staking.unbond(validator, unbond_amount, 0).is_ok());
        
        let v = staking.get_validator(&validator).unwrap();
        assert_eq!(v.self_stake, U256::from(22_000_000_000_000_000_000u128)); // 22 MERK remaining
        assert_eq!(v.unbonding, unbond_amount);
    }

    #[test]
    fn test_calculate_rewards() {
        let mut staking = StakingContract::new();
        let validator = Address::ZERO;
        let stake = U256::from(100_000_000_000_000_000_000u128);
        
        staking.register_validator(validator, stake, 1000).unwrap();
        
        let rewards = staking.calculate_rewards(validator, 365).unwrap();
        assert!(rewards > U256::ZERO);
    }
}
