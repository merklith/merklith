//! State management for accounts

use std::collections::HashMap;

/// Account state manager
#[derive(Debug)]
pub struct AccountState {
    balances: HashMap<merklith_types::Address, merklith_types::U256>,
    nonces: HashMap<merklith_types::Address, u64>,
}

impl AccountState {
    /// Create a new account state
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            nonces: HashMap::new(),
        }
    }

    /// Get account balance
    pub fn get_balance(&self, address: &merklith_types::Address) -> merklith_types::U256 {
        self.balances.get(address).copied().unwrap_or(merklith_types::U256::ZERO)
    }

    /// Set account balance
    pub fn set_balance(
        &mut self,
        address: merklith_types::Address,
        balance: merklith_types::U256,
    ) {
        self.balances.insert(address, balance);
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &merklith_types::Address) -> u64 {
        self.nonces.get(address).copied().unwrap_or(0)
    }

    /// Increment account nonce
    pub fn increment_nonce(&mut self,
        address: merklith_types::Address) {
        let current = self.get_nonce(&address);
        self.nonces.insert(address, current + 1);
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self::new()
    }
}
