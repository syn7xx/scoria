//! Scoria — save clipboard content (text and images) to an Obsidian vault.
//!
//! - `scoria::engine` — config, vault, clipboard, updates, autostart, hotkey parsing, settings validation
//! - `scoria::ui` — platform settings (GTK / AppKit)
//! - `scoria::app` — system tray and desktop integration
//! - `scoria::i18n` — strings (EN / RU)

use std::path::PathBuf;

use anyhow::Result;

pub mod app;
pub mod engine;
pub mod i18n;
pub mod ui;

/// Read clipboard / selection and write into the configured vault.
pub fn perform_save() -> Result<PathBuf> {
    let cfg = engine::config::load_or_create()?;
    let content = engine::clipboard::read()?;
    engine::vault::save(&cfg, &content)
}
