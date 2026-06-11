# Prompt Password — Implementation Plan | 实施文档

> Version: 1.0 | Date: 2026-06-11
> Issue: SYD-2

---

## 1. Technology Selection | 技术选型

### 1.1 Cross-Platform Framework: Tauri 2.x

| Option | Bundle Size | Memory | Hotkey Support | Verdict |
|--------|------------|--------|----------------|---------|
| **Tauri 2.x** | ~3–5 MB | ~10–20 MB | Native via `tauri-plugin-global-shortcut` | **Selected** |
| Electron | ~80–120 MB | ~100+ MB | Requires `globalShortcut` module | Rejected — too heavy for a utility app |

**Rationale**: Prompt Password is a lightweight utility that should launch instantly. Tauri's Rust backend provides native hotkey registration, small bundle, and low memory. Tauri 2.x stabilised cross-platform support and has first-class plugins for global shortcuts, clipboard, and filesystem.

### 1.2 Frontend: Vanilla TypeScript + Preact

| Option | Bundle Size | Dev Experience | Verdict |
|--------|------------|----------------|---------|
| **Preact + TypeScript** | ~4 KB runtime | Lightweight, hooks-based | **Selected** |
| React | ~40 KB runtime | Familiar ecosystem | Rejected — unnecessary overhead for a single-panel UI |
| Svelte | ~2 KB runtime | Compile-time | Viable alternative, but Preact's TSX is more conventional |

**Rationale**: The UI is a single search panel — a text input and a filtered list. Preact provides React-like ergonomics at minimal cost.

### 1.3 Data Storage: SQLite (via `rusqlite`) + AES-256-GCM Encryption

| Option | Query Performance | Encryption | Verdict |
|--------|-------------------|------------|---------|
| **SQLite + AES-256-GCM** | Indexed fuzzy search | Field-level encryption in Rust | **Selected** |
| JSON file | Linear scan for search | Whole-file encryption | Rejected — no indexing, poor search at scale |
| sled / redb | Key-value only | Manual | Rejected — overkill, no SQL ergonomics |

**Rationale**: SQLite provides indexed queries for fuzzy search. Password fields are encrypted at rest using AES-256-GCM with a key derived from a user master password via Argon2id. Non-sensitive fields (name, URL, tags) are stored in plaintext for fast search.

### 1.4 Dependency Summary

| Layer | Technology | Version |
|-------|-----------|---------|
| Shell | Tauri | 2.x |
| Backend | Rust | 1.80+ |
| Frontend | Preact + TypeScript | Preact 10.x, TS 5.x |
| Database | SQLite via `rusqlite` | 0.32+ |
| Encryption | `aes-gcm` + `argon2` crates | latest |
| Password Gen | `rand` crate | 0.8+ |
| Fuzzy Search | `fuse-rust` or custom SQLite FTS5 | — |
| Build | Vite | 6.x |
| Bundle | tauri-bundler | (via Tauri CLI) |

---

## 2. Architecture Design | 架构设计

### 2.1 System Layers

```
┌─────────────────────────────────────────────────┐
│             Global Hotkey Layer                  │  Rust (tauri-plugin-global-shortcut)
│   Registers system-wide shortcut, shows/hides   │
│   the search window on key press                │
├─────────────────────────────────────────────────┤
│             Search Panel UI                      │  Preact + TypeScript (renderer)
│   Text input → fuzzy matching → filtered list   │
│   Password masking (Ab******) → clipboard copy  │
├─────────────────────────────────────────────────┤
│             Data & Crypto Layer                  │  Rust (core)
│   SQLite storage, field-level encryption,       │
│   CRUD commands, Argon2id key derivation        │
└─────────────────────────────────────────────────┘
```

### 2.2 Module Breakdown

