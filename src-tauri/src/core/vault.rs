use rusqlite::Connection;

use crate::crypto::{self, VaultKey};
use crate::db::queries;

const VERIFIER_AAD: &str = "vault:verifier:v1";
const VERIFIER_PLAINTEXT: &str = "prompt-password:vault:v1";

#[derive(Default)]
pub struct VaultSession {
    key: Option<VaultKey>,
}

impl VaultSession {
    pub fn key(&self) -> Result<&[u8], String> {
        self.key
            .as_deref()
            .map(|key| key as &[u8])
            .ok_or_else(|| "Vault is locked".to_string())
    }

    pub fn lock(&mut self) {
        self.key = None;
    }

    fn unlock_with_key(&mut self, key: VaultKey) {
        self.key = Some(key);
    }
}

pub fn entry_aad(id: &str) -> String {
    format!("entry:{}", id)
}

pub fn template_aad(id: &str) -> String {
    format!("template:{}", id)
}

pub fn unlock_or_initialize(
    conn: &Connection,
    session: &mut VaultSession,
    master_password: &str,
) -> Result<(), String> {
    if master_password.is_empty() {
        return Err("Master password is required".to_string());
    }

    let key = match queries::get_vault_meta(conn)
        .map_err(|e| format!("Failed to read vault metadata: {}", e))?
    {
        Some(_) => verify_master_password(conn, master_password)?,
        None => {
            let salt = crypto::random_salt();
            let key = crypto::derive_key(master_password, &salt)?;
            let verifier = crypto::encrypt_secret(&key[..], VERIFIER_AAD, VERIFIER_PLAINTEXT)?;
            queries::store_vault_meta(
                conn,
                &crypto::encode_base64(&salt),
                &verifier.ciphertext_b64,
                &verifier.nonce_b64,
            )
            .map_err(|e| format!("Failed to initialize vault metadata: {}", e))?;
            key
        }
    };

    migrate_legacy_plaintext_entries(conn, &key[..])?;
    session.unlock_with_key(key);
    Ok(())
}

pub fn verify_master_password(conn: &Connection, master_password: &str) -> Result<VaultKey, String> {
    if master_password.is_empty() {
        return Err("Master password is required".to_string());
    }

    let meta = queries::get_vault_meta(conn)
        .map_err(|e| format!("Failed to read vault metadata: {}", e))?
        .ok_or_else(|| "Vault is not initialized".to_string())?;
    let salt = crypto::decode_base64(&meta.salt_b64)?;
    let key = crypto::derive_key(master_password, &salt)?;
    let verifier = crypto::decrypt_secret(
        &key[..],
        VERIFIER_AAD,
        &meta.verifier_enc,
        &meta.verifier_nonce,
    )
    .map_err(|_| "Master password is incorrect".to_string())?;
    if verifier != VERIFIER_PLAINTEXT {
        return Err("Master password is incorrect".to_string());
    }

    Ok(key)
}

fn migrate_legacy_plaintext_entries(conn: &Connection, key: &[u8]) -> Result<(), String> {
    let entries = queries::legacy_plaintext_entries(conn)
        .map_err(|e| format!("Failed to inspect plaintext entries: {}", e))?;

    for entry in entries {
        let sealed = crypto::encrypt_secret(key, &entry_aad(&entry.id), &entry.password)?;
        queries::update_entry_secret(conn, &entry.id, &sealed.ciphertext_b64, &sealed.nonce_b64)
            .map_err(|e| format!("Failed to migrate entry {}: {}", entry.id, e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_conn() -> Connection {
        db::init::initialize_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn first_unlock_migrates_legacy_plaintext_and_rejects_later_wrong_password() {
        let conn = test_conn();
        let legacy = queries::insert_entry(
            &conn,
            "GitHub",
            "",
            "",
            "",
            "octocat",
            "plain-secret",
            "",
            "dev",
        )
        .unwrap();
        let mut session = VaultSession::default();

        unlock_or_initialize(&conn, &mut session, "master-password").unwrap();

        let migrated = queries::get_entry(&conn, &legacy.id).unwrap();
        assert_ne!(migrated.password_enc, "plain-secret");
        assert!(!migrated.password_nonce.is_empty());
        assert_eq!(
            crate::crypto::decrypt_secret(
                session.key().unwrap(),
                &entry_aad(&legacy.id),
                &migrated.password_enc,
                &migrated.password_nonce,
            )
            .unwrap(),
            "plain-secret"
        );

        let mut wrong_session = VaultSession::default();
        assert!(unlock_or_initialize(&conn, &mut wrong_session, "wrong-password").is_err());
    }
}
