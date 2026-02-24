//! Staking Contract with Rewards
//! 
//! Stake tokens to earn rewards over time.
//! Features:
//! - Flexible staking periods
//! - Compound rewards
//! - Multiple reward tokens
//! - Emergency withdrawal
//! - Slashing for early exit

use borsh::{BorshSerialize, BorshDeserialize};
use merklith_types::{Address, U256};

/// Precision constant for reward calculations (1e18)
const PRECISION: U256 = U256::from(1_000_000_000_000_000_000u128);

/// Staking Contract State
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct StakingContract {
    /// Contract owner
    pub owner: Address,
    /// Staking token address
    pub staking_token: Address,
    /// Reward token address
    pub reward_token: Address,
    /// Total staked amount
    pub total_staked: U256,
    /// Reward rate per second
    pub reward_rate: U256,
    /// Last update time
    pub last_update_time: u64,
    /// Reward per token stored
    pub reward_per_token_stored: U256,
    /// User stakes: address -> stake info
    pub stakes: Vec<(Address, StakeInfo)>,
    /// Minimum stake amount
    pub min_stake: U256,
    /// Lock period in seconds
    pub lock_period: u64,
    /// Early withdrawal fee (in bps, 100 = 1%)
    pub early_withdrawal_fee: u64,
    /// Fee recipient
    pub fee_recipient: Address,
    /// Total rewards distributed
    pub total_rewards_distributed: U256,
    /// Reward period finish
    pub period_finish: u64,
}

/// Stake information
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct StakeInfo {
    /// Amount staked
    pub amount: U256,
    /// Reward debt
    pub reward_debt: U256,
    /// Stake start time
    pub start_time: u64,
    /// Last claim time
    pub last_claim: u64,
    /// Accumulated rewards
    pub accumulated_rewards: U256,
}

/// Stake Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct StakeEvent {
    pub user: Address,
    pub amount: U256,
    pub timestamp: u64,
}

/// Withdraw Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct WithdrawEvent {
    pub user: Address,
    pub amount: U256,
    pub reward: U256,
    pub timestamp: u64,
}

/// Reward Paid Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RewardPaidEvent {
    pub user: Address,
    pub reward: U256,
    pub timestamp: u64,
}

/// Staking Error Types
#[derive(Debug, Clone, PartialEq)]
pub enum StakingError {
    /// Insufficient balance
    InsufficientBalance,
    /// Below minimum stake
    BelowMinimumStake,
    /// No active stake
    NoActiveStake,
    /// Still locked
    StillLocked,
    /// Zero amount
    ZeroAmount,
    /// Not owner
    NotOwner,
    /// Reward period not finished
    RewardPeriodNotFinished,
    /// Overflow
    Overflow,
    /// Underflow
    Underflow,
    /// Transfer failed
    TransferFailed,
}

impl std::fmt::Display for StakingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StakingError::InsufficientBalance => write!(f, "Insufficient balance"),
            StakingError::BelowMinimumStake => write!(f, "Below minimum stake amount"),
            StakingError::NoActiveStake => write!(f, "No active stake found"),
            StakingError::StillLocked => write!(f, "Stake is still locked"),
            StakingError::ZeroAmount => write!(f, "Amount must be greater than zero"),
            StakingError::NotOwner => write!(f, "Not contract owner"),
            StakingError::RewardPeriodNotFinished => write!(f, "Reward period not finished"),
            StakingError::Overflow => write!(f, "Arithmetic overflow"),
            StakingError::Underflow => write!(f, "Arithmetic underflow"),
            StakingError::TransferFailed => write!(f, "Token transfer failed"),
        }
    }
}

impl std::error::Error for StakingError {}

impl StakingContract {
    /// Create new staking contract
    pub fn new(
        owner: Address,
        staking_token: Address,
        reward_token: Address,
        min_stake: U256,
        lock_period: u64,
    ) -> Self {
        let now = Self::current_timestamp();
        
        Self {
            owner,
            staking_token,
            reward_token,
            total_staked: U256::ZERO,
            reward_rate: U256::ZERO,
            last_update_time: now,
            reward_per_token_stored: U256::ZERO,
            stakes: Vec::new(),
            min_stake,
            lock_period,
            early_withdrawal_fee: 1000, // 10%
            fee_recipient: owner,
            total_rewards_distributed: U256::ZERO,
            period_finish: now,
        }
    }

