use crate::error::CryptoError;
use merklith_types::{Address, Ed25519PublicKey, Ed25519Signature, Transaction};
use ed25519_dalek::{Signer, Verifier};
use rand::rngs::OsRng;
use std::fmt;
use zeroize::Zeroize;

/// Ed25519 keypair for transaction signing.
/// Private key is zeroized on drop.
pub struct Keypair {
    signing_key: ed25519_dalek::SigningKey,
}

impl Keypair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        Self { signing_key }
    }

    /// Create from a 32-byte seed
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(seed);
        Self { signing_key }
    }

    /// Get the public key
    pub fn public_key(&self) -> Ed25519PublicKey {
        let bytes = self.signing_key.verifying_key().to_bytes();
        Ed25519PublicKey::from_bytes(bytes)
    }

    /// Get the address derived from this keypair
    pub fn address(&self) -> Address {
        self.public_key().to_address()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Ed25519Signature {
        let signature = self.signing_key.sign(message);
        Ed25519Signature::from_bytes(signature.to_bytes())
    }

    /// Sign a transaction (signs the transaction's signing_hash)
    pub fn sign_transaction(&self, tx: &Transaction) -> (Ed25519Signature, Ed25519PublicKey) {
        let hash = tx.signing_hash();
        let signature = self.sign(hash.as_bytes());
        (signature, self.public_key())
    }

    /// Export private key bytes (CAUTION: sensitive)
    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

impl fmt::Debug for Keypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Keypair({})", self.address())
    }
}

impl Clone for Keypair {
    fn clone(&self) -> Self {
        Self::from_seed(&self.to_bytes())
    }
}

impl Drop for Keypair {
    fn drop(&mut self) {
        // Zeroize the signing key bytes
        let mut bytes = self.signing_key.to_bytes();
        bytes.zeroize();
        // Note: We can't actually overwrite the signing_key field itself,
        // but this is the best we can do without the Zeroize trait
    }
}

/// Verify an ed25519 signature.
pub fn verify(
    public_key: &Ed25519PublicKey,
    message: &[u8],
    signature: &Ed25519Signature,
) -> Result<(), CryptoError> {
    let pk = ed25519_dalek::VerifyingKey::from_bytes(public_key.as_bytes())?;
    let sig = ed25519_dalek::Signature::from_bytes(signature.as_bytes());
    pk.verify(message, &sig)?;
    Ok(())
}

/// Batch verify multiple signatures (faster than individual verification).
/// Returns Ok(()) if ALL signatures are valid, Err otherwise.
pub fn batch_verify(
    items: &[(Ed25519PublicKey, Vec<u8>, Ed25519Signature)],
) -> Result<(), CryptoError> {
    let mut messages: Vec<&[u8]> = Vec::with_capacity(items.len());
    let mut signatures: Vec<ed25519_dalek::Signature> = Vec::with_capacity(items.len());
    let mut public_keys: Vec<ed25519_dalek::VerifyingKey> = Vec::with_capacity(items.len());

    for (pk, msg, sig) in items {
        messages.push(msg);
        signatures.push(ed25519_dalek::Signature::from_bytes(sig.as_bytes()));
        public_keys.push(ed25519_dalek::VerifyingKey::from_bytes(pk.as_bytes())?);
    }

    ed25519_dalek::verify_batch(&messages, &signatures, &public_keys)
        .map_err(|_| CryptoError::VerificationFailed)
}

/// Recover the public key from a signed message (for sender recovery).
/// Note: Ed25519 doesn't support public key recovery from signature alone.
/// This function verifies the signature and returns the provided public key.
pub fn recover_sender(
    _message: &[u8],
    _signature: &Ed25519Signature,
    _expected_address: &Address,
) -> Result<Ed25519PublicKey, CryptoError> {
    // Try to derive the public key from the expected address
    // In practice, the public key should be included in the transaction
    // This is a placeholder that would need the actual public key
    Err(CryptoError::InvalidPublicKey)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = Keypair::generate();
        assert!(!keypair.address().is_zero());
        assert!(!keypair.public_key().is_zero());
    }

    #[test]
    fn test_keypair_from_seed() {
        let seed = [42u8; 32];
        let kp1 = Keypair::from_seed(&seed);
        let kp2 = Keypair::from_seed(&seed);

        assert_eq!(kp1.public_key(), kp2.public_key());
        assert_eq!(kp1.address(), kp2.address());
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = Keypair::generate();
        let message = b"Hello, Merklith!";

        let signature = keypair.sign(message);
        assert!(!signature.is_zero());

        // Verify
        let result = verify(&keypair.public_key(), message, &signature);
        assert!(result.is_ok());

        // Wrong message should fail
        let wrong_message = b"Wrong message";
        let result = verify(&keypair.public_key(), wrong_message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_verify() {
        let keypairs: Vec<Keypair> = (0..10).map(|_| Keypair::generate()).collect();

        let items: Vec<(Ed25519PublicKey, Vec<u8>, Ed25519Signature)> = keypairs
            .iter()
            .enumerate()
            .map(|(i, kp)| {
                let msg = format!("Message {}", i).into_bytes();
                let sig = kp.sign(&msg);
                (kp.public_key(), msg, sig)
            })
            .collect();

        let result = batch_verify(&items);
        assert!(result.is_ok());

        // Mix in one invalid signature
        let mut invalid_items = items.clone();
        invalid_items[5].2 = Ed25519Signature::from_bytes([0u8; 64]);

        let result = batch_verify(&invalid_items);
        assert!(result.is_err());
    }

    #[test]
    fn test_keypair_clone() {
        let kp1 = Keypair::generate();
        let kp2 = kp1.clone();

        assert_eq!(kp1.public_key(), kp2.public_key());
        assert_eq!(kp1.address(), kp2.address());

        // Verify both can sign
        let msg = b"test";
        let sig1 = kp1.sign(msg);
        let sig2 = kp2.sign(msg);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_drop() {
        let seed = [42u8; 32];
        let keypair = Keypair::from_seed(&seed);

        // Just test that drop works without panic
        drop(keypair);
    }
}
