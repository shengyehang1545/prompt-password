# macOS Full-Screen Space Activation Design

## Goal

When the global shortcut is pressed while another macOS application is full screen, Prompt Password must appear in that same full-screen Space and accept keyboard input immediately. Existing monitor placement, hide/show behavior, search reset rules, Windows compatibility, and encrypted data remain unchanged.

## Root Cause

The current panel only uses `CanJoinAllSpaces`. That makes it visible on ordinary Spaces, but a full-screen application has a dedicated Space. The panel therefore remains associated with its previous Space, and AppKit does not make the WebView key window for the full-screen Space. Tauri's normal focus calls are not sufficient for this transition.

## Design

On macOS, configure the native `NSWindow` collection behavior on the main thread:

- remove `CanJoinAllSpaces`;
- add `MoveToActiveSpace`, which moves the panel to the active Space when it becomes active;
- add `FullScreenAuxiliary`, which allows the panel to display alongside the full-screen window;
- set `hidesOnDeactivate(false)` so clicking another application does not hide the panel.

When showing the panel, after moving it to the pointer monitor, dispatch a main-thread AppKit activation sequence: `NSApplication.activateIgnoringOtherApps(true)`, `makeKeyAndOrderFront`, and `orderFrontRegardless`. Then emit the existing frontend `panel-shown` event. This sequence does not simulate Control+Arrow or force the user away from the full-screen application; it joins that application's Space.

Windows and other platforms keep the existing Tauri-only implementation through `cfg(target_os = "macos")` guards.

## Side Effects and Limits

The panel will overlay the full-screen application while open, which is intentional. It will not remain pinned to every Space; switching Spaces can hide it until the next shortcut. Full-screen applications using exclusive presentation or system-protected surfaces may still prevent overlays. No accessibility permission, database migration, or data write is required.

## Verification

Automated checks cover Rust compilation, all existing unit tests, Clippy, and native behavior flag composition. Manual macOS checks cover a normal desktop, a full-screen app, immediate typing after shortcut, Escape/close/hotkey hide, and multi-monitor placement. The installed bundle version will be bumped from 0.1.2 to 0.1.7 while retaining the same bundle identifier and application data path.
