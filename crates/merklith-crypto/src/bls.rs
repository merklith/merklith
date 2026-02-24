use crate::error::CryptoError;
use merklith_types::{BLSPublicKey, BLSSignature};
use blst::min_pk::*;
use rand::RngCore;

/// BLS12-381 keypair for committee attestations.
pub struct BLSKeypair {
    secret_key: SecretKey,
}

impl BLSKeypair {
    /// Generate a new random keypair using cryptographically secure randomness
    pub fn generate() -> Result<Self, CryptoError> {
        let mut rng = rand::thread_rng();
        let mut ikm = [0u8; 32]; // IKM (Input Keying Material)
        // Fill with cryptographically secure random bytes
        rng.fill_bytes(&mut ikm);
        let secret_key = SecretKey::key_gen(&ikm, &[])
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("{:?}", e)))?;
        Ok(Self { secret_key })
    }

    /// Create from 32-byte secret key bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        // BLS secret keys are scalar field elements
        // For simplicity, we use key_gen with the bytes as IKM
        let secret_key = SecretKey::key_gen(bytes, &[])
            .map_err(|_| CryptoError::InvalidPrivateKey)?;
        Ok(Self { secret_key })
    }

    /// Get the public key
    pub fn public_key(&self) -> BLSPublicKey {
        let pk = self.secret_key.sk_to_pk();
        let bytes = pk.to_bytes();
        BLSPublicKey::from_bytes(&bytes).unwrap_or_else(|_| {
            // This should never happen with valid BLS keys, but we handle it gracefully
            BLSPublicKey::from_bytes(&[0u8; 48]).expect("Zero public key is valid")
        })
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> BLSSignature {
        let signature = self.secret_key.sign(message, b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_", &[]);
        let bytes = signature.to_bytes();
        BLSSignature::from_bytes(&bytes).unwrap_or_else(|_| {
            // This should never happen with valid signatures, but we handle it gracefully
            BLSSignature::from_bytes(&[0u8; 96]).expect("Zero signature is valid")
        })
    }

    /// Get secret key bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        // Note: blst doesn't expose raw secret key bytes directly
        // We serialize it through to_bytes
        self.secret_key.serialize()
    }
}

/// Verify a BLS signature
pub fn bls_verify(
    public_key: &BLSPublicKey,
    message: &[u8],
    signature: &BLSSignature,
) -> Result<(), CryptoError> {
    let pk = PublicKey::from_bytes(public_key.as_bytes())
        .map_err(|_| CryptoError::InvalidPublicKey)?;
    let sig = Signature::from_bytes(signature.as_bytes())
        .map_err(|_| CryptoError::InvalidSignature)?;

    let result = sig.verify(
        true, // check for infinity
        message,
        b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_",
        &[],
        &pk,
        true, // use distinct messages
    );

    if result == blst::BLST_ERROR::BLST_SUCCESS {
        Ok(())
    } else {
        Err(CryptoError::VerificationFailed)
    }
}

