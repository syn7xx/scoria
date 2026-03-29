use anyhow::{Context, Result};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

#[cfg(target_os = "linux")]
use crate::settings_gui;
use crate::{autostart, config, hotkey, i18n, update};

const MENU_SAVE: &str = "scoria.save";
const MENU_SETTINGS: &str = "scoria.settings";
const MENU_CONFIG: &str = "scoria.config";
const MENU_UPDATE: &str = "scoria.update";
const MENU_QUIT: &str = "scoria.quit";

#[derive(Debug)]
enum UserEvent {
    Menu(tray_icon::menu::MenuEvent),
    HotKey(GlobalHotKeyEvent),
    /// Fired by the config-watcher thread when config.toml is modified.
    ConfigChanged,
}

// ---------------------------------------------------------------------------
// Menu items — kept alive so we can call set_text() after a language change.
// ---------------------------------------------------------------------------

struct MenuItems {
    save: MenuItem,
    settings: MenuItem,
    config_item: MenuItem,
    update: MenuItem,
    quit: MenuItem,
}

impl MenuItems {
    fn build() -> Result<(Menu, Self)> {
        let save = MenuItem::with_id(MENU_SAVE, i18n::menu_save(), true, None);
        let settings = MenuItem::with_id(MENU_SETTINGS, i18n::menu_settings(), true, None);
        let config_item = MenuItem::with_id(MENU_CONFIG, i18n::menu_config(), true, None);
        let update = MenuItem::with_id(MENU_UPDATE, i18n::menu_update(), true, None);
        let quit = MenuItem::with_id(MENU_QUIT, i18n::menu_quit(), true, None);
        let menu = Menu::new();

        menu.append(&save)?;
        menu.append(&settings)?;
        menu.append(&config_item)?;
        menu.append(&update)?;
        menu.append(&quit)?;

        Ok((
            menu,
            Self {
                save,
                settings,
                config_item,
                update,
                quit,
            },
        ))
    }

    fn refresh_labels(&self) {
        self.save.set_text(i18n::menu_save());
        self.settings.set_text(i18n::menu_settings());
        self.config_item.set_text(i18n::menu_config());
        self.update.set_text(i18n::menu_update());
        self.quit.set_text(i18n::menu_quit());
    }
}

// ---------------------------------------------------------------------------
// Config-file watcher — sends ConfigChanged when config.toml is modified.
// ---------------------------------------------------------------------------

