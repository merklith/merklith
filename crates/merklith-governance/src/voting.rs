//! Voting power calculation with time-lock and quadratic voting.
//!
//! Voting power = sqrt(tokens) * lock_multiplier

use merklith_types::U256;
use crate::error::GovernanceError;

/// Lock duration options with multipliers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockDuration {
    /// No lock (1x multiplier)
    None,
    /// 1 week lock (1.25x multiplier)
    OneWeek,
    /// 1 month lock (1.5x multiplier)
    OneMonth,
    /// 3 months lock (2x multiplier)
    ThreeMonths,
    /// 6 months lock (3x multiplier)
    SixMonths,
    /// 1 year lock (4x multiplier)
    OneYear,
}

impl LockDuration {
    /// Get the multiplier for this lock duration.
    /// Returns basis points (10000 = 1x).
    pub fn multiplier_bps(&self) -> u16 {
        match self {
            LockDuration::None => 10_000,
            LockDuration::OneWeek => 12_500,
            LockDuration::OneMonth => 15_000,
            LockDuration::ThreeMonths => 20_000,
            LockDuration::SixMonths => 30_000,
            LockDuration::OneYear => 40_000,
        }
    }

    /// Get lock period in blocks (at 6s block time).
    pub fn blocks(&self) -> u64 {
        match self {
            LockDuration::None => 0,
            LockDuration::OneWeek => 100_800,  // 7 days
            LockDuration::OneMonth => 432_000, // 30 days
            LockDuration::ThreeMonths => 1_296_000, // 90 days
            LockDuration::SixMonths => 2_592_000,   // 180 days
            LockDuration::OneYear => 5_184_000,     // 365 days
        }
    }

    /// Calculate unlock block.
    pub fn unlock_block(&self, current_block: u64) -> u64 {
        current_block + self.blocks()
    }
}

/// Vote lock record.
#[derive(Debug, Clone)]
pub struct VoteLock {
    /// Amount of tokens locked
    pub amount: U256,
    /// Lock duration type
    pub duration: LockDuration,
    /// Block when locked
    pub locked_at: u64,
    /// Block when can unlock
    pub unlocks_at: u64,
}

impl VoteLock {
    /// Create a new vote lock.
    pub fn new(amount: U256, duration: LockDuration, current_block: u64) -> Self {
        Self {
            amount,
            duration,
            locked_at: current_block,
            unlocks_at: duration.unlock_block(current_block),
        }
    }

    /// Check if lock has expired.
    pub fn is_expired(&self, current_block: u64) -> bool {
        current_block >= self.unlocks_at
    }

    /// Get voting power for this lock.
    /// 
    /// Uses quadratic voting: voting_power = sqrt(amount) * multiplier
    pub fn voting_power(&self) -> U256 {
        // Calculate square root of amount
        let sqrt_amount = integer_sqrt(self.amount);
        
        // Apply multiplier
        let multiplier = U256::from(self.duration.multiplier_bps());
        (sqrt_amount * multiplier) / U256::from(10_000)
    }

    /// Calculate the raw token voting power (without quadratic).
    pub fn raw_voting_power(&self) -> U256 {
        let multiplier = U256::from(self.duration.multiplier_bps());
        (self.amount * multiplier) / U256::from(10_000)
    }
}

/// Integer square root using Newton's method.
/// Returns floor(sqrt(n)).
pub fn integer_sqrt(n: U256) -> U256 {
    if n <= U256::ONE {
        return n;
    }

    let mut x = n;
    let mut y = (x + U256::ONE) / U256::from(2u64);

    while y < x {
        x = y;
        y = (x + n / x) / U256::from(2u64);
    }

    x
}

/// Calculate voting power for a given token amount and lock duration.
/// 
/// Formula: voting_power = sqrt(tokens) * lock_multiplier
pub fn calculate_voting_power(tokens: U256, duration: LockDuration) -> U256 {
    let sqrt_tokens = integer_sqrt(tokens);
    let multiplier = U256::from(duration.multiplier_bps());
    (sqrt_tokens * multiplier) / U256::from(10_000)
}

/// Calculate raw voting power (without quadratic formula).
pub fn calculate_raw_voting_power(tokens: U256, duration: LockDuration) -> U256 {
    let multiplier = U256::from(duration.multiplier_bps());
    (tokens * multiplier) / U256::from(10_000)
}

