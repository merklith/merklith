//! Cross-Chain Bridge Contract
//! 
//! Bridge assets between MERKLITH and other blockchains.
//! Features:
//! - Lock and mint mechanism
//! - Multi-sig validators
//! - Replay protection
//! - Emergency pause
//! - Fee management

use borsh::{BorshSerialize, BorshDeserialize};
use merklith_types::{Address, U256, Hash};

/// Bridge Contract State
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BridgeContract {
    /// Contract owner
    pub owner: Address,
    /// Minimum validators required
    pub min_validators: u64,
    /// Validators list
    pub validators: Vec<Address>,
    /// Supported chains: chain_id -> is_supported
    pub supported_chains: Vec<(u64, bool)>,
    /// Supported tokens: token -> is_supported
    pub supported_tokens: Vec<(Address, bool)>,
    /// Wrapped tokens: original_chain -> original_token -> wrapped_token
    pub wrapped_tokens: Vec<((u64, Address), Address)>,
    /// Locked tokens: (chain_id, token, user) -> amount
    pub locked_tokens: Vec<((u64, Address, Address), U256)>,
    /// Processed transactions (replay protection)
    pub processed_txs: Vec<(Hash, bool)>,
    /// Bridge fees (in bps, 100 = 1%)
    pub bridge_fee: u64,
    /// Fee recipient
    pub fee_recipient: Address,
    /// Daily limits: (chain_id, token) -> amount
    pub daily_limits: Vec<((u64, Address), U256)>,
    /// Daily transferred: (chain_id, token, day) -> amount
    pub daily_transferred: Vec<((u64, Address, u64), U256)>,
    /// Paused state
    pub paused: bool,
    /// Emergency stop
    pub emergency_stopped: bool,
    /// Bridge nonce for transactions
    pub nonce: u64,
}

/// Bridge Event
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BridgeEvent {
    pub direction: BridgeDirection,
    pub from_chain: u64,
    pub to_chain: u64,
    pub token: Address,
    pub amount: U256,
    pub sender: Address,
    pub recipient: Address,
    pub tx_hash: Hash,
    pub nonce: u64,
}

/// Bridge Direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum BridgeDirection {
    Lock,   // Lock on source, mint on destination
    Unlock, // Burn on source, unlock on destination
}

/// Validator Signature
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ValidatorSignature {
    pub validator: Address,
    pub signature: Vec<u8>,
}

/// Bridge Request
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BridgeRequest {
    pub from_chain: u64,
    pub to_chain: u64,
    pub token: Address,
    pub amount: U256,
    pub sender: Address,
    pub recipient: Address,
    pub nonce: u64,
    pub signatures: Vec<ValidatorSignature>,
}

/// Bridge Error Types
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeError {
    /// Not owner
    NotOwner,
    /// Not validator
    NotValidator,
    /// Chain not supported
    ChainNotSupported,
    /// Token not supported
    TokenNotSupported,
    /// Insufficient signatures
    InsufficientSignatures,
    /// Invalid signature
    InvalidSignature,
    /// Transaction already processed
    AlreadyProcessed,
    /// Daily limit exceeded
    DailyLimitExceeded,
    /// Amount too low (must be greater than fee)
    AmountTooLow,
    /// Insufficient locked amount
    InsufficientLocked,
    /// Contract paused
    ContractPaused,
    /// Emergency stopped
    EmergencyStopped,
    /// Zero amount
    ZeroAmount,
    /// Zero address
    ZeroAddress,
    /// Overflow
    Overflow,
    /// Underflow
    Underflow,
    /// Divide by zero
    DivideByZero,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::NotOwner => write!(f, "Not contract owner"),
            BridgeError::NotValidator => write!(f, "Not a validator"),
            BridgeError::ChainNotSupported => write!(f, "Chain not supported"),
            BridgeError::TokenNotSupported => write!(f, "Token not supported"),
            BridgeError::InsufficientSignatures => write!(f, "Insufficient validator signatures"),
            BridgeError::InvalidSignature => write!(f, "Invalid signature"),
            BridgeError::AlreadyProcessed => write!(f, "Transaction already processed"),
            BridgeError::DailyLimitExceeded => write!(f, "Daily limit exceeded"),
            BridgeError::AmountTooLow => write!(f, "Amount too low (must be greater than fee)"),
            BridgeError::InsufficientLocked => write!(f, "Insufficient locked amount"),
            BridgeError::ContractPaused => write!(f, "Contract is paused"),
            BridgeError::EmergencyStopped => write!(f, "Emergency stop activated"),
            BridgeError::ZeroAmount => write!(f, "Amount must be greater than zero"),
            BridgeError::ZeroAddress => write!(f, "Zero address not allowed"),
            BridgeError::Overflow => write!(f, "Arithmetic overflow"),
            BridgeError::Underflow => write!(f, "Arithmetic underflow"),
            BridgeError::DivideByZero => write!(f, "Divide by zero"),
        }
    }
}

