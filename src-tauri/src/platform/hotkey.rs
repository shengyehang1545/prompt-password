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
            ShortcutProfile::Macos => "Cmd+Shift+K",
            ShortcutProfile::WindowsCompatible => "Ctrl+Shift+K",
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
    fn macos_profile_uses_command_shift_k() {
        let spec = hotkey_for_profile(ShortcutProfile::Macos);

        assert_eq!(spec.label(), "Cmd+Shift+K");
    }

    #[test]
    fn windows_compatible_profile_uses_control_shift_k() {
        let spec = hotkey_for_profile(ShortcutProfile::WindowsCompatible);

        assert_eq!(spec.label(), "Ctrl+Shift+K");
    }
}
