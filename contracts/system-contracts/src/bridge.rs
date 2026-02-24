//! Bridge Contract
//!
//! Cross-chain bridge for asset transfers.

use merklith_types::{Address, Hash, U256};
use std::collections::HashMap;

/// Bridge state.
#[derive(Debug)]
pub struct BridgeContract {
    /// Supported chains
    supported_chains: HashMap<u64, ChainInfo>,
    /// Pending transfers
    pending_transfers: HashMap<Hash, Transfer>,
    /// Completed transfers
    completed_transfers: HashMap<Hash, bool>,
    /// Validators
    validators: HashMap<Address, bool>,
    /// Required signatures
    required_signatures: u32,
    /// Transfer nonces
    nonces: HashMap<Address, u64>,
}

/// Chain info.
#[derive(Debug, Clone)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub bridge_address: [u8; 20],
    pub enabled: bool,
}

/// Transfer request.
#[derive(Debug, Clone)]
pub struct Transfer {
    pub id: Hash,
    pub sender: Address,
    pub recipient: [u8; 20],
    pub amount: U256,
    pub target_chain: u64,
    pub nonce: u64,
    pub signatures: Vec<([u8; 32], [u8; 32])>, // (r, s)
}

impl BridgeContract {
    /// Create new bridge.
    pub fn new(required_signatures: u32) -> Self {
        Self {
            supported_chains: HashMap::new(),
            pending_transfers: HashMap::new(),
            completed_transfers: HashMap::new(),
            validators: HashMap::new(),
            required_signatures,
            nonces: HashMap::new(),
        }
    }

    /// Add supported chain.
    pub fn add_chain(
        &mut self,
        chain_id: u64,
        bridge_address: [u8; 20],
    ) {
        let info = ChainInfo {
            chain_id,
            bridge_address,
            enabled: true,
        };
        self.supported_chains.insert(chain_id, info);
    }

    /// Remove supported chain.
    pub fn remove_chain(
        &mut self,
        chain_id: u64,
    ) {
        self.supported_chains.remove(&chain_id);
    }

    /// Add validator.
    pub fn add_validator(
        &mut self,
        validator: Address,
    ) {
        self.validators.insert(validator, true);
    }

    /// Remove validator.
    pub fn remove_validator(
        &mut self,
        validator: Address,
    ) {
        self.validators.remove(&validator);
    }

    /// Initiate transfer.
    pub fn initiate_transfer(
        &mut self,
        sender: Address,
        recipient: [u8; 20],
        amount: U256,
        target_chain: u64,
    ) -> Result<Hash, String> {
        // Check chain is supported
        let chain = self.supported_chains
            .get(&target_chain)
            .ok_or("Chain not supported")?;
        
        if !chain.enabled {
            return Err("Chain disabled".to_string());
        }

        // Get and increment nonce
        let nonce = *self.nonces.entry(sender).or_insert(0);
        self.nonces.insert(sender, nonce + 1);

        // Create transfer ID
        let id = self.generate_transfer_id(sender, recipient, amount, target_chain, nonce + 1);

        let transfer = Transfer {
            id,
            sender,
            recipient,
            amount,
            target_chain,
            nonce: nonce + 1,
            signatures: vec![],
        };

        self.pending_transfers.insert(id, transfer);

        Ok(id)
    }

    /// Sign transfer (by validator).
    pub fn sign_transfer(
        &mut self,
        transfer_id: Hash,
        validator: Address,
        signature: ([u8; 32], [u8; 32]),
    ) -> Result<(), String> {
        if !self.validators.contains_key(&validator) {
            return Err("Not a validator".to_string());
        }

        let transfer = self.pending_transfers
            .get_mut(&transfer_id)
            .ok_or("Transfer not found")?;

        transfer.signatures.push(signature);

        Ok(())
    }