fn watch_config_bg(proxy: EventLoopProxy<UserEvent>) {
    std::thread::spawn(move || {
        let Ok(path) = config::config_path() else {
            return;
        };
        let mut last_modified = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());

        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));

            let new_modified = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());

            if new_modified != last_modified {
                last_modified = new_modified;
                let _ = proxy.send_event(UserEvent::ConfigChanged);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Icon
// ---------------------------------------------------------------------------

fn scoria_icon() -> Icon {
    let decoder = png::Decoder::new(ICON_PNG);
    let mut reader = decoder.read_info().expect("decode icon PNG header");
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("decode icon PNG frame");

    buf.truncate(info.buffer_size());
    Icon::from_rgba(buf, info.width, info.height).expect("build tray icon")
}

const ICON_PNG: &[u8] = include_bytes!("../assets/scoria-32.png");

// ---------------------------------------------------------------------------
// Hotkey
// ---------------------------------------------------------------------------

fn try_register_hotkey(cfg: &config::Config) -> (Option<u32>, Option<GlobalHotKeyManager>) {
    let Some(ref spec) = cfg.hotkey else {
        eprintln!("scoria: no hotkey configured; use tray menu or `scoria save`.");
        return (None, None);
    };

    let hk = match hotkey::parse_hotkey(spec) {
        Ok(hk) => hk,
        Err(e) => {
            eprintln!("scoria: invalid hotkey {spec:?}: {e}");
            return (None, None);
        }
    };

    let mgr = match GlobalHotKeyManager::new() {
        Ok(mgr) => mgr,
        Err(e) => {
            eprintln!(
                "scoria: global hotkeys unavailable ({e}). Bind `scoria save` in your DE instead."
            );
            return (None, None);
        }
    };

    match mgr.register(hk) {
        Ok(()) => {
            eprintln!("scoria: registered hotkey {spec}");
            (Some(hk.id()), Some(mgr))
        }
        Err(e) => {
            eprintln!("scoria: could not register hotkey {spec}: {e}");
            (None, None)
        }
    }
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

fn notify(summary: &str, body: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("notify-send")
            .args(["-a", "Scoria", "-i", "scoria", "-t", "3000", summary, body])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display notification \"{}\" with title \"{}\"",
            body.replace('\\', "\\\\").replace('"', "\\\""),
            summary.replace('\\', "\\\\").replace('"', "\\\""),
        );
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn();
    }
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

fn do_save() {
    match crate::perform_save() {
        Ok(p) => {
            let body = i18n::notif_saved_body(&p.display().to_string());
            eprintln!("scoria: {body}");
            notify(i18n::notif_saved_title(), &body);
        }
        Err(e) => {
            let msg = format!("{e:#}");
            eprintln!("scoria: save failed: {msg}");
            notify(i18n::notif_save_failed(), &msg);
        }
    }
}

fn open_settings() {
    #[cfg(target_os = "linux")]
    settings_gui::open();
    #[cfg(target_os = "macos")]
    {
        let launched = std::env::current_exe().ok().and_then(|exe| {
            std::process::Command::new(exe).arg("settings-gui").spawn().ok()
        });

        if launched.is_none() {
            config::open_in_editor();
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    config::open_in_editor();
}

fn check_for_updates_bg() {
    std::thread::spawn(|| {
        if let update::CheckResult::UpdateAvailable(tag) = update::check() {
            let msg = format!("v{} → {tag}", update::current_version());

            eprintln!("scoria: update available: {msg}");
            notify(
                i18n::notif_update_available(),
                &i18n::notif_update_available_body(&msg),
            );
        }
    });
}

fn do_update() {
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
                eprintln!("scoria: updated to {tag}");
                notify(i18n::notif_updated(), &i18n::notif_updated_body(&tag));
            }
            Err(e) => {
                let msg = format!("{e:#}");

                eprintln!("scoria: update failed: {msg}");
                notify(i18n::notif_update_failed(), &msg);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Event handler
// ---------------------------------------------------------------------------

fn handle_menu(id: &str, control_flow: &mut ControlFlow) {
    match id {
        MENU_SAVE => do_save(),
        MENU_SETTINGS => open_settings(),
        MENU_CONFIG => config::open_in_editor(),
        MENU_UPDATE => do_update(),
        MENU_QUIT => *control_flow = ControlFlow::Exit,
        _ => {}
    }
}

fn on_config_changed(tray: &TrayIcon, menu_items: &MenuItems) {
    let Ok(cfg) = config::load() else { return };

    i18n::apply(&cfg.language);
    menu_items.refresh_labels();

    let _ = tray.set_tooltip(Some(i18n::tooltip()));
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run() -> Result<()> {
    let cfg = config::load_or_create()?;

    i18n::apply(&cfg.language);

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    tray_icon::menu::MenuEvent::set_event_handler(Some({
        let proxy = proxy.clone();

        move |e| {
            let _ = proxy.send_event(UserEvent::Menu(e));
        }
    }));

    GlobalHotKeyEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::HotKey(e));
    }));

    let (menu, menu_items) = MenuItems::build()?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(i18n::tooltip())
        .with_icon(scoria_icon())
        .build()
        .context("create tray icon")?;

    autostart::apply(cfg.autostart);
    
    let (hotkey_id, hk_manager) = try_register_hotkey(&cfg);

    check_for_updates_bg();
    watch_config_bg(event_loop.create_proxy());

    event_loop.run(move |event, _elwt, control_flow| {
        let _ = &hk_manager;
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::Menu(e)) => {
                handle_menu(e.id.as_ref(), control_flow);
            }
            Event::UserEvent(UserEvent::HotKey(e))
                if e.state == HotKeyState::Pressed && hotkey_id == Some(e.id) =>
            {
                handle_menu(MENU_SAVE, control_flow);
            }
            Event::UserEvent(UserEvent::ConfigChanged) => {
                on_config_changed(&tray, &menu_items);
            }
            _ => {}
        }
    });
}
