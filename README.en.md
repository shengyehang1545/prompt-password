# Prompt Password

Prompt Password is a local password quick-search utility, currently focused on macOS with a Windows-compatible path kept in the codebase. It runs in the background, opens from a global hotkey, and lets you search for a saved password and copy it quickly.

Passwords are stored in a local SQLite database. Secret password values are encrypted with a key derived from the Master Password.

## Features

- macOS background app with menu bar tray controls.
- Global hotkey: `Cmd+Shift+K` on macOS, `Ctrl+Shift+K` for the Windows-compatible profile.
- Local encrypted vault unlocked by the Master Password.
- Search by name, URL, description, alias, username, and tags.
- Click a search result to copy the password.
- Add, edit, and delete password entries.
- Password templates. A template only fills the password field and is not linked to existing entries.
- Built-in password generator: 12 characters by default, with at least one digit, one uppercase letter, one lowercase letter, and exactly 2 special characters from `!@#$%^&*,?`.
- Settings page for launch at login, hotkey customization, vault locking, import, and export.
- Export formats:
  - Encrypted JSON backup for restore and import.
  - Plaintext CSV, protected by Master Password verification.
  - Plaintext TXT, protected by Master Password verification.
- Encrypted JSON import. Existing entries or templates with the same `id` or name are skipped instead of overwritten.

## Tech Stack

- Tauri 2
- Rust
- Preact + TypeScript
- SQLite / `rusqlite`
- AES-256-GCM
- Argon2id

## Quick Start

Install dependencies:

```bash
npm install
```

Start in development mode:

```bash
npm run tauri dev
```

Build the macOS app:

```bash
npm run tauri build -- --bundles app
```

The built app is generated at:

```text
src-tauri/target/release/bundle/macos/prompt-password.app
```

Run checks:

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

## Usage

### First Launch

1. Open the app.
2. Enter a new Master Password.
3. The app creates the local encrypted vault.
4. Use the same Master Password to unlock it later.

The Master Password is not saved. If it is lost, the encrypted vault cannot be recovered.

### Open And Search

- Press `Cmd+Shift+K` on macOS to show or hide the search panel.
- Type a keyword to search entries.
- Click an entry to copy its password.
- Use arrow keys to select an entry, then press `Enter` to copy.
- Press `Escape` or click outside the window to hide the panel.

### Add Passwords

1. Click the `+` button next to the search field.
2. Fill in name, URL, username, tags, and password.
3. Click `Generate` to create a password with the default rule.
4. Save the entry.

When editing an existing entry, the password field does not show the old password. Leave it empty to keep the current password, or enter/generate a new password to replace it.

### Password Templates

Templates are useful for reusable password values.

- Selecting a template only fills the current form's password field.
- Templates are not linked to password entries.
- Editing a template does not affect existing entries.
- Deleting a template does not affect existing entries.

### Settings

Click the gear button next to the search field to open Settings.

Current settings:

- Launch at login: starts the macOS app in the background.
- Hotkey: changes the global hotkey and re-registers it immediately.
- Import/export: imports encrypted JSON and exports JSON/CSV/TXT.
- Lock vault: clears the in-memory vault key and returns to the unlock screen.

### Import And Export

Encrypted JSON backup:

- Requires Master Password verification before export.
- Does not contain plaintext passwords.
- Suitable for backup and sync workflows.
- Import requires the same Master Password to decrypt.

CSV / TXT export:

- Requires Master Password verification before export.
- Contains plaintext passwords.
- Intended only for temporary migration or manual inspection.
- CSV quotes every field and escapes quotes, so passwords containing commas, newlines, or quotes keep the file structure valid.

## Data And Security Notes

- Data is stored locally.
- Password values are encrypted at rest.
- Names, URLs, usernames, aliases, descriptions, and tags are stored in plaintext to support fast local search.
- The app decrypts password values briefly when copying, applying a template, exporting, or importing.
- JSON backup files are encrypted.
- CSV and TXT export files are plaintext.

## Sync Recommendation

The safest short-term sync workflow is encrypted JSON backup:

1. Export encrypted JSON on one device.
2. Put the JSON file in iCloud Drive, Dropbox, OneDrive, or another sync folder.
3. Import the JSON file on another device.

Direct SQLite database sync is not recommended. Concurrent writes can cause overwrite conflicts or database corruption.

A better future sync design would synchronize end-to-end encrypted change logs instead of a live database file.

## Project Structure

```text
prompt-password/
├── src/                         # Preact frontend
│   ├── components/              # Search, forms, settings
│   ├── hooks/                   # Frontend hooks
│   ├── lib/                     # Tauri invoke wrappers
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                   # Rust / Tauri app layer
│   ├── src/
│   │   ├── commands/            # Tauri commands exposed to the frontend
│   │   ├── core/                # Vault and entry logic
│   │   ├── crypto/              # Encryption and key derivation
│   │   ├── db/                  # SQLite schema and queries
│   │   ├── platform/            # Hotkey, tray, login item, window behavior
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

## License

Private project for personal use.