/// Aggregate multiple BLS signatures into one.
/// Used for committee attestations (100 sigs → 1 aggregate sig).
pub fn bls_aggregate_signatures(
    signatures: &[BLSSignature],
) -> Result<BLSSignature, CryptoError> {
    if signatures.is_empty() {
        return Err(CryptoError::BLSAggregationError(
            "Cannot aggregate empty signature list".to_string(),
        ));
    }

    let sigs: Vec<Signature> = signatures
        .iter()
        .map(|s| {
            Signature::from_bytes(s.as_bytes())
                .map_err(|_e| CryptoError::InvalidSignature)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let aggregate = AggregateSignature::aggregate(
        &sigs.iter().collect::<Vec<_>>(),
        true, // check for infinity
    )
    .map_err(|e| CryptoError::BLSAggregationError(format!("{:?}", e)))?;

    let bytes = aggregate.to_signature().to_bytes();
    BLSSignature::from_bytes(&bytes)
        .map_err(|e| CryptoError::BLSAggregationError(format!("{:?}", e)))
}

/// Aggregate multiple BLS public keys into one.
pub fn bls_aggregate_public_keys(
    public_keys: &[BLSPublicKey],
) -> Result<BLSPublicKey, CryptoError> {
    if public_keys.is_empty() {
        return Err(CryptoError::BLSAggregationError(
            "Cannot aggregate empty public key list".to_string(),
        ));
    }

    let pks: Vec<PublicKey> = public_keys
        .iter()
        .map(|pk| {
            PublicKey::from_bytes(pk.as_bytes())
                .map_err(|_e| CryptoError::InvalidPublicKey)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let aggregate = AggregatePublicKey::aggregate(
        &pks.iter().collect::<Vec<_>>(),
        true, // check for infinity
    )
    .map_err(|e| CryptoError::BLSAggregationError(format!("{:?}", e)))?;

    let bytes = aggregate.to_public_key().to_bytes();
    BLSPublicKey::from_bytes(&bytes)
        .map_err(|e| CryptoError::BLSAggregationError(format!("{:?}", e)))
}

/// Verify an aggregate signature against multiple public keys (same message).
/// This is the fast path for committee attestation verification.
pub fn bls_verify_aggregate(
    public_keys: &[BLSPublicKey],
    message: &[u8],
    aggregate_signature: &BLSSignature,
) -> Result<(), CryptoError> {
    let agg_pk = bls_aggregate_public_keys(public_keys)?;
    bls_verify(&agg_pk, message, aggregate_signature)
}

/// Verify an aggregate signature where each pubkey signed a different message.
pub fn bls_verify_multi(
    items: &[(BLSPublicKey, Vec<u8>)],
    aggregate_signature: &BLSSignature,
) -> Result<(), CryptoError> {
    let pks: Vec<PublicKey> = items
        .iter()
        .map(|(pk, _)| {
            PublicKey::from_bytes(pk.as_bytes())
                .map_err(|_| CryptoError::InvalidPublicKey)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let msgs: Vec<&[u8]> = items.iter().map(|(_, msg)| msg.as_slice()).collect();

    let sig = Signature::from_bytes(aggregate_signature.as_bytes())
        .map_err(|_| CryptoError::InvalidSignature)?;

    let pk_refs: Vec<&PublicKey> = pks.iter().collect();

    let result = sig.aggregate_verify(
        true, // check for infinity
        &msgs,
        b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_",
        &pk_refs,
        true, // use distinct messages
    );

    if result == blst::BLST_ERROR::BLST_SUCCESS {
        Ok(())
    } else {
        Err(CryptoError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bls_keypair_generation() {
        let keypair = BLSKeypair::generate().unwrap();
        let pk = keypair.public_key();
        assert!(!pk.is_zero());
    }

    #[test]
    fn test_bls_sign_and_verify() {
        let keypair = BLSKeypair::generate().unwrap();
        let message = b"Test message for BLS";

        let signature = keypair.sign(message);
        assert!(!signature.is_zero());

        // Verify
        let result = bls_verify(&keypair.public_key(), message, &signature);
        assert!(result.is_ok());

        // Wrong message should fail
        let wrong_message = b"Wrong message";
        let result = bls_verify(&keypair.public_key(), wrong_message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_bls_aggregate_signatures() {
        let keypairs: Vec<BLSKeypair> = (0..10)
            .map(|i| {
                let seed = [i as u8; 32];
                BLSKeypair::from_bytes(&seed).unwrap()
            })
            .collect();

        let message = b"Common message";

        // All sign the same message
        let signatures: Vec<BLSSignature> = keypairs.iter().map(|kp| kp.sign(message)).collect();

        // Aggregate
        let aggregate = bls_aggregate_signatures(&signatures).unwrap();

        // Verify aggregate
        let pks: Vec<BLSPublicKey> = keypairs.iter().map(|kp| kp.public_key()).collect();
        let result = bls_verify_aggregate(&pks, message, &aggregate);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bls_aggregate_public_keys() {
        let keypairs: Vec<BLSKeypair> = (0..5)
            .map(|i| {
                let seed = [i as u8; 32];
                BLSKeypair::from_bytes(&seed).unwrap()
            })
            .collect();

        let pks: Vec<BLSPublicKey> = keypairs.iter().map(|kp| kp.public_key()).collect();

        let agg_pk = bls_aggregate_public_keys(&pks).unwrap();
        assert!(!agg_pk.is_zero());
    }

    #[test]
    fn test_bls_verify_multi() {
        let keypairs: Vec<BLSKeypair> = (0..5)
            .map(|i| {
                let seed = [i as u8; 32];
                BLSKeypair::from_bytes(&seed).unwrap()
            })
            .collect();

        // Different messages for each keypair
        let items: Vec<(BLSPublicKey, Vec<u8>)> = keypairs
            .iter()
            .enumerate()
            .map(|(i, kp)| {
                let msg = format!("Message {}", i).into_bytes();
                (kp.public_key(), msg)
            })
            .collect();

        // Sign each with corresponding key
        let signatures: Vec<BLSSignature> = items
            .iter()
            .enumerate()
            .map(|(i, (_, msg))| keypairs[i].sign(msg))
            .collect();

        // Aggregate
        let aggregate = bls_aggregate_signatures(&signatures).unwrap();

        // Verify multi
        let result = bls_verify_multi(&items, &aggregate);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bls_empty_aggregate_fails() {
        let result = bls_aggregate_signatures(&[]);
        assert!(result.is_err());

        let result = bls_aggregate_public_keys(&[]);
        assert!(result.is_err());
    }
}