impl std::error::Error for BridgeError {}

impl BridgeContract {
    /// Create new bridge contract
    pub fn new(
        owner: Address,
        min_validators: u64,
        validators: Vec<Address>,
    ) -> Self {
        Self {
            owner,
            min_validators,
            validators,
            supported_chains: Vec::new(),
            supported_tokens: Vec::new(),
            wrapped_tokens: Vec::new(),
            locked_tokens: Vec::new(),
            processed_txs: Vec::new(),
            bridge_fee: 50, // 0.5%
            fee_recipient: owner,
            daily_limits: Vec::new(),
            daily_transferred: Vec::new(),
            paused: false,
            emergency_stopped: false,
            nonce: 0,
        }
    }

    /// Lock tokens on source chain
    pub fn lock(
        &mut self,
        caller: Address,
        token: Address,
        amount: U256,
        to_chain: u64,
        recipient: Address,
    ) -> Result<BridgeEvent, BridgeError> {
        self.check_active()?;

        if amount == U256::ZERO {
            return Err(BridgeError::ZeroAmount);
        }

        if recipient == Address::ZERO {
            return Err(BridgeError::ZeroAddress);
        }

        if !self.is_chain_supported(to_chain) {
            return Err(BridgeError::ChainNotSupported);
        }

        if !self.is_token_supported(token) {
            return Err(BridgeError::TokenNotSupported);
        }

        // Check daily limit
        self.check_daily_limit(to_chain, token, amount)?;

        // Calculate fee with minimum fee enforcement to prevent zero-fee transfers
        let min_fee = U256::from(1u64); // Minimum 1 unit fee
        let calculated_fee = amount
            .checked_mul(&U256::from(self.bridge_fee)).ok_or(BridgeError::Overflow)?
            .checked_div(&U256::from(10000u64)).ok_or(BridgeError::DivideByZero)?;
        
        // Ensure fee is at least min_fee (prevents zero-fee draining attacks)
        let fee = calculated_fee.max(min_fee);
        
        // Ensure amount is greater than fee
        if amount <= fee {
            return Err(BridgeError::AmountTooLow);
        }
        
        let net_amount = amount.checked_sub(&fee).ok_or(BridgeError::Underflow)?;

        // Lock tokens
        self.increase_locked(to_chain, token, caller, net_amount)?;

        // Update daily transferred
        self.increase_daily_transferred(to_chain, token, amount)?;

        // Generate nonce
        let nonce = self.nonce;
        self.nonce += 1;

        // Create transaction hash
        let tx_hash = self.generate_tx_hash(
            17001, // MERKLITH chain ID
            to_chain,
            token,
            amount,
            caller,
            recipient,
            nonce,
        );

        Ok(BridgeEvent {
            direction: BridgeDirection::Lock,
            from_chain: 17001,
            to_chain,
            token,
            amount: net_amount,
            sender: caller,
            recipient,
            tx_hash,
            nonce,
        })
    }

    /// Unlock tokens on destination chain (called by validators)
    pub fn unlock(
        &mut self,
        request: BridgeRequest,
    ) -> Result<BridgeEvent, BridgeError> {
        self.check_active()?;

        // Verify transaction not already processed
        let tx_hash = self.generate_tx_hash(
            request.from_chain,
            request.to_chain,
            request.token,
            request.amount,
            request.sender,
            request.recipient,
            request.nonce,
        );

        if self.is_processed(tx_hash) {
            return Err(BridgeError::AlreadyProcessed);
        }

        // Verify signatures
        self.verify_signatures(&request, tx_hash)?;

        // Mark as processed
        self.processed_txs.push((tx_hash, true));

        // Unlock tokens
        let wrapped_token = self.get_wrapped_token(request.from_chain, request.token)
            .ok_or(BridgeError::TokenNotSupported)?;

        // In production, mint wrapped tokens to recipient

        Ok(BridgeEvent {
            direction: BridgeDirection::Unlock,
            from_chain: request.from_chain,
            to_chain: request.to_chain,
            token: wrapped_token,
            amount: request.amount,
            sender: request.sender,
            recipient: request.recipient,
            tx_hash,
            nonce: request.nonce,
        })
    }

