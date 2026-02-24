use crate::address::Address;
use crate::error::TypesError;
use crate::hash::Hash;
use crate::signature::{Ed25519PublicKey, Ed25519Signature};
use crate::u256::U256;
use std::fmt;

/// Transaction type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub enum TransactionType {
    /// Legacy transaction
    #[default]
    Legacy,
    /// EIP-1559 transaction
    Eip1559,
    /// Batch transaction
    Batch,
}

/// Access list entry for warm storage slots
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct AccessListEntry {
    pub address: Address,
    pub storage_keys: Vec<Hash>,
}

/// Unsigned transaction data.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct Transaction {
    /// Transaction type
    pub tx_type: TransactionType,
    /// Chain ID (replay protection)
    pub chain_id: u64,
    /// Sender's nonce (increments with each TX)
    pub nonce: u64,
    /// Recipient address (None = contract creation)
    pub to: Option<Address>,
    /// MERK value to transfer (in Spark)
    pub value: U256,
    /// Maximum gas units this TX can consume
    pub gas_limit: u64,
    /// Maximum total fee per gas (base + priority)
    pub max_fee_per_gas: U256,
    /// Maximum priority fee per gas (tip to validator)
    pub max_priority_fee_per_gas: U256,
    /// Input data (contract call data or init code)
    pub data: Vec<u8>,
    /// Access list for warm storage slots (optional optimization)
    pub access_list: Vec<AccessListEntry>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        chain_id: u64,
        nonce: u64,
        to: Option<Address>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: U256,
    ) -> Self {
        Self {
            tx_type: TransactionType::Legacy,
            chain_id,
            nonce,
            to,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            data: Vec::new(),
            access_list: Vec::new(),
        }
    }

    /// Check if this is a contract creation transaction
    pub fn is_create(&self) -> bool {
        self.to.is_none()
    }

    /// Compute the hash that should be signed
    pub fn signing_hash(&self) -> Hash {
        // Simple serialization for signing
        // In production, use a proper canonical serialization
        let mut data = Vec::new();
        data.extend_from_slice(&self.chain_id.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        if let Some(to) = self.to {
            data.extend_from_slice(to.as_bytes());
        } else {
            data.extend_from_slice(&[0u8; 20]);
        }
        data.extend_from_slice(&self.value.to_le_bytes());
        data.extend_from_slice(&self.gas_limit.to_le_bytes());
        data.extend_from_slice(&self.max_fee_per_gas.to_le_bytes());
        data.extend_from_slice(&self.max_priority_fee_per_gas.to_le_bytes());
        data.extend_from_slice(&self.data);
        Hash::compute(&data)
    }

    /// Add data to the transaction
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Add access list to the transaction
    pub fn with_access_list(mut self, access_list: Vec<AccessListEntry>) -> Self {
        self.access_list = access_list;
        self
    }
}

/// Transaction with signature attached.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct SignedTransaction {
    pub tx: Transaction,
    pub signature: Ed25519Signature,
    /// Sender public key (included for recovery)
    pub public_key: Ed25519PublicKey,
}

impl SignedTransaction {
    /// Create a new signed transaction
    pub fn new(tx: Transaction, signature: Ed25519Signature, public_key: Ed25519PublicKey) -> Self {
        Self {
            tx,
            signature,
            public_key,
        }
    }

    /// Compute the transaction hash
    pub fn hash(&self) -> Hash {
        // Include signature in hash
        let mut data = Vec::new();
        let signing_hash = self.tx.signing_hash();
        data.extend_from_slice(signing_hash.as_bytes());
        data.extend_from_slice(self.signature.as_bytes());
        data.extend_from_slice(self.public_key.as_bytes());
        Hash::compute(&data)
    }

    /// Get the sender address
    pub fn sender(&self) -> Address {
        self.public_key.to_address()
    }

    /// Check if this is a contract creation
    pub fn is_create(&self) -> bool {
        self.tx.is_create()
    }

