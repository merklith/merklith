//! ERC721 NFT Contract Example
//! 
//! Non-Fungible Token implementation for MERKLITH blockchain.
//! Full ERC721 standard with metadata and enumeration support.

use borsh::{BorshSerialize, BorshDeserialize};
use merklith_types::Address;

/// ERC721 Token Contract State
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ERC721Token {
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Contract owner
    pub owner: Address,
    /// Token URI base
    pub base_uri: String,
    /// Token ID counter
    pub token_counter: u64,
    /// Token ID -> Owner
    pub owners: Vec<(u64, Address)>,
    /// Owner -> Balance
    pub balances: Vec<(Address, u64)>,
    /// Token ID -> Approved address
    pub token_approvals: Vec<(u64, Address)>,
    /// Owner -> Operator -> Approved
    pub operator_approvals: Vec<(Address, Vec<(Address, bool)>)>,
    /// Token ID -> Token URI
    pub token_uris: Vec<(u64, String)>,
    /// Paused state
    pub paused: bool,
}

/// ERC721 Transfer Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub token_id: u64,
}

/// ERC721 Approval Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ApprovalEvent {
    pub owner: Address,
    pub approved: Address,
    pub token_id: u64,
}

/// ERC721 ApprovalForAll Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ApprovalForAllEvent {
    pub owner: Address,
    pub operator: Address,
    pub approved: bool,
}

/// ERC721 Error Types
#[derive(Debug, Clone, PartialEq)]
pub enum ERC721Error {
    /// Invalid token ID
    InvalidTokenId,
    /// Not the owner
    NotOwner,
    /// Not approved
    NotApproved,
    /// Transfer to zero address
    TransferToZero,
    /// Token already minted
    AlreadyMinted,
    /// Token not minted
    NotMinted,
    /// Contract is paused
    ContractPaused,
    /// Self approval
    SelfApproval,
    /// Invalid recipient
    InvalidRecipient,
}

impl std::fmt::Display for ERC721Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ERC721Error::InvalidTokenId => write!(f, "Invalid token ID"),
            ERC721Error::NotOwner => write!(f, "Not the token owner"),
            ERC721Error::NotApproved => write!(f, "Not approved for this token"),
            ERC721Error::TransferToZero => write!(f, "Cannot transfer to zero address"),
            ERC721Error::AlreadyMinted => write!(f, "Token already minted"),
            ERC721Error::NotMinted => write!(f, "Token not minted"),
            ERC721Error::ContractPaused => write!(f, "Contract is paused"),
            ERC721Error::SelfApproval => write!(f, "Cannot approve self"),
            ERC721Error::InvalidRecipient => write!(f, "Invalid recipient"),
        }
    }
}

impl std::error::Error for ERC721Error {}

impl ERC721Token {
    /// Create new NFT contract
    pub fn new(name: String, symbol: String, owner: Address) -> Self {
        Self {
            name,
            symbol,
            owner,
            base_uri: String::new(),
            token_counter: 0,
            owners: Vec::new(),
            balances: Vec::new(),
            token_approvals: Vec::new(),
            operator_approvals: Vec::new(),
            token_uris: Vec::new(),
            paused: false,
        }
    }

    /// Set base URI (owner only)
    pub fn set_base_uri(
        &mut self, caller: Address, uri: String) -> Result<(), ERC721Error> {
        if caller != self.owner {
            return Err(ERC721Error::NotOwner);
        }
        self.base_uri = uri;
        Ok(())
    }

    /// Get token name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get token symbol
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get owner of token
    pub fn owner_of(&self, token_id: u64) -> Result<Address, ERC721Error> {
        self.owners
            .iter()
            .find(|(id, _)| *id == token_id)
            .map(|(_, addr)| *addr)
            .ok_or(ERC721Error::NotMinted)
    }

    /// Get balance of owner
    pub fn balance_of(&self, owner: Address) -> u64 {
        self.balances
            .iter()
            .find(|(addr, _)| *addr == owner)
            .map(|(_, balance)| *balance)
            .unwrap_or(0)
    }

