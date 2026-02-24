use crate::hash::Hash;
use crate::u256::U256;
use std::fmt;

/// On-chain account state (stored in the state trie).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct Account {
    /// Transaction count (nonce)
    pub nonce: u64,
    /// Balance in Spark
    pub balance: U256,
    /// Hash of contract code (Hash::ZERO for EOAs)
    pub code_hash: Hash,
    /// Root hash of the account's storage trie (Hash::ZERO if empty)
    pub storage_root: Hash,
    /// Account type flags
    pub account_type: AccountType,
}

/// Account type enumeration
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub enum AccountType {
    /// Externally owned account (keypair)
    #[default]
    EOA,
    /// Smart contract
    Contract,
    /// Smart account (native account abstraction)
    SmartAccount,
    /// System contract (pre-deployed)
    System,
}

impl Account {
    /// Create a new empty account
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new EOA with given balance
    pub fn new_eoa(balance: U256) -> Self {
        Self {
            nonce: 0,
            balance,
            code_hash: Hash::ZERO,
            storage_root: Hash::ZERO,
            account_type: AccountType::EOA,
        }
    }

    /// Create a new contract account
    pub fn new_contract(code_hash: Hash, balance: U256) -> Self {
        Self {
            nonce: 0,
            balance,
            code_hash,
            storage_root: Hash::ZERO,
            account_type: AccountType::Contract,
        }
    }

    /// Check if account is empty (nonce=0, balance=0, no code)
    pub fn is_empty(&self) -> bool {
        self.nonce == 0 && self.balance.is_zero() && self.code_hash.is_zero()
    }

    /// Check if this is a contract account
    pub fn is_contract(&self) -> bool {
        matches!(
            self.account_type,
            AccountType::Contract | AccountType::SmartAccount | AccountType::System
        )
    }

    /// Check if account has code deployed
    pub fn has_code(&self) -> bool {
        !self.code_hash.is_zero()
    }

    /// Check if this is a system contract
    pub fn is_system(&self) -> bool {
        matches!(self.account_type, AccountType::System)
    }

    /// Increment nonce
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    /// Add balance
    pub fn add_balance(&mut self, amount: U256) {
        self.balance = self.balance.saturating_add(&amount);
    }

    /// Subtract balance
    pub fn sub_balance(&mut self, amount: U256) {
        self.balance = self.balance.saturating_sub(&amount);
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountType::EOA => write!(f, "EOA"),
            AccountType::Contract => write!(f, "Contract"),
            AccountType::SmartAccount => write!(f, "SmartAccount"),
            AccountType::System => write!(f, "System"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_empty() {
        let acc = Account::new();
        assert!(acc.is_empty());
        assert!(!acc.is_contract());
        assert!(!acc.has_code());
    }

    #[test]
    fn test_account_eoa() {
        let acc = Account::new_eoa(U256::from(1000u64));
        assert!(!acc.is_empty());
        assert!(!acc.is_contract());
        assert!(!acc.has_code());
        assert_eq!(acc.balance, U256::from(1000u64));
    }

    #[test]
    fn test_account_contract() {
        let code_hash = Hash::compute(b"code");
        let acc = Account::new_contract(code_hash, U256::from(0u64));
        assert!(!acc.is_empty());
        assert!(acc.is_contract());
        assert!(acc.has_code());
        assert_eq!(acc.code_hash, code_hash);
    }

    #[test]
    fn test_account_nonce() {
        let mut acc = Account::new();
        assert_eq!(acc.nonce, 0);
        acc.increment_nonce();
        assert_eq!(acc.nonce, 1);
    }

    #[test]
    fn test_account_balance() {
        let mut acc = Account::new_eoa(U256::from(1000u64));
        acc.add_balance(U256::from(500u64));
        assert_eq!(acc.balance, U256::from(1500u64));

        acc.sub_balance(U256::from(300u64));
        assert_eq!(acc.balance, U256::from(1200u64));
    }

    #[test]
    fn test_account_types() {
        assert_eq!(format!("{}", AccountType::EOA), "EOA");
        assert_eq!(format!("{}", AccountType::Contract), "Contract");
        assert_eq!(format!("{}", AccountType::System), "System");
    }
}