    /// Calculate effective gas price given a base fee
    pub fn effective_gas_price(&self, base_fee: &U256,
    ) -> U256 {
        let priority = self.tx.max_priority_fee_per_gas.min(
            self.tx.max_fee_per_gas.saturating_sub(base_fee)
        );
        base_fee.saturating_add(&priority)
    }

    /// Calculate maximum possible cost of this transaction
    pub fn max_cost(&self) -> U256 {
        let gas_cost = self.tx.max_fee_per_gas
            .checked_mul(&U256::from(self.tx.gas_limit))
            .unwrap_or(U256::MAX);
        gas_cost.saturating_add(&self.tx.value)
    }

    /// Verify the signature
    pub fn verify_signature(&self) -> Result<(), TypesError> {
        // This would use ed25519-dalek in production
        // For now, just check that signature and public key are not zero
        if self.signature.is_zero() {
            return Err(TypesError::InvalidSignatureLength {
                expected: 64,
                actual: 0,
            });
        }
        if self.public_key.is_zero() {
            return Err(TypesError::InvalidPublicKeyLength {
                expected: 32,
                actual: 0,
            });
        }
        Ok(())
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction {{ chain_id: {}, nonce: {}, to: {:?}, value: {} }}",
            self.chain_id, self.nonce, self.to, self.value
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_new() {
        let tx = Transaction::new(
            1, // chain_id
            0, // nonce
            Some(Address::ZERO),
            U256::from(1000u64),
            21000, // gas_limit
            U256::from(1000000000u64), // max_fee_per_gas
            U256::from(1000000u64), // max_priority_fee_per_gas
        );

        assert_eq!(tx.chain_id, 1);
        assert_eq!(tx.nonce, 0);
        assert!(!tx.is_create());
    }

    #[test]
    fn test_transaction_create() {
        let tx = Transaction::new(
            1,
            0,
            None, // No recipient = contract creation
            U256::ZERO,
            100000,
            U256::from(1000000000u64),
            U256::from(1000000u64),
        );

        assert!(tx.is_create());
    }

    #[test]
    fn test_signed_transaction() {
        let tx = Transaction::new(
            1,
            0,
            Some(Address::ZERO),
            U256::from(1000u64),
            21000,
            U256::from(1000000000u64),
            U256::from(1000000u64),
        );

        let sig = Ed25519Signature::from_bytes([1u8; 64]);
        let pk = Ed25519PublicKey::from_bytes([2u8; 32]);

        let signed = SignedTransaction::new(tx, sig, pk);

        assert!(!signed.hash().is_zero());
        assert_eq!(signed.sender(), pk.to_address());
        assert!(!signed.is_create());
    }

    #[test]
    fn test_effective_gas_price() {
        let tx = Transaction::new(
            1,
            0,
            Some(Address::ZERO),
            U256::ZERO,
            21000,
            U256::from(100u64),
            U256::from(10u64),
        );

        let sig = Ed25519Signature::from_bytes([1u8; 64]);
        let pk = Ed25519PublicKey::from_bytes([2u8; 32]);
        let signed = SignedTransaction::new(tx, sig, pk);

        let base_fee = U256::from(50u64);
        let effective = signed.effective_gas_price(&base_fee);

        // Should be base_fee + min(priority, max_fee - base_fee)
        assert_eq!(effective, U256::from(60u64));
    }

    #[test]
    fn test_max_cost() {
        let tx = Transaction::new(
            1,
            0,
            Some(Address::ZERO),
            U256::from(1000u64),
            21000,
            U256::from(10u64),
            U256::from(1u64),
        );

        let sig = Ed25519Signature::from_bytes([1u8; 64]);
        let pk = Ed25519PublicKey::from_bytes([2u8; 32]);
        let signed = SignedTransaction::new(tx, sig, pk);

        let max_cost = signed.max_cost();
        // gas_limit * max_fee_per_gas + value
        assert_eq!(max_cost, U256::from(211000u64));
    }

    #[test]
    fn test_access_list_entry() {
        let entry = AccessListEntry {
            address: Address::ZERO,
            storage_keys: vec![Hash::compute(b"key1"), Hash::compute(b"key2")],
        };

        assert_eq!(entry.storage_keys.len(), 2);
    }
}
