# Cross-Platform Initialization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current Tauri/Preact prototype into a test-backed macOS-first, Windows-compatible initialization baseline.

**Architecture:** Keep one shared Rust core for storage, validation, platform policy, and clipboard behavior. Keep the Preact panel as the shared UI. Move operating-system differences behind small Rust platform adapters so macOS polish does not fork the product.

**Tech Stack:** Tauri 2, Rust 2021, rusqlite, tauri-plugin-global-shortcut, tauri-plugin-clipboard-manager, Preact 10, TypeScript, Vite.

---

## File Structure

- Modify `src-tauri/src/db/init.rs`: expose a testable SQLite initialization helper and keep path-based app initialization.
- Modify `src-tauri/src/db/queries.rs`: add database behavior tests for insert, empty search, and FTS metadata search.
- Create `src-tauri/src/core/mod.rs`: module boundary for pure Rust product logic.
- Create `src-tauri/src/core/entry_input.rs`: normalize and validate create-entry input outside Tauri command glue.
- Modify `src-tauri/src/commands/entry.rs`: call the new input normalizer before inserting entries.
- Create `src-tauri/src/platform/mod.rs`: module boundary for OS-specific adapters.
- Create `src-tauri/src/platform/hotkey.rs`: default cross-platform shortcut policy and Tauri shortcut conversion.
- Create `src-tauri/src/platform/window.rs`: shared main-window toggle, focus, hide-on-blur, and configure helpers.
- Modify `src-tauri/src/lib.rs`: wire platform adapters into app setup and shortcut handler.
- Modify `src-tauri/src/commands/clipboard.rs`: encode the post-copy window policy and hide the panel after successful copy.
- Modify `README.md`: align the project description with the committed Tauri/Rust direction.

## Task 1: Add Rust Database Core Tests

**Files:**
- Modify: `src-tauri/src/db/init.rs`
- Modify: `src-tauri/src/db/queries.rs`

- [ ] **Step 1: Write the failing initialization test**

Append this test module to `src-tauri/src/db/init.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_connection_creates_entries_schema() {
        let conn = initialize_connection(Connection::open_in_memory().unwrap()).unwrap();

        let table_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN ('entries', 'entries_fts')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let trigger_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'trigger' AND name IN ('entries_ai', 'entries_ad', 'entries_au')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 2);
        assert_eq!(trigger_count, 3);
    }
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml initialize_connection_creates_entries_schema
```

Expected: FAIL because `initialize_connection` is not defined.

- [ ] **Step 3: Implement the testable initialization helper**

Replace `src-tauri/src/db/init.rs` with:

```rust
use rusqlite::{Connection, Result};

pub fn initialize(db_path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    initialize_connection(conn)
}

pub fn initialize_connection(conn: Connection) -> Result<Connection> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id          TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
            name        TEXT NOT NULL,
            url         TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            alias       TEXT NOT NULL DEFAULT '',
            password_enc TEXT NOT NULL,
            password_nonce TEXT NOT NULL DEFAULT '',
            tags        TEXT NOT NULL DEFAULT '',
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
            name, url, description, alias, tags,
            content=entries,
            content_rowid=rowid
        );

        CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, name, url, description, alias, tags)
            VALUES (new.rowid, new.name, new.url, new.description, new.alias, new.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, name, url, description, alias, tags)
            VALUES ('delete', old.rowid, old.name, old.url, old.description, old.alias, old.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, name, url, description, alias, tags)
            VALUES ('delete', old.rowid, old.name, old.url, old.description, old.alias, old.tags);
            INSERT INTO entries_fts(rowid, name, url, description, alias, tags)
            VALUES (new.rowid, new.name, new.url, new.description, new.alias, new.tags);
        END;",
    )?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_connection_creates_entries_schema() {
        let conn = initialize_connection(Connection::open_in_memory().unwrap()).unwrap();

        let table_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN ('entries', 'entries_fts')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let trigger_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'trigger' AND name IN ('entries_ai', 'entries_ad', 'entries_au')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 2);
        assert_eq!(trigger_count, 3);
    }
}
```

