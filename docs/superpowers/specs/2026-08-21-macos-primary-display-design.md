# macOS Primary Display Panel Design

## Goal

Always show the quick-search panel centered in the operating system's primary display, regardless of the mouse position or which secondary display is active. Preserve the existing window lifecycle, focus behavior, macOS Space policy, frontend state, Windows behavior, and encrypted data.

## Behavior

The show path asks Tauri for `primary_monitor()`. When a primary monitor is available, it centers the panel in that monitor's work area, respecting the menu bar and Dock. If the monitor lookup fails, the window keeps its current position and is still shown. Cursor position and `available_monitors()` are no longer used for placement.

The change is limited to display selection and its unit-test coverage. No full-screen window flags, native class changes, frontend code, database files, or version fields are changed.

## Verification

Run the Rust test suite, Clippy, frontend build, and `git diff --check`. Manual verification uses the existing shortcut with the pointer on both the primary and secondary displays; the panel should remain centered on the primary display in both cases.
