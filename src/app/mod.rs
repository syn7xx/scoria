//! Application shell: system tray and desktop integration.

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod tray;