- [ ] **Step 4: Run the initialization test and verify it passes**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml initialize_connection_creates_entries_schema
```

Expected: PASS.

- [ ] **Step 5: Write failing query tests**

Append this test module to `src-tauri/src/db/queries.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        crate::db::init::initialize_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn insert_entry_persists_all_fields() {
        let conn = test_conn();

        let entry = insert_entry(
            &conn,
            "GitHub",
            "https://github.com",
            "Developer account",
            "work login",
            "secret-123",
            "dev,code",
        )
        .unwrap();

        assert_eq!(entry.name, "GitHub");
        assert_eq!(entry.url, "https://github.com");
        assert_eq!(entry.description, "Developer account");
        assert_eq!(entry.alias, "work login");
        assert_eq!(entry.password_enc, "secret-123");
        assert_eq!(entry.password_nonce, "");
        assert_eq!(entry.tags, "dev,code");
        assert!(!entry.id.is_empty());
        assert!(!entry.created_at.is_empty());
        assert!(!entry.updated_at.is_empty());
    }

    #[test]
    fn empty_search_lists_recent_entries() {
        let conn = test_conn();
        insert_entry(&conn, "First", "", "", "", "one", "").unwrap();
        insert_entry(&conn, "Second", "", "", "", "two", "").unwrap();

        let results = search_entries(&conn, "").unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Second");
        assert_eq!(results[1].name, "First");
    }

    #[test]
    fn search_entries_matches_metadata_fields() {
        let conn = test_conn();
        insert_entry(
            &conn,
            "GitHub",
            "https://github.com",
            "Developer account",
            "work login",
            "secret-123",
            "dev,code",
        )
        .unwrap();

        let alias_results = search_entries(&conn, "work").unwrap();
        let tag_results = search_entries(&conn, "code").unwrap();

        assert_eq!(alias_results.len(), 1);
        assert_eq!(alias_results[0].name, "GitHub");
        assert_eq!(tag_results.len(), 1);
        assert_eq!(tag_results[0].name, "GitHub");
    }
}
```

- [ ] **Step 6: Run query tests and verify the current failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml db::queries::tests
```

Expected: FAIL if ordering or FTS behavior does not match the tests. If all tests pass, keep the tests and continue because they document current required behavior.

- [ ] **Step 7: Fix recent-entry ordering if needed**

If `empty_search_lists_recent_entries` fails because timestamps are equal at second precision, replace the `list_entries` SQL in `src-tauri/src/db/queries.rs` with:

```rust
pub fn list_entries(conn: &Connection) -> Result<Vec<EntrySearchResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, description, alias, tags, password_enc
         FROM entries
         ORDER BY updated_at DESC, rowid DESC
         LIMIT 20",
    )?;

    let results = stmt
        .query_map([], |row| {
            Ok(EntrySearchResult {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                alias: row.get(4)?,
                tags: row.get(5)?,
                password_enc: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(results)
}
```

- [ ] **Step 8: Run database tests and commit**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml db::
```

Expected: PASS.

Commit:

```bash
git add src-tauri/src/db/init.rs src-tauri/src/db/queries.rs
git commit -m "test: cover password entry database core"
```

## Task 2: Extract Create-Entry Validation Into Core Logic

**Files:**
- Create: `src-tauri/src/core/mod.rs`
- Create: `src-tauri/src/core/entry_input.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/entry.rs`

- [ ] **Step 1: Write failing validation tests**

Create `src-tauri/src/core/mod.rs`:

```rust
pub mod entry_input;
```

Create `src-tauri/src/core/entry_input.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_create_entry_rejects_blank_name() {
        let result = normalize_create_entry(
            "   ".to_string(),
            Some("https://example.com".to_string()),
            None,
            None,
            "secret".to_string(),
            None,
        );

        assert_eq!(result.unwrap_err(), "name is required");
    }

    #[test]
    fn normalize_create_entry_rejects_empty_password() {
        let result = normalize_create_entry(
            "Example".to_string(),
            None,
            None,
            None,
            "".to_string(),
            None,
        );

        assert_eq!(result.unwrap_err(), "password is required");
    }

    #[test]
    fn normalize_create_entry_trims_metadata() {
        let input = normalize_create_entry(
            "  GitHub  ".to_string(),
            Some("  https://github.com  ".to_string()),
            Some("  Developer account  ".to_string()),
            Some("  work login  ".to_string()),
            "secret".to_string(),
            Some("  dev,code  ".to_string()),
        )
        .unwrap();

        assert_eq!(input.name, "GitHub");
        assert_eq!(input.url, "https://github.com");
        assert_eq!(input.description, "Developer account");
        assert_eq!(input.alias, "work login");
        assert_eq!(input.password, "secret");
        assert_eq!(input.tags, "dev,code");
    }
}
```

Modify `src-tauri/src/lib.rs` so the module exists:

```rust
mod commands;
mod core;
mod db;
```

- [ ] **Step 2: Run validation tests and verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml core::entry_input::tests
```

