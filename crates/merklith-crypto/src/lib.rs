//! Merklith Crypto - Cryptographic primitives for the Merklith blockchain.
//!
//! This crate provides:
//! - Ed25519 signatures (transaction signing)
//! - BLS12-381 aggregate signatures (committee attestations)
//! - Blake3 hashing
//! - VRF (Verifiable Random Function)
//! - Merkle trees and proofs
//! - Encrypted keystore

pub mod ed25519;
pub mod bls;
pub mod hash;
pub mod vrf;
pub mod merkle;
pub mod keystore;
pub mod error;

pub use ed25519::{Keypair, verify as ed25519_verify, batch_verify as ed25519_batch_verify};
pub use bls::{
    BLSKeypair, bls_verify, bls_aggregate_signatures, 
    bls_aggregate_public_keys, bls_verify_aggregate, bls_verify_multi
};
pub use vrf::{VRFOutput, vrf_prove, vrf_verify, vrf_output_to_index};
pub use merkle::{MerkleTree, MerkleProof, merkle_hash_pair};
pub use keystore::{encrypt_keystore, decrypt_keystore, create_keystore, check_keystore};
pub use error::CryptoError;