```
prompt-password/
├── src-tauri/                          # Rust backend
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                     # Tauri entry, window config
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── search.rs              # Tauri command: fuzzy search entries
│   │   │   ├── entry.rs               # Tauri command: CRUD for entries
│   │   │   ├── clipboard.rs           # Tauri command: copy to clipboard
│   │   │   └── generator.rs           # Tauri command: random password generation
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── init.rs                # SQLite init, migrations
│   │   │   └── queries.rs             # SQL query helpers
│   │   ├── crypto/
│   │   │   ├── mod.rs
│   │   │   ├── kdf.rs                 # Argon2id key derivation
│   │   │   ├── encrypt.rs             # AES-256-GCM encrypt
│   │   │   └── decrypt.rs             # AES-256-GCM decrypt
│   │   ├── generator/
│   │   │   ├── mod.rs
│   │   │   └── password.rs            # Random password generation
│   │   └── hotkey.rs                  # Global shortcut registration
│   └── tauri.conf.json
├── src/                                # Frontend (Vite + Preact)
│   ├── App.tsx                         # Root component
│   ├── components/
│   │   ├── SearchInput.tsx             # Search text input
│   │   ├── EntryList.tsx              # Filtered result list
│   │   ├── EntryItem.tsx              # Single entry row (masked password)
│   │   ├── MaskedPassword.tsx         # "Ab******" display component
│   │   └── PasswordGenerator.tsx      # Random password generator widget
│   ├── hooks/
│   │   ├── useSearch.ts               # Fuzzy search hook (debounced)
│   │   └── useClipboard.ts            # Clipboard copy hook
│   ├── lib/
│   │   └── tauri.ts                   # Typed Tauri invoke wrappers
│   ├── main.tsx                        # Preact entry
│   └── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
└── IMPLEMENTATION.md                   # This file
```

### 2.3 IPC Contract (Tauri Commands)

| Command | Params | Returns | Description |
|---------|--------|---------|-------------|
| `search_entries` | `query: string` | `Vec<EntrySearchResult>` | Fuzzy search across name/url/description/tags |
| `get_entry` | `id: string` | `Entry` | Full entry with decrypted password |
| `create_entry` | `EntryInput` | `Entry` | Create new entry |
| `update_entry` | `id: string, EntryInput` | `Entry` | Update existing entry |
| `delete_entry` | `id: string` | `()` | Delete entry |
| `copy_to_clipboard` | `text: string` | `()` | Copy password to system clipboard |
| `unlock_vault` | `master_password: string` | `bool` | Derive key, verify, store in memory |
| `lock_vault` | — | `()` | Clear decryption key from memory |
| `generate_password` | `GeneratorConfig` | `string` | Generate a random password and copy to clipboard |

### 2.4 Password Generator

A built-in password generator available when creating or editing entries. One click generates a strong random password, fills the password field, and copies it to clipboard.

**Default rules**: 12 characters, includes uppercase + lowercase + digits + 2 special characters. User can configure length and special character count.

**Character pool**:
- Uppercase: `A-Z`
- Lowercase: `a-z`
- Digits: `0-9`
- Special characters: `!@#$%^&*()-_=+`

**Generation algorithm**: Use `rand` crate's CSPRNG (`OsRng`). Shuffle guaranteed minimum characters (1 upper, 1 lower, 1 digit, N special) with Fisher-Yates, then fill remaining slots from the full pool.

```rust
struct GeneratorConfig {
    length: u32,              // default: 12
    special_count: u32,       // default: 2
}
```

**UI flow**: In the entry form, a "Generate" button next to the password field. Clicking it generates a password, fills the field, and copies to clipboard with a brief "Copied!" toast.

---

## 3. Data Model | 数据模型

### 3.1 Password Entry Schema