    /// Add validator (owner only)
    pub fn add_validator(
        &mut self,
        caller: Address,
        validator: Address,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        if !self.validators.contains(&validator) {
            self.validators.push(validator);
        }

        Ok(())
    }

    /// Remove validator (owner only)
    pub fn remove_validator(
        &mut self,
        caller: Address,
        validator: Address,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        self.validators.retain(|v| *v != validator);

        Ok(())
    }

    /// Add supported chain (owner only)
    pub fn add_chain(
        &mut self,
        caller: Address,
        chain_id: u64,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        if let Some(pos) = self.supported_chains.iter().position(|(id, _)| *id == chain_id) {
            self.supported_chains[pos].1 = true;
        } else {
            self.supported_chains.push((chain_id, true));
        }

        Ok(())
    }

    /// Add supported token (owner only)
    pub fn add_token(
        &mut self,
        caller: Address,
        token: Address,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        if let Some(pos) = self.supported_tokens.iter().position(|(t, _)| *t == token) {
            self.supported_tokens[pos].1 = true;
        } else {
            self.supported_tokens.push((token, true));
        }

        Ok(())
    }

    /// Register wrapped token (owner only)
    pub fn register_wrapped_token(
        &mut self,
        caller: Address,
        original_chain: u64,
        original_token: Address,
        wrapped_token: Address,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        self.wrapped_tokens.push(((original_chain, original_token), wrapped_token));

        Ok(())
    }

    /// Set daily limit (owner only)
    pub fn set_daily_limit(
        &mut self,
        caller: Address,
        chain_id: u64,
        token: Address,
        limit: U256,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        if let Some(pos) = self.daily_limits.iter().position(|((c, t), _)| *c == chain_id && *t == token) {
            self.daily_limits[pos].1 = limit;
        } else {
            self.daily_limits.push(((chain_id, token), limit));
        }

        Ok(())
    }

    /// Pause bridge (owner only)
    pub fn pause(
        &mut self,
        caller: Address,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        self.paused = true;
        Ok(())
    }

    /// Unpause bridge (owner only)
    pub fn unpause(
        &mut self,
        caller: Address,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        self.paused = false;
        Ok(())
    }

    /// Emergency stop (owner only)
    pub fn emergency_stop(
        &mut self,
        caller: Address,
    ) -> Result<(), BridgeError> {
        if caller != self.owner {
            return Err(BridgeError::NotOwner);
        }

        self.emergency_stopped = true;
        Ok(())
    }

    /// Check if active
    fn check_active(&self,
    ) -> Result<(), BridgeError> {
        if self.emergency_stopped {
            return Err(BridgeError::EmergencyStopped);
        }
        if self.paused {
            return Err(BridgeError::ContractPaused);
        }
        Ok(())
    }

    /// Check if chain is supported
    fn is_chain_supported(&self,
        chain_id: u64,
    ) -> bool {
        self.supported_chains
            .iter()
            .any(|(id, supported)| *id == chain_id && *supported)
    }

    /// Check if token is supported
    fn is_token_supported(&self,
        token: Address,
    ) -> bool {
        self.supported_tokens
            .iter()
            .any(|(t, supported)| *t == token && *supported)
    }

    /// Get wrapped token
    fn get_wrapped_token(
        &self,
        chain_id: u64,
        token: Address,
    ) -> Option<Address> {
        self.wrapped_tokens
            .iter()
            .find(|((c, t), _)| *c == chain_id && *t == token)
            .map(|(_, wrapped)| *wrapped)
    }

    /// Check daily limit
    fn check_daily_limit(
        &self,
        chain_id: u64,
        token: Address,
        amount: U256,
    ) -> Result<(), BridgeError> {
        let daily_limit = self.daily_limits
            .iter()
            .find(|((c, t), _)| *c == chain_id && *t == token)
            .map(|(_, limit)| *limit)
            .unwrap_or(U256::MAX);

        let day = Self::current_day();
        let transferred = self.get_daily_transferred(chain_id, token, day);

        let new_total = transferred.checked_add(&amount).ok_or(BridgeError::Overflow)?;
        
        if new_total > daily_limit {
            return Err(BridgeError::DailyLimitExceeded);
        }

        Ok(())
    }

    /// Get daily transferred amount
    fn get_daily_transferred(
        &self,
        chain_id: u64,
        token: Address,
        day: u64,
    ) -> U256 {
        self.daily_transferred
            .iter()
            .find(|((c, t, d), _)| *c == chain_id && *t == token && *d == day)
            .map(|(_, amount)| *amount)
            .unwrap_or(U256::ZERO)
    }