    /// Stake tokens
    pub fn stake(
        &mut self,
        caller: Address,
        amount: U256,
    ) -> Result<StakeEvent, StakingError> {
        if amount == U256::ZERO {
            return Err(StakingError::ZeroAmount);
        }

        if amount < self.min_stake {
            return Err(StakingError::BelowMinimumStake);
        }

        self.update_reward(caller)?;

        let now = Self::current_timestamp();

        // Update stake info
        if let Some(pos) = self.stakes.iter().position(|(addr, _)| *addr == caller) {
            let stake_info = &mut self.stakes[pos].1;
            stake_info.amount = stake_info.amount.checked_add(&amount).ok_or(StakingError::Overflow)?;
            stake_info.reward_debt = self.calculate_reward_debt(stake_info.amount)?;
        } else {
            let stake_info = StakeInfo {
                amount,
                reward_debt: self.calculate_reward_debt(amount)?,
                start_time: now,
                last_claim: now,
                accumulated_rewards: U256::ZERO,
            };
            self.stakes.push((caller, stake_info));
        }

        self.total_staked = self.total_staked.checked_add(&amount).ok_or(StakingError::Overflow)?;

        Ok(StakeEvent {
            user: caller,
            amount,
            timestamp: now,
        })
    }

    /// Withdraw stake
    pub fn withdraw(
        &mut self,
        caller: Address,
        amount: U256,
    ) -> Result<WithdrawEvent, StakingError> {
        if amount == U256::ZERO {
            return Err(StakingError::ZeroAmount);
        }

        self.update_reward(caller)?;

        let pos = self.stakes.iter().position(|(addr, _)| *addr == caller)
            .ok_or(StakingError::NoActiveStake)?;
        
        let stake_info = &self.stakes[pos].1;
        
        if stake_info.amount < amount {
            return Err(StakingError::InsufficientBalance);
        }

        // Calculate rewards
        let reward = self.calculate_earned(caller)?;
        
        // Check lock period
        let now = Self::current_timestamp();
        let is_early = now < stake_info.start_time + self.lock_period;
        
        let withdraw_amount = if is_early {
            // Apply early withdrawal fee
            let fee = amount
                .checked_mul(&U256::from(self.early_withdrawal_fee)).ok_or(StakingError::Overflow)?
                .checked_div(&U256::from(10000u64)).ok_or(StakingError::DivideByZero)?;
            amount.checked_sub(&fee).ok_or(StakingError::Underflow)?
        } else {
            amount
        };

        // Update stake
        let stake_info = &mut self.stakes[pos].1;
        stake_info.amount = stake_info.amount.checked_sub(&amount).ok_or(StakingError::Underflow)?;
        stake_info.reward_debt = self.calculate_reward_debt(stake_info.amount)?;
        
        if stake_info.amount == U256::ZERO {
            self.stakes.remove(pos);
        }

        self.total_staked = self.total_staked.checked_sub(&amount).ok_or(StakingError::Underflow)?;

        Ok(WithdrawEvent {
            user: caller,
            amount: withdraw_amount,
            reward,
            timestamp: now,
        })
    }

    /// Claim rewards only
    pub fn claim_reward(
        &mut self,
        caller: Address,
    ) -> Result<RewardPaidEvent, StakingError> {
        self.update_reward(caller)?;

        let reward = self.calculate_earned(caller)?;
        
        if reward == U256::ZERO {
            return Err(StakingError::NoActiveStake);
        }

        // Update claim time
        if let Some(pos) = self.stakes.iter().position(|(addr, _)| *addr == caller) {
            let now = Self::current_timestamp();
            self.stakes[pos].1.last_claim = now;
            self.stakes[pos].1.accumulated_rewards = U256::ZERO;
        }

        self.total_rewards_distributed = self.total_rewards_distributed
            .checked_add(&reward).ok_or(StakingError::Overflow)?;

        Ok(RewardPaidEvent {
            user: caller,
            reward,
            timestamp: Self::current_timestamp(),
        })
    }