    /// Complete transfer.
    pub fn complete_transfer(
        &mut self,
        transfer_id: Hash,
    ) -> Result<(), String> {
        let transfer = self.pending_transfers
            .get(&transfer_id)
            .ok_or("Transfer not found")?;

        if transfer.signatures.len() < self.required_signatures as usize {
            return Err("Insufficient signatures".to_string());
        }

        if self.completed_transfers.contains_key(&transfer_id) {
            return Err("Already completed".to_string());
        }

        // Mark as completed
        self.completed_transfers.insert(transfer_id, true);
        
        // In real implementation, would mint/release tokens on target chain
        
        Ok(())
    }

    /// Verify transfer is completed.
    pub fn is_completed(&self,
        transfer_id: &Hash,
    ) -> bool {
        self.completed_transfers.contains_key(transfer_id)
    }

    /// Get pending transfer.
    pub fn get_pending(&self,
        transfer_id: &Hash,
    ) -> Option<&Transfer> {
        self.pending_transfers.get(transfer_id)
    }

    /// Generate unique transfer ID.
    fn generate_transfer_id(
        &self,
        sender: Address,
        recipient: [u8; 20],
        amount: U256,
        target_chain: u64,
        nonce: u64,
    ) -> Hash {
        let mut data = Vec::new();
        data.extend_from_slice(sender.as_bytes());
        data.extend_from_slice(&recipient);
        data.extend_from_slice(&amount.to_be_bytes());
        data.extend_from_slice(&target_chain.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());

        let hash = blake3::hash(&data);
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        Hash::from_bytes(result)
    }

    /// Get nonce for address.
    pub fn get_nonce(&self,
        address: &Address,
    ) -> u64 {
        self.nonces.get(address).copied().unwrap_or(0)
    }

    /// Get validator count.
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Get supported chains.
    pub fn supported_chains(&self) -> Vec<&ChainInfo> {
        self.supported_chains.values().collect()
    }
}

impl Default for BridgeContract {
    fn default() -> Self {
        Self::new(3) // Default 3 signatures required
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_chain() {
        let mut bridge = BridgeContract::default();
        bridge.add_chain(1, [0u8; 20]);
        
        assert_eq!(bridge.supported_chains().len(), 1);
    }

    #[test]
    fn test_initiate_transfer() {
        let mut bridge = BridgeContract::default();
        bridge.add_chain(1, [0u8; 20]);
        
        let sender = Address::ZERO;
        let recipient = [1u8; 20];
        let amount = U256::from(1000u64);
        
        let id = bridge.initiate_transfer(sender, recipient, amount, 1).unwrap();
        
        assert!(bridge.get_pending(&id).is_some());
        assert_eq!(bridge.get_nonce(&sender), 1);
    }

    #[test]
    fn test_sign_and_complete() {
        let mut bridge = BridgeContract::default();
        bridge.add_chain(1, [0u8; 20]);
        
        let validator = Address::from_bytes([1u8; 20]);
        bridge.add_validator(validator);
        
        let sender = Address::ZERO;
        let id = bridge.initiate_transfer(sender, [2u8; 20], U256::from(100), 1).unwrap();
        
        // Sign transfer
        let sig = ([0u8; 32], [0u8; 32]);
        bridge.sign_transfer(id, validator, sig).unwrap();
        
        // Complete (if enough signatures)
        // In this test, required_signatures is 3, so this should fail
        assert!(bridge.complete_transfer(id).is_err());
        
        // Add more signatures
        for i in 2..=3 {
            let v = Address::from_bytes([i as u8; 20]);
            bridge.add_validator(v);
            bridge.sign_transfer(id, v, sig).unwrap();
        }
        
        // Now should succeed
        assert!(bridge.complete_transfer(id).is_ok());
        assert!(bridge.is_completed(&id));
    }

    #[test]
    fn test_unsupported_chain() {
        let mut bridge = BridgeContract::default();
        
        let result = bridge.initiate_transfer(Address::ZERO, [1u8; 20], U256::from(100), 999);
        
        assert!(result.is_err());
    }
}
