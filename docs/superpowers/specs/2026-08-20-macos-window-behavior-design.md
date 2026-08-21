# macOS Window Behavior Design

## Goal

Make Prompt Password behave like a reliable macOS quick-search panel: the global shortcut shows it centered on the primary display, activates it across Spaces, focuses the search field, and the same shortcut or an explicit close control hides it. Ordinary searches reset after hiding, while unfinished add/edit forms remain intact.

## Current Application

Prompt Password is a Tauri 2 desktop password manager with a Preact frontend and Rust backend. It stores encrypted secrets in a local SQLite database under the application data directory, keeps the application resident in the menu bar, and exposes a global shortcut for fast password lookup and copy.

## Scope

This change targets macOS first, while keeping the display-placement path portable. macOS uses the primary-monitor lookup to show the panel on the primary display; Windows retains its existing pointer-monitor placement and skips the macOS-only Space policy. The database schema, encryption format, app identifier, and application data directory remain unchanged.

## Window Lifecycle

The Rust window module owns show/hide behavior. Showing the panel performs these actions in order:

1. Resolve the operating system's primary monitor.
2. Center the panel inside the primary monitor's available work area.
3. On macOS, make the panel available on the active Space and activate/focus the application window. On Windows, keep the normal workspace behavior.
4. Show and focus the WebView window.
5. Emit a frontend event indicating that the panel has been shown.

The global shortcut remains a toggle. If the panel is visible it hides; if hidden it follows the show sequence above. Losing focus no longer hides the panel. The tray Show action uses the same show sequence. Escape and the new visible close button hide the panel without terminating the resident application.

For macOS Spaces, the panel will use the native window collection behavior that allows it to appear on the currently active Space. This avoids private Mission Control APIs and preserves compatibility with normal macOS security restrictions. The panel is moved to the primary display before it is ordered front and focused.

## Frontend State and Focus

The frontend listens for the Rust panel-shown event. On receipt it:

- focuses and selects the search input after the window is active;
- resets selection and keyboard-copy state;
- clears the query and therefore clears results when no add/edit form is open;
- preserves the add/edit form and its draft when a form is open.

Hiding from Escape, the explicit close button, or the shortcut follows the same reset rule. To cover shortcut hiding, the frontend resets ordinary search state when the window loses visibility/focus only if there is no add/edit form. Modal settings and secure import/export prompts are not treated as add-entry drafts and may be closed when the panel is hidden so the next quick search starts cleanly.

## Components and Boundaries

- `src-tauri/src/platform/window.rs`: cross-platform primary-monitor selection and window placement, macOS workspace policy, show/hide toggle, and panel-shown event emission.
- `src/App.tsx`: application-level visibility lifecycle, search reset rules, and explicit close action.
- `src/components/SearchInput.tsx`: stable input ref behavior and reaction to a focus request signal.
- `src/components/SearchInput.css`: close-button styling consistent with existing search actions.
- `src-tauri/Cargo.toml`: macOS-only native window dependency if required by the final implementation.

No persistence code changes are required.

## Error Handling

Monitor lookup and placement are best-effort. If the primary monitor cannot be resolved, the existing window position is retained and the panel is still shown. Failure to apply a macOS collection behavior must not prevent showing or focusing the panel. Window show/focus failures remain non-fatal because the shortcut callback cannot return errors to the user.

## Verification

Automated verification covers pure monitor-position calculation and frontend build/type checking. Rust unit tests verify centering with negative monitor coordinates and mixed monitor layouts. Existing Rust tests ensure data and settings behavior remain intact.

Manual macOS verification covers:

1. Shortcut from each connected display places the panel on the primary display.
2. Shortcut from another Space brings the panel to that active Space without manually switching desktops.
3. Search input accepts typing immediately after every show.
4. Clicking another application leaves the panel visible.
5. The shortcut, Escape, and close button each hide it.
6. A normal search is empty after reopening.
7. An unfinished add/edit form survives hide and reopen.
8. Existing vault entries remain readable after installing the updated app.

## Deployment Safety

Build a new macOS `.app`, quit the running Prompt Password process, back up the currently installed application bundle, and replace only `/Applications/prompt-password.app`. Do not copy, delete, migrate, or modify the app data directory. Because the bundle identifier and database location stay unchanged, existing encrypted data is reused by the new binary.
