use crate::error::CryptoError;
use merklith_types::{Address, Hash};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng as AesRng},
    Aes256Gcm, Key, Nonce,
};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

/// Encrypted keystore file format.
/// Uses argon2id for key derivation, AES-256-GCM for encryption.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeystoreFile {
    pub version: u32,  // 1
    pub id: String,    // UUID
    pub address: String,
    pub crypto: KeystoreCrypto,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeystoreCrypto {
    pub cipher: String,         // "aes-256-gcm"
    pub ciphertext: String,     // hex
    pub cipherparams: CipherParams,
    pub kdf: String,            // "argon2id"
    pub kdfparams: KdfParams,
    pub mac: String,            // hex, blake3(derived_key[16..32] || ciphertext)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CipherParams {
    pub iv: String,  // hex, 12 bytes nonce
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KdfParams {
    pub salt: String,           // hex, 16 bytes
    pub parallelism: u32,       // 4
    pub memory_cost: u32,       // 65536 (64 MB)
    pub time_cost: u32,         // 3
    pub output_len: u32,        // 32
}

/// Encrypt a private key with a password and save to keystore file.
pub fn encrypt_keystore(
    secret_key: &[u8; 32],
    password: &str,
    path: &Path,
) -> Result<(), CryptoError> {
    // Generate salt
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 parameters
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(65536, 3, 4, Some(32))
            .map_err(|e| CryptoError::EncryptionFailed(format!("{:?}", e)))?,
    );

    // Derive key from password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::KeyDerivationFailed(format!("{:?}", e)))?;

    let derived_key = password_hash.hash.ok_or_else(|| {
        CryptoError::KeyDerivationFailed("No hash generated".to_string())
    })?;

    // Generate random nonce (IV)
    let nonce_bytes = AesRng.gen::<[u8; 12]>();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt secret key
    let key: &Key<Aes256Gcm> = Key::<Aes256Gcm>::from_slice(&derived_key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let ciphertext = cipher
        .encrypt(nonce, secret_key.as_ref())
        .map_err(|e| CryptoError::EncryptionFailed(format!("{:?}", e)))?;

    // Calculate MAC (Message Authentication Code)
    let mut mac_data = Vec::new();
    mac_data.extend_from_slice(&derived_key.as_bytes()[16..32]);
    mac_data.extend_from_slice(&ciphertext);
    let mac = blake3::hash(&mac_data);

    // Create keystore file structure
    let keystore = KeystoreFile {
        version: 1,
        id: uuid::Uuid::new_v4().to_string(),
        address: crate::ed25519::Keypair::from_seed(secret_key)
            .address()
            .to_string(),
        crypto: KeystoreCrypto {
            cipher: "aes-256-gcm".to_string(),
            ciphertext: hex::encode(&ciphertext),
            cipherparams: CipherParams {
                iv: hex::encode(nonce_bytes),
            },
            kdf: "argon2id".to_string(),
            kdfparams: KdfParams {
                salt: salt.to_string(),
                parallelism: 4,
                memory_cost: 65536,
                time_cost: 3,
                output_len: 32,
            },
            mac: mac.to_string(),
        },
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&keystore)
        .map_err(|e| CryptoError::Serialization(e.to_string()))?;

    // Write to file
    std::fs::write(path, json)?;

    Ok(())
}

/// Decrypt a keystore file with a password.
pub fn decrypt_keystore(
    path: &Path,
    password: &str,
) -> Result<[u8; 32], CryptoError> {
    // Read file
    let json = std::fs::read_to_string(path)?;

    // Parse JSON
    let keystore: KeystoreFile = serde_json::from_str(&json)
        .map_err(|e| CryptoError::KeystoreError(format!("Parse error: {}", e)))?;

    // Verify version
    if keystore.version != 1 {
        return Err(CryptoError::KeystoreError(
            format!("Unsupported keystore version: {}", keystore.version)
        ));
    }

    // Decode salt
    let salt = SaltString::from_b64(&keystore.crypto.kdfparams.salt
    ).map_err(|_| CryptoError::KeystoreError("Invalid salt".to_string()))?;

    // Argon2 parameters
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(
            keystore.crypto.kdfparams.memory_cost,
            keystore.crypto.kdfparams.time_cost,
            keystore.crypto.kdfparams.parallelism,
            Some(keystore.crypto.kdfparams.output_len as usize),
        ).map_err(|e| CryptoError::KeystoreError(format!("{:?}", e)))?,
    );

    // Derive key from password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| CryptoError::InvalidPassword)?;

    let derived_key = password_hash.hash.ok_or_else(|| {
        CryptoError::KeyDerivationFailed("No hash generated".to_string())
    })?;

    // Decode ciphertext
    let ciphertext = hex::decode(&keystore.crypto.ciphertext)
        .map_err(|_| CryptoError::KeystoreError("Invalid ciphertext".to_string()))?;

    // Verify MAC
    let mut mac_data = Vec::new();
    mac_data.extend_from_slice(&derived_key.as_bytes()[16..32]);
    mac_data.extend_from_slice(&ciphertext);
    let expected_mac = blake3::hash(&mac_data);

    let actual_mac = Hash::from_str(&keystore.crypto.mac)
        .map_err(|_| CryptoError::KeystoreError("Invalid MAC".to_string()))?;

    if expected_mac.as_bytes() != actual_mac.as_bytes() {
        return Err(CryptoError::InvalidPassword);
    }

    // Decode nonce (IV)
    let nonce_bytes = hex::decode(&keystore.crypto.cipherparams.iv)
        .map_err(|_| CryptoError::KeystoreError("Invalid IV".to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decrypt
    let key: &Key<Aes256Gcm> = Key::<Aes256Gcm>::from_slice(&derived_key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| CryptoError::DecryptionFailed("Decryption failed".to_string()))?;

    if plaintext.len() != 32 {
        return Err(CryptoError::KeystoreError(
            format!("Invalid key length: {}", plaintext.len())
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&plaintext);

    Ok(key)
}

/// Create a new random keystore file.
pub fn create_keystore(
    password: &str,
    path: &Path,
) -> Result<Address, CryptoError> {
    // Generate random key
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);

    encrypt_keystore(&key, password, path)?;

    let keypair = crate::ed25519::Keypair::from_seed(&key);
    Ok(keypair.address())
}

/// Check if a keystore file exists and is valid.
pub fn check_keystore(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    match std::fs::read_to_string(path) {
        Ok(json) => serde_json::from_str::<KeystoreFile>(&json).is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_keystore() {
        let temp_file = NamedTempFile::new().unwrap();
        let password = "test_password_123";
        let secret_key = [42u8; 32];

        // Encrypt
        encrypt_keystore(&secret_key, password, temp_file.path()).unwrap();

        // Decrypt
        let decrypted = decrypt_keystore(temp_file.path(), password).unwrap();

        assert_eq!(secret_key, decrypted);
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let temp_file = NamedTempFile::new().unwrap();
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let secret_key = [42u8; 32];

        encrypt_keystore(&secret_key, password, temp_file.path()).unwrap();

        let result = decrypt_keystore(temp_file.path(), wrong_password);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_keystore() {
        let temp_file = NamedTempFile::new().unwrap();
        let password = "test_password";

        let address = create_keystore(password, temp_file.path()).unwrap();

        assert!(!address.is_zero());
        assert!(check_keystore(temp_file.path()));
    }

    #[test]
    fn test_check_keystore_invalid() {
        let temp_file = NamedTempFile::new().unwrap();

        // Write invalid JSON
        std::fs::write(temp_file.path(), b"invalid json").unwrap();

        assert!(!check_keystore(temp_file.path()));
    }

    #[test]
    fn test_check_keystore_nonexistent() {
        let path = Path::new("/nonexistent/path/to/keystore.json");
        assert!(!check_keystore(path));
    }
}
