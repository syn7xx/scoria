use std::time::Duration;

use tao::event_loop::EventLoopProxy;

use crate::engine::config;

use super::UserEvent;

pub(crate) fn watch_config_bg(proxy: EventLoopProxy<UserEvent>) {
    std::thread::spawn(move || {
        let Ok(path) = config::config_path() else {
            return;
        };
        let mut last_modified = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());

        loop {
            std::thread::sleep(Duration::from_secs(2));

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
