use std::path::{Path, PathBuf};

pub fn apply(enabled: bool) {
    tracing::debug!(enabled = enabled, "applying autostart setting");
    let Ok(exe) = std::env::current_exe() else {
        tracing::warn!("could not determine executable path for autostart");
        return;
    };
    if enabled {
        enable(&exe);
        tracing::info!("autostart enabled");
    } else {
        disable();
        tracing::info!("autostart disabled");
    }
}

// ---------------------------------------------------------------------------
// Linux: ~/.config/autostart/scoria.desktop
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn autostart_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("autostart").join("scoria.desktop"))
}

#[cfg(target_os = "linux")]
fn enable(exe: &Path) {
    let Some(path) = autostart_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Scoria\n\
         Comment=Save clipboard to Obsidian vault\n\
         Exec={exe} run\n\
         Icon=scoria\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n",
        exe = exe.display()
    );
    let _ = std::fs::write(&path, content);
}

#[cfg(target_os = "linux")]
fn disable() {
    if let Some(path) = autostart_path() {
        let _ = std::fs::remove_file(path);
    }
}

// ---------------------------------------------------------------------------
// macOS: ~/Library/LaunchAgents/<label>.plist
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
const LAUNCH_AGENT_LABEL: &str = "com.github.syn7xx.scoria";

#[cfg(target_os = "macos")]
fn launch_agent_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join("Library")
            .join("LaunchAgents")
            .join(format!("{LAUNCH_AGENT_LABEL}.plist"))
    })
}

#[cfg(target_os = "macos")]
fn enable(exe: &Path) {
    let Some(path) = launch_agent_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
"#,
        label = LAUNCH_AGENT_LABEL,
        exe = exe.display()
    );
    let _ = std::fs::write(&path, content);
}

#[cfg(target_os = "macos")]
fn disable() {
    if let Some(path) = launch_agent_path() {
        let _ = std::fs::remove_file(path);
    }
}

// ---------------------------------------------------------------------------
// Other platforms: no-op
// ---------------------------------------------------------------------------

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn enable(_exe: &Path) {}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn disable() {}

#[cfg(test)]
#[path = "autostart_tests.rs"]
mod autostart_tests;
