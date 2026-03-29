mod autostart;
mod clipboard;
mod config;
mod hotkey;
mod i18n;
mod update;
mod vault;

#[cfg(target_os = "linux")]
mod settings_gui;
#[cfg(target_os = "macos")]
mod settings_macos;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod tray;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in the system tray (default on Linux).
    Run,
    /// Save selection or clipboard to Obsidian vault.
    Save,
    /// Open the graphical settings window (macOS AppKit helper process).
    #[cfg(target_os = "macos")]
    SettingsGui,
}

pub fn perform_save() -> Result<PathBuf> {
    let cfg = config::load_or_create()?;
    let content = clipboard::read()?;
    vault::save(&cfg, &content)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => {
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            return tray::run();
            #[cfg(not(any(target_os = "linux", target_os = "macos")))]
            anyhow::bail!("tray mode is only supported on Linux and macOS")
        }
        Commands::Save => {
            let path = perform_save()?;
            
            println!("{}", path.display());
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Commands::SettingsGui => settings_macos::run_blocking(),
    }
}
