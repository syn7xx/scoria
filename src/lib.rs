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

/// Initialize tracing (logging) subsystem.
pub fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,scoria=debug"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .init();
}

/// Read clipboard / selection and write into the configured vault.
pub fn perform_save() -> Result<PathBuf> {
    tracing::debug!("starting save operation");
    let cfg = engine::config::load_or_create()?;
    let content = engine::clipboard::read()?;
    let path = engine::vault::save(&cfg, &content)?;
    tracing::info!(path = %path.display(), "saved clipboard content");
    Ok(path)
}
