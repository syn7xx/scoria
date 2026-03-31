//! Engine: domain logic and side effects without UI (config, vault, clipboard, updates, etc.).

pub mod autostart;
pub mod clipboard;
pub mod config;
pub mod hotkey;
pub(crate) mod path_safety;
pub mod settings;
pub mod update;
pub mod vault;
