use super::*;
use crate::engine::config::SaveTarget;

fn valid_draft() -> SettingsDraft {
    SettingsDraft {
        vault_path: "/test/vault".into(),
        target: SaveTarget::NewFileInFolder,
        folder: "scoria".into(),
        append_file: "Notes.md".into(),
        filename_template: "clip-%Y%m%d.md".into(),
        prepend_timestamp_header: true,
        hotkey_raw: "Ctrl+S".into(),
        autostart: true,
        auto_update: false,
        language: "en".into(),
    }
}

#[test]
fn test_validate_valid_draft() {
    let draft = valid_draft();
    let result = validate_and_build(draft);
    assert!(result.is_ok());
}

#[test]
fn test_validate_empty_folder() {
    let mut draft = valid_draft();
    draft.folder = "   ".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SettingsValidationError::EmptySubfolder));
}

#[test]
fn test_validate_empty_append_file() {
    let mut draft = valid_draft();
    draft.append_file = "   ".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SettingsValidationError::EmptyAppend));
}

#[test]
fn test_validate_empty_template() {
    let mut draft = valid_draft();
    draft.filename_template = "   ".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SettingsValidationError::EmptyTemplate));
}

#[test]
fn test_validate_invalid_hotkey() {
    let mut draft = valid_draft();
    draft.hotkey_raw = "Invalid+Key+XYZ".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_err());
    match result.unwrap_err() {
        SettingsValidationError::InvalidHotkey(msg) => {
            assert!(!msg.is_empty());
        }
        _ => panic!("expected InvalidHotkey"),
    }
}

#[test]
fn test_validate_empty_hotkey_is_ok() {
    let mut draft = valid_draft();
    draft.hotkey_raw = "".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().hotkey, None);
}

#[test]
fn test_validate_valid_hotkey_is_stored() {
    let mut draft = valid_draft();
    draft.hotkey_raw = "Ctrl+Shift+S".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().hotkey, Some("Ctrl+Shift+S".into()));
}

#[test]
fn test_validate_trims_whitespace() {
    let mut draft = valid_draft();
    draft.folder = "  scoria  ".into();
    draft.append_file = "  Notes.md  ".into();
    draft.filename_template = "  clip.md  ".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_ok());
    let cfg = result.unwrap();
    assert_eq!(cfg.folder, "scoria");
    assert_eq!(cfg.append_file, "Notes.md");
    assert_eq!(cfg.filename_template, "clip.md");
}

#[test]
fn test_validate_appends_target() {
    let mut draft = valid_draft();
    draft.target = SaveTarget::AppendToFile;
    
    let result = validate_and_build(draft);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().target, SaveTarget::AppendToFile);
}

#[test]
fn test_validate_prepend_timestamp() {
    let mut draft = valid_draft();
    draft.prepend_timestamp_header = false;
    
    let result = validate_and_build(draft);
    assert!(result.is_ok());
    assert!(!result.unwrap().prepend_timestamp_header);
}

#[test]
fn test_validate_autostart() {
    let mut draft = valid_draft();
    draft.autostart = true;
    
    let result = validate_and_build(draft);
    assert!(result.is_ok());
    assert!(result.unwrap().autostart);
}

#[test]
fn test_validate_language() {
    let mut draft = valid_draft();
    draft.language = "ru".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().language, "ru");
}

#[test]
fn test_validate_vault_path_trimmed() {
    let mut draft = valid_draft();
    draft.vault_path = "  /test/vault  ".into();
    
    let result = validate_and_build(draft);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().vault_path.to_string_lossy(), "/test/vault");
}

#[test]
fn test_settings_draft_clone() {
    let draft = valid_draft();
    let cloned = draft.clone();
    assert_eq!(draft.folder, cloned.folder);
    assert_eq!(draft.append_file, cloned.append_file);
}
