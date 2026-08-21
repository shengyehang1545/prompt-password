# macOS Window Behavior Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the quick-search panel appear and focus on the primary display, remain visible after outside clicks, toggle with the global shortcut, close explicitly, and reset search-only state while preserving entry-form drafts.

**Architecture:** Rust owns the cross-platform window lifecycle and emits `panel-shown`/`panel-hidden` events. Primary-monitor selection and centering use Tauri APIs on macOS and Windows; only macOS enables the native all-Spaces collection behavior. Preact consumes lifecycle events to reset transient search state and issue an explicit focus request.

**Tech Stack:** Tauri 2, Rust, Preact, TypeScript, CSS, SQLite (unchanged)

---

## File Map

- Modify `src-tauri/src/platform/window.rs`: window placement, workspace policy, lifecycle events, show/hide/toggle behavior, and position unit tests.
- Create `src-tauri/src/commands/window.rs`: frontend-safe hide command that routes through the lifecycle owner.
- Modify `src-tauri/src/commands/mod.rs`: export the window command module.
- Modify `src-tauri/src/lib.rs`: register the hide command.
- Modify `src/lib/tauri.ts`: expose the typed hide command to Preact.
- Modify `src/hooks/useSearch.ts`: expose one operation that clears query, results, loading, errors, and pending debounce work.
- Modify `src/App.tsx`: listen for panel lifecycle events, apply the form-preservation rule, and render an explicit close button.
- Modify `src/components/SearchInput.tsx`: focus only when its focus request changes.
- Modify `src/components/SearchInput.css`: style the explicit panel-close action.
- Modify `README.md`: document the revised behavior.

### Task 1: Cross-platform placement and lifecycle owner

**Files:**
- Modify: `src-tauri/src/platform/window.rs`

- [ ] **Step 1: Write failing centering tests**

Add tests that expect the panel to center in a work area, including negative display coordinates and a panel larger than the work area:

```rust
#[test]
fn centers_window_in_monitor_work_area() {
    assert_eq!(centered_axis(1728, 1440, 960), 1968);
}

#[test]
fn centers_window_on_monitor_with_negative_origin() {
    assert_eq!(centered_axis(-1920, 1920, 960), -1440);
}

#[test]
fn clamps_oversized_window_to_work_area_origin() {
    assert_eq!(centered_axis(100, 800, 1000), 100);
}
```

- [ ] **Step 2: Run the test to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml platform::window::tests -- --nocapture`

Expected: compilation fails because `centered_axis` is not defined.

- [ ] **Step 3: Implement the lifecycle owner**

Implement `centered_axis(origin, available, window)` with saturating integer conversion, then add:

```rust
pub const PANEL_SHOWN_EVENT: &str = "panel-shown";
pub const PANEL_HIDDEN_EVENT: &str = "panel-hidden";

fn move_to_primary_monitor(window: &WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.primary_monitor()? {
        let area = monitor.work_area();
        let size = window.outer_size()?;
        let x = centered_axis(area.position.x, area.size.width, size.width);
        let y = centered_axis(area.position.y, area.size.height, size.height);
        window.set_position(tauri::PhysicalPosition::new(x, y))?;
    }
    Ok(())
}
```

Configure `set_visible_on_all_workspaces(true)` only under `#[cfg(target_os = "macos")]`. Remove the `Focused(false)` hide listener. Make show best-effort move, then show, focus, and emit `panel-shown`. Make hide emit `panel-hidden`, then hide. Route shortcut toggle and tray show through those functions.

- [ ] **Step 4: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml platform::window::tests -- --nocapture`

Expected: all new window placement tests pass.

### Task 2: Backend/frontend hide bridge

**Files:**
- Create: `src-tauri/src/commands/window.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: Add a lifecycle-aware hide command**

Create:

```rust
use tauri::AppHandle;

#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    crate::platform::window::hide_main_window(&app);
}
```

