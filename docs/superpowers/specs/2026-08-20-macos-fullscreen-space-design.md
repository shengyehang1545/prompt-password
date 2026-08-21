# macOS Full-Screen Space Panel Design

## Goal

Make the Prompt Password quick panel appear over a standard macOS full-screen application without switching the user to another Space, while preserving the already-working normal-desktop behavior, search focus, hide controls, and encrypted data.

## Safety boundary

The current Tauri `WebviewWindow` remains the stable production path until the native panel behavior is proven separately. The experiment must not mutate the Objective-C class of an existing Tauri `NSWindow`, must not read or migrate the vault database, and must not require repeated restarts of the installed application.

The first implementation step is a standalone AppKit probe. It creates a real `NSPanel` at construction time, displays a harmless text field, and reports the panel's collection behavior, Space membership, key-window state, and first-responder result. The probe is built outside the application bundle and is never used as the password manager or given access to its data.

## Candidate native behavior

The probe evaluates the supported AppKit behavior on a real `NSPanel`:

- `FullScreenAuxiliary` allows the panel to accompany a full-screen application.
- `CanJoinAllApplications` allows the panel to join another application's full-screen Space.
- `MoveToActiveSpace` is tested as a separate candidate because it follows the active Space and can conflict with pointer-monitor placement on multi-display setups.
- `CanJoinAllSpaces` is not used in the production candidate because it can leave the panel attached to the wrong Space.
- `NonactivatingPanel`, `hidesOnDeactivate = false`, floating behavior, and a high window level are tested only on the real `NSPanel`; no runtime `setClass` conversion is allowed.

The probe must record whether ordering the panel changes the active application/Space. A passing result requires both: the panel is visible in the triggering full-screen Space and its text field accepts input immediately. If no candidate satisfies both without destabilizing the normal desktop, production falls back to the stable 0.1.7 behavior.

## Integration boundary

The probe passes the focus portion, but Tauri 2.11/tao creates a custom `TaoWindow` subclass with an internal `focusable` ivar. Changing that object to `NSPanel` at runtime is incompatible and caused the earlier white-screen failure. Production integration must therefore retain the `TaoWindow` class and use only compatible `NSWindow` operations: collection behavior, `hidesOnDeactivate`, style-mask changes, level, and main-thread ordering. No `setClass`/`object_setClass` call is allowed.

The first production candidate was `CanJoinAllApplications + FullScreenAuxiliary` with the existing pointer-monitor placement and a native ordering sequence. It was tested in a standard Chrome full-screen Space and remained on the triggering desktop, so it was not integrated. The remaining path would require an upstream tao/Tauri creation-time extension or a separately hosted native panel, both of which are a larger architecture change.

The existing Rust monitor placement and frontend lifecycle events remain unchanged. `panel-shown` is emitted only after native ordering and focus are complete.

Windows keeps the current Tauri-only path and does not link AppKit.

## Verification

Automated checks cover the frontend build, all Rust tests, Clippy, and probe compilation. Manual verification is one controlled pass on a normal desktop and one standard Chrome/Safari full-screen window. It checks pointer-monitor placement, immediate typing, Escape/close/hotkey hide, and that the browser is not forced out of full screen. No DRM, exclusive-display, lock-screen, authorization, or other system-protected surface is expected to be coverable.

## Deployment safety

Restore and verify the stable 0.1.7 bundle before any native experiment. Record only bundle metadata and database/WAL checksums. Replace `/Applications/prompt-password.app` at most once after all code changes are complete, and never alter `/Users/shengye/Library/Application Support/com.shengye.prompt-password/`.

## Decision

Production remains on the stable 0.1.7 Tauri window. The standalone real `NSPanel` probe demonstrated that AppKit's candidate collection behaviors can display a native panel and accept input, but applying the same behaviors after Tauri has created a `TaoWindow` does not move the webview panel into another application's full-screen Space. Further runtime window-property experiments are intentionally stopped to avoid repeating the earlier white-screen and focus regressions.
