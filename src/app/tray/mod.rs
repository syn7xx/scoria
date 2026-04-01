//! System tray: notifications, hotkey, config watcher.

mod actions;
mod hotkey_reg;
#[cfg(not(target_os = "linux"))]
mod icon;
mod menu;
mod notify;
#[cfg(not(target_os = "linux"))]
mod watch;

#[cfg(target_os = "linux")]
use anyhow::{Context, Result};
#[cfg(not(target_os = "linux"))]
use anyhow::Result;
use crate::engine::{autostart, config};
use crate::i18n;

#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(not(target_os = "linux"))]
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
#[cfg(not(target_os = "linux"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(target_os = "linux"))]
use std::sync::Arc;
#[cfg(not(target_os = "linux"))]
use tao::event::Event;
#[cfg(not(target_os = "linux"))]
use tao::event_loop::{ControlFlow, EventLoopBuilder};
#[cfg(not(target_os = "linux"))]
use tray_icon::TrayIconBuilder;

#[cfg(target_os = "linux")]
pub fn run() -> Result<()> {
    use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
    use ksni::blocking::TrayMethods;

    #[derive(Clone)]
    struct LinuxState {
        should_exit: Arc<AtomicBool>,
    }

    struct LinuxTray {
        state: LinuxState,
    }

    static LINUX_ICON_PIXMAP: OnceLock<Vec<ksni::Icon>> = OnceLock::new();

    fn linux_icon_pixmap() -> Vec<ksni::Icon> {
        LINUX_ICON_PIXMAP
            .get_or_init(|| {
                let decoder = png::Decoder::new(std::io::Cursor::new(include_bytes!(
                    "../../../assets/scoria-32.png"
                )));
                let mut reader = match decoder.read_info() {
                    Ok(reader) => reader,
                    Err(err) => {
                        tracing::warn!(error = %err, "failed to decode tray icon header");
                        return Vec::new();
                    }
                };
                let Some(output_size) = reader.output_buffer_size() else {
                    tracing::warn!("failed to determine tray icon output buffer size");
                    return Vec::new();
                };
                let mut data = vec![0u8; output_size];
                let info = match reader.next_frame(&mut data) {
                    Ok(info) => info,
                    Err(err) => {
                        tracing::warn!(error = %err, "failed to decode tray icon frame");
                        return Vec::new();
                    }
                };
                data.truncate(info.buffer_size());
                for pixel in data.chunks_exact_mut(4) {
                    // ksni expects ARGB32, while PNG decoding yields RGBA.
                    pixel.rotate_right(1);
                }

                vec![ksni::Icon {
                    width: info.width as i32,
                    height: info.height as i32,
                    data,
                }]
            })
            .clone()
    }

    impl ksni::Tray for LinuxTray {
        fn id(&self) -> String {
            "scoria".into()
        }

        fn title(&self) -> String {
            "Scoria".into()
        }

        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: i18n::tooltip().to_string(),
                ..Default::default()
            }
        }

        fn icon_name(&self) -> String {
            "scoria".into()
        }

        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            linux_icon_pixmap()
        }

        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            use ksni::menu::StandardItem;
            vec![
                StandardItem {
                    label: i18n::menu_save().into(),
                    activate: Box::new(|_| actions::do_save()),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: i18n::menu_settings().into(),
                    activate: Box::new(|_| actions::open_settings()),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: i18n::menu_config().into(),
                    activate: Box::new(|_| config::open_in_editor()),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: if actions::update_check_in_progress() {
                        i18n::menu_update_checking().into()
                    } else {
                        i18n::menu_update().into()
                    },
                    enabled: !actions::update_check_in_progress(),
                    activate: Box::new(|_| actions::do_update()),
                    ..Default::default()
                }
                .into(),
                ksni::MenuItem::Separator,
                StandardItem {
                    label: i18n::menu_quit().into(),
                    activate: Box::new(|tray: &mut Self| {
                        tray.state.should_exit.store(true, Ordering::SeqCst);
                    }),
                    ..Default::default()
                }
                .into(),
            ]
        }

        fn watcher_online(&self) {
            tracing::info!("status notifier watcher is online");
        }

        fn watcher_offline(&self, reason: ksni::OfflineReason) -> bool {
            tracing::warn!(?reason, "status notifier watcher is offline");
            true
        }
    }

    let cfg = config::load_or_create()?;
    i18n::apply(&cfg.language);
    autostart::apply(cfg.autostart);
    if cfg.auto_update {
        actions::check_for_updates_bg();
    }

    let should_exit = Arc::new(AtomicBool::new(false));
    let state = LinuxState {
        should_exit: should_exit.clone(),
    };

    ctrlc::set_handler({
        let should_exit = should_exit.clone();
        move || {
            tracing::info!("received shutdown signal");
            should_exit.store(true, Ordering::SeqCst);
        }
    })?;

    let tray_handle = LinuxTray { state: state.clone() }
        .assume_sni_available(true)
        .spawn()
        .context("start ksni tray service")?;

    let (hotkey_id, hk_manager) = hotkey_reg::try_register_hotkey(&cfg);
    let hotkey_id = Arc::new(AtomicU32::new(hotkey_id.unwrap_or(0)));
    let hk_manager = Arc::new(Mutex::new(hk_manager));

    GlobalHotKeyEvent::set_event_handler(Some({
        let hotkey_id = hotkey_id.clone();
        move |e: GlobalHotKeyEvent| {
            if e.state == HotKeyState::Pressed && hotkey_id.load(Ordering::SeqCst) == e.id {
                actions::do_save();
            }
        }
    }));

    let config_mtime = Arc::new(Mutex::new(
        std::fs::metadata(config::config_path()?)
            .ok()
            .and_then(|m| m.modified().ok()),
    ));

    tracing::info!("scoria started successfully (linux ksni)");
    while !should_exit.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_secs(2));

        let new_modified = config::config_path()
            .ok()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());

        let changed = {
            let mut last = match config_mtime.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if *last != new_modified {
                *last = new_modified;
                true
            } else {
                false
            }
        };

        if !changed {
            continue;
        }

        actions::on_config_changed_linux();

        if let Ok(cfg) = config::load() {
            autostart::apply(cfg.autostart);
            let mut mgr = match hk_manager.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    tracing::warn!("hotkey manager lock was poisoned, recovering");
                    poisoned.into_inner()
                }
            };
            *mgr = None;
            hotkey_id.store(0, Ordering::SeqCst);
            let (new_id, new_mgr) = hotkey_reg::try_register_hotkey(&cfg);
            *mgr = new_mgr;
            if let Some(id) = new_id {
                hotkey_id.store(id, Ordering::SeqCst);
            }

            let _ = tray_handle.update(|_| {});
            tracing::info!("config changed, hotkey reloaded");
        }
    }

    tray_handle.shutdown().wait();
    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[derive(Debug)]
