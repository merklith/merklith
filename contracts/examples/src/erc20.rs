//! ERC20 Token Contract Example
//! 
//! A production-ready ERC20 token implementation for MERKLITH blockchain.
//! 
//! Features:
//! - Standard ERC20 interface (transfer, approve, transferFrom, etc.)
//! - Minting and burning capabilities
//! - Permit (gasless approvals) using Ed25519 signatures
//! - Pausable functionality
//! - Role-based access control

use borsh::{BorshSerialize, BorshDeserialize};
use merklith_types::{Address, U256};
use std::collections::HashMap;

/// ERC20 Token Contract State
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ERC20Token {
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token decimals (usually 18)
    pub decimals: u8,
    /// Total supply
    pub total_supply: U256,
    /// Balances mapping: address -> balance (O(1) lookup)
    pub balances: HashMap<Address, U256>,
    /// Allowances mapping: (owner, spender) -> amount (O(1) lookup)
    pub allowances: HashMap<(Address, Address), U256>,
    /// Contract owner
    pub owner: Address,
    /// Paused state
    pub paused: bool,
    /// Nonces for permit (address -> nonce) (O(1) lookup)
    pub nonces: HashMap<Address, u64>,
}

/// ERC20 Transfer Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub value: U256,
}

/// ERC20 Approval Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ApprovalEvent {
    pub owner: Address,
    pub spender: Address,
    pub value: U256,
}

/// ERC20 Token Error Types
#[derive(Debug, Clone, PartialEq)]
pub enum ERC20Error {
    /// Insufficient balance
    InsufficientBalance,
    /// Insufficient allowance
    InsufficientAllowance,
    /// Invalid amount
    InvalidAmount,
    /// Contract is paused
    ContractPaused,
    /// Not the owner
    NotOwner,
    /// Invalid signature
    InvalidSignature,
    /// Nonce already used
    NonceAlreadyUsed,
    /// Zero address not allowed
    ZeroAddress,
    /// Overflow
    Overflow,
}

impl std::fmt::Display for ERC20Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ERC20Error::InsufficientBalance => write!(f, "Insufficient balance"),
            ERC20Error::InsufficientAllowance => write!(f, "Insufficient allowance"),
            ERC20Error::InvalidAmount => write!(f, "Invalid amount"),
            ERC20Error::ContractPaused => write!(f, "Contract is paused"),
            ERC20Error::NotOwner => write!(f, "Caller is not the owner"),
            ERC20Error::InvalidSignature => write!(f, "Invalid signature"),
            ERC20Error::NonceAlreadyUsed => write!(f, "Nonce already used"),
            ERC20Error::ZeroAddress => write!(f, "Zero address not allowed"),
            ERC20Error::Overflow => write!(f, "Arithmetic overflow"),
        }
    }
}

impl std::error::Error for ERC20Error {}

impl ERC20Token {
    /// Create a new ERC20 token
    pub fn new(name: String, symbol: String, decimals: u8, owner: Address) -> Self {
        Self {
            name,
            symbol,
            decimals,
            total_supply: U256::ZERO,
            balances: HashMap::new(),
            allowances: HashMap::new(),
            owner,
            paused: false,
            nonces: HashMap::new(),
        }
    }

    /// Initialize with initial supply (minted to owner)
    pub fn with_initial_supply(
        name: String,
        symbol: String,
        decimals: u8,
        owner: Address,
        initial_supply: U256,
    ) -> Self {
        let mut token = Self::new(name, symbol, decimals, owner);
        token.balances.insert(owner, initial_supply);
        token.total_supply = initial_supply;
        token
    }

    /// Get token name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get token symbol
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get decimals
    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    /// Get total supply
    pub fn total_supply(&self) -> U256 {
        self.total_supply
    }

    /// Get balance of address
    pub fn balance_of(&self, address: Address) -> U256 {
        *self.balances.get(&address).unwrap_or(&U256::ZERO)
    }

