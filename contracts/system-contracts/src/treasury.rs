//! Treasury Contract
//!
//! Manages protocol funds and fee distribution.

use merklith_types::{Address, U256};
use std::collections::HashMap;

/// Treasury state.
#[derive(Debug)]
pub struct TreasuryContract {
    /// Treasury balance
    balance: U256,
    /// Authorized spenders
    authorized: HashMap<Address, bool>,
    /// Spending limits (spender -> limit)
    limits: HashMap<Address, U256>,
    /// Spending proposals
    proposals: Vec<SpendingProposal>,
    /// Fee recipients
    fee_recipients: Vec<(Address, u16)>, // (address, basis points)
}

/// Spending proposal.
#[derive(Debug, Clone)]
pub struct SpendingProposal {
    pub id: u64,
    pub recipient: Address,
    pub amount: U256,
    pub description: String,
    pub approved: bool,
    pub executed: bool,
}

impl TreasuryContract {
    /// Create new treasury.
    pub fn new() -> Self {
        Self {
            balance: U256::ZERO,
            authorized: HashMap::new(),
            limits: HashMap::new(),
            proposals: vec![],
            fee_recipients: vec![],
        }
    }

    /// Deposit funds.
    pub fn deposit(
        &mut self,
        amount: U256,
    ) {
        self.balance += amount;
    }

    /// Authorize a spender.
    pub fn authorize(
        &mut self,
        spender: Address,
        limit: U256,
    ) {
        self.authorized.insert(spender, true);
        self.limits.insert(spender, limit);
    }

    /// Revoke authorization.
    pub fn revoke(
        &mut self,
        spender: Address,
    ) {
        self.authorized.insert(spender, false);
    }

    /// Create spending proposal.
    pub fn propose_spending(
        &mut self,
        id: u64,
        recipient: Address,
        amount: U256,
        description: String,
    ) -> Result<(), String> {
        if amount > self.balance {
            return Err("Insufficient balance".to_string());
        }

        let proposal = SpendingProposal {
            id,
            recipient,
            amount,
            description,
            approved: false,
            executed: false,
        };

        self.proposals.push(proposal);
        Ok(())
    }

    /// Approve spending proposal.
    pub fn approve_spending(
        &mut self,
        id: u64,
        approver: Address,
    ) -> Result<(), String> {
        if !self.authorized.get(&approver).copied().unwrap_or(false) {
            return Err("Not authorized".to_string());
        }

        let proposal = self.proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or("Proposal not found")?;

        proposal.approved = true;
        Ok(())
    }

    /// Execute approved spending.
    pub fn execute_spending(
        &mut self,
        id: u64,
    ) -> Result<(), String> {
        let proposal = self.proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or("Proposal not found")?;

        if !proposal.approved {
            return Err("Not approved".to_string());
        }

        if proposal.executed {
            return Err("Already executed".to_string());
        }

        if proposal.amount > self.balance {
            return Err("Insufficient balance".to_string());
        }

        self.balance -= proposal.amount;
        proposal.executed = true;

        // In real implementation, would transfer to recipient
        Ok(())
    }

    /// Add fee recipient.
    pub fn add_fee_recipient(
        &mut self,
        recipient: Address,
        share_bps: u16,
    ) {
        self.fee_recipients.push((recipient, share_bps));
    }

    /// Distribute fees.
    pub fn distribute_fees(
        &mut self,
        amount: U256,
    ) -> Result<(), String> {
        if amount > self.balance {
            return Err("Insufficient balance".to_string());
        }

        let total_bps: u16 = self.fee_recipients.iter()
            .map(|(_, bps)| bps)
            .sum();

        if total_bps > 10000 {
            return Err("Invalid fee distribution".to_string());
        }

        for (recipient, bps) in &self.fee_recipients {
            let share = amount * U256::from(*bps) / U256::from(10000);
            // In real implementation, would transfer to recipient
        }

        Ok(())
    }

    /// Get balance.
    pub fn balance(&self) -> U256 {
        self.balance
    }

    /// Get spending limit for address.
    pub fn get_limit(&self,
        spender: &Address,
    ) -> U256 {
        self.limits.get(spender).copied().unwrap_or(U256::ZERO)
    }
}

impl Default for TreasuryContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit() {
        let mut treasury = TreasuryContract::new();
        let amount = U256::from(1000u64);
        
        treasury.deposit(amount);
        assert_eq!(treasury.balance(), amount);
    }

    #[test]
    fn test_authorize() {
        let mut treasury = TreasuryContract::new();
        let spender = Address::ZERO;
        let limit = U256::from(500u64);
        
        treasury.authorize(spender, limit);
        assert_eq!(treasury.get_limit(&spender), limit);
    }

    #[test]
    fn test_propose_spending() {
        let mut treasury = TreasuryContract::new();
        treasury.deposit(U256::from(1000u64));
        
        let result = treasury.propose_spending(
            1,
            Address::from_bytes([1u8; 20]),
            U256::from(500u64),
            "Test".to_string(),
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_propose_exceeds_balance() {
        let mut treasury = TreasuryContract::new();
        
        let result = treasury.propose_spending(
            1,
            Address::ZERO,
            U256::from(1000u64),
            "Test".to_string(),
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_add_fee_recipient() {
        let mut treasury = TreasuryContract::new();
        treasury.add_fee_recipient(Address::ZERO, 5000); // 50%
        
        assert_eq!(treasury.fee_recipients.len(), 1);
    }
}
