//! Minimal localisation (EN / RU, easily extendable).
//!
//! ## Adding a new language
//!
//! 1. Add a variant to [`Lang`].
//! 2. Create `src/i18n/xx.rs` with `pub(super) static XX: T = T { ... };`.
//! 3. Add `mod xx;` and one match arm in [`tr()`].
//! 4. Add one combo-box entry in `ui/gtk.rs` / `ui/macos.rs`.
//!
//! Call [`apply`] with `Config.language` before building any UI.
//! Falls back to system locale auto-detection on first use.

mod en;
mod ru;

use std::sync::RwLock;

use en::EN;
use ru::RU;

// RwLock allows apply() to be called multiple times (e.g. after saving
// settings), so language changes take effect immediately without restarting.
static LANG: RwLock<Option<Lang>> = RwLock::new(None);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Ru,
}

/// Apply a language setting. May be called any number of times; each call
/// takes effect immediately for all subsequent UI string lookups.
/// `""` = auto-detect from system locale.
pub fn apply(setting: &str) {
    let lang = if setting.is_empty() {
        detect()
    } else {
        parse(setting)
    };
    *LANG.write().unwrap() = Some(lang);
}

/// Current language. Auto-detects from system locale on first call if
/// [`apply`] has not been called yet.
pub fn current() -> Lang {
    if let Some(lang) = *LANG.read().unwrap() {
        return lang;
    }
    let lang = detect();
    *LANG.write().unwrap() = Some(lang);
    lang
}

fn parse(s: &str) -> Lang {
    if s.to_lowercase().starts_with("ru") {
        Lang::Ru
    } else {
        Lang::En
    }
}

fn detect() -> Lang {
    for var in ["LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES"] {
        if let Ok(v) = std::env::var(var) {
            if !v.is_empty() {
                return parse(&v);
            }
        }
    }
    Lang::En
}

/// Returns the active translation table.
///
/// Use `tr().field` for direct field access, or the convenience functions below.
pub fn tr() -> &'static T {
    match current() {
        Lang::En => &EN,
        Lang::Ru => &RU,
    }
}

// ---------------------------------------------------------------------------
// Translation table
// ---------------------------------------------------------------------------

/// All translatable strings.
/// Format templates use `{placeholder}` tokens — see the wrapper functions below.
pub struct T {
    // ─── Tray menu ───────────────────────────────────────────────────────────
    pub menu_save: &'static str,
    pub menu_settings: &'static str,
    pub menu_config: &'static str,
    pub menu_update: &'static str,
    pub menu_quit: &'static str,
    pub tooltip: &'static str,

    // ─── Notifications ───────────────────────────────────────────────────────
    pub notif_saved_title: &'static str,
    pub notif_saved_body: &'static str, // {path}
    pub notif_save_failed: &'static str,
    pub notif_update_available: &'static str,
    pub notif_update_available_body: &'static str, // {msg}
    pub notif_updating: &'static str,
    pub notif_downloading: &'static str, // {tag}
    pub notif_updated: &'static str,
    pub notif_updated_body: &'static str, // {tag}
    pub notif_up_to_date: &'static str,
    pub notif_up_to_date_body: &'static str, // {ver}
    pub notif_update_failed: &'static str,
    pub notif_checking: &'static str,
    pub notif_looking: &'static str,
    pub notif_unreachable: &'static str,

    // ─── Settings UI ─────────────────────────────────────────────────────────
    pub settings_title: &'static str,
    pub settings_vault: &'static str,
    pub settings_save_mode: &'static str,
    pub settings_folder: &'static str,
    pub settings_append: &'static str,
    pub settings_template: &'static str,
    pub settings_timestamp: &'static str,
    pub settings_autostart: &'static str,
    pub settings_auto_update: &'static str,
    pub settings_hotkey_x11: &'static str,
    pub settings_hotkey: &'static str,
    pub settings_hotkey_wayland: &'static str,
    pub settings_hotkey_hint: &'static str,
    pub settings_hotkey_placeholder: &'static str,
    pub settings_browse: &'static str,
    pub settings_detect: &'static str,
    pub settings_cancel: &'static str,
    pub settings_save: &'static str,
    pub settings_raw: &'static str,
    pub settings_lang: &'static str,