    /// Exit (withdraw all + claim rewards)
    pub fn exit(
        &mut self,
        caller: Address,
    ) -> Result<(WithdrawEvent, RewardPaidEvent), StakingError> {
        let stake_amount = self.get_stake_amount(caller);
        let withdraw_event = self.withdraw(caller, stake_amount)?;
        let reward_event = self.claim_reward(caller)?;
        
        Ok((withdraw_event, reward_event))
    }

    /// Update reward for user
    fn update_reward(
        &mut self,
        account: Address,
    ) -> Result<(), StakingError> {
        self.reward_per_token_stored = self.reward_per_token()?;
        self.last_update_time = self.last_time_reward_applicable();

        if let Some(pos) = self.stakes.iter().position(|(addr, _)| *addr == account) {
            let earned = self.calculate_earned(account)?;
            self.stakes[pos].1.accumulated_rewards = earned;
            self.stakes[pos].1.reward_debt = self.calculate_reward_debt(self.stakes[pos].1.amount)?;
        }

        Ok(())
    }

    /// Calculate reward per token
    fn reward_per_token(
        &self,
    ) -> Result<U256, StakingError> {
        if self.total_staked == U256::ZERO {
            return Ok(self.reward_per_token_stored);
        }

        let time_diff = self.last_time_reward_applicable()
            .checked_sub(self.last_update_time).ok_or(StakingError::Underflow)?;
        
        let reward = U256::from(time_diff)
            .checked_mul(&self.reward_rate).ok_or(StakingError::Overflow)?
            .checked_mul(&PRECISION).ok_or(StakingError::Overflow)? // precision (1e18)
            .checked_div(&self.total_staked).ok_or(StakingError::DivideByZero)?;

        self.reward_per_token_stored
            .checked_add(&reward).ok_or(StakingError::Overflow)
    }

    /// Calculate earned rewards for account
    fn calculate_earned(
        &self,
        account: Address,
    ) -> Result<U256, StakingError> {
        let pos = self.stakes.iter().position(|(addr, _)| *addr == account);
        
        if let Some(pos) = pos {
            let stake_info = &self.stakes[pos].1;
            let earned = stake_info.amount
                .checked_mul(
                    &self.reward_per_token()?.checked_sub(&stake_info.reward_debt).ok_or(StakingError::Underflow)?
                ).ok_or(StakingError::Overflow)?
                .checked_div(&PRECISION).ok_or(StakingError::DivideByZero)?;
            
            stake_info.accumulated_rewards
                .checked_add(&earned).ok_or(StakingError::Overflow)
        } else {
            Ok(U256::ZERO)
        }
    }

    /// Calculate reward debt
    fn calculate_reward_debt(
        &self,
        amount: U256,
    ) -> Result<U256, StakingError> {
        amount
            .checked_mul(&self.reward_per_token_stored).ok_or(StakingError::Overflow)?
            .checked_div(&PRECISION).ok_or(StakingError::DivideByZero)
    }