/// Voting power tracker for an address.
#[derive(Debug, Default, Clone)]
pub struct VotingPowerTracker {
    /// Active locks
    pub locks: Vec<VoteLock>,
    /// Total locked tokens
    pub total_locked: U256,
}

impl VotingPowerTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a lock.
    pub fn lock(
        &mut self,
        amount: U256,
        duration: LockDuration,
        current_block: u64,
    ) -> Result<(), GovernanceError> {
        if amount == U256::ZERO {
            return Err(GovernanceError::InsufficientBalance(
                "Cannot lock zero tokens".to_string()
            ));
        }

        let vote_lock = VoteLock::new(amount, duration, current_block);
        self.locks.push(vote_lock);
        self.total_locked += amount;

        Ok(())
    }

    /// Unlock expired locks.
    /// 
    /// Returns the total amount unlocked.
    pub fn unlock_expired(&mut self, current_block: u64) -> U256 {
        let mut unlocked = U256::ZERO;
        let mut remaining = Vec::new();

        for lock in self.locks.drain(..) {
            if lock.is_expired(current_block) {
                unlocked += lock.amount;
                self.total_locked -= lock.amount;
            } else {
                remaining.push(lock);
            }
        }

        self.locks = remaining;
        unlocked
    }

    /// Get total voting power (quadratic).
    pub fn total_voting_power(&self) -> U256 {
        let mut total = U256::ZERO;
        for lock in &self.locks {
            total = total + lock.voting_power();
        }
        total
    }

    /// Get total raw voting power (non-quadratic).
    pub fn total_raw_voting_power(&self) -> U256 {
        let mut total = U256::ZERO;
        for lock in &self.locks {
            total = total + lock.raw_voting_power();
        }
        total
    }

    /// Get number of active locks.
    pub fn lock_count(&self) -> usize {
        self.locks.len()
    }
}

/// Calculate cost for quadratic voting.
/// 
/// In quadratic voting, cost = votes^2 (not linear).
/// This ensures that expressing strong preferences is more expensive.
pub fn quadratic_cost(votes: U256) -> U256 {
    votes * votes
}

