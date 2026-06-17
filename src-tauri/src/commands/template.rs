use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::core::vault::{template_aad, VaultSession};
use crate::crypto;
use crate::db::queries::{self, PasswordTemplateSearchResult};

#[derive(Debug, Clone, serde::Serialize)]
pub struct UsedPasswordTemplate {
    pub password: String,
}

#[tauri::command]
pub fn search_password_templates(
    query: String,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<Vec<PasswordTemplateSearchResult>, String> {
    if query.len() > 256 {
        return Err("Template search query too long (max 256 characters)".to_string());
    }
    {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        vault.key()?;
    }

    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    queries::search_password_templates(&db, &query)
        .map_err(|e| format!("Failed to search password templates: {}", e))
}

#[tauri::command]
pub fn create_password_template(
    name: String,
    description: Option<String>,
    password: String,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<PasswordTemplateSearchResult, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("template name is required".to_string());
    }
    if password.is_empty() {
        return Err("template password is required".to_string());
    }

    let description = description.unwrap_or_default().trim().to_string();
    let id = crypto::generate_id();
    let sealed = {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        crypto::encrypt_secret(vault.key()?, &template_aad(&id), &password)?
    };
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    let template = queries::insert_password_template_with_id(
        &db,
        &id,
        &name,
        &description,
        &sealed.ciphertext_b64,
        &sealed.nonce_b64,
    )
    .map_err(|e| format!("Failed to create password template: {}", e))?;

    Ok(PasswordTemplateSearchResult {
        id: template.id,
        name: template.name,
        description: template.description,
        password_preview: "********".to_string(),
    })
}

#[tauri::command]
pub fn update_password_template(
    id: String,
    name: String,
    description: Option<String>,
    password: Option<String>,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<PasswordTemplateSearchResult, String> {
    if id.trim().is_empty() {
        return Err("template id is required".to_string());
    }
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("template name is required".to_string());
    }
    let description = description.unwrap_or_default().trim().to_string();
    let password = password.unwrap_or_default();

    let sealed = if password.is_empty() {
        None
    } else {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        Some(crypto::encrypt_secret(
            vault.key()?,
            &template_aad(&id),
            &password,
        )?)
    };

    if sealed.is_none() {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        vault.key()?;
    }

    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;

    match sealed {
        Some(sealed) => queries::update_password_template_metadata_and_secret(
            &db,
            &id,
            &name,
            &description,
            &sealed.ciphertext_b64,
            &sealed.nonce_b64,
        ),
        None => queries::update_password_template_metadata(&db, &id, &name, &description),
    }
    .map_err(|e| format!("Failed to update password template {}: {}", id, e))
}

#[tauri::command]
pub fn delete_password_template(
    id: String,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err("template id is required".to_string());
    }
    {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        vault.key()?;
    }
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    queries::delete_password_template(&db, &id)
        .map_err(|e| format!("Failed to delete password template {}: {}", id, e))
}

#[tauri::command]
pub fn use_password_template(
    id: String,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<UsedPasswordTemplate, String> {
    let template = {
        let db = conn
            .lock()
            .map_err(|e| format!("Failed to access database: {}", e))?;
        queries::get_password_template(&db, &id)
            .map_err(|e| format!("Failed to retrieve password template {}: {}", id, e))?
    };
    let password = {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        crypto::decrypt_secret(
            vault.key()?,
            &template_aad(&template.id),
            &template.password_enc,
            &template.password_nonce,
        )?
    };

    Ok(UsedPasswordTemplate { password })
}
