# Prompt Password Cross-Platform Initialization Design

Date: 2026-06-16
Branch: feat/cross-platform-initialization

## Decision

Prompt Password will stay as one cross-platform Tauri 2 application instead of splitting into separate native macOS and Windows apps.

The product direction is macOS-first, Windows-compatible:

- Keep one shared Rust core for storage, search, encryption, migrations, and clipboard policy.
- Keep one shared Preact UI for the search panel, entry list, and entry form.
- Isolate OS-specific behavior behind small platform adapters for hotkeys, window behavior, startup, permissions, and packaging.

This keeps macOS polish high without duplicating security-sensitive code across two applications.

## Goals

- Build a lightweight password reminder and quick-copy tool.
- Summon the app with a global hotkey like an input method or launcher.
- Search password hints and metadata quickly.
- Copy the selected password to the clipboard with minimal friction.
- Store data locally and prepare for encrypted-at-rest password storage.
- Keep the same vault format usable on macOS and Windows.

## Non-Goals

- Do not build separate SwiftUI and Windows-native products.
- Do not add cloud sync in the initialization phase.
- Do not optimize for Linux in the first pass.
- Do not expose plaintext password storage as a production-ready security model.

## Architecture

The application has three layers.

### Rust Core

Rust owns all security-sensitive and platform-sensitive work:

- SQLite database setup and migrations.
- Entry CRUD and search queries.
- Future password encryption and vault unlock flow.
- Clipboard copy behavior.
- Global shortcut registration through Tauri plugins.
- Window show, hide, focus, and blur handling.

The Rust core should avoid macOS-only assumptions unless they live behind a platform adapter.

### UI Layer

Preact owns the interactive panel:

- Search input.
- Result list.
- Keyboard navigation.
- Add-entry form.
- Masked password display.
- Error and empty states.

The UI should call typed wrappers in `src/lib/tauri.ts` instead of invoking Tauri commands directly from components.

### Platform Adapter Layer

Platform differences should be centralized instead of scattered:

- macOS hotkey default: `Cmd+Shift+P`.
- Windows hotkey default: `Ctrl+Shift+P`.
- macOS window behavior: compact floating panel, focus-on-summon, hide-on-blur.
- Windows window behavior: compact always-on-top panel with equivalent focus and hide behavior where supported.
- Future startup integration: LaunchAgent on macOS, Startup Apps or registry-backed setup on Windows.
- Future permission help: macOS accessibility guidance, Windows hotkey conflict messaging.

## Data Model

The current entry model is acceptable for initialization:

- `id`
- `name`
- `url`
- `description`
- `alias`
- `password_enc`
- `password_nonce`
- `tags`
- `created_at`
- `updated_at`

For Phase 1, the project may keep plaintext passwords in the existing `password_enc` field as a temporary development representation. The field name should remain encryption-ready so the schema does not need a disruptive rename later.

Phase 2 should replace plaintext storage with encrypted password payloads:

- Derive a vault key from a master password.
- Encrypt only the password field at first.
- Keep searchable metadata plaintext for fast local search.
- Keep `password_nonce` populated once encryption is enabled.

## Product Behavior

The initial user flow is:

1. Press the global hotkey.
2. The compact search panel appears and focuses the input.
3. Type part of a site, name, alias, description, or tag.
4. Navigate results with arrow keys.
5. Press Enter or click a result to copy the password.
6. Hide the panel after copy or when focus is lost.

Adding an entry should stay available from the panel for now. A separate full settings window is not required in the initialization phase.

## Error Handling

The app should surface short, specific errors:

- Database initialization failure.
- Search command failure.
- Entry creation validation failure.
- Clipboard copy failure.
- Global hotkey registration failure.

Errors shown in the panel should be actionable but compact. Detailed diagnostics belong in logs, not the main UI.

## Testing Strategy

Initialization should add tests around the Rust core first:

- Database schema initializes successfully.
- Entry insertion persists all fields.
- Empty search lists recent entries.
- Non-empty search matches metadata through FTS.
- Required fields are validated before insertion.

Frontend verification should use the existing TypeScript build as the baseline until UI tests are added:

- `npm run build`
- `cargo test --manifest-path src-tauri/Cargo.toml`

Later UI behavior can be covered with browser or Tauri integration tests, using Firefox only when Playwright is required.

## Repository Workflow

Work should continue on feature branches created from `main`.

The current initialization branch is:

`feat/cross-platform-initialization`

Local agent and automation folders must stay ignored:

- `.gh-multica/`
- `.codex/`
- `.multica/`

## Acceptance Criteria

- The repository has a `main` branch on `origin`.
- New initialization work happens on a branch based on `main`.
- The design keeps one shared Tauri/Rust application.
- The codebase has a clear place for OS-specific behavior.
- The next implementation plan can be written without deciding between SwiftUI, Electron, and Tauri again.
