use global_hotkey::GlobalHotKeyManager;

use crate::engine::{config, hotkey};

pub(crate) fn try_register_hotkey(
    cfg: &config::Config,
) -> (Option<u32>, Option<GlobalHotKeyManager>) {
    let Some(ref spec) = cfg.hotkey else {
        tracing::debug!("no hotkey configured");
        return (None, None);
    };

    let hk = match hotkey::parse_hotkey(spec) {
        Ok(hk) => hk,
        Err(e) => {
            tracing::warn!(hotkey = %spec, error = %e, "invalid hotkey");
            return (None, None);
        }
    };

    let mgr = match GlobalHotKeyManager::new() {
        Ok(mgr) => mgr,
        Err(e) => {
            tracing::warn!(error = %e, "global hotkeys unavailable");
            return (None, None);
        }
    };

    match mgr.register(hk) {
        Ok(()) => {
            tracing::info!(hotkey = %spec, "hotkey registered");
            (Some(hk.id()), Some(mgr))
        }
        Err(e) => {
            tracing::warn!(hotkey = %spec, error = %e, "could not register hotkey");
            (None, None)
        }
    }
}
