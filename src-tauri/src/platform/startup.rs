#[cfg(target_os = "macos")]
const LAUNCH_AGENT_LABEL: &str = "com.shengye.prompt-password";

pub fn set_auto_start_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        set_macos_auto_start_enabled(enabled)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = enabled;
        Err("Auto-start is not implemented on this platform yet".to_string())
    }
}

#[cfg(target_os = "macos")]
fn set_macos_auto_start_enabled(enabled: bool) -> Result<(), String> {
    let plist_path = macos_launch_agent_path()?;
    if enabled {
        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to resolve executable path: {}", e))?;
        let plist = macos_launch_agent_plist(&exe.to_string_lossy());
        if let Some(parent) = plist_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;
        }
        std::fs::write(&plist_path, plist)
            .map_err(|e| format!("Failed to write LaunchAgent: {}", e))?;
    } else if plist_path.exists() {
        std::fs::remove_file(&plist_path)
            .map_err(|e| format!("Failed to remove LaunchAgent: {}", e))?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_launch_agent_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or_else(|| "HOME is not set".to_string())?;
    Ok(std::path::PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", LAUNCH_AGENT_LABEL)))
}

#[cfg(target_os = "macos")]
pub fn macos_launch_agent_plist(executable_path: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <false/>
</dict>
</plist>
"#,
        LAUNCH_AGENT_LABEL,
        escape_xml(executable_path)
    )
}

#[cfg(target_os = "macos")]
fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_launch_agent_plist_escapes_executable_path() {
        let plist = super::macos_launch_agent_plist("/Applications/A&B.app/Contents/MacOS/prompt");

        assert!(plist.contains("<string>com.shengye.prompt-password</string>"));
        assert!(plist.contains("/Applications/A&amp;B.app/Contents/MacOS/prompt"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
    }
}