```sql
CREATE TABLE entries (
    id          TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name        TEXT NOT NULL,                        -- 名称 (plaintext, searchable)
    url         TEXT NOT NULL DEFAULT '',              -- 网站链接 (plaintext, searchable)
    description TEXT NOT NULL DEFAULT '',              -- 描述 (plaintext, searchable)
    alias       TEXT NOT NULL DEFAULT '',              -- 密码别称 (plaintext, searchable)
    password_enc TEXT NOT NULL,                        -- 密码 (AES-256-GCM encrypted, base64)
    password_nonce TEXT NOT NULL,                      -- AES nonce (base64)
    tags        TEXT NOT NULL DEFAULT '',              -- 标签, comma-separated (plaintext, searchable)
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- FTS5 virtual table for fuzzy search
CREATE VIRTUAL TABLE entries_fts USING fts5(
    name, url, description, alias, tags,
    content=entries,
    content_rowid=rowid
);

-- Triggers to keep FTS in sync
CREATE TRIGGER entries_ai AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts(rowid, name, url, description, alias, tags)
    VALUES (new.rowid, new.name, new.url, new.description, new.alias, new.tags);
END;

CREATE TRIGGER entries_ad AFTER DELETE ON entries BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, name, url, description, alias, tags)
    VALUES ('delete', old.rowid, old.name, old.url, old.description, old.alias, old.tags);
END;

CREATE TRIGGER entries_au AFTER UPDATE ON entries BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, name, url, description, alias, tags)
    VALUES ('delete', old.rowid, old.name, old.url, old.description, old.alias, old.tags);
    INSERT INTO entries_fts(rowid, name, url, description, alias, tags)
    VALUES (new.rowid, new.name, new.url, new.description, new.alias, new.tags);
END;
```

### 3.2 Encryption Scheme

```
Master Password (user input)
        │
        ▼
   Argon2id (m=65536, t=3, p=2)
        │
        ▼
   256-bit DEK (Data Encryption Key)
        │
        ▼
   AES-256-GCM encrypt/decrypt password field
        │
        ├── Random 96-bit nonce per entry
        ├── Associated data: entry id (binds ciphertext to entry)
        └── Output: base64(ciphertext + tag)
```

**Key Management**:
- DEK is derived on `unlock_vault` and held in a `Zeroizing` wrapper (`zeroize` crate) in Rust process memory.
- DEK is never written to disk.
- On `lock_vault` or app exit, DEK memory is zeroed.
- Master password verification: store a known-plaintext verification blob (encrypted with DEK) at vault creation; attempt decrypt on unlock to verify.

### 3.3 TypeScript Types

```typescript
interface Entry {
  id: string;
  name: string;
  url: string;
  description: string;
  alias: string;
  password: string;        // decrypted, only in memory after unlock
  tags: string[];          // split from comma-separated storage
  createdAt: string;
  updatedAt: string;
}

interface EntrySearchResult {
  id: string;
  name: string;
  url: string;
  description: string;
  alias: string;
  tags: string[];
  passwordPreview: string;  // e.g. "Ab******"
}

interface EntryInput {
  name: string;
  url?: string;
  description?: string;
  alias?: string;
  password: string;
  tags?: string[];
}
```

---

## 4. Phased Implementation Plan | 分阶段实施计划

### Phase 1: Minimum Viable — Hotkey + Search + Copy

> Goal: Press hotkey → type query → see results → copy password. No encryption yet.

| Step | Action | Files | Dependencies | Risk |
|------|--------|-------|-------------|------|
| 1.1 | Initialize Tauri 2.x project with Vite + Preact + TypeScript | `package.json`, `src-tauri/Cargo.toml`, `vite.config.ts`, `tsconfig.json` | None | Low |
| 1.2 | Configure Tauri window: frameless, always-on-top, centered, hide-on-blur | `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs` | 1.1 | Low |
| 1.3 | Register global hotkey (default `Cmd+Shift+P` / `Ctrl+Shift+P`) | `src-tauri/src/hotkey.rs` | 1.1 | Medium — OS permissions |
| 1.4 | Create search panel UI: input + result list | `src/App.tsx`, `src/components/SearchInput.tsx`, `src/components/EntryList.tsx`, `src/components/EntryItem.tsx` | 1.2 | Low |
| 1.5 | Implement SQLite storage with plaintext passwords (no encryption) | `src-tauri/src/db/init.rs`, `src-tauri/src/db/queries.rs` | 1.1 | Low |
| 1.6 | Implement Tauri commands: search, copy, CRUD | `src-tauri/src/commands/search.rs`, `src-tauri/src/commands/entry.rs`, `src-tauri/src/commands/clipboard.rs` | 1.4, 1.5 | Low |
| 1.7 | Wire frontend to backend: search + copy flow | `src/hooks/useSearch.ts`, `src/hooks/useClipboard.ts`, `src/lib/tauri.ts` | 1.4, 1.6 | Low |
| 1.8 | Keyboard navigation: arrow keys + Enter to select | `src/components/EntryList.tsx` | 1.7 | Low |

