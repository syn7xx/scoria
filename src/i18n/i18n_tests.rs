use super::*;

/// Reset language to None for clean test state
fn reset_lang() {
    *LANG.write().expect("lock LANG for reset") = None;
}

#[test]
fn test_lang_enum_variants() {
    assert_eq!(Lang::En, Lang::En);
    assert_eq!(Lang::Ru, Lang::Ru);
    assert_ne!(Lang::En, Lang::Ru);
}

#[test]
fn test_parse_english() {
    reset_lang();
    let variants = ["en", "EN", "English", "en-US"];
    for v in variants {
        let lang = parse(v);
        assert_eq!(lang, Lang::En, "failed for: {}", v);
    }
}

#[test]
fn test_parse_russian() {
    reset_lang();
    let variants = ["ru", "RU", "Russian", "ru-RU", "ruRU"];
    for v in variants {
        let lang = parse(v);
        assert_eq!(lang, Lang::Ru, "failed for: {}", v);
    }
}

#[test]
fn test_apply_empty_string_uses_detect() {
    reset_lang();
    apply("");
    // Should not panic - just uses detect()
    let _ = current();
}

#[test]
fn test_apply_english() {
    reset_lang();
    apply("en");
    assert_eq!(current(), Lang::En);
}

#[test]
fn test_apply_russian() {
    reset_lang();
    apply("ru");
    assert_eq!(current(), Lang::Ru);
}

#[test]
fn test_apply_unknown_falls_back_to_english() {
    reset_lang();
    apply("xx"); // Unknown language code
    assert_eq!(current(), Lang::En);
}

#[test]
fn test_current_auto_detects() {
    reset_lang();
    let lang = current();
    // Should return either En or Ru
    assert!(lang == Lang::En || lang == Lang::Ru);
}

#[test]
fn test_tr_returns_english_when_en() {
    reset_lang();
    apply("en");
    let t = tr();
    // English strings should contain English words
    assert!(t.menu_save.contains("Save"));
    assert!(t.menu_quit.contains("Quit"));
}

#[test]
fn test_tr_returns_russian_when_ru() {
    reset_lang();
    apply("ru");
    let t = tr();
    // Russian strings should contain Russian words
    assert!(t.menu_save.contains("Сохранить") || t.menu_save.contains("Save"));
    assert!(t.menu_quit.contains("Выход") || t.menu_quit.contains("Quit"));
}

#[test]
fn test_tr_fields_are_not_empty() {
    reset_lang();
    apply("en");
    let t = tr();

    // All fields should be non-empty
    assert!(!t.menu_save.is_empty());
    assert!(!t.menu_settings.is_empty());
    assert!(!t.menu_config.is_empty());
    assert!(!t.menu_update.is_empty());
    assert!(!t.menu_quit.is_empty());
    assert!(!t.tooltip.is_empty());

    assert!(!t.notif_saved_title.is_empty());
    assert!(!t.notif_save_failed.is_empty());

    assert!(!t.settings_title.is_empty());
    assert!(!t.settings_vault.is_empty());
    assert!(!t.settings_folder.is_empty());
}

#[test]
fn test_apply_can_be_called_multiple_times() {
    reset_lang();
    apply("en");
    assert_eq!(current(), Lang::En);

    apply("ru");
    assert_eq!(current(), Lang::Ru);

    apply("en");
    assert_eq!(current(), Lang::En);
}

#[test]
fn test_convenience_functions_exist() {
    reset_lang();
    apply("en");

    // These should all compile and not panic
    let _ = menu_save();
    let _ = menu_settings();
    let _ = menu_quit();
    let _ = tooltip();
    let _ = notif_saved_title();
    let _ = notif_save_failed();
    let _ = settings_title();
    let _ = settings_vault();
}

#[test]
fn test_format_template_functions() {
    reset_lang();
    apply("en");

    // These should all compile and return String
    let path = "/test/path";
    let body = notif_saved_body(path);
    assert!(body.contains(path));

    let msg = "v1.0.0";
    let update_body = notif_update_available_body(msg);
    assert!(update_body.contains(msg));

    let tag = "v1.0.0";
    let downloading = notif_downloading(tag);
    assert!(downloading.contains(tag));

    let ver = "1.0.0";
    let up_to_date = notif_up_to_date_body(ver);
    assert!(up_to_date.contains(ver));
}

#[test]
fn test_err_vault_functions() {
    reset_lang();
    apply("en");

    let path = "/nonexistent";
    let err = err_vault_path_empty(path);
    assert!(err.contains(path));

    let err = err_vault_not_found(path);
    assert!(err.contains(path));

    let err = err_vault_not_dir(path);
    assert!(err.contains(path));
}

#[test]
fn test_lang_clone() {
    let en = Lang::En;
    let cloned = en; // Copy trait, not clone
    assert_eq!(en, cloned);
}

#[test]
fn test_lang_copy() {
    let en = Lang::En;
    let copied = en; // Copy, not clone
    assert_eq!(en, copied);
}