    /// Get approved address for token
    pub fn get_approved(&self, token_id: u64) -> Option<Address> {
        self.token_approvals
            .iter()
            .find(|(id, _)| *id == token_id)
            .map(|(_, addr)| *addr)
    }

    /// Check if operator is approved for all
    pub fn is_approved_for_all(&self, owner: Address, operator: Address,
    ) -> bool {
        self.operator_approvals
            .iter()
            .find(|(addr, _)| *addr == owner)
            .and_then(|(_, ops)| {
                ops.iter().find(|(op, _)| *op == operator)
                    .map(|(_, approved)| *approved)
            })
            .unwrap_or(false)
    }

    /// Check if spender is approved
    fn is_approved(
        &self,
        spender: Address,
        token_id: u64,
    ) -> Result<bool, ERC721Error> {
        let owner = self.owner_of(token_id)?;
        
        Ok(spender == owner
            || self.get_approved(token_id) == Some(spender)
            || self.is_approved_for_all(owner, spender))
    }

    /// Transfer token (internal)
    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: u64,
    ) -> Result<(), ERC721Error> {
        if self.paused {
            return Err(ERC721Error::ContractPaused);
        }

        if to == Address::ZERO {
            return Err(ERC721Error::TransferToZero);
        }

        let owner = self.owner_of(token_id)?;
        if owner != from {
            return Err(ERC721Error::NotOwner);
        }

        // Clear approvals
        self.token_approvals.retain(|(id, _)| *id != token_id);

        // Update balances
        self.update_balance(from, -1)?;
        self.update_balance(to, 1)?;

        // Update owner
        if let Some(pos) = self.owners.iter().position(|(id, _)| *id == token_id) {
            self.owners[pos].1 = to;
        }

        Ok(())
    }

    /// Update balance
    fn update_balance(
        &mut self,
        owner: Address,
        delta: i64,
    ) -> Result<(), ERC721Error> {
        let current = self.balance_of(owner);
        
        if delta < 0 && current < (-delta) as u64 {
            return Err(ERC721Error::NotOwner);
        }

        let new_balance = if delta >= 0 {
            current + delta as u64
        } else {
            current - (-delta) as u64
        };

        if let Some(pos) = self.balances.iter().position(|(addr, _)| *addr == owner) {
            if new_balance == 0 {
                self.balances.remove(pos);
            } else {
                self.balances[pos].1 = new_balance;
            }
        } else if new_balance > 0 {
            self.balances.push((owner, new_balance));
        }

        Ok(())
    }

    /// Transfer from
    pub fn transfer_from(
        &mut self,
        caller: Address,
        from: Address,
        to: Address,
        token_id: u64,
    ) -> Result<TransferEvent, ERC721Error> {
        if !self.is_approved(caller, token_id)? {
            return Err(ERC721Error::NotApproved);
        }

        self._transfer(from, to, token_id)?;

        Ok(TransferEvent { from, to, token_id })
    }

    /// Safe transfer from
    pub fn safe_transfer_from(
        &mut self,
        caller: Address,
        from: Address,
        to: Address,
        token_id: u64,
    ) -> Result<TransferEvent, ERC721Error> {
        // Same as transfer_from for now
        // In real implementation, check if recipient is contract
        self.transfer_from(caller, from, to, token_id)
    }

    /// Approve address for token
    pub fn approve(
        &mut self,
        caller: Address,
        approved: Address,
        token_id: u64,
    ) -> Result<ApprovalEvent, ERC721Error> {
        if self.paused {
            return Err(ERC721Error::ContractPaused);
        }

        let owner = self.owner_of(token_id)?;
        if caller != owner && !self.is_approved_for_all(owner, caller) {
            return Err(ERC721Error::NotApproved);
        }

        if approved == owner {
            return Err(ERC721Error::SelfApproval);
        }

        // Update approval
        if let Some(pos) = self.token_approvals.iter().position(|(id, _)| *id == token_id) {
            self.token_approvals[pos].1 = approved;
        } else {
            self.token_approvals.push((token_id, approved));
        }

        Ok(ApprovalEvent {
            owner,
            approved,
            token_id,
        })
    }

    /// Set approval for all
    pub fn set_approval_for_all(
        &mut self,
        caller: Address,
        operator: Address,
        approved: bool,
    ) -> Result<ApprovalForAllEvent, ERC721Error> {
        if operator == caller {
            return Err(ERC721Error::SelfApproval);
        }

        // Update operator approval
        if let Some(pos) = self.operator_approvals.iter().position(|(addr, _)| *addr == caller) {
            let ops = &mut self.operator_approvals[pos].1;
            if let Some(op_pos) = ops.iter().position(|(op, _)| *op == operator) {
                ops[op_pos].1 = approved;
            } else {
                ops.push((operator, approved));
            }
        } else {
            self.operator_approvals.push((caller, vec![(operator, approved)]));
        }

        Ok(ApprovalForAllEvent {
            owner: caller,
            operator,
            approved,
        })
    }

    /// Mint new token (owner only)
    pub fn mint(
        &mut self,
        caller: Address,
        to: Address,
        uri: Option<String>,
    ) -> Result<TransferEvent, ERC721Error> {
        if caller != self.owner {
            return Err(ERC721Error::NotOwner);
        }

        if self.paused {
            return Err(ERC721Error::ContractPaused);
        }

        if to == Address::ZERO {
            return Err(ERC721Error::TransferToZero);
        }

        let token_id = self.token_counter;
        self.token_counter += 1;

        // Add owner
        self.owners.push((token_id, to));

        // Update balance
        self.update_balance(to, 1)?;

        // Set URI if provided
        if let Some(token_uri) = uri {
            self.token_uris.push((token_id, token_uri));
        }

        Ok(TransferEvent {
            from: Address::ZERO,
            to,
            token_id,
        })
    }

    /// Burn token
    pub fn burn(
        &mut self,
        caller: Address,
        token_id: u64,
    ) -> Result<TransferEvent, ERC721Error> {
        if self.paused {
            return Err(ERC721Error::ContractPaused);
        }

        let owner = self.owner_of(token_id)?;
        if caller != owner && !self.is_approved(caller, token_id)? {
            return Err(ERC721Error::NotApproved);
        }

        // Clear approvals
        self.token_approvals.retain(|(id, _)| *id != token_id);

        // Update balance
        self.update_balance(owner, -1)?;

        // Remove owner
        self.owners.retain(|(id, _)| *id != token_id);

        // Remove URI
        self.token_uris.retain(|(id, _)| *id != token_id);

        Ok(TransferEvent {
            from: owner,
            to: Address::ZERO,
            token_id,
        })
    }

    /// Get token URI
    pub fn token_uri(&self, token_id: u64) -> Result<String, ERC721Error> {
        self.owner_of(token_id)?; // Check if token exists

        if let Some((_, uri)) = self.token_uris.iter().find(|(id, _)| *id == token_id) {
            Ok(uri.clone())
        } else if !self.base_uri.is_empty() {
            Ok(format!("{}{}", self.base_uri, token_id))
        } else {
            Ok(String::new())
        }
    }

    /// Get total supply
    pub fn total_supply(&self) -> u64 {
        self.owners.len() as u64
    }

    /// Check if token exists
    pub fn exists(&self, token_id: u64) -> bool {
        self.owners.iter().any(|(id, _)| *id == token_id)
    }

    /// Pause contract (owner only)
    pub fn pause(&mut self, caller: Address) -> Result<(), ERC721Error> {
        if caller != self.owner {
            return Err(ERC721Error::NotOwner);
        }
        self.paused = true;
        Ok(())
    }

    /// Unpause contract (owner only)
    pub fn unpause(&mut self, caller: Address) -> Result<(), ERC721Error> {
        if caller != self.owner {
            return Err(ERC721Error::NotOwner);
        }
        self.paused = false;
        Ok(())
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get token by index (for enumeration)
    pub fn token_by_index(&self, index: u64) -> Option<u64> {
        self.owners.get(index as usize).map(|(id, _)| *id)
    }

    /// Get token of owner by index
    pub fn token_of_owner_by_index(
        &self,
        owner: Address,
        index: u64,
    ) -> Option<u64> {
        self.owners
            .iter()
            .filter(|(_, addr)| *addr == owner)
            .nth(index as usize)
            .map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_nft() -> ERC721Token {
        let owner = Address::from_bytes([1u8; 20]);
        ERC721Token::new("Test NFT".to_string(), "TNFT".to_string(), owner)
    }

    #[test]
    fn test_initialization() {
        let nft = create_test_nft();
        assert_eq!(nft.name(), "Test NFT");
        assert_eq!(nft.symbol(), "TNFT");
        assert_eq!(nft.total_supply(), 0);
    }

    #[test]
    fn test_mint() {
        let owner = Address::from_bytes([1u8; 20]);
        let recipient = Address::from_bytes([2u8; 20]);
        let mut nft = create_test_nft();

        let result = nft.mint(owner, recipient, Some("ipfs://test/1".to_string()));
        assert!(result.is_ok());

        assert_eq!(nft.total_supply(), 1);
        assert_eq!(nft.balance_of(recipient), 1);
        assert_eq!(nft.owner_of(0).unwrap(), recipient);
    }

    #[test]
    fn test_transfer() {
        let owner = Address::from_bytes([1u8; 20]);
        let recipient = Address::from_bytes([2u8; 20]);
        let buyer = Address::from_bytes([3u8; 20]);
        let mut nft = create_test_nft();

        // Mint to owner
        nft.mint(owner, owner, None).unwrap();

        // Transfer
        let result = nft.transfer_from(owner, owner, recipient, 0);
        assert!(result.is_ok());

        assert_eq!(nft.balance_of(owner), 0);
        assert_eq!(nft.balance_of(recipient), 1);
        assert_eq!(nft.owner_of(0).unwrap(), recipient);
    }

    #[test]
    fn test_approve() {
        let owner = Address::from_bytes([1u8; 20]);
        let approved = Address::from_bytes([2u8; 20]);
        let mut nft = create_test_nft();

        nft.mint(owner, owner, None).unwrap();

        let result = nft.approve(owner, approved, 0);
        assert!(result.is_ok());
        assert_eq!(nft.get_approved(0), Some(approved));
    }

    #[test]
    fn test_approve_all() {
        let owner = Address::from_bytes([1u8; 20]);
        let operator = Address::from_bytes([2u8; 20]);
        let mut nft = create_test_nft();

        let result = nft.set_approval_for_all(owner, operator, true);
        assert!(result.is_ok());
        assert!(nft.is_approved_for_all(owner, operator));
    }

    #[test]
    fn test_burn() {
        let owner = Address::from_bytes([1u8; 20]);
        let mut nft = create_test_nft();

        nft.mint(owner, owner, None).unwrap();
        assert_eq!(nft.total_supply(), 1);

        let result = nft.burn(owner, 0);
        assert!(result.is_ok());
        assert_eq!(nft.total_supply(), 0);
        assert!(!nft.exists(0));
    }

    #[test]
    fn test_token_uri() {
        let owner = Address::from_bytes([1u8; 20]);
        let mut nft = create_test_nft();

        nft.mint(owner, owner, Some("custom://uri".to_string())).unwrap();

        let uri = nft.token_uri(0).unwrap();
        assert_eq!(uri, "custom://uri");
    }

    #[test]
    fn test_pause() {
        let owner = Address::from_bytes([1u8; 20]);
        let recipient = Address::from_bytes([2u8; 20]);
        let mut nft = create_test_nft();

        nft.mint(owner, owner, None).unwrap();
        nft.pause(owner).unwrap();

        // Should not be able to transfer while paused
        let result = nft.transfer_from(owner, owner, recipient, 0);
        assert!(matches!(result, Err(ERC721Error::ContractPaused)));
    }
}