**Phase 1 Acceptance Criteria**:
- [ ] Global hotkey shows/hides the search panel
- [ ] Typing in search input filters entries in real-time
- [ ] Selecting an entry copies password to clipboard
- [ ] Clicking outside the panel hides it
- [ ] App launches on macOS and Windows

---

### Phase 2: Core Experience — Secure Display + Encryption

> Goal: Full security model — encrypted vault, password masking, unlock flow.

| Step | Action | Files | Dependencies | Risk |
|------|--------|-------|-------------|------|
| 2.1 | Implement Argon2id key derivation | `src-tauri/src/crypto/kdf.rs` | 1.5 | Low |
| 2.2 | Implement AES-256-GCM encrypt/decrypt | `src-tauri/src/crypto/encrypt.rs`, `src-tauri/src/crypto/decrypt.rs` | 2.1 | Medium — correctness critical |
| 2.3 | Add vault migration: encrypt existing plaintext passwords | `src-tauri/src/db/init.rs` (migration) | 2.2 | Medium — data loss risk |
| 2.4 | Implement `unlock_vault` / `lock_vault` commands | `src-tauri/src/commands/mod.rs` | 2.1, 2.2 | Low |
| 2.5 | Add unlock screen UI (master password input) | `src/components/UnlockScreen.tsx` | 2.4 | Low |
| 2.6 | Implement password masking component (first 2 chars + asterisks) | `src/components/MaskedPassword.tsx` | 1.4 | Low |
| 2.7 | Update search results to return masked passwords | `src-tauri/src/commands/search.rs` | 2.3, 2.6 | Low |
| 2.8 | Add entry management UI (add/edit/delete) | `src/components/EntryForm.tsx`, `src/components/EntryDetail.tsx` | 1.6, 2.3 | Low |
| 2.9 | Auto-lock after inactivity timeout (5 min) | `src-tauri/src/main.rs` | 2.4 | Low |
| 2.10 | Implement password generator: Rust command + frontend widget | `src-tauri/src/generator/password.rs`, `src-tauri/src/commands/generator.rs`, `src/components/PasswordGenerator.tsx` | 1.6, 2.8 | Low |

**Phase 2 Acceptance Criteria**:
- [ ] Vault is encrypted at rest — passwords not readable without master password
- [ ] Search results show `Ab******` format, never full password
- [ ] Selecting an entry copies decrypted password to clipboard
- [ ] Unlock screen appears on app launch and after auto-lock
- [ ] Can add, edit, and delete entries through UI
- [ ] Master password verification rejects wrong passwords
- [ ] New entry form has "Generate" button that creates a random password (default 12 chars, 2 special) and copies to clipboard

---

### Phase 3: Edge Cases — Error Handling, Import/Export, Polish

> Goal: Production-ready robustness.

| Step | Action | Files | Dependencies | Risk |
|------|--------|-------|-------------|------|
| 3.1 | Error handling: DB errors, crypto errors, clipboard failures | All command files | 2.x | Low |
| 3.2 | Empty state UI (no entries, no search results) | `src/components/EmptyState.tsx` | 1.4 | Low |
| 3.3 | CSV import for bulk entry creation | `src-tauri/src/commands/import.rs` | 2.3 | Low |
| 3.4 | Encrypted JSON export/backup | `src-tauri/src/commands/export.rs` | 2.2 | Medium — export security |
| 3.5 | Duplicate detection on entry creation | `src-tauri/src/commands/entry.rs` | 1.6 | Low |
| 3.6 | Password strength indicator on entry form | `src/components/PasswordStrength.tsx` | 2.8 | Low |
| 3.7 | Confirmation dialog for destructive actions (delete, overwrite) | `src/components/ConfirmDialog.tsx` | 2.8 | Low |
| 3.8 | Accessibility: keyboard-only navigation, ARIA labels | All UI components | 1.x | Low |

**Phase 3 Acceptance Criteria**:
- [ ] All error states show user-friendly messages (no panics, no raw errors)
- [ ] Can import entries from CSV
- [ ] Can export encrypted backup of all entries
- [ ] Duplicate names trigger a warning
- [ ] Delete requires confirmation
- [ ] Full keyboard navigation without mouse