Expected: FAIL because `normalize_create_entry` and `NormalizedEntryInput` are not defined.

- [ ] **Step 3: Implement the normalizer**

Replace `src-tauri/src/core/entry_input.rs` with:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedEntryInput {
    pub name: String,
    pub url: String,
    pub description: String,
    pub alias: String,
    pub password: String,
    pub tags: String,
}

pub fn normalize_create_entry(
    name: String,
    url: Option<String>,
    description: Option<String>,
    alias: Option<String>,
    password: String,
    tags: Option<String>,
) -> Result<NormalizedEntryInput, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("name is required".to_string());
    }

    if password.is_empty() {
        return Err("password is required".to_string());
    }

    Ok(NormalizedEntryInput {
        name,
        url: trim_optional(url),
        description: trim_optional(description),
        alias: trim_optional(alias),
        password,
        tags: trim_optional(tags),
    })
}

fn trim_optional(value: Option<String>) -> String {
    value.unwrap_or_default().trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_create_entry_rejects_blank_name() {
        let result = normalize_create_entry(
            "   ".to_string(),
            Some("https://example.com".to_string()),
            None,
            None,
            "secret".to_string(),
            None,
        );

        assert_eq!(result.unwrap_err(), "name is required");
    }

    #[test]
    fn normalize_create_entry_rejects_empty_password() {
        let result = normalize_create_entry(
            "Example".to_string(),
            None,
            None,
            None,
            "".to_string(),
            None,
        );

        assert_eq!(result.unwrap_err(), "password is required");
    }

    #[test]
    fn normalize_create_entry_trims_metadata() {
        let input = normalize_create_entry(
            "  GitHub  ".to_string(),
            Some("  https://github.com  ".to_string()),
            Some("  Developer account  ".to_string()),
            Some("  work login  ".to_string()),
            "secret".to_string(),
            Some("  dev,code  ".to_string()),
        )
        .unwrap();

        assert_eq!(input.name, "GitHub");
        assert_eq!(input.url, "https://github.com");
        assert_eq!(input.description, "Developer account");
        assert_eq!(input.alias, "work login");
        assert_eq!(input.password, "secret");
        assert_eq!(input.tags, "dev,code");
    }
}
```

- [ ] **Step 4: Run validation tests and verify they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml core::entry_input::tests
```

Expected: PASS.

- [ ] **Step 5: Wire the normalizer into the Tauri command**

Replace `src-tauri/src/commands/entry.rs` with:

```rust
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::core::entry_input::normalize_create_entry;
use crate::db::queries::{self, Entry};

#[tauri::command]
pub fn get_entry(id: String, conn: State<'_, Mutex<Connection>>) -> Result<Entry, String> {
    let db = conn.lock().map_err(|e| format!("Failed to access database: {}", e))?;
    queries::get_entry(&db, &id).map_err(|e| format!("Failed to retrieve entry {}: {}", id, e))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_entry(
    name: String,
    url: Option<String>,
    description: Option<String>,
    alias: Option<String>,
    password: String,
    tags: Option<String>,
    conn: State<'_, Mutex<Connection>>,
) -> Result<Entry, String> {
    let input = normalize_create_entry(name, url, description, alias, password, tags)?;

    let db = conn.lock().map_err(|e| format!("Failed to access database: {}", e))?;
    queries::insert_entry(
        &db,
        &input.name,
        &input.url,
        &input.description,
        &input.alias,
        &input.password,
        &input.tags,
    )
    .map_err(|e| format!("Failed to create entry: {}", e))
}
```

- [ ] **Step 6: Run Rust tests and commit**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

Commit:

```bash
git add src-tauri/src/core/mod.rs src-tauri/src/core/entry_input.rs src-tauri/src/lib.rs src-tauri/src/commands/entry.rs
git commit -m "refactor: centralize entry input validation"
```

## Task 3: Add Platform Adapter Modules

