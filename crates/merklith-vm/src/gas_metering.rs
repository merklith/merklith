/// Gas schedule for VM operations.
#[derive(Debug, Clone, Copy)]
pub struct GasSchedule {
    // Storage
    pub storage_read_cold: u64,      // 200
    pub storage_read_warm: u64,      // 50
    pub storage_write_new: u64,      // 5,000
    pub storage_write_update: u64,   // 2,500
    pub storage_delete_refund: u64,  // 2,500

    // Transfers & calls
    pub transfer: u64,               // 2,100
    pub call_base: u64,              // 700
    pub call_value_transfer: u64,    // 9,000
    pub call_new_account: u64,       // 25,000

    // Contract creation
    pub create_base: u64,            // 32,000
    pub create_per_byte: u64,        // 200

    // Events
    pub log_base: u64,               // 375
    pub log_per_topic: u64,          // 375
    pub log_per_byte: u64,           // 8

    // Crypto
    pub keccak256_base: u64,         // 30
    pub keccak256_per_word: u64,     // 6
    pub blake3_base: u64,            // 20
    pub blake3_per_word: u64,        // 4
    pub ed25519_verify: u64,         // 3,000
    pub bls_verify: u64,             // 12,000

    // Memory
    pub memory_per_page: u64,        // 3  (per 64KB page)

    // TX
    pub tx_base: u64,                // 21,000
    pub tx_per_data_zero_byte: u64,  // 4
    pub tx_per_data_nonzero_byte: u64, // 16
    pub tx_create: u64,              // 53,000
    pub tx_access_list_address: u64, // 2,400
    pub tx_access_list_storage: u64, // 1,900
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            // Storage
            storage_read_cold: 200,
            storage_read_warm: 50,
            storage_write_new: 5_000,
            storage_write_update: 2_500,
            storage_delete_refund: 2_500,

            // Transfers & calls
            transfer: 2_100,
            call_base: 700,
            call_value_transfer: 9_000,
            call_new_account: 25_000,

            // Contract creation
            create_base: 32_000,
            create_per_byte: 200,

            // Events
            log_base: 375,
            log_per_topic: 375,
            log_per_byte: 8,

            // Crypto
            keccak256_base: 30,
            keccak256_per_word: 6,
            blake3_base: 20,
            blake3_per_word: 4,
            ed25519_verify: 3_000,
            bls_verify: 12_000,

            // Memory
            memory_per_page: 3,

            // TX
            tx_base: 21_000,
            tx_per_data_zero_byte: 4,
            tx_per_data_nonzero_byte: 16,
            tx_create: 53_000,
            tx_access_list_address: 2_400,
            tx_access_list_storage: 1_900,
        }
    }
}

/// Gas tracking during execution.
#[derive(Debug, Clone)]
pub struct GasTracker {
    /// Gas limit for this execution
    limit: u64,
    /// Gas already used
    used: u64,
    /// Gas refunded
    refunded: u64,
    /// Gas schedule
    schedule: GasSchedule,
}

impl GasTracker {
    /// Create a new gas tracker.
    pub fn new(limit: u64, schedule: GasSchedule) -> Self {
        Self {
            limit,
            used: 0,
            refunded: 0,
            schedule,
        }
    }

    /// Create with default schedule.
    pub fn with_default_schedule(limit: u64) -> Self {
        Self::new(limit, GasSchedule::default())
    }

    /// Get gas limit.
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Get gas used.
    pub fn used(&self) -> u64 {
        self.used
    }

    /// Get gas refunded.
    pub fn refunded(&self) -> u64 {
        self.refunded
    }

    /// Get remaining gas.
    pub fn remaining(&self) -> u64 {
        self.limit - self.used
    }

    /// Get effective gas (used - refunded, never negative).
    pub fn effective_gas(&self) -> u64 {
        self.used.saturating_sub(self.refunded)
    }

    /// Charge gas.
    pub fn charge(&mut self, amount: u64) -> Result<(), crate::error::VmError> {
        // Use checked_add to prevent overflow
        let new_used = self.used.checked_add(amount)
            .ok_or(crate::error::VmError::OutOfGas {
                used: u64::MAX,
                limit: self.limit,
            })?;
        
        if new_used > self.limit {
            Err(crate::error::VmError::OutOfGas {
                used: new_used,
                limit: self.limit,
            })
        } else {
            self.used = new_used;
            Ok(())
        }
    }

    /// Charge gas for storage write.
    pub fn charge_storage_write(&mut self, is_new: bool) -> Result<(), crate::error::VmError> {
        let cost = if is_new {
            self.schedule.storage_write_new
        } else {
            self.schedule.storage_write_update
        };
        self.charge(cost)
    }

    /// Charge gas for storage read.
    pub fn charge_storage_read(&mut self, is_cold: bool) -> Result<(), crate::error::VmError> {
        let cost = if is_cold {
            self.schedule.storage_read_cold
        } else {
            self.schedule.storage_read_warm
        };
        self.charge(cost)
    }

    /// Charge gas for memory expansion.
    pub fn charge_memory(&mut self, pages: u64) -> Result<(), crate::error::VmError> {
        let cost = pages * self.schedule.memory_per_page;
        self.charge(cost)
    }

    /// Refund gas (for storage deletion).
    pub fn refund(&mut self, amount: u64) {
        self.refunded += amount;
    }

    /// Get gas schedule.
    pub fn schedule(&self) -> &GasSchedule {
        &self.schedule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_tracker() {
        let mut tracker = GasTracker::with_default_schedule(100_000);
        assert_eq!(tracker.limit(), 100_000);
        assert_eq!(tracker.remaining(), 100_000);

        tracker.charge(10_000).unwrap();
        assert_eq!(tracker.used(), 10_000);
        assert_eq!(tracker.remaining(), 90_000);
    }

    #[test]
    fn test_gas_tracker_out_of_gas() {
        let mut tracker = GasTracker::with_default_schedule(1_000);
        assert!(tracker.charge(10_000).is_err());
    }

    #[test]
    fn test_gas_refund() {
        let mut tracker = GasTracker::with_default_schedule(100_000);
        tracker.charge(10_000).unwrap();
        tracker.refund(2_500);
        assert_eq!(tracker.refunded(), 2_500);
        assert_eq!(tracker.effective_gas(), 7_500);
    }

    #[test]
    fn test_storage_gas() {
        let mut tracker = GasTracker::with_default_schedule(100_000);
        tracker.charge_storage_write(true).unwrap();
        assert_eq!(tracker.used(), 5_000);

        tracker.charge_storage_write(false).unwrap();
        assert_eq!(tracker.used(), 5_000 + 2_500);
    }
}