---

### Phase 4: Optimization — Performance, Customization, Monitoring

> Goal: Speed and polish for daily use.

| Step | Action | Files | Dependencies | Risk |
|------|--------|-------|-------------|------|
| 4.1 | Hotkey customization in settings | `src-tauri/src/hotkey.rs`, `src/components/Settings.tsx` | 1.3 | Medium — OS-specific quirks |
| 4.2 | Search performance: benchmark with 1000+ entries, optimize FTS5 ranking | `src-tauri/src/db/queries.rs` | 1.5 | Low |
| 4.3 | Window animation: fade-in/out on show/hide | `src/App.tsx`, CSS | 1.2 | Low |
| 4.4 | Clipboard auto-clear: clear password from clipboard after 30s | `src-tauri/src/commands/clipboard.rs` | 1.6 | Medium — OS clipboard API limits |
| 4.5 | Dark/light theme support | CSS variables, `src/App.tsx` | 1.4 | Low |
| 4.6 | App auto-start on login (optional setting) | `src-tauri/tauri.conf.json`, `src/components/Settings.tsx` | 1.1 | Medium — OS-specific |
| 4.7 | Crash reporting / error logging to local file | `src-tauri/src/main.rs` | 3.1 | Low |

**Phase 4 Acceptance Criteria**:
- [ ] User can change the global hotkey
- [ ] Search responds within 50ms for 1000 entries
- [ ] Password is automatically cleared from clipboard after 30 seconds
- [ ] UI supports dark and light themes
- [ ] Optional auto-start on login

---

## 5. Risk Assessment | 风险评估

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| **macOS Accessibility permissions** for global hotkey | High — hotkey won't register | Medium | Prompt user to grant permission; provide fallback tray icon to open panel |
| **Windows hotkey conflicts** with other apps | Medium — hotkey may not register | Medium | Allow hotkey customization; detect registration failure and notify user |
| **Encryption key in memory** could be swapped to disk | High — key exposed in swap file | Low | Use `mlock` (Unix) / `VirtualLock` (Windows) to prevent paging; zeroize on lock |
| **SQLite corruption** from crash during write | Medium — data loss | Low | Use WAL mode; periodic backup in Phase 3 export; transactional writes |
| **Clipboard security** — other apps can read clipboard | Medium — password exposed briefly | Medium | Auto-clear clipboard after 30s (Phase 4); document risk to user |
| **Cross-platform UI differences** — font rendering, window behavior | Low — visual inconsistencies | Medium | Test on both platforms; use CSS normalisation; frameless window minimises OS chrome |
| **Argon2id performance** on low-end hardware | Low — slow unlock | Low | Use conservative parameters (m=65536, t=3, p=2); benchmark on target hardware in Phase 2 |

---

## 6. Success Criteria | 成功标准

### Phase 1 Checklist
- [ ] App launches on macOS and Windows
- [ ] `Cmd/Ctrl+Shift+P` toggles search panel globally
- [ ] Fuzzy search returns results within 100ms for 100 entries
- [ ] Selecting a result copies password to clipboard
- [ ] Panel hides on blur or Escape

### Phase 2 Checklist
- [ ] Passwords are AES-256-GCM encrypted at rest
- [ ] Master password is required on launch
- [ ] Password preview shows `Ab******` format (first 2 chars visible)
- [ ] Full password only copied to clipboard, never displayed
- [ ] Auto-lock after 5 minutes of inactivity
- [ ] Add/edit/delete entries through UI

### Phase 3 Checklist
- [ ] No unhandled errors or panics in normal use
- [ ] CSV import works with common formats
- [ ] Encrypted JSON export/restore works
- [ ] Full keyboard navigation
- [ ] Confirmation required for destructive actions

### Phase 4 Checklist
- [ ] Custom hotkey registration works on both platforms
- [ ] Search under 50ms for 1000+ entries
- [ ] Clipboard auto-clears after 30 seconds
- [ ] Dark and light themes available
- [ ] Optional auto-start on login
