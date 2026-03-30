//! System tray: Tao event loop, menu, notifications, hotkey, config watcher.

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

#[derive(Debug)]
enum UserEvent {
    Menu(tray_icon::menu::MenuEvent),
    HotKey(GlobalHotKeyEvent),
    /// Fired by the config-watcher thread when config.toml is modified.
    ConfigChanged,
}

/// Start the tray icon and event loop (Linux / macOS).
pub fn run() -> Result<()> {
    let cfg = config::load_or_create()?;

    i18n::apply(&cfg.language);

    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        // Tray: no Dock icon (Tao defaults to Regular).
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }

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

    let (menu, menu_items) = menu::MenuItems::build()?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(i18n::tooltip())
        .with_icon(icon::scoria_icon())
        .build()
        .context("create tray icon")?;

    autostart::apply(cfg.autostart);

    let (hotkey_id, hk_manager) = hotkey_reg::try_register_hotkey(&cfg);

    actions::check_for_updates_bg();
    watch::watch_config_bg(event_loop.create_proxy());

    event_loop.run(move |event, _elwt, control_flow| {
        let _ = &hk_manager;
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::Menu(e)) => {
                actions::handle_menu(e.id.as_ref(), control_flow);
            }
            Event::UserEvent(UserEvent::HotKey(e))
                if e.state == HotKeyState::Pressed && hotkey_id == Some(e.id) =>
            {
                actions::handle_menu(menu::MENU_SAVE, control_flow);
            }
            Event::UserEvent(UserEvent::ConfigChanged) => {
                actions::on_config_changed(&tray, &menu_items);
            }
            _ => {}
        }
    });
}