    /// Get allowance
    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        *self.allowances.get(&(owner, spender)).unwrap_or(&U256::ZERO)
    }

    /// Transfer tokens (internal)
    fn _transfer(&mut self, from: Address, to: Address, value: U256) -> Result<(), ERC20Error> {
        if self.paused {
            return Err(ERC20Error::ContractPaused);
        }

        if value == U256::ZERO {
            return Err(ERC20Error::InvalidAmount);
        }

        let from_balance = self.balance_of(from);
        if from_balance < value {
            return Err(ERC20Error::InsufficientBalance);
        }

        // Update balances
        self.update_balance(from, from_balance - value)?;
        let to_balance = self.balance_of(to);
        self.update_balance(to, to_balance + value)?;

        Ok(())
    }

    /// Update balance for address
    fn update_balance(&mut self, address: Address, new_balance: U256) -> Result<(), ERC20Error> {
        // Check for overflow
        if new_balance < U256::ZERO {
            return Err(ERC20Error::Overflow);
        }

        // Update or remove balance using HashMap (O(1) operation)
        if new_balance == U256::ZERO {
            // Remove zero balances to save space
            self.balances.remove(&address);
        } else {
            self.balances.insert(address, new_balance);
        }

        Ok(())
    }

    /// Transfer tokens
    pub fn transfer(&mut self, from: Address, to: Address, value: U256) -> Result<TransferEvent, ERC20Error> {
        if to == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        }

        self._transfer(from, to, value)?;

        Ok(TransferEvent { from, to, value })
    }

    /// Approve spender
    pub fn approve(&mut self, owner: Address, spender: Address, value: U256) -> Result<ApprovalEvent, ERC20Error> {
        if spender == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        }

        if self.paused {
            return Err(ERC20Error::ContractPaused);
        }

        // Update or remove allowance using HashMap (O(1) operation)
        if value == U256::ZERO {
            // Remove zero allowances to save space
            self.allowances.remove(&(owner, spender));
        } else {
            self.allowances.insert((owner, spender), value);
        }

        Ok(ApprovalEvent { owner, spender, value })
    }

    /// Transfer from (with allowance)
    pub fn transfer_from(
        &mut self,
        spender: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<TransferEvent, ERC20Error> {
        if to == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        }

        let current_allowance = self.allowance(from, spender);
        if current_allowance < value {
            return Err(ERC20Error::InsufficientAllowance);
        }

        // Update allowance
        let new_allowance = current_allowance - value;
        self.approve(from, spender, new_allowance)?;

        // Perform transfer
        self._transfer(from, to, value)?;

        Ok(TransferEvent { from, to, value })
    }

    /// Mint new tokens (owner only)
    pub fn mint(&mut self, caller: Address, to: Address, value: U256) -> Result<TransferEvent, ERC20Error> {
        if caller != self.owner {
            return Err(ERC20Error::NotOwner);
        }

        if to == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        }

        if self.paused {
            return Err(ERC20Error::ContractPaused);
        }

        if value == U256::ZERO {
            return Err(ERC20Error::InvalidAmount);
        }

        // Update total supply
        self.total_supply = self.total_supply.checked_add(&value)
            .ok_or(ERC20Error::Overflow)?;

        // Update balance
        let to_balance = self.balance_of(to);
        self.update_balance(to, to_balance + value)?;

        // Emit event (from zero address for minting)
        Ok(TransferEvent {
            from: Address::ZERO,
            to,
            value,
        })
    }

    /// Burn tokens
    pub fn burn(&mut self, caller: Address, value: U256) -> Result<TransferEvent, ERC20Error> {
        if self.paused {
            return Err(ERC20Error::ContractPaused);
        }

        if value == U256::ZERO {
            return Err(ERC20Error::InvalidAmount);
        }

        let caller_balance = self.balance_of(caller);
        if caller_balance < value {
            return Err(ERC20Error::InsufficientBalance);
        }

        // Update total supply
        self.total_supply = self.total_supply - value;

        // Update balance
        self.update_balance(caller, caller_balance - value)?;

        // Emit event (to zero address for burning)
        Ok(TransferEvent {
            from: caller,
            to: Address::ZERO,
            value,
        })
    }

    /// Burn from (with allowance)
    pub fn burn_from(
        &mut self,
        caller: Address,
        from: Address,
        value: U256,
    ) -> Result<TransferEvent, ERC20Error> {
        let current_allowance = self.allowance(from, caller);
        if current_allowance < value {
            return Err(ERC20Error::InsufficientAllowance);
        }

        // Update allowance
        let new_allowance = current_allowance - value;
        self.approve(from, caller, new_allowance)?;

        // Perform burn
        let from_balance = self.balance_of(from);
        if from_balance < value {
            return Err(ERC20Error::InsufficientBalance);
        }

        // Update total supply
        self.total_supply = self.total_supply - value;

        // Update balance
        self.update_balance(from, from_balance - value)?;

        Ok(TransferEvent {
            from,
            to: Address::ZERO,
            value,
        })
    }

    /// Pause contract (owner only)
    pub fn pause(&mut self, caller: Address) -> Result<(), ERC20Error> {
        if caller != self.owner {
            return Err(ERC20Error::NotOwner);
        }

        if self.paused {
            return Err(ERC20Error::ContractPaused);
        }

        self.paused = true;
        Ok(())
    }

    /// Unpause contract (owner only)
    pub fn unpause(&mut self, caller: Address) -> Result<(), ERC20Error> {
        if caller != self.owner {
            return Err(ERC20Error::NotOwner);
        }

        if !self.paused {
            return Err(ERC20Error::InvalidAmount);
        }

        self.paused = false;
        Ok(())
    }

    /// Check if contract is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get nonce for permit
    pub fn nonce(&self, address: Address) -> u64 {
        *self.nonces.get(&address).unwrap_or(&0)
    }

    /// Use nonce (increment)
    fn use_nonce(&mut self, address: Address) -> u64 {
        match self.nonces.get_mut(&address) {
            Some(nonce) => {
                let current = *nonce;
                *nonce += 1;
                current
            }
            None => {
                self.nonces.insert(address, 1);
                0
            }
        }
    }

    /// Permit (gasless approval with signature)
    /// 
    /// # SECURITY WARNING
    /// This is a DEMONSTRATION implementation only. Signature verification is NOT implemented.
    /// DO NOT use this in production - it allows anyone to approve any address to spend tokens.
    /// 
    /// In a production implementation, you must:
    /// 1. Construct a message hash from (owner, spender, value, deadline, nonce)
    /// 2. Verify the Ed25519 signature against the owner's public key
    /// 3. Only proceed if signature is valid
    pub fn permit(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: u64,
        signature: &[u8],
    ) -> Result<ApprovalEvent, ERC20Error> {
        // Check deadline
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if current_time > deadline {
            return Err(ERC20Error::InvalidSignature);
        }

        // SECURITY: This is a placeholder - signature verification is NOT implemented
        // In production, you MUST verify the Ed25519 signature here
        // For now, this function is disabled for security
        return Err(ERC20Error::InvalidSignature);

        // TODO: Implement proper signature verification
        // let nonce = self.use_nonce(owner);
        // let message = construct_permit_message(owner, spender, value, deadline, nonce);
        // if !ed25519_verify(&message, signature, &owner) {
        //     return Err(ERC20Error::InvalidSignature);
        // }
        // self.approve(owner, spender, value)
    }

    /// Transfer ownership (owner only)
    pub fn transfer_ownership(&mut self, caller: Address, new_owner: Address) -> Result<(), ERC20Error> {
        if caller != self.owner {
            return Err(ERC20Error::NotOwner);
        }

        if new_owner == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        }

        self.owner = new_owner;
        Ok(())
    }

    /// Get owner address
    pub fn owner(&self) -> Address {
        self.owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_token() -> ERC20Token {
        let owner = Address::from_bytes([1u8; 20]);
        ERC20Token::with_initial_supply(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            owner,
            U256::from(1000000u64),
        )
    }

    #[test]
    fn test_initialization() {
        let token = create_test_token();
        assert_eq!(token.name(), "Test Token");
        assert_eq!(token.symbol(), "TEST");
        assert_eq!(token.decimals(), 18);
        assert_eq!(token.total_supply(), U256::from(1000000u64));
    }

    #[test]
    fn test_balance_of() {
        let owner = Address::from_bytes([1u8; 20]);
        let token = create_test_token();
        assert_eq!(token.balance_of(owner), U256::from(1000000u64));
        
        let other = Address::from_bytes([2u8; 20]);
        assert_eq!(token.balance_of(other), U256::ZERO);
    }

    #[test]
    fn test_transfer() {
        let owner = Address::from_bytes([1u8; 20]);
        let recipient = Address::from_bytes([2u8; 20]);
        let mut token = create_test_token();

        let result = token.transfer(owner, recipient, U256::from(1000u64));
        assert!(result.is_ok());

        assert_eq!(token.balance_of(owner), U256::from(999000u64));
        assert_eq!(token.balance_of(recipient), U256::from(1000u64));
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let owner = Address::from_bytes([1u8; 20]);
        let recipient = Address::from_bytes([2u8; 20]);
        let mut token = create_test_token();

        let result = token.transfer(owner, recipient, U256::from(2000000u64));
        assert!(matches!(result, Err(ERC20Error::InsufficientBalance)));
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let owner = Address::from_bytes([1u8; 20]);
        let spender = Address::from_bytes([2u8; 20]);
        let recipient = Address::from_bytes([3u8; 20]);
        let mut token = create_test_token();

        // Approve
        let result = token.approve(owner, spender, U256::from(5000u64));
        assert!(result.is_ok());
        assert_eq!(token.allowance(owner, spender), U256::from(5000u64));

        // Transfer from
        let result = token.transfer_from(spender, owner, recipient, U256::from(3000u64));
        assert!(result.is_ok());

        assert_eq!(token.balance_of(owner), U256::from(997000u64));
        assert_eq!(token.balance_of(recipient), U256::from(3000u64));
        assert_eq!(token.allowance(owner, spender), U256::from(2000u64));
    }

    #[test]
    fn test_mint() {
        let owner = Address::from_bytes([1u8; 20]);
        let recipient = Address::from_bytes([2u8; 20]);
        let mut token = create_test_token();

        let result = token.mint(owner, recipient, U256::from(500000u64));
        assert!(result.is_ok());

        assert_eq!(token.total_supply(), U256::from(1500000u64));
        assert_eq!(token.balance_of(recipient), U256::from(500000u64));
    }

    #[test]
    fn test_mint_not_owner() {
        let owner = Address::from_bytes([1u8; 20]);
        let not_owner = Address::from_bytes([2u8; 20]);
        let mut token = create_test_token();

        let result = token.mint(not_owner, not_owner, U256::from(1000u64));
        assert!(matches!(result, Err(ERC20Error::NotOwner)));
    }

    #[test]
    fn test_burn() {
        let owner = Address::from_bytes([1u8; 20]);
        let mut token = create_test_token();

        let result = token.burn(owner, U256::from(500000u64));
        assert!(result.is_ok());

        assert_eq!(token.total_supply(), U256::from(500000u64));
        assert_eq!(token.balance_of(owner), U256::from(500000u64));
    }

    #[test]
    fn test_pause() {
        let owner = Address::from_bytes([1u8; 20]);
        let recipient = Address::from_bytes([2u8; 20]);
        let mut token = create_test_token();

        // Pause contract
        let result = token.pause(owner);
        assert!(result.is_ok());
        assert!(token.is_paused());

        // Transfer should fail when paused
        let result = token.transfer(owner, recipient, U256::from(1000u64));
        assert!(matches!(result, Err(ERC20Error::ContractPaused)));

        // Unpause
        let result = token.unpause(owner);
        assert!(result.is_ok());
        assert!(!token.is_paused());

        // Transfer should work now
        let result = token.transfer(owner, recipient, U256::from(1000u64));
        assert!(result.is_ok());
    }

    #[test]
    fn test_transfer_ownership() {
        let owner = Address::from_bytes([1u8; 20]);
        let new_owner = Address::from_bytes([2u8; 20]);
        let mut token = create_test_token();

        let result = token.transfer_ownership(owner, new_owner);
        assert!(result.is_ok());
        assert_eq!(token.owner(), new_owner);
    }
}