Export the module, register `commands::window::hide_main_window` in `generate_handler!`, and add:

```ts
export function hideMainWindow(): Promise<void> {
  return invoke("hide_main_window");
}
```

- [ ] **Step 2: Check backend and frontend compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml && npm run build`

Expected: both commands exit successfully.

### Task 3: Search reset, form preservation, and deterministic focus

**Files:**
- Modify: `src/hooks/useSearch.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/SearchInput.tsx`

- [ ] **Step 1: Add an explicit search reset operation**

In `useSearch`, clear the pending timer and synchronously clear all transient search state:

```ts
const reset = () => {
  if (timerRef.current) clearTimeout(timerRef.current);
  timerRef.current = null;
  setQuery("");
  setResults([]);
  setLoading(false);
  setError(null);
};
```

Return `reset` from the hook.

- [ ] **Step 2: Consume lifecycle events in App**

Use `getCurrentWindow().listen("panel-shown", ...)` and `listen("panel-hidden", ...)`. Track `showForm` in a ref so listeners are stable. On either lifecycle event, reset highlight/copy selection; when no form is open, call `reset`, close settings/secure dialogs, and on `panel-shown` increment a numeric focus request. Replace direct `getCurrentWindow().hide()` calls with `hideMainWindow()`.

- [ ] **Step 3: Make SearchInput focus request-driven**

Add `focusRequest: number` to `SearchInputProps`; remove the permanent `onFocusChanged` listener; use an animation-frame callback keyed by `focusRequest`:

```ts
useEffect(() => {
  const frame = requestAnimationFrame(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  });
  return () => cancelAnimationFrame(frame);
}, [focusRequest]);
```

This prevents a window focus event from stealing focus from an open entry form.

- [ ] **Step 4: Run the frontend build**

Run: `npm run build`

Expected: TypeScript and Vite builds complete without errors.

### Task 4: Explicit close control and documentation

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/components/SearchInput.css`
- Modify: `README.md`

- [ ] **Step 1: Add the close control**

Add a search-bar button after Settings with `aria-label="Hide window"`, `title="Hide"`, and an X icon. Its click calls the lifecycle-aware hide command. Reuse the 28px search action geometry and add a danger-neutral hover state that remains consistent with the existing dark UI.

- [ ] **Step 2: Update user-facing behavior docs**

Document that the shortcut toggles the panel on the primary display, outside clicks no longer hide it, Escape/X hide it, searches reset, and add/edit drafts persist.

- [ ] **Step 3: Run all automated checks**

Run: `npm run build`

Expected: frontend build passes.

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all Rust tests pass.

Run: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`

Expected: no warnings or errors.

### Task 5: macOS package and safe installed-app replacement

**Files:**
- Build output: `src-tauri/target/release/bundle/macos/prompt-password.app`
- Installed bundle: `/Applications/prompt-password.app`

- [ ] **Step 1: Record data and bundle locations without reading secrets**

Resolve the installed bundle and application data directory. Record only database file metadata and checksum, never database content or passwords.

- [ ] **Step 2: Build the macOS application**

Run: `npm run tauri build -- --bundles app`

Expected: `src-tauri/target/release/bundle/macos/prompt-password.app` exists.

- [ ] **Step 3: Smoke-test the built binary**

Launch the built bundle, verify its process remains running, exercise shortcut show/hide and visible close behavior on Firefox-free native WebView, then quit only the test bundle process.

- [ ] **Step 4: Replace the installed bundle without touching app data**

Quit the running installed app, move the existing `/Applications/prompt-password.app` to a timestamped backup beside the build artifacts, copy the new bundle into `/Applications`, and launch it. Do not alter `~/Library/Application Support/com.shengye.prompt-password/`.

- [ ] **Step 5: Verify data preservation**

Compare database path, size, and checksum with the pre-install record, then unlock and confirm existing entries are searchable. The app bundle may change; the database checksum must remain unchanged until the user performs a data-changing action.
