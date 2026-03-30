//! CLI entry point; logic lives in the library crate (`lib.rs`).

use anyhow::Result;
use clap::{Parser, Subcommand};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use scoria::app::tray;

#[cfg(target_os = "macos")]
use scoria::ui::macos;

use scoria::perform_save;

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

fn main() -> Result<()> {
    #[cfg(target_os = "macos")]
    macos::set_process_name();

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
        Commands::SettingsGui => macos::run_blocking(),
    }
}
