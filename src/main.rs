//! CLI entry point; logic lives in the library crate (`lib.rs`).

use anyhow::Result;
use clap::{Parser, Subcommand};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use scoria::app::tray;

#[cfg(target_os = "macos")]
use scoria::ui::macos;

#[cfg(target_os = "linux")]
use scoria::ui::gtk;
#[cfg(target_os = "windows")]
use scoria::ui::windows;

use scoria::perform_save;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in the system tray (default command).
    Run,
    /// Save selection or clipboard to Obsidian vault.
    Save,
    /// Open the graphical settings window.
    SettingsGui,
}

fn main() -> Result<()> {
    scoria::init_logging();

    #[cfg(target_os = "macos")]
    macos::set_process_name();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => {
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            return tray::run();
            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
            anyhow::bail!("tray mode is only supported on Linux, macOS and Windows")
        }
        Commands::Save => {
            let path = perform_save()?;

            println!("{}", path.display());
            Ok(())
        }
        Commands::SettingsGui => {
            #[cfg(target_os = "macos")]
            let _ = macos::run_blocking();
            #[cfg(target_os = "linux")]
            gtk::open();
            #[cfg(target_os = "windows")]
            windows::open()?;

            Ok(())
        }
    }
}
