# macOS Full-Screen Space Implementation Plan

> Execute in the current checkout. Do not create a git worktree. Keep the installed app and its data untouched until the final controlled deployment step.

**Goal:** Validate and, only if safe, integrate a native macOS panel that can appear over a standard full-screen app while preserving the stable Prompt Password behavior.

**Baseline:** Commit `5cc9043` / app version `0.1.7` is the known-good production baseline. The previous 0.1.10 experiment is preserved at `/tmp/prompt-password-macos-space-experiment.patch` and must not be deployed.

---

## Task 1: Restore and verify the stable baseline

**Files:** `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/src/platform/window.rs`

- [x] Restore source and version fields to the stable 0.1.7 implementation.
- [x] Record bundle identifier/version and database/WAL metadata without reading secrets.
- [x] Run `npm run build`, Rust tests, and Clippy against the restored source.
- [x] Build the 0.1.7 `.app` bundle and verify its bundle metadata.
- [x] Replace `/Applications/prompt-password.app` once, preserving the app data directory, then confirm the process starts. Do not perform feature testing in this step.

## Task 2: Build an isolated AppKit panel probe

**Files (local experiment only; not part of the production bundle):**

- Local source: `/tmp/prompt-password-macos-panel-probe-20260821/`
- Compiled probe: `/tmp/prompt-password-nspanel-probe`

- [x] Create a standalone accessory application that constructs a real `NSPanel` at startup.
- [x] Add a text field and log `isOnActiveSpace`, `isKeyWindow`, `isMainWindow`, active application state, collection behavior, and first-responder status.
- [x] Support explicit candidate modes: `auxiliary` (`CanJoinAllApplications + FullScreenAuxiliary`), `active-space` (the previous pair plus `MoveToActiveSpace`), and `all-spaces` (control case only).
- [x] Apply `NonactivatingPanel`, `hidesOnDeactivate = false`, floating behavior, and the chosen level only to the real `NSPanel`.
- [x] Never call Objective-C `setClass` on a Tauri-created window and never load the Prompt Password database.
- [x] Compile the probe with `swiftc -framework AppKit`; do not launch it automatically from the main app.

## Task 3: One controlled native verification

- [x] Run the probe manually once per candidate only when the user is not doing unrelated work.
- [x] Capture the initial probe logs. All three modes became visible and key. The first `first_responder_is_input=false` reading was a probe bug: `NSTextField` editing uses a field-editor `NSTextView`, and `makeFirstResponder` returned `true`.
- [x] Check the normal desktop for all three candidates; each reported `make_first_responder_result=true` and `first_responder_is_field_editor=true`.
- [x] Check a standard Chrome full-screen Space without sending full-screen toggle shortcuts; the safe TaoWindow candidate remained on the triggering desktop instead of appearing in the browser's full-screen Space.
- [x] Record the result: the candidate did not satisfy the Space requirement, so no production integration was kept.
- [x] Stop the candidate and restore the stable Prompt Password bundle; the browser was not toggled or otherwise changed by the test.

## Task 4: Integrate only a passing candidate

- [x] Test a safe `TaoWindow` candidate using `CanJoinAllApplications + FullScreenAuxiliary` and compatible `NSWindow` operations. It failed to join the browser's full-screen Space.
- [x] Preserve the existing cross-platform monitor placement, `panel-shown`/`panel-hidden` lifecycle, search reset, draft preservation, and Windows path by restoring the stable implementation.
- [x] Keep all macOS-only code behind `cfg(target_os = "macos")`; no production-only behavior flags or diagnostics remain.
- [x] Do not use runtime class mutation or unbounded diagnostic logging in production.
- [x] Stop integration and document that cross-Space behavior requires an upstream tao/Tauri creation-time extension or a separately hosted native panel.

## Task 5: Final validation and deployment

- [x] Run `npm run build`.
- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`.
- [x] Build release `.app` bundles and verify identifier/version `com.shengye.prompt-password` / `0.1.7`.
- [x] Record database/WAL/SHM checksums before and after the controlled replacement; all three remained unchanged.
- [x] Restore `/Applications/prompt-password.app` to the stable 0.1.7 bundle and confirm the process starts.
