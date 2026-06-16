use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutProfile {
    Macos,
    WindowsCompatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeySpec {
    profile: ShortcutProfile,
}

impl HotkeySpec {
    pub fn label(self) -> &'static str {
        match self.profile {
            ShortcutProfile::Macos => "Cmd+Shift+P",
            ShortcutProfile::WindowsCompatible => "Ctrl+Shift+P",
        }
    }

    pub fn to_tauri_shortcut(self) -> Shortcut {
        match self.profile {
            ShortcutProfile::Macos => {
                Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyP)
            }
            ShortcutProfile::WindowsCompatible => {
                Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyP)
            }
        }
    }
}

pub fn current_profile() -> ShortcutProfile {
    if cfg!(target_os = "macos") {
        ShortcutProfile::Macos
    } else {
        ShortcutProfile::WindowsCompatible
    }
}

pub fn default_hotkey_spec() -> HotkeySpec {
    hotkey_for_profile(current_profile())
}

pub fn hotkey_for_profile(profile: ShortcutProfile) -> HotkeySpec {
    HotkeySpec { profile }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_profile_uses_command_shift_p() {
        let spec = hotkey_for_profile(ShortcutProfile::Macos);

        assert_eq!(spec.label(), "Cmd+Shift+P");
    }

    #[test]
    fn windows_compatible_profile_uses_control_shift_p() {
        let spec = hotkey_for_profile(ShortcutProfile::WindowsCompatible);

        assert_eq!(spec.label(), "Ctrl+Shift+P");
    }
}
