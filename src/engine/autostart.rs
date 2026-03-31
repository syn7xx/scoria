use std::path::Path;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::path::PathBuf;

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn create_dir_all_warn(path: &Path, context: &str) {
    if let Err(e) = std::fs::create_dir_all(path) {
        tracing::warn!(path = %path.display(), error = %e, "{context}");
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn remove_file_warn_if_exists(path: &Path, context: &str) {
    if let Err(e) = std::fs::remove_file(path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(path = %path.display(), error = %e, "{context}");
        }
    }
}

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
fn desktop_escape_arg(arg: &Path) -> String {
    let raw = arg.display().to_string();
    let escaped = raw.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(target_os = "linux")]
fn enable(exe: &Path) {
    let Some(path) = autostart_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        create_dir_all_warn(parent, "failed to create autostart directory");
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
        exe = desktop_escape_arg(exe)
    );
    if let Err(e) = std::fs::write(&path, content) {
        tracing::warn!(path = %path.display(), error = %e, "failed to write linux autostart entry");
    }
}

#[cfg(target_os = "linux")]
fn disable() {
    if let Some(path) = autostart_path() {
        remove_file_warn_if_exists(&path, "failed to remove linux autostart entry");
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
        create_dir_all_warn(parent, "failed to create launch agents directory");
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
    if let Err(e) = std::fs::write(&path, content) {
        tracing::warn!(path = %path.display(), error = %e, "failed to write macOS launch agent");
    }
}

#[cfg(target_os = "macos")]
fn disable() {
    if let Some(path) = launch_agent_path() {
        remove_file_warn_if_exists(&path, "failed to remove macOS launch agent");
    }
}

// ---------------------------------------------------------------------------
// Other platforms: no-op
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(target_os = "windows")]
const RUN_VALUE_NAME: &str = "Scoria";

#[cfg(target_os = "windows")]
fn run_value_data(exe: &Path) -> String {
    format!("\"{}\" run", exe.display())
}

#[cfg(target_os = "windows")]
fn enable(exe: &Path) {
    let value = run_value_data(exe);
    let status = std::process::Command::new("reg")
        .args(["add", RUN_KEY, "/v", RUN_VALUE_NAME, "/t", "REG_SZ", "/d"])
        .arg(value)
        .arg("/f")
        .status();
    if status.map(|s| !s.success()).unwrap_or(true) {
        tracing::warn!("failed to add Windows Run key for autostart");
    }
}

#[cfg(target_os = "windows")]
fn disable() {
    let status = std::process::Command::new("reg")
        .args(["delete", RUN_KEY, "/v", RUN_VALUE_NAME, "/f"])
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            tracing::warn!(code = ?s.code(), "reg delete failed for autostart");
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to run reg delete for autostart");
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn enable(_exe: &Path) {}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn disable() {}

#[cfg(test)]
#[path = "autostart_tests.rs"]
mod autostart_tests;