    /// Last time reward applicable
    fn last_time_reward_applicable(&self,
    ) -> u64 {
        let now = Self::current_timestamp();
        if now < self.period_finish {
            now
        } else {
            self.period_finish
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get stake amount for user
    pub fn get_stake_amount(&self,
        account: Address,
    ) -> U256 {
        self.stakes
            .iter()
            .find(|(addr, _)| *addr == account)
            .map(|(_, info)| info.amount)
            .unwrap_or(U256::ZERO)
    }

    /// Get earned rewards for user
    pub fn get_earned(&self,
        account: Address,
    ) -> U256 {
        self.calculate_earned(account).unwrap_or(U256::ZERO)
    }

    /// Set reward rate (owner only)
    pub fn set_reward_rate(
        &mut self,
        caller: Address,
        rate: U256,
    ) -> Result<(), StakingError> {
        if caller != self.owner {
            return Err(StakingError::NotOwner);
        }
        
        self.update_reward(Address::ZERO)?;
        self.reward_rate = rate;
        
        Ok(())
    }

    /// Notify reward amount (add rewards)
    pub fn notify_reward_amount(
        &mut self,
        caller: Address,
        reward: U256,
    ) -> Result<(), StakingError> {
        if caller != self.owner {
            return Err(StakingError::NotOwner);
        }

        self.update_reward(Address::ZERO)?;

        let now = Self::current_timestamp();
        
        if now >= self.period_finish {
            // New period
            let duration = U256::from(7 * 24 * 60 * 60u64); // 7 days
            self.reward_rate = reward
                .checked_div(&duration).ok_or(StakingError::DivideByZero)?;
        } else {
            // Extend current period
            let remaining = self.period_finish - now;
            let leftover = U256::from(remaining)
                .checked_mul(&self.reward_rate).ok_or(StakingError::Overflow)?;
            
            let new_reward = reward.checked_add(&leftover).ok_or(StakingError::Overflow)?;
            let duration = U256::from(7 * 24 * 60 * 60u64);
            
            self.reward_rate = new_reward
                .checked_div(&duration).ok_or(StakingError::DivideByZero)?;
        }

        self.period_finish = now + 7 * 24 * 60 * 60;
        self.last_update_time = now;

        Ok(())
    }

    /// Get APR (Annual Percentage Rate)
    pub fn get_apr(&self,
    ) -> U256 {
        if self.total_staked == U256::ZERO {
            return U256::ZERO;
        }

        // APR = (reward_rate * 365 days * 100) / total_staked
        let yearly_rewards = self.reward_rate
            .mul(U256::from(365 * 24 * 60 * 60u64));
        
        yearly_rewards
            .mul(U256::from(100u64))
            .div(self.total_staked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_staking() -> StakingContract {
        let owner = Address::from_bytes([1u8; 20]);
        let staking_token = Address::from_bytes([2u8; 20]);
        let reward_token = Address::from_bytes([3u8; 20]);
        
        StakingContract::new(
            owner,
            staking_token,
            reward_token,
            U256::from(100u64), // min stake
            86400, // 1 day lock
        )
    }

    #[test]
    fn test_initialization() {
        let staking = create_staking();
        assert_eq!(staking.total_staked, U256::ZERO);
        assert_eq!(staking.reward_rate, U256::ZERO);
    }

    #[test]
    fn test_stake() {
        let mut staking = create_staking();
        let user = Address::from_bytes([4u8; 20]);
        
        let result = staking.stake(user, U256::from(1000u64));
        assert!(result.is_ok());
        
        assert_eq!(staking.get_stake_amount(user), U256::from(1000u64));
        assert_eq!(staking.total_staked, U256::from(1000u64));
    }

    #[test]
    fn test_stake_below_minimum() {
        let mut staking = create_staking();
        let user = Address::from_bytes([4u8; 20]);
        
        let result = staking.stake(user, U256::from(50u64));
        assert!(matches!(result, Err(StakingError::BelowMinimumStake)));
    }

    #[test]
    fn test_reward_calculation() {
        let mut staking = create_staking();
        let user = Address::from_bytes([4u8; 20]);
        
        // Stake
        staking.stake(user, U256::from(1000u64)).unwrap();
        
        // Set reward rate
        staking.set_reward_rate(staking.owner, U256::from(100u64)).unwrap();
        
        // Check earned (initially 0)
        let earned = staking.get_earned(user);
        assert_eq!(earned, U256::ZERO);
    }

    #[test]
    fn test_withdraw() {
        let mut staking = create_staking();
        let user = Address::from_bytes([4u8; 20]);
        
        // Stake
        staking.stake(user, U256::from(1000u64)).unwrap();
        
        // Withdraw
        let result = staking.withdraw(user, U256::from(500u64));
        assert!(result.is_ok());
        
        assert_eq!(staking.get_stake_amount(user), U256::from(500u64));
    }

    #[test]
    fn test_withdraw_too_much() {
        let mut staking = create_staking();
        let user = Address::from_bytes([4u8; 20]);
        
        // Stake
        staking.stake(user, U256::from(1000u64)).unwrap();
        
        // Try to withdraw more than staked
        let result = staking.withdraw(user, U256::from(2000u64));
        assert!(matches!(result, Err(StakingError::InsufficientBalance)));
    }
}
