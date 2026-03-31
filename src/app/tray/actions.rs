use tao::event_loop::ControlFlow;
use tray_icon::TrayIcon;

use crate::engine::{config, update};
use crate::i18n;

use super::menu::{MENU_CONFIG, MENU_QUIT, MENU_SAVE, MENU_SETTINGS, MENU_UPDATE, MenuItems};
use super::notify::notify;

#[cfg(target_os = "linux")]
use crate::ui::gtk;

pub(crate) fn do_save() {
    match crate::perform_save() {
        Ok(p) => {
            let body = i18n::notif_saved_body(&p.display().to_string());
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

pub(crate) fn open_settings() {
    #[cfg(target_os = "linux")]
    gtk::open();
    #[cfg(target_os = "macos")]
    {
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
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    config::open_in_editor();
}

pub(crate) fn check_for_updates_bg() {
    std::thread::spawn(|| {
        if let update::CheckResult::UpdateAvailable(tag) = update::check() {
            let msg = format!("v{} → {tag}", update::current_version());

            tracing::info!(tag = %tag, "update available");
            notify(
                i18n::notif_update_available(),
                &i18n::notif_update_available_body(&msg),
            );
        }
    });
}

pub(crate) fn do_update() {
    std::thread::spawn(|| {
        let tag = match update::cached_tag() {
            Some(cached) => cached.clone(),
            None => {
                notify(i18n::notif_checking(), i18n::notif_looking());
                match update::check() {
                    update::CheckResult::UpdateAvailable(tag) => tag,
                    update::CheckResult::UpToDate => {
                        let ver = update::current_version();

                        notify(i18n::notif_up_to_date(), &i18n::notif_up_to_date_body(ver));
                        return;
                    }
                    update::CheckResult::Unreachable => {
                        notify(i18n::notif_update_failed(), i18n::notif_unreachable());
                        return;
                    }
                }
            }
        };

        notify(i18n::notif_updating(), &i18n::notif_downloading(&tag));
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
    });
}

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

pub(crate) fn on_config_changed(tray: &TrayIcon, menu_items: &MenuItems) {
    let Ok(cfg) = config::load() else { return };

    i18n::apply(&cfg.language);
    menu_items.refresh_labels();

    let _ = tray.set_tooltip(Some(i18n::tooltip()));
}
