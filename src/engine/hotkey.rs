use std::str::FromStr;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};

/// Parse a hotkey string in friendly format (`Ctrl+Shift+S`) or native format (`Control+KeyS`).
pub fn parse_hotkey(spec: &str) -> anyhow::Result<HotKey> {
    if let Ok(hk) = spec.parse::<HotKey>() {
        return Ok(hk);
    }

    let (mod_parts, key_part) = split_spec(spec)?;
    let mods = parse_modifiers(&mod_parts)?;
    let code = friendly_code(&key_part)?;
    Ok(HotKey::new(Some(mods), code))
}

fn split_spec(spec: &str) -> anyhow::Result<(Vec<String>, String)> {
    let parts: Vec<&str> = spec
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    anyhow::ensure!(!parts.is_empty(), "empty hotkey");
    let key = parts[parts.len() - 1].to_string();
    let mods = parts[..parts.len() - 1]
        .iter()
        .map(|s| s.to_string())
        .collect();
    Ok((mods, key))
}

fn parse_modifiers(parts: &[String]) -> anyhow::Result<Modifiers> {
    let mut mods = Modifiers::empty();
    for p in parts {
        mods |= match p.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Modifiers::CONTROL,
            "alt" => Modifiers::ALT,
            "shift" => Modifiers::SHIFT,
            "super" | "meta" | "win" | "cmd" => Modifiers::SUPER,
            other => anyhow::bail!("unknown modifier: {other}"),
        };
    }
    Ok(mods)
}

/// Map friendly key names (`s`, `3`, `f1`, `space`) to `keyboard_types::Code`.
fn friendly_code(s: &str) -> anyhow::Result<Code> {
    let s = s.trim();
    if s.len() == 1 {
        let Some(c) = s.chars().next() else {
            anyhow::bail!("empty key");
        };
        if c.is_ascii_alphabetic() {
            let name = format!("Key{}", c.to_ascii_uppercase());
            return Code::from_str(&name).map_err(|e| anyhow::anyhow!("{e}"));
        }
        if c.is_ascii_digit() {
            let name = format!("Digit{c}");
            return Code::from_str(&name).map_err(|e| anyhow::anyhow!("{e}"));
        }
    }

    let lower = s.to_ascii_lowercase();
    let alias = match lower.as_str() {
        "space" => "Space",
        "tab" => "Tab",
        "enter" | "return" => "Enter",
        "escape" | "esc" => "Escape",
        "minus" | "-" => "Minus",
        "equal" | "=" => "Equal",
        "comma" => "Comma",
        "period" | "." => "Period",
        "slash" | "/" => "Slash",
        "backslash" | "\\" => "Backslash",
        "semicolon" | ";" => "Semicolon",
        "quote" | "'" => "Quote",
        "bracketleft" | "[" => "BracketLeft",
        "bracketright" | "]" => "BracketRight",
        "grave" | "`" => "Backquote",
        other => other, // try as-is (F1, F12, etc.)
    };

    Code::from_str(alias)
        .or_else(|_| {
            // Title-case fallback for f1 → F1, f12 → F12
            let mut tc = alias.to_string();
            if let Some(first) = tc.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            Code::from_str(&tc)
        })
        .map_err(|_| anyhow::anyhow!("unknown key: {s} (try: a-z, 0-9, Space, F1-F12, etc.)"))
}

#[cfg(test)]
#[path = "hotkey_tests.rs"]
mod hotkey_tests;
