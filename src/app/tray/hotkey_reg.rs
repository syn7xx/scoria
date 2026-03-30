use global_hotkey::GlobalHotKeyManager;

use crate::engine::{config, hotkey};

pub(crate) fn try_register_hotkey(
    cfg: &config::Config,
) -> (Option<u32>, Option<GlobalHotKeyManager>) {
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