enum UserEvent {
    Menu(tray_icon::menu::MenuEvent),
    HotKey(GlobalHotKeyEvent),
    ConfigChanged,
    UpdateStateChanged,
    Shutdown,
}

/// Start the tray icon and event loop.
#[cfg(not(target_os = "linux"))]
pub fn run() -> Result<()> {
    let cfg = config::load_or_create()?;

    i18n::apply(&cfg.language);

    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }

    let proxy = event_loop.create_proxy();

    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = should_exit.clone();
    let proxy_clone = proxy.clone();

    ctrlc::set_handler(move || {
        tracing::info!("received shutdown signal");
        should_exit_clone.store(true, Ordering::SeqCst);
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

    let (menu, menu_items) = menu::MenuItems::build()?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(i18n::tooltip())
        .with_icon(icon::scoria_icon().context("load tray icon")?)
        .build()
        .context("create tray icon")?;

    autostart::apply(cfg.autostart);
    if cfg.auto_update {
        actions::check_for_updates_bg();
    }

    let (hotkey_id, hk_manager) = hotkey_reg::try_register_hotkey(&cfg);
    let hotkey_id = Arc::new(std::sync::atomic::AtomicU32::new(hotkey_id.unwrap_or(0)));
    let hotkey_id_clone = hotkey_id.clone();
    let hk_manager = Arc::new(std::sync::Mutex::new(hk_manager));
    let hk_manager_clone = hk_manager.clone();

    watch::watch_config_bg(event_loop.create_proxy(), should_exit.clone());

    tracing::info!("scoria started successfully");

    event_loop.run(move |event, _elwt, control_flow| {
        if should_exit.load(Ordering::SeqCst) {
            tracing::info!("shutting down gracefully");
            *control_flow = ControlFlow::Exit;
            return;
        }

        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::Menu(e)) => {
                if e.id.as_ref() == menu::MENU_UPDATE {
                    let proxy = proxy.clone();
                    actions::do_update_with_state_hook(move || {
                        let _ = proxy.send_event(UserEvent::UpdateStateChanged);
                    });
                } else {
                    actions::handle_menu(e.id.as_ref(), control_flow);
                }
            }
            Event::UserEvent(UserEvent::HotKey(e))
                if e.state == HotKeyState::Pressed
                    && hotkey_id_clone.load(Ordering::SeqCst) == e.id =>
            {
                actions::handle_menu(menu::MENU_SAVE, control_flow);
            }
            Event::UserEvent(UserEvent::ConfigChanged) => {
                actions::on_config_changed(&tray, &menu_items);
                menu_items.update.set_enabled(!actions::update_check_in_progress());
                menu_items.update.set_text(if actions::update_check_in_progress() {
                    i18n::menu_update_checking()
                } else {
                    i18n::menu_update()
                });
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
                    hotkey_id_clone.store(0, std::sync::atomic::Ordering::SeqCst);
                    let (new_id, new_mgr) = hotkey_reg::try_register_hotkey(&cfg);
                    *mgr = new_mgr;
                    if let Some(id) = new_id {
                        hotkey_id_clone.store(id, std::sync::atomic::Ordering::SeqCst);
                    }

                    tracing::info!("config changed, hotkey reloaded");
                }
            }
            Event::UserEvent(UserEvent::UpdateStateChanged) => {
                menu_items.update.set_enabled(!actions::update_check_in_progress());
                menu_items.update.set_text(if actions::update_check_in_progress() {
                    i18n::menu_update_checking()
                } else {
                    i18n::menu_update()
                });
            }
            Event::UserEvent(UserEvent::Shutdown) => {
                tracing::info!("shutdown event received, exiting");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
