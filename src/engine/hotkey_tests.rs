use super::*;

#[test]
fn test_parse_hotkey_simple_letter() {
    // Single letter key
    let result = parse_hotkey("Ctrl+S");
    assert!(result.is_ok());
    let hk = result.expect("Ctrl+S should parse");
    let _ = hk.id(); // Just verify it was created
}

#[test]
fn test_parse_hotkey_with_shift() {
    let result = parse_hotkey("Shift+A");
    assert!(result.is_ok());
}

#[test]
fn test_parse_hotkey_with_alt() {
    let result = parse_hotkey("Alt+Enter");
    assert!(result.is_ok());
}

#[test]
fn test_parse_hotkey_function_keys() {
    for key in ["F1", "F12"] {
        let result = parse_hotkey(key);
        assert!(result.is_ok(), "failed for key: {}", key);
    }
}

#[test]
fn test_parse_hotkey_special_keys() {
    let special_keys = ["Space", "Tab", "Enter", "Escape", "Esc"];

    for key in special_keys {
        let result = parse_hotkey(key);
        assert!(result.is_ok(), "failed for key: {}", key);
    }
}

#[test]
fn test_parse_hotkey_with_super() {
    let variants = ["Super+S", "Meta+S", "Win+S", "Cmd+S"];

    for spec in variants {
        let result = parse_hotkey(spec);
        assert!(result.is_ok(), "failed for spec: {}", spec);
    }
}

#[test]
fn test_parse_hotkey_numeric() {
    // Digits 0-9
    for i in 0..=9 {
        let result = parse_hotkey(&format!("Ctrl+{}", i));
        assert!(result.is_ok(), "failed for Ctrl+{}", i);
    }
}

#[test]
fn test_parse_hotkey_punctuation() {
    // Punctuation keys
    let punctuation = ["Comma", "Period", "Minus", "Equal", "Slash"];

    for key in punctuation {
        let result = parse_hotkey(key);
        assert!(result.is_ok(), "failed for key: {}", key);
    }
}

#[test]
fn test_parse_hotkey_empty_string() {
    let result = parse_hotkey("");
    assert!(result.is_err());
}

#[test]
fn test_parse_hotkey_invalid_modifier() {
    let result = parse_hotkey("Invalid+S");
    assert!(result.is_err());
}

#[test]
fn test_parse_hotkey_invalid_key() {
    let result = parse_hotkey("Ctrl+InvalidKeyXYZ");
    assert!(result.is_err());
}

#[test]
fn test_parse_hotkey_case_insensitive_modifiers() {
    // Modifiers should be case-insensitive
    let variants = ["CTRL+S", "ctrl+s", "Ctrl+s", "cTrL+s"];

    for spec in variants {
        let result = parse_hotkey(spec);
        assert!(result.is_ok(), "failed for spec: {}", spec);
    }
}

#[test]
fn test_parse_hotkey_three_modifiers() {
    let result = parse_hotkey("Ctrl+Shift+Alt+S");
    assert!(result.is_ok());
}

#[test]
fn test_parse_hotkey_native_format() {
    // Native format from global-hotkey (Control+KeyS)
    let result = parse_hotkey("Control+KeyS");
    assert!(result.is_ok());
}

#[test]
fn test_parse_hotkey_brackets() {
    let result = parse_hotkey("Ctrl+[");
    assert!(result.is_ok());
}

#[test]
fn test_parse_hotkey_semicolon() {
    let result = parse_hotkey("Ctrl+;");
    assert!(result.is_ok());
}
