//! Platform-specific settings windows.

#[cfg(target_os = "linux")]
pub mod gtk;
#[cfg(target_os = "macos")]
pub mod macos;