    /// Increase daily transferred
    fn increase_daily_transferred(
        &mut self,
        chain_id: u64,
        token: Address,
        amount: U256,
    ) -> Result<(), BridgeError> {
        let day = Self::current_day();
        
        if let Some(pos) = self.daily_transferred.iter().position(|((c, t, d), _)| {
            *c == chain_id && *t == token && *d == day
        }) {
            self.daily_transferred[pos].1 = self.daily_transferred[pos].1
                .checked_add(&amount).ok_or(BridgeError::Overflow)?;
        } else {
            self.daily_transferred.push(((chain_id, token, day), amount));
        }
        
        Ok(())
    }

    /// Increase locked tokens
    fn increase_locked(
        &mut self,
        chain_id: u64,
        token: Address,
        user: Address,
        amount: U256,
    ) -> Result<(), BridgeError> {
        let key = (chain_id, token, user);
        
        if let Some(pos) = self.locked_tokens.iter().position(|(k, _)| *k == key) {
            self.locked_tokens[pos].1 = self.locked_tokens[pos].1
                .checked_add(&amount).ok_or(BridgeError::Overflow)?;
        } else {
            self.locked_tokens.push((key, amount));
        }
        
        Ok(())
    }

    /// Check if transaction is processed
    fn is_processed(&self,
        tx_hash: Hash,
    ) -> bool {
        self.processed_txs
            .iter()
            .any(|(h, processed)| *h == tx_hash && *processed)
    }

    /// Verify validator signatures
    /// Verify bridge request signatures
    ///
    /// # SECURITY WARNING
    /// This is a DEMONSTRATION implementation only. Signature verification is NOT implemented.
    /// DO NOT use this in production - it allows anyone to forge bridge transfers.
    ///
    /// In a production implementation, you must:
    /// 1. Verify each validator's Ed25519 signature against the transaction hash
    /// 2. Ensure signatures are from distinct validators
    /// 3. Only proceed if min_validators valid signatures are present
    fn verify_signatures(
        &self,
        _request: &BridgeRequest,
        _tx_hash: Hash,
    ) -> Result<(), BridgeError> {
        // SECURITY: This is a placeholder - signature verification is NOT implemented
        // In production, you MUST verify Ed25519 signatures here
        // For now, this function always fails for security
        return Err(BridgeError::InsufficientSignatures);

        // TODO: Implement proper signature verification
        // if request.signatures.len() < self.min_validators as usize {
        //     return Err(BridgeError::InsufficientSignatures);
        // }
        //
        // let mut valid_count = 0;
        // let mut seen_validators = std::collections::HashSet::new();
        //
        // for sig in &request.signatures {
        //     if !self.validators.contains(&sig.validator) {
        //         return Err(BridgeError::NotValidator);
        //     }
        //
        //     // Check for duplicate validator
        //     if !seen_validators.insert(sig.validator) {
        //         return Err(BridgeError::DuplicateSignature);
        //     }
        //
        //     // Verify Ed25519 signature
        //     if !ed25519_verify(&tx_hash.as_bytes(), &sig.signature, &sig.validator) {
        //         return Err(BridgeError::InvalidSignature);
        //     }
        //
        //     valid_count += 1;
        // }
        //
        // if valid_count < self.min_validators as usize {
        //     return Err(BridgeError::InsufficientSignatures);
        // }
        //
        // Ok(())
    }

    /// Generate transaction hash
    fn generate_tx_hash(
        &self,
        from_chain: u64,
        to_chain: u64,
        token: Address,
        amount: U256,
        sender: Address,
        recipient: Address,
        nonce: u64,
    ) -> Hash {
        // In production: use proper hashing
        let mut data = Vec::new();
        data.extend_from_slice(&from_chain.to_be_bytes());
        data.extend_from_slice(&to_chain.to_be_bytes());
        data.extend_from_slice(token.as_bytes());
        data.extend_from_slice(&amount.to_be_bytes());
        data.extend_from_slice(sender.as_bytes());
        data.extend_from_slice(recipient.as_bytes());
        data.extend_from_slice(&nonce.to_be_bytes());
        
        Hash::compute(&data)
    }

