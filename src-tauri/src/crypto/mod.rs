use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand_core::{OsRng, RngCore};
use zeroize::Zeroizing;

const ARGON2_MEMORY_KIB: u32 = 65_536;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 2;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const SALT_LEN: usize = 16;
const ID_LEN: usize = 16;

pub type VaultKey = Zeroizing<[u8; KEY_LEN]>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedSecret {
    pub ciphertext_b64: String,
    pub nonce_b64: String,
}

pub fn derive_key(master_password: &str, salt: &[u8]) -> Result<VaultKey, String> {
    if salt.len() < SALT_LEN {
        return Err("vault salt is too short".to_string());
    }

    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(KEY_LEN),
    )
    .map_err(|e| format!("Failed to configure Argon2id: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0_u8; KEY_LEN]);

    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut key[..])
        .map_err(|e| format!("Failed to derive vault key: {}", e))?;

    Ok(key)
}

pub fn encrypt_secret(
    key: &[u8],
    associated_data: &str,
    plaintext: &str,
) -> Result<SealedSecret, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to initialize AES-256-GCM: {}", e))?;
    let mut nonce_bytes = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext.as_bytes(),
                aad: associated_data.as_bytes(),
            },
        )
        .map_err(|_| "Failed to encrypt secret".to_string())?;

    Ok(SealedSecret {
        ciphertext_b64: BASE64.encode(ciphertext),
        nonce_b64: BASE64.encode(nonce_bytes),
    })
}

pub fn decrypt_secret(
    key: &[u8],
    associated_data: &str,
    ciphertext_b64: &str,
    nonce_b64: &str,
) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to initialize AES-256-GCM: {}", e))?;
    let ciphertext = decode_base64(ciphertext_b64)?;
    let nonce_bytes = decode_base64(nonce_b64)?;
    if nonce_bytes.len() != NONCE_LEN {
        return Err("Invalid AES-GCM nonce length".to_string());
    }

    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: &ciphertext,
                aad: associated_data.as_bytes(),
            },
        )
        .map_err(|_| "Failed to decrypt secret".to_string())?;

    String::from_utf8(plaintext).map_err(|_| "Secret is not valid UTF-8".to_string())
}

pub fn random_salt() -> [u8; SALT_LEN] {
    let mut salt = [0_u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn generate_id() -> String {
    let mut bytes = [0_u8; ID_LEN];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn encode_base64(bytes: &[u8]) -> String {
    BASE64.encode(bytes)
}

pub fn decode_base64(value: &str) -> Result<Vec<u8>, String> {
    BASE64
        .decode(value)
        .map_err(|e| format!("Invalid base64 data: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_key_encrypts_and_decrypts_secret_with_associated_data() {
        let salt = [7_u8; 16];
        let key = derive_key("correct horse battery staple", &salt).unwrap();

        let sealed = encrypt_secret(&key[..], "entry:abc123", "secret-123").unwrap();

        assert_ne!(sealed.ciphertext_b64, "secret-123");
        assert!(!sealed.nonce_b64.is_empty());
        assert_eq!(
            decrypt_secret(
                &key[..],
                "entry:abc123",
                &sealed.ciphertext_b64,
                &sealed.nonce_b64
            )
            .unwrap(),
            "secret-123"
        );

        let wrong_key = derive_key("wrong password", &salt).unwrap();
        assert!(decrypt_secret(
            &wrong_key[..],
            "entry:abc123",
            &sealed.ciphertext_b64,
            &sealed.nonce_b64
        )
        .is_err());
    }
}
