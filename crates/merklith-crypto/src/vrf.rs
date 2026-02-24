//! VRF (Verifiable Random Function) implementation using Ed25519.
//!
//! This is a simplified VRF implementation that uses Ed25519 signatures
//! as the basis for verifiable randomness.

use crate::error::CryptoError;
use merklith_types::{Ed25519PublicKey, Ed25519Signature, Hash};

/// VRF output and proof.
/// Used for committee selection — generates verifiable randomness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VRFOutput {
    /// The random output (32 bytes, uniformly distributed)
    pub output: Hash,
    /// Proof that the output was correctly generated (the signature)
    pub proof: Ed25519Signature,
}

impl VRFOutput {
    /// Create a new VRF output
    pub fn new(output: Hash, proof: Ed25519Signature) -> Self {
        Self { output, proof }
    }
}

/// Generate a VRF output.
///
/// Given a secret key and a message (typically block hash as seed),
/// produces a deterministic random output with a proof that anyone
/// can verify using the corresponding public key.
pub fn vrf_prove(
    secret_key: &crate::ed25519::Keypair,
    message: &[u8],
) -> VRFOutput {
    // Sign the message
    let signature = secret_key.sign(message);

    // Hash the signature to get the output
    let output = Hash::compute(signature.as_bytes());

    VRFOutput::new(output, signature)
}

/// Verify a VRF output.
pub fn vrf_verify(
    public_key: &Ed25519PublicKey,
    message: &[u8],
    output: &VRFOutput,
) -> Result<(), CryptoError> {
    // Verify the signature
    crate::ed25519::verify(public_key, message, &output.proof)?;

    // Verify the output matches the signature hash
    let expected_output = Hash::compute(output.proof.as_bytes());
    if expected_output != output.output {
        return Err(CryptoError::VRFProofInvalid);
    }

    Ok(())
}

/// Convert VRF output to a number in range [0, max).
/// Used for weighted committee selection.
pub fn vrf_output_to_index(output: &Hash, max: u64) -> u64 {
    let bytes = output.as_bytes();
    // Use first 8 bytes as u64
    let num = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    num % max
}

/// Generate deterministic randomness from VRF output
pub fn vrf_to_randomness(output: &Hash) -> [u8; 32] {
    *output.as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ed25519::Keypair;

    #[test]
    fn test_vrf_prove_and_verify() {
        let keypair = Keypair::generate();
        let message = b"Block hash as VRF seed";

        let output = vrf_prove(&keypair, message);
        assert!(!output.output.is_zero());

        // Verify
        let result = vrf_verify(&keypair.public_key(), message, &output);
        assert!(result.is_ok());

        // Wrong message
        let wrong_message = b"Wrong message";
        let result = vrf_verify(&keypair.public_key(), wrong_message, &output);
        assert!(result.is_err());

        // Wrong public key
        let wrong_keypair = Keypair::generate();
        let result = vrf_verify(&wrong_keypair.public_key(), message, &output);
        assert!(result.is_err());
    }

    #[test]
    fn test_vrf_determinism() {
        let keypair = Keypair::generate();
        let message = b"Same message";

        let output1 = vrf_prove(&keypair, message);
        let output2 = vrf_prove(&keypair, message);

        // Same key + same message = same output
        assert_eq!(output1.output, output2.output);
        assert_eq!(output1.proof, output2.proof);
    }

    #[test]
    fn test_vrf_different_inputs() {
        let keypair = Keypair::generate();

        let output1 = vrf_prove(&keypair, b"message1");
        let output2 = vrf_prove(&keypair, b"message2");

        // Different messages = different outputs
        assert_ne!(output1.output, output2.output);
    }

    #[test]
    fn test_vrf_output_to_index() {
        let output = Hash::compute(b"test");

        let index = vrf_output_to_index(&output, 100);
        assert!(index < 100);

        // Deterministic
        let index2 = vrf_output_to_index(&output, 100);
        assert_eq!(index, index2);
    }

    #[test]
    fn test_vrf_to_randomness() {
        let output = Hash::compute(b"test");
        let randomness = vrf_to_randomness(&output);

        // Should be 32 bytes
        assert_eq!(randomness.len(), 32);

        // Should match output bytes
        assert_eq!(randomness, *output.as_bytes());
    }

    #[test]
    fn test_vrf_different_keys() {
        let keypair1 = Keypair::generate();
        let keypair2 = Keypair::generate();
        let message = b"same message";

        let output1 = vrf_prove(&keypair1, message);
        let output2 = vrf_prove(&keypair2, message);

        // Different keys should produce different outputs
        assert_ne!(output1.output, output2.output);

        // Each should verify with its own key
        assert!(vrf_verify(&keypair1.public_key(), message, &output1).is_ok());
        assert!(vrf_verify(&keypair2.public_key(), message, &output2).is_ok());
    }
}