**Files:**
- Create: `src-tauri/src/platform/mod.rs`
- Create: `src-tauri/src/platform/hotkey.rs`
- Create: `src-tauri/src/platform/window.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing hotkey policy tests**

Create `src-tauri/src/platform/mod.rs`:

```rust
pub mod hotkey;
pub mod window;
```

Create `src-tauri/src/platform/hotkey.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_profile_uses_command_shift_p() {
        let spec = hotkey_for_profile(ShortcutProfile::Macos);

        assert_eq!(spec.label(), "Cmd+Shift+P");
    }

    #[test]
    fn windows_compatible_profile_uses_control_shift_p() {
        let spec = hotkey_for_profile(ShortcutProfile::WindowsCompatible);

        assert_eq!(spec.label(), "Ctrl+Shift+P");
    }
}
```

Create `src-tauri/src/platform/window.rs`:

```rust
pub const MAIN_WINDOW_LABEL: &str = "main";
```

Modify `src-tauri/src/lib.rs` so the module exists:

```rust
mod commands;
mod core;
mod db;
mod platform;
```

- [ ] **Step 2: Run platform tests and verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml platform::hotkey::tests
```

Expected: FAIL because `ShortcutProfile` and `hotkey_for_profile` are not defined.

- [ ] **Step 3: Implement hotkey policy and conversion**

Replace `src-tauri/src/platform/hotkey.rs` with:

```rust
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutProfile {
    Macos,
    WindowsCompatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeySpec {
    profile: ShortcutProfile,
}

impl HotkeySpec {
    pub fn label(self) -> &'static str {
        match self.profile {
            ShortcutProfile::Macos => "Cmd+Shift+P",
            ShortcutProfile::WindowsCompatible => "Ctrl+Shift+P",
        }
    }

    pub fn to_tauri_shortcut(self) -> Shortcut {
        match self.profile {
            ShortcutProfile::Macos => {
                Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyP)
            }
            ShortcutProfile::WindowsCompatible => {
                Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyP)
            }
        }
    }
}

pub fn current_profile() -> ShortcutProfile {
    if cfg!(target_os = "macos") {
        ShortcutProfile::Macos
    } else {
        ShortcutProfile::WindowsCompatible
    }
}

pub fn default_hotkey_spec() -> HotkeySpec {
    hotkey_for_profile(current_profile())
}

pub fn hotkey_for_profile(profile: ShortcutProfile) -> HotkeySpec {
    HotkeySpec { profile }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_profile_uses_command_shift_p() {
        let spec = hotkey_for_profile(ShortcutProfile::Macos);

        assert_eq!(spec.label(), "Cmd+Shift+P");
    }

    #[test]
    fn windows_compatible_profile_uses_control_shift_p() {
        let spec = hotkey_for_profile(ShortcutProfile::WindowsCompatible);

        assert_eq!(spec.label(), "Ctrl+Shift+P");
    }
}
```

- [ ] **Step 4: Run hotkey tests and verify they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml platform::hotkey::tests
```

Expected: PASS.

- [ ] **Step 5: Implement window adapter helpers**

Replace `src-tauri/src/platform/window.rs` with:

```rust
use tauri::{Manager, WebviewWindow};

pub const MAIN_WINDOW_LABEL: &str = "main";

pub fn configure_main_window(window: &WebviewWindow) {
    #[cfg(debug_assertions)]
    {
        window.open_devtools();
    }

    hide_on_blur(window);
}

pub fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn hide_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window
            .hide()
            .map_err(|e| format!("Failed to hide main window: {}", e))?;
    }

    Ok(())
}

fn hide_on_blur(window: &WebviewWindow) {
    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = w.hide();
        }
    });
}
```

- [ ] **Step 6: Wire adapters into Tauri setup**

Replace `src-tauri/src/lib.rs` with:

```rust
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

