//! Validated construction of [`Config`] from UI field values (shared by GTK and AppKit).

use std::path::PathBuf;

use crate::engine::config::{Config, SaveTarget};
use crate::engine::hotkey;
use crate::engine::path_safety;

/// Raw values from the settings form before validation.
#[derive(Debug, Clone)]
pub struct SettingsDraft {
    pub vault_path: String,
    pub target: SaveTarget,
    pub folder: String,
    pub append_file: String,
    pub filename_template: String,
    pub prepend_timestamp_header: bool,
    pub hotkey_raw: String,
    pub autostart: bool,
    pub auto_update: bool,
    pub language: String,
}

/// Non-empty field checks and hotkey parse failure (same rules on all platforms).
#[derive(Debug)]
pub enum SettingsValidationError {
    EmptySubfolder,
    EmptyAppend,
    EmptyTemplate,
    InvalidHotkey(String),
    InvalidPath(String),
}

fn validate_path_component(s: &str, field_name: &str) -> Result<(), SettingsValidationError> {
    path_safety::validate_relative_fragment(s, field_name)
        .map_err(SettingsValidationError::InvalidPath)
}

/// Trim fields, validate, and build a [`Config`] ready to save.
pub fn validate_and_build(draft: SettingsDraft) -> Result<Config, SettingsValidationError> {
    let folder = draft.folder.trim();
    if folder.is_empty() {
        return Err(SettingsValidationError::EmptySubfolder);
    }
    validate_path_component(folder, "folder")?;

    let append_file = draft.append_file.trim();
    if append_file.is_empty() {
        return Err(SettingsValidationError::EmptyAppend);
    }
    validate_path_component(append_file, "append_file")?;

    let filename_template = draft.filename_template.trim();
    if filename_template.is_empty() {
        return Err(SettingsValidationError::EmptyTemplate);
    }
    // Filename template shouldn't have path separators
    if filename_template.contains('/') || filename_template.contains('\\') {
        return Err(SettingsValidationError::InvalidPath(
            "filename_template cannot contain path separators".into(),
        ));
    }

    let hotkey = match draft.hotkey_raw.trim() {
        "" => None,
        h => match hotkey::parse_hotkey(h) {
            Ok(_) => Some(h.to_string()),
            Err(e) => return Err(SettingsValidationError::InvalidHotkey(e.to_string())),
        },
    };

    Ok(Config {
        vault_path: PathBuf::from(draft.vault_path.trim()),
        target: draft.target,
        folder: folder.to_string(),
        append_file: append_file.to_string(),
        filename_template: filename_template.to_string(),
        prepend_timestamp_header: draft.prepend_timestamp_header,
        hotkey,
        autostart: draft.autostart,
        auto_update: draft.auto_update,
        language: draft.language,
    })
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod settings_tests;
