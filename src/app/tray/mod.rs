//! System tray: Tao event loop, menu, notifications, hotkey, config watcher.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{Context, Result};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::TrayIconBuilder;

use crate::engine::{autostart, config};
use crate::i18n;

mod actions;
mod hotkey_reg;
mod icon;
mod menu;
mod notify;
mod watch;

#[cfg(target_os = "linux")]
fn status_notifier_watcher_available() -> bool {
    // GNOME on Ubuntu often doesn't provide a KDE-style StatusNotifierWatcher.
    // Tao/tray implementations may then fail silently or not display the icon.
    // We check D-Bus service existence before creating the tray icon.
    use dbus::blocking::Connection;
    use std::time::Duration;

    let conn = match Connection::new_session() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let proxy = conn.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(500),
    );

    // NameHasOwner returns a single boolean, so the typed return must be a 1-tuple: (bool,)
    let (has_owner,): (bool,) = proxy
        .method_call(
            "org.freedesktop.DBus",
            "NameHasOwner",
            ("org.kde.StatusNotifierWatcher",),
        )
        .unwrap_or((false,));

    has_owner
}

#[derive(Debug)]
enum UserEvent {
    Menu(tray_icon::menu::MenuEvent),
    HotKey(GlobalHotKeyEvent),
    /// Fired by the config-watcher thread when config.toml is modified.
    ConfigChanged,
    /// Fired by signal handler to trigger shutdown.
    Shutdown,
}

/// Start the tray icon and event loop (Linux / macOS).
pub fn run() -> Result<()> {
    let cfg = config::load_or_create()?;

    i18n::apply(&cfg.language);

    #[cfg(target_os = "linux")]
    let tray_supported = status_notifier_watcher_available();
    #[cfg(not(target_os = "linux"))]
    let tray_supported = true;

    #[cfg(target_os = "linux")]
    if !tray_supported {
        notify::notify(
            "Scoria: tray icon unavailable...",
            "org.kde.StatusNotifierWatcher is not available. GNOME/AppIndicator barrier detected; tray menu is disabled.",
        );
    }

    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        // Tray: no Dock icon (Tao defaults to Regular).
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }

    let proxy = event_loop.create_proxy();

    // Graceful shutdown flag
    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = should_exit.clone();
    let proxy_clone = proxy.clone();

    // Set up signal handler - wake up event loop
    ctrlc::set_handler(move || {
        tracing::info!("received shutdown signal");
        should_exit_clone.store(true, Ordering::SeqCst);
        // Send event to wake up the event loop
        let _ = proxy_clone.send_event(UserEvent::Shutdown);
    })?;

    tray_icon::menu::MenuEvent::set_event_handler(Some({
        let proxy = proxy.clone();

        move |e| {
            let _ = proxy.send_event(UserEvent::Menu(e));
        }
    }));

    GlobalHotKeyEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::HotKey(e));
    }));

    let (tray, menu_items): (Option<tray_icon::TrayIcon>, Option<menu::MenuItems>) =
        if tray_supported {
            let (menu, menu_items) = menu::MenuItems::build()?;
            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip(i18n::tooltip())
                .with_icon(icon::scoria_icon())
                .build()
                .context("create tray icon")?;
            (Some(tray), Some(menu_items))
        } else {
            (None, None)
        };

    autostart::apply(cfg.autostart);

    // Check for updates only if auto_update is enabled
    if cfg.auto_update {
        actions::check_for_updates_bg();
    }

    let (hotkey_id, hk_manager) = hotkey_reg::try_register_hotkey(&cfg);

    // Store hotkey data in thread-safe way for reload capability
    use std::sync::atomic::{AtomicU32, Ordering};
    let hotkey_id = Arc::new(AtomicU32::new(hotkey_id.unwrap_or(0)));
    let hotkey_id_clone = hotkey_id.clone();

    let hk_manager = Arc::new(std::sync::Mutex::new(hk_manager));
    let hk_manager_clone = hk_manager.clone();

    watch::watch_config_bg(event_loop.create_proxy(), should_exit.clone());

    tracing::info!("scoria started successfully");

    event_loop.run(move |event, _elwt, control_flow| {
        // Check for graceful shutdown signal
        if should_exit.load(Ordering::SeqCst) {
            tracing::info!("shutting down gracefully");
            *control_flow = ControlFlow::Exit;
            return;
        }

        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::Menu(e)) => {
                actions::handle_menu(e.id.as_ref(), control_flow);
            }
            Event::UserEvent(UserEvent::HotKey(e))
                if e.state == HotKeyState::Pressed
                    && hotkey_id_clone.load(Ordering::SeqCst) == e.id =>
            {
                actions::handle_menu(menu::MENU_SAVE, control_flow);
            }
            Event::UserEvent(UserEvent::ConfigChanged) => {
                if let (Some(tray), Some(menu_items)) = (tray.as_ref(), menu_items.as_ref()) {
                    actions::on_config_changed(tray, menu_items);
                }

                // Reload autostart and hotkey on config change
                if let Ok(cfg) = config::load() {
                    autostart::apply(cfg.autostart);

                    let mut mgr = match hk_manager_clone.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            tracing::warn!("hotkey manager lock was poisoned, recovering");
                            poisoned.into_inner()
                        }
                    };
                    *mgr = None;
                    // Clear active id before re-register to avoid keeping stale id
                    // when parsing/registration fails for the new config value.
                    hotkey_id_clone.store(0, std::sync::atomic::Ordering::SeqCst);

                    let (new_id, new_mgr) = hotkey_reg::try_register_hotkey(&cfg);
                    *mgr = new_mgr;
                    if let Some(id) = new_id {
                        hotkey_id_clone.store(id, std::sync::atomic::Ordering::SeqCst);
                    }

                    tracing::info!("config changed, hotkey reloaded");
                }
            }
            Event::UserEvent(UserEvent::Shutdown) => {
                tracing::info!("shutdown event received, exiting");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
