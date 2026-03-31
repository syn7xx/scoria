//! CLI entry point; logic lives in the library crate (`lib.rs`).
//!
//! Windows: `windows_subsystem = "windows"` avoids a console for the default tray run; for
//! `scoria save`, `attach_to_parent_console` attaches to the parent shell so `println!` works.
#![cfg_attr(windows, windows_subsystem = "windows")]

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

/// Attach to the parent console so stdout/tracing work for `scoria save` with `windows_subsystem`.
#[cfg(windows)]
fn attach_to_parent_console() {
    type BOOL = i32;
    type DWORD = u32;
    #[link(name = "kernel32", kind = "system")]
    extern "system" {
        fn AttachConsole(dw_process_id: DWORD) -> BOOL;
    }
    const ATTACH_PARENT_PROCESS: DWORD = 0xFFFF_FFFF;
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Run);

    #[cfg(windows)]
    if matches!(command, Commands::Save) {
        attach_to_parent_console();
    }

    scoria::init_logging();

    #[cfg(target_os = "macos")]
    macos::set_process_name();

    match command {
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