    /// Get current day
    fn current_day() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() / 86400
    }

    /// Get locked amount
    pub fn get_locked_amount(
        &self,
        chain_id: u64,
        token: Address,
        user: Address,
    ) -> U256 {
        self.locked_tokens
            .iter()
            .find(|((c, t, u), _)| *c == chain_id && *t == token && *u == user)
            .map(|(_, amount)| *amount)
            .unwrap_or(U256::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_bridge() -> BridgeContract {
        let owner = Address::from_bytes([1u8; 20]);
        let validator1 = Address::from_bytes([2u8; 20]);
        let validator2 = Address::from_bytes([3u8; 20]);
        let validator3 = Address::from_bytes([4u8; 20]);
        
        BridgeContract::new(
            owner,
            2, // min validators
            vec![validator1, validator2, validator3],
        )
    }

    #[test]
    fn test_initialization() {
        let bridge = create_bridge();
        assert_eq!(bridge.min_validators, 2);
        assert_eq!(bridge.validators.len(), 3);
    }

    #[test]
    fn test_add_chain() {
        let mut bridge = create_bridge();
        let owner = bridge.owner;
        
        bridge.add_chain(owner, 1).unwrap(); // Ethereum
        assert!(bridge.is_chain_supported(1));
    }

    #[test]
    fn test_add_chain_not_owner() {
        let mut bridge = create_bridge();
        let not_owner = Address::from_bytes([5u8; 20]);
        
        let result = bridge.add_chain(not_owner, 1);
        assert!(matches!(result, Err(BridgeError::NotOwner)));
    }

    #[test]
    fn test_lock() {
        let mut bridge = create_bridge();
        let owner = bridge.owner;
        let user = Address::from_bytes([5u8; 20]);
        let token = Address::from_bytes([6u8; 20]);
        
        // Setup
        bridge.add_chain(owner, 1).unwrap();
        bridge.add_token(owner, token).unwrap();
        bridge.set_daily_limit(owner, 1, token, U256::from(10000u64)).unwrap();
        
        // Lock
        let result = bridge.lock(user, token, U256::from(100u64), 1, user);
        assert!(result.is_ok());
        
        let event = result.unwrap();
        assert_eq!(event.direction, BridgeDirection::Lock);
        // Fee is 0.5% = 0.5, but minimum fee is 1 to prevent zero-fee draining
        // So net_amount = 100 - 1 = 99
        assert_eq!(event.amount, U256::from(99u64));
    }

    #[test]
    fn test_lock_chain_not_supported() {
        let mut bridge = create_bridge();
        let user = Address::from_bytes([5u8; 20]);
        let token = Address::from_bytes([6u8; 20]);
        
        let result = bridge.lock(user, token, U256::from(100u64), 999, user);
        assert!(matches!(result, Err(BridgeError::ChainNotSupported)));
    }

    #[test]
    fn test_daily_limit() {
        let mut bridge = create_bridge();
        let owner = bridge.owner;
        let user = Address::from_bytes([5u8; 20]);
        let token = Address::from_bytes([6u8; 20]);
        
        // Setup
        bridge.add_chain(owner, 1).unwrap();
        bridge.add_token(owner, token).unwrap();
        bridge.set_daily_limit(owner, 1, token, U256::from(100u64)).unwrap();
        
        // First lock should work
        let result = bridge.lock(user, token, U256::from(50u64), 1, user);
        assert!(result.is_ok());
        
        // Second lock should work (total 100)
        let result = bridge.lock(user, token, U256::from(50u64), 1, user);
        assert!(result.is_ok());
        
        // Third lock should fail (limit exceeded)
        let result = bridge.lock(user, token, U256::from(10u64), 1, user);
        assert!(matches!(result, Err(BridgeError::DailyLimitExceeded)));
    }

    #[test]
    fn test_pause() {
        let mut bridge = create_bridge();
        let owner = bridge.owner;
        let user = Address::from_bytes([5u8; 20]);
        let token = Address::from_bytes([6u8; 20]);
        
        // Setup
        bridge.add_chain(owner, 1).unwrap();
        bridge.add_token(owner, token).unwrap();
        
        // Pause
        bridge.pause(owner).unwrap();
        
        // Try to lock (should fail)
        let result = bridge.lock(user, token, U256::from(100u64), 1, user);
        assert!(matches!(result, Err(BridgeError::ContractPaused)));
    }

    #[test]
    fn test_emergency_stop() {
        let mut bridge = create_bridge();
        let owner = bridge.owner;
        
        bridge.emergency_stop(owner).unwrap();
        assert!(bridge.emergency_stopped);
        
        let user = Address::from_bytes([5u8; 20]);
        let token = Address::from_bytes([6u8; 20]);
        
        let result = bridge.lock(user, token, U256::from(100u64), 1, user);
        assert!(matches!(result, Err(BridgeError::EmergencyStopped)));
    }
}