mod commands;
mod core;
mod db;
mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        platform::window::toggle_main_window(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::search::search_entries,
            commands::entry::get_entry,
            commands::entry::create_entry,
            commands::clipboard::copy_to_clipboard,
        ])
        .setup(|app| {
            let window = app
                .get_webview_window(platform::window::MAIN_WINDOW_LABEL)
                .expect("main window must exist");
            platform::window::configure_main_window(&window);

            let hotkey = platform::hotkey::default_hotkey_spec();
            app.global_shortcut()
                .register(hotkey.to_tauri_shortcut())
                .unwrap_or_else(|e| {
                    panic!("failed to register global shortcut {}: {}", hotkey.label(), e)
                });

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            let db_path = app_data_dir.join("prompt-password.db");
            let conn = db::init::initialize(&db_path).expect("failed to initialize database");
            app.manage(Mutex::new(conn));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Run full Rust tests and build**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

Expected: both PASS.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/platform/mod.rs src-tauri/src/platform/hotkey.rs src-tauri/src/platform/window.rs src-tauri/src/lib.rs
git commit -m "refactor: isolate platform shortcut and window behavior"
```

## Task 4: Hide Panel After Successful Copy

**Files:**
- Modify: `src-tauri/src/commands/clipboard.rs`

- [ ] **Step 1: Write the failing clipboard policy test**

Replace `src-tauri/src/commands/clipboard.rs` with:

```rust
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostCopyWindowAction {
    HideMainPanel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_copy_hides_main_panel() {
        assert_eq!(post_copy_window_action(), PostCopyWindowAction::HideMainPanel);
    }
}

#[tauri::command]
pub fn copy_to_clipboard(text: String, app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))
}
```

- [ ] **Step 2: Run the clipboard test and verify it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml successful_copy_hides_main_panel
```

Expected: FAIL because `post_copy_window_action` is not defined.

- [ ] **Step 3: Implement post-copy hide behavior**

Replace `src-tauri/src/commands/clipboard.rs` with:

```rust
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::platform::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostCopyWindowAction {
    HideMainPanel,
}

pub fn post_copy_window_action() -> PostCopyWindowAction {
    PostCopyWindowAction::HideMainPanel
}

fn apply_post_copy_window_action(
    app: &tauri::AppHandle,
    action: PostCopyWindowAction,
) -> Result<(), String> {
    match action {
        PostCopyWindowAction::HideMainPanel => window::hide_main_window(app),
    }
}

#[tauri::command]
pub fn copy_to_clipboard(text: String, app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    apply_post_copy_window_action(&app, post_copy_window_action())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_copy_hides_main_panel() {
        assert_eq!(post_copy_window_action(), PostCopyWindowAction::HideMainPanel);
    }
}
```

- [ ] **Step 4: Run clipboard test and verify it passes**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml successful_copy_hides_main_panel
```

Expected: PASS.

- [ ] **Step 5: Run build verification and commit**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

Expected: both PASS.

Commit:

```bash
git add src-tauri/src/commands/clipboard.rs
git commit -m "feat: hide panel after copying password"
```

## Task 5: Align README With Tauri Direction

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace outdated planned Electron structure**

Edit `README.md` so the technology and structure sections say:

````markdown
## Technology Direction | 技术方向

- **Shell**: Tauri 2.x
- **Core**: Rust
- **UI**: Preact + TypeScript
- **Storage**: SQLite through `rusqlite`
- **Target behavior**: macOS-first, Windows-compatible

Prompt Password uses one shared codebase. Rust owns storage, search, clipboard, hotkeys, and platform adapters. Preact owns the compact search panel and entry form.

## Project Structure | 项目结构

```text
prompt-password/
├── src/                         # Shared Preact UI
│   ├── components/              # Search panel and entry form components
│   ├── hooks/                   # UI data and clipboard hooks
│   ├── lib/                     # Typed Tauri invoke wrappers
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                   # Rust/Tauri application shell
│   ├── src/
│   │   ├── commands/            # Tauri commands exposed to the UI
│   │   ├── core/                # Pure product logic
│   │   ├── db/                  # SQLite initialization and queries
│   │   ├── platform/            # OS-specific adapters
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/superpowers/
├── package.json
└── vite.config.ts
```
````

- [ ] **Step 2: Verify README no longer references Electron as the planned implementation**

Run:

```bash
rg -n "Electron|src/main|renderer|Technology choices" README.md
```

Expected: no output.

- [ ] **Step 3: Run final project verification**

Run:

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
git status --short --ignored .gh-multica .codex .multica
```

Expected:

```text
!! .gh-multica/
```

The build and Rust tests should exit 0.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: align readme with tauri architecture"
```

## Final Verification

- [ ] **Step 1: Run all verification commands**

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
git status --short --branch
git status --ignored --short .gh-multica .codex .multica
```

Expected:

- `npm run build` exits 0.
- `cargo test --manifest-path src-tauri/Cargo.toml` exits 0.
- `git status --short --branch` shows the current branch with no unstaged or untracked application files.
- `git status --ignored --short .gh-multica .codex .multica` shows `.gh-multica/` as ignored if it exists locally.

- [ ] **Step 2: Push the implementation branch**

```bash
git push
```

Expected: branch `feat/cross-platform-initialization` is updated on `origin`.
