use anyhow::{Result, bail};

#[cfg(any(target_os = "macos", test))]
use crate::i18n;

pub enum Content {
    Text(String),
    Image { data: Vec<u8>, ext: &'static str },
}

fn run_bytes(cmd: &str, args: &[&str]) -> Option<Vec<u8>> {
    let out = std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    if !out.stdout.is_empty() {
        Some(out.stdout)
    } else {
        None
    }
}

fn run_bytes_timeout(cmd: &str, args: &[&str]) -> Option<Vec<u8>> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .filter(|o| !o.stdout.is_empty())
        .map(|o| o.stdout)
}

fn run_text_timeout(cmd: &str, args: &[&str]) -> Option<String> {
    run_bytes_timeout(cmd, args).and_then(|bytes| {
        let s = String::from_utf8_lossy(&bytes).into_owned();
        if s.is_empty() { None } else { Some(s) }
    })
}

fn run_text(cmd: &str, args: &[&str]) -> Option<String> {
    let bytes = run_bytes(cmd, args)?;
    let s = String::from_utf8_lossy(&bytes).into_owned();
    if s.is_empty() { None } else { Some(s) }
}

fn read_arboard() -> Option<Content> {
    let mut cb = arboard::Clipboard::new().ok()?;
    if let Ok(img) = cb.get_image() {
        if let Some(data) = encode_rgba_png(img.width, img.height, &img.bytes) {
            return Some(Content::Image { data, ext: "png" });
        }
    }
    if let Ok(text) = cb.get_text() {
        if !text.is_empty() {
            return Some(Content::Text(text));
        }
    }
    None
}

#[cfg(any(target_os = "macos", test))]
fn choose_clipboard_content(arboard: Option<Content>, pb_text: Option<String>) -> Result<Content> {
    if let Some(c) = arboard {
        match c {
            // pbpaste handles text reliably; if we have text from pbpaste, prefer it.
            // Images are only available via arboard, so keep arboard for images.
            Content::Text(_) => {
                if let Some(t) = pb_text {
                    return Ok(Content::Text(t));
                }
                Ok(c)
            }
            Content::Image { .. } => Ok(c),
        }
    } else if let Some(t) = pb_text {
        Ok(Content::Text(t))
    } else {
        bail!("{}", i18n::err_nothing_to_save_selection())
    }
}

