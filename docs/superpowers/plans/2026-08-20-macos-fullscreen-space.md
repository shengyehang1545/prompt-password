# macOS Full-Screen Space Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the quick panel join the active macOS full-screen Space and receive keyboard input immediately after the global shortcut.

**Architecture:** Use direct AppKit bindings only on macOS. The existing Tauri monitor placement remains cross-platform; native collection behavior is configured once and native activation runs on the main thread each time the panel is shown.

**Tech Stack:** Tauri 2, Rust, `objc2`, `objc2-app-kit`, AppKit, Preact (unchanged)

---

### Task 1: Add macOS AppKit bindings and behavior helpers

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/platform/window.rs`

- [ ] **Step 1: Add target-specific dependencies**

Add macOS-only dependencies so non-macOS builds do not link AppKit:

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = { version = "0.6", default-features = false }
objc2-app-kit = { version = "0.3", default-features = false, features = ["std", "NSApplication", "NSWindow"] }
```

- [ ] **Step 2: Add a pure behavior composition test**

Add a macOS-gated helper that composes `MoveToActiveSpace` and `FullScreenAuxiliary` while removing `CanJoinAllSpaces`, and test the bit flags:

```rust
#[cfg(target_os = "macos")]
fn panel_collection_behavior(current: objc2_app_kit::NSWindowCollectionBehavior) -> objc2_app_kit::NSWindowCollectionBehavior {
    (current & !objc2_app_kit::NSWindowCollectionBehavior::CanJoinAllSpaces)
        | objc2_app_kit::NSWindowCollectionBehavior::MoveToActiveSpace
        | objc2_app_kit::NSWindowCollectionBehavior::FullScreenAuxiliary
}
```

The test must assert the result contains the two required flags and does not contain `CanJoinAllSpaces`.

- [ ] **Step 3: Implement main-thread native configuration**

In `configure_main_window`, replace the current `set_visible_on_all_workspaces(true)` call with a macOS-only `run_on_main_thread` closure. Obtain `window.ns_window()`, cast it to `&NSWindow`, set the composed behavior, and call `setHidesOnDeactivate(false)`. Ignore failures so window startup remains best-effort.

- [ ] **Step 4: Run focused checks**

Run: `cargo test --manifest-path src-tauri/Cargo.toml platform::window::tests -- --nocapture`

Expected: all window behavior tests pass.

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: macOS compilation succeeds.

### Task 2: Activate the window in the current full-screen Space

**Files:**
- Modify: `src-tauri/src/platform/window.rs`

- [ ] **Step 1: Add a macOS activation helper**

Add a macOS-only helper that schedules this closure on the window's main thread:

```rust
let native_window = window.clone();
let _ = window.run_on_main_thread(move || {
    let Ok(pointer) = native_window.ns_window() else { return };
    let ns_window: &objc2_app_kit::NSWindow = unsafe { &*pointer.cast() };
    let marker = objc2::MainThreadMarker::new().expect("window activation runs on main thread");
    let app = objc2_app_kit::NSApplication::sharedApplication(marker);
    app.activateIgnoringOtherApps(true);
    ns_window.makeKeyAndOrderFront(None);
    ns_window.orderFrontRegardless();
});
```

- [ ] **Step 2: Call activation after show/focus**

In `show_window`, keep monitor placement, `show`, and Tauri `set_focus`, then call the macOS activation helper before emitting `panel-shown`. Other platforms compile to no-op.

- [ ] **Step 3: Build the frontend and Rust tests**

Run: `npm run build && cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`

Expected: frontend build succeeds, all Rust tests pass, and Clippy emits no warnings.

### Task 3: Bump, package, and deploy safely

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Bump version to 0.1.7**

Update only the project package version fields and the `prompt-password` package entry in Cargo.lock. Keep identifier `com.shengye.prompt-password` unchanged.

- [ ] **Step 2: Build the macOS app**

Run: `npm run tauri build -- --bundles app`

Expected: `src-tauri/target/release/bundle/macos/prompt-password.app` exists and reports version `0.1.7`.

- [ ] **Step 3: Replace the installed bundle**

Quit `/Applications/prompt-password.app`, move it to the macOS Trash as a recoverable replacement backup only if needed, copy the 0.1.7 bundle into `/Applications`, and launch it. Do not alter `/Users/shengye/Library/Application Support/com.shengye.prompt-password/`.

- [ ] **Step 4: Verify**

Confirm bundle identifier/version, running process, unchanged database and WAL checksums, shortcut display on a normal desktop and a full-screen app, immediate typing focus, and normal hide controls.
