use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(target_os = "linux"))]
use tao::event_loop::ControlFlow;

use crate::engine::{config, update};
use crate::i18n;

use super::notify::notify;

#[cfg(not(target_os = "linux"))]
use super::menu::MenuItems;
#[cfg(not(target_os = "linux"))]
use super::menu::{MENU_CONFIG, MENU_QUIT, MENU_SAVE, MENU_SETTINGS, MENU_UPDATE};
#[cfg(not(target_os = "linux"))]
use tray_icon::TrayIcon;

#[cfg(target_os = "windows")]
const RELEASES_URL: &str = "https://github.com/syn7xx/scoria/releases/latest";
static UPDATE_CHECK_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

pub(crate) fn do_save() {
    match crate::perform_save() {
        Ok(p) => {
            #[cfg(target_os = "windows")]
            let display_path = sanitize_windows_path_for_notification(&p);
            #[cfg(not(target_os = "windows"))]
            let display_path = p.display().to_string();
            let body = i18n::notif_saved_body(&display_path);
            tracing::info!(path = %p.display(), "save completed");
            notify(i18n::notif_saved_title(), &body);
        }
        Err(e) => {
            let msg = format!("{e:#}");
            tracing::error!(error = %msg, "save failed");
            notify(i18n::notif_save_failed(), &msg);
        }
    }
}

#[cfg(target_os = "windows")]
fn sanitize_windows_path_for_notification(path: &std::path::Path) -> String {
    let raw = path.display().to_string();
    if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    if let Some(rest) = raw.strip_prefix(r"\\?\") {
        return rest.to_string();
    }
    raw
}

pub(crate) fn open_settings() {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    open_settings_gui_process_or_config();
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    config::open_in_editor();
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn open_settings_gui_process_or_config() {
    let launched = std::env::current_exe().ok().and_then(|exe| {
        std::process::Command::new(exe)
            .arg("settings-gui")
            .spawn()
            .ok()
    });
    if launched.is_none() {
        config::open_in_editor();
    }
}

pub(crate) fn check_for_updates_bg() {
    std::thread::spawn(|| {
        if let update::CheckResult::UpdateAvailable(tag) = update::check() {
            #[cfg(target_os = "windows")]
            let msg = i18n::notif_manual_update_body(&tag);
            #[cfg(not(target_os = "windows"))]
            let msg = i18n::notif_update_available_body(&format!(
                "v{} → {tag}",
                update::current_version()
            ));

            tracing::info!(tag = %tag, "update available");
            notify(i18n::notif_update_available(), &msg);
        }
    });
}

pub(crate) fn update_check_in_progress() -> bool {
    UPDATE_CHECK_IN_PROGRESS.load(Ordering::SeqCst)
}

pub(crate) fn do_update() {
    do_update_with_state_hook(|| {});
}

pub(crate) fn do_update_with_state_hook<F>(on_state_change: F)
where
    F: Fn() + Send + Sync + 'static,
{
    if UPDATE_CHECK_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    let on_state_change = std::sync::Arc::new(on_state_change);
    on_state_change();

    #[cfg(target_os = "windows")]
    {
        let on_state_change_done = on_state_change.clone();
        std::thread::spawn(move || {
            let finish = || {
                UPDATE_CHECK_IN_PROGRESS.store(false, Ordering::SeqCst);
                on_state_change_done();
            };
            let tag = match update::cached_tag() {
                Some(cached) => cached.clone(),
                None => {
                    match update::check() {
                        update::CheckResult::UpdateAvailable(tag) => tag,
                        update::CheckResult::UpToDate => {
                            let ver = update::current_version();
                            notify(i18n::notif_up_to_date(), &i18n::notif_up_to_date_body(ver));
                            finish();
                            return;
                        }
                        update::CheckResult::Unreachable => {
                            notify(i18n::notif_update_failed(), i18n::notif_unreachable());
                            finish();
                            return;
                        }
                    }
                }
            };

            tracing::info!(tag = %tag, "manual update required on Windows");
            notify(
                i18n::notif_update_available(),
                &i18n::notif_manual_update_body(&tag),
            );
            open_releases_page();
            finish();
        });
        return;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let on_state_change_done = on_state_change.clone();
        std::thread::spawn(move || {
            let finish = || {
                UPDATE_CHECK_IN_PROGRESS.store(false, Ordering::SeqCst);
                on_state_change_done();
            };
        let tag = match update::cached_tag() {
            Some(cached) => cached.clone(),
            None => {
                match update::check() {
                    update::CheckResult::UpdateAvailable(tag) => tag,
                    update::CheckResult::UpToDate => {
                        let ver = update::current_version();

                        notify(i18n::notif_up_to_date(), &i18n::notif_up_to_date_body(ver));
                        finish();
                        return;
                    }
                    update::CheckResult::Unreachable => {
                        notify(i18n::notif_update_failed(), i18n::notif_unreachable());
                        finish();
                        return;
                    }
                }
            }
        };

        match update::apply(&tag) {
            Ok(()) => {
                tracing::info!(tag = %tag, "update applied");
                notify(i18n::notif_updated(), &i18n::notif_updated_body(&tag));
            }
            Err(e) => {
                let msg = format!("{e:#}");

                tracing::error!(error = %msg, "update failed");
                notify(i18n::notif_update_failed(), &msg);
            }
        }
            finish();
        });
    }
}

#[cfg(target_os = "windows")]
fn open_releases_page() {
    let status = std::process::Command::new("cmd")
        .args(["/C", "start", "", RELEASES_URL])
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => tracing::warn!(code = ?s.code(), "failed to open releases page"),
        Err(e) => tracing::warn!(error = %e, "failed to run cmd start for releases page"),
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn handle_menu(id: &str, control_flow: &mut ControlFlow) {
    match id {
        MENU_SAVE => do_save(),
        MENU_SETTINGS => open_settings(),
        MENU_CONFIG => config::open_in_editor(),
        MENU_UPDATE => do_update(),
        MENU_QUIT => *control_flow = ControlFlow::Exit,
        _ => {}
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn on_config_changed(tray: &TrayIcon, menu_items: &MenuItems) {
    let Ok(cfg) = config::load() else { return };

    i18n::apply(&cfg.language);
    menu_items.refresh_labels();

    let _ = tray.set_tooltip(Some(i18n::tooltip()));
}

#[cfg(target_os = "linux")]
pub(crate) fn on_config_changed_linux() {
    let Ok(cfg) = config::load() else { return };
    i18n::apply(&cfg.language);
}