fn encode_rgba_png(width: usize, height: usize, rgba: &[u8]) -> Option<Vec<u8>> {
    use std::io::Cursor;
    let mut buf = Cursor::new(Vec::new());
    let mut encoder = png::Encoder::new(&mut buf, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(rgba).ok()?;
    drop(writer);
    Some(buf.into_inner())
}

// ---------------------------------------------------------------------------
// Linux: wl-paste -> xclip -> arboard -> xsel
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use crate::i18n;

    const IMAGE_MIMES: &[(&str, &str)] = &[
        ("image/png", "png"),
        ("image/jpeg", "jpg"),
        ("image/webp", "webp"),
        ("image/gif", "gif"),
        ("image/bmp", "bmp"),
        ("image/svg+xml", "svg"),
    ];

    fn wl_mime_types(primary: bool) -> Vec<String> {
        let mut args = vec!["--list-types"];
        if primary {
            args.push("--primary");
        }
        run_text_timeout("wl-paste", &args)
            .map(|s| s.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn has_mime(types: &[String], prefix: &str) -> bool {
        types.iter().any(|t| t.starts_with(prefix))
    }

    fn image_mime(types: &[String]) -> Option<(&'static str, &'static str)> {
        IMAGE_MIMES
            .iter()
            .find(|(mime, _)| has_mime(types, mime))
            .copied()
    }

    fn read_wl(primary: bool) -> Option<Content> {
        let types = wl_mime_types(primary);
        if types.is_empty() {
            return None;
        }
        if let Some((mime, ext)) = image_mime(&types) {
            let mut args = vec!["--no-newline", "--type", mime];
            if primary {
                args.push("--primary");
            }
            if let Some(data) = run_bytes_timeout("wl-paste", &args) {
                return Some(Content::Image { data, ext });
            }
        }
        let mut args = vec!["--no-newline"];
        if primary {
            args.push("--primary");
        }
        run_text("wl-paste", &args).map(Content::Text)
    }

    fn read_xclip(primary: bool) -> Option<Content> {
        let sel = if primary { "primary" } else { "clipboard" };
        for (target, ext) in [("image/png", "png"), ("image/jpeg", "jpg")] {
            if let Some(data) = run_bytes_timeout("xclip", &["-selection", sel, "-target", target, "-o"]) {
                return Some(Content::Image { data, ext });
            }
        }
        run_text("xclip", &["-selection", sel, "-o"]).map(Content::Text)
    }

    fn read_any(primary: bool) -> Option<Content> {
        read_wl(primary)
            .or_else(|| read_xclip(primary))
            .or_else(|| {
                if primary {
                    run_text_timeout("xsel", &["--primary", "--output"]).map(Content::Text)
                } else {
                    read_arboard()
                }
            })
    }

    pub fn read() -> Result<Content> {
        if let Some(c) = read_any(true) {
            return Ok(c);
        }

        if let Some(c) = read_any(false) {
            return Ok(c);
        }

        bail!("{}", i18n::err_nothing_to_save_selection())
    }
}

// ---------------------------------------------------------------------------
// macOS: arboard (primary) -> pbpaste (fallback)
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use crate::i18n;
    use std::time::Duration;

    fn copy_selection_to_clipboard_via_cmd_c() -> bool {
        // Best-effort: ask the focused app to copy the current selection into the system clipboard.
        // Requires the user's Accessibility permissions for System Events.
        let script = r#"tell application "System Events" to keystroke "c" using {command down}"#;
        let status = std::process::Command::new("osascript")
            .args(["-e", script])
            .status();

        // Let the clipboard update propagate.
        std::thread::sleep(Duration::from_millis(200));

        status.map(|s| s.success()).unwrap_or(false)
    }

    pub fn read() -> Result<Content> {
        // Always copy the current selection into the clipboard first. Reading the clipboard
        // before this would return whatever was there last time (e.g. after a previous save),
        // so the hotkey would ignore the new selection while non-empty stale data remained.
        let cmd_c_ok = copy_selection_to_clipboard_via_cmd_c();

        if !cmd_c_ok {
            // Synthetic Cmd+C failed (e.g. Accessibility permission). User may have copied
            // manually; read the clipboard as-is.
            let arboard_initial = read_arboard();
            let pb_text_initial = run_text_timeout("pbpaste", &[]);
            return choose_clipboard_content(arboard_initial, pb_text_initial);
        }

        // Wait for clipboard propagation after Cmd+C.
        //
        // Reliability trick for text:
        // - `arboard` may still contain the *old* clipboard text when the global Cmd+C keystroke
        //   is in-flight.
        // - `pbpaste` usually updates faster for text.
        // Therefore:
        // - Prefer `pbpaste` text as soon as it becomes available.
        // - Return `arboard` images immediately (pbpaste won't help for images).
        // - Only if pbpaste never updates within the retry window, fall back to the last arboard
        //   text value.
        let mut last_arboard_text: Option<String> = None;

        for attempt in 0..8 {
            std::thread::sleep(Duration::from_millis(100));

            let arboard = read_arboard();
            let pb_text = run_text_timeout("pbpaste", &[]);

            match arboard {
                Some(Content::Image { data, ext }) => {
                    // Keep as-is; pbpaste isn't suitable for images.
                    return Ok(Content::Image { data, ext });
                }
                Some(Content::Text(t)) => {
                    // Store and keep waiting for pbpaste to update.
                    last_arboard_text = Some(t);
                }
                None => {}
            }

            if let Some(t) = pb_text {
                return Ok(Content::Text(t));
            }

            if attempt == 7 {
                if let Some(t) = last_arboard_text {
                    return Ok(Content::Text(t));
                }
                bail!("{}", i18n::err_nothing_to_save_selection());
            }
        }

        bail!("{}", i18n::err_nothing_to_save_selection())
    }
}

// ---------------------------------------------------------------------------
// Other platforms: arboard only
// ---------------------------------------------------------------------------

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod platform {
    use super::*;
    use crate::i18n;

    pub fn read() -> Result<Content> {
        if let Some(c) = read_arboard() {
            return Ok(c);
        }
        bail!("{}", i18n::err_nothing_to_save())
    }
}

/// Read content: on Linux tries primary selection first, then clipboard.
/// On macOS/other: reads clipboard via arboard.
pub fn read() -> Result<Content> {
    platform::read()
}

#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod clipboard_tests;