    // ─── Save targets ────────────────────────────────────────────────────────
    pub save_target_new_file: &'static str,
    pub save_target_append: &'static str,

    // ─── Runtime errors (bubble up to notifications) ─────────────────────────
    /// "Nothing to save — select text or copy something first" (Linux primary+clipboard)
    pub err_nothing_to_save_selection: &'static str,
    /// "Nothing to save — copy something first" (macOS / other)
    pub err_nothing_to_save: &'static str,
    pub err_text_empty: &'static str,
    pub err_image_empty: &'static str,
    pub err_vault_path_empty: &'static str, // {path}
    pub err_vault_not_found: &'static str,  // {path}
    pub err_vault_not_dir: &'static str,    // {path}

    // ─── Alerts ──────────────────────────────────────────────────────────────
    pub alert_no_config: &'static str,
    pub alert_invalid: &'static str,
    pub alert_empty_subfolder: &'static str,
    pub alert_empty_append: &'static str,
    pub alert_empty_template: &'static str,
    pub alert_invalid_hotkey: &'static str,
    pub alert_save_failed: &'static str,
    pub alert_no_vaults: &'static str,
    pub alert_no_vaults_body: &'static str,
    pub alert_no_vaults_gtk: &'static str,
    pub alert_choose_vault: &'static str,
    pub alert_multiple_vaults: &'static str,
    pub alert_pick_vault: &'static str,
    pub alert_vault_open: &'static str,
}

// ---------------------------------------------------------------------------
// Convenience API — generated by macro for static fields, written manually
// for format-template fields (they need argument substitution).
// ---------------------------------------------------------------------------

macro_rules! tr_fns {
    ($($name:ident),+ $(,)?) => {
        $(pub fn $name() -> &'static str { tr().$name })+
    };
}

tr_fns!(
    menu_save,
    menu_settings,
    menu_config,
    menu_update,
    menu_quit,
    tooltip,
    notif_saved_title,
    notif_save_failed,
    notif_update_available,
    notif_updating,
    notif_updated,
    notif_up_to_date,
    notif_update_failed,
    notif_checking,
    notif_looking,
    notif_unreachable,
    settings_title,
    settings_vault,
    settings_save_mode,
    settings_folder,
    settings_append,
    settings_template,
    settings_timestamp,
    settings_autostart,
    settings_auto_update,
    settings_hotkey_x11,
    settings_hotkey,
    settings_hotkey_wayland,
    settings_hotkey_hint,
    settings_hotkey_placeholder,
    settings_browse,
    settings_detect,
    settings_cancel,
    settings_save,
    settings_raw,
    settings_lang,
    save_target_new_file,
    save_target_append,
    err_nothing_to_save_selection,
    err_nothing_to_save,
    err_text_empty,
    err_image_empty,
    alert_no_config,
    alert_invalid,
    alert_empty_subfolder,
    alert_empty_append,
    alert_empty_template,
    alert_invalid_hotkey,
    alert_save_failed,
    alert_no_vaults,
    alert_no_vaults_body,
    alert_no_vaults_gtk,
    alert_choose_vault,
    alert_multiple_vaults,
    alert_pick_vault,
    alert_vault_open,
);

// Format-template helpers — substitute the named `{placeholder}` token.

pub fn notif_saved_body(path: &str) -> String {
    tr().notif_saved_body.replace("{path}", path)
}
pub fn notif_update_available_body(msg: &str) -> String {
    tr().notif_update_available_body.replace("{msg}", msg)
}
pub fn notif_downloading(tag: &str) -> String {
    tr().notif_downloading.replace("{tag}", tag)
}
pub fn notif_updated_body(tag: &str) -> String {
    tr().notif_updated_body.replace("{tag}", tag)
}
pub fn notif_up_to_date_body(ver: &str) -> String {
    tr().notif_up_to_date_body.replace("{ver}", ver)
}
pub fn err_vault_path_empty(path: &str) -> String {
    tr().err_vault_path_empty.replace("{path}", path)
}
pub fn err_vault_not_found(path: &str) -> String {
    tr().err_vault_not_found.replace("{path}", path)
}
pub fn err_vault_not_dir(path: &str) -> String {
    tr().err_vault_not_dir.replace("{path}", path)
}

#[cfg(test)]
#[path = "i18n_tests.rs"]
mod i18n_tests;