/// Calculate maximum votes given a budget.
/// 
/// Returns floor(sqrt(budget)).
pub fn max_votes_from_budget(budget: U256) -> U256 {
    integer_sqrt(budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt(U256::ZERO), U256::ZERO);
        assert_eq!(integer_sqrt(U256::ONE), U256::ONE);
        assert_eq!(integer_sqrt(U256::from(4u64)), U256::from(2u64));
        assert_eq!(integer_sqrt(U256::from(9u64)), U256::from(3u64));
        assert_eq!(integer_sqrt(U256::from(15u64)), U256::from(3u64)); // floor(sqrt(15)) = 3
        assert_eq!(integer_sqrt(U256::from(16u64)), U256::from(4u64));
        assert_eq!(integer_sqrt(U256::from(100u64)), U256::from(10u64));
    }

    #[test]
    fn test_lock_duration_multipliers() {
        assert_eq!(LockDuration::None.multiplier_bps(), 10_000);
        assert_eq!(LockDuration::OneWeek.multiplier_bps(), 12_500);
        assert_eq!(LockDuration::OneYear.multiplier_bps(), 40_000);
    }

    #[test]
    fn test_vote_lock_creation() {
        let lock = VoteLock::new(
            U256::from(1000u64),
            LockDuration::OneMonth,
            100,
        );

        assert_eq!(lock.amount, U256::from(1000u64));
        assert_eq!(lock.locked_at, 100);
        assert_eq!(lock.unlocks_at, 100 + LockDuration::OneMonth.blocks());
    }

    #[test]
    fn test_vote_lock_expiration() {
        let lock = VoteLock::new(
            U256::from(1000u64),
            LockDuration::OneWeek,
            100,
        );

        assert!(!lock.is_expired(100));
        assert!(!lock.is_expired(100_799));
        assert!(!lock.is_expired(100_800)); // Expires at 100 + 100800 = 100900
        assert!(lock.is_expired(100_900));
        assert!(lock.is_expired(200_000));
    }

    #[test]
    fn test_voting_power_calculation() {
        // 100 tokens, no lock
        let power_none = calculate_voting_power(U256::from(100u64), LockDuration::None);
        // sqrt(100) * 1.0 = 10
        assert_eq!(power_none, U256::from(10u64));

        // 100 tokens, 1 year lock
        let power_year = calculate_voting_power(U256::from(100u64), LockDuration::OneYear);
        // sqrt(100) * 4.0 = 40
        assert_eq!(power_year, U256::from(40u64));

        // 10000 tokens, no lock
        let power_10k = calculate_voting_power(U256::from(10000u64), LockDuration::None);
        // sqrt(10000) * 1.0 = 100
        assert_eq!(power_10k, U256::from(100u64));
    }

    #[test]
    fn test_raw_voting_power() {
        // 100 tokens, 1 year lock
        let raw = calculate_raw_voting_power(U256::from(100u64), LockDuration::OneYear);
        // 100 * 4.0 = 400
        assert_eq!(raw, U256::from(400u64));
    }

    #[test]
    fn test_voting_power_tracker() {
        let mut tracker = VotingPowerTracker::new();

        // Lock some tokens
        tracker.lock(
            U256::from(100u64),
            LockDuration::None,
            100,
        ).unwrap();

        tracker.lock(
            U256::from(400u64),
            LockDuration::None,
            100,
        ).unwrap();

        // sqrt(100) + sqrt(400) = 10 + 20 = 30
        let power = tracker.total_voting_power();
        assert_eq!(power, U256::from(30u64));

        // Total locked
        assert_eq!(tracker.total_locked, U256::from(500u64));
    }

    #[test]
    fn test_unlock_expired() {
        let mut tracker = VotingPowerTracker::new();

        // Lock with 1 week duration
        tracker.lock(
            U256::from(100u64),
            LockDuration::OneWeek,
            100,
        ).unwrap();

        // Lock with 1 year duration
        tracker.lock(
            U256::from(200u64),
            LockDuration::OneYear,
            100,
        ).unwrap();

        assert_eq!(tracker.lock_count(), 2);

        // Unlock at block 100_900 (after 1 week but before 1 year)
        let unlocked = tracker.unlock_expired(100_900);
        
        // Only first lock should unlock
        assert_eq!(unlocked, U256::from(100u64));
        assert_eq!(tracker.lock_count(), 1);
    }

    #[test]
    fn test_quadratic_cost() {
        // Cost = votes^2
        assert_eq!(quadratic_cost(U256::from(1u64)), U256::from(1u64));
        assert_eq!(quadratic_cost(U256::from(2u64)), U256::from(4u64));
        assert_eq!(quadratic_cost(U256::from(10u64)), U256::from(100u64));
        assert_eq!(quadratic_cost(U256::from(100u64)), U256::from(10000u64));
    }

    #[test]
    fn test_max_votes_from_budget() {
        // With budget of 100, can cast 10 votes (10^2 = 100)
        assert_eq!(max_votes_from_budget(U256::from(100u64)), U256::from(10u64));
        
        // With budget of 50, can cast 7 votes (7^2 = 49 <= 50)
        assert_eq!(max_votes_from_budget(U256::from(50u64)), U256::from(7u64));
    }

    #[test]
    fn test_lock_zero_fails() {
        let mut tracker = VotingPowerTracker::new();
        let result = tracker.lock(U256::ZERO, LockDuration::None, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_quadratic_voting_fairness() {
        // Demonstrate quadratic voting property:
        // Large token holders have diminishing returns
        
        let small_holder = U256::from(100u64);   // 100 tokens
        let medium_holder = U256::from(1000u64);  // 1000 tokens (10x)
        let large_holder = U256::from(10000u64);  // 10000 tokens (100x)

        let small_power = calculate_voting_power(small_holder, LockDuration::None);
        let medium_power = calculate_voting_power(medium_holder, LockDuration::None);
        let large_power = calculate_voting_power(large_holder, LockDuration::None);

        // sqrt(100) = 10
        assert_eq!(small_power, U256::from(10u64));
        
        // sqrt(1000) = 31.6...
        // Should be about 3x, not 10x
        assert!(medium_power > U256::from(30u64));
        assert!(medium_power < U256::from(32u64));

        // sqrt(10000) = 100
        // Should be about 10x, not 100x
        assert_eq!(large_power, U256::from(100u64));
    }
}
