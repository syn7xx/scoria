use anyhow::{bail, Result};

#[cfg(any(target_os = "macos", test))]
use crate::i18n;

pub enum Content {
    Text(String),
    Image { data: Vec<u8>, ext: &'static str },
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn run_bytes_timeout(cmd: &str, args: &[&str]) -> Option<Vec<u8>> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .filter(|o| !o.stdout.is_empty())
        .map(|o| o.stdout)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn run_text_timeout(cmd: &str, args: &[&str]) -> Option<String> {
    run_bytes_timeout(cmd, args).and_then(|bytes| {
        let s = String::from_utf8_lossy(&bytes).into_owned();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    })
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
            // Prefer arboard text. Some apps expose clipboard representations that
            // pbpaste can decode differently, which may degrade Cyrillic glyphs.
            Content::Text(_) => Ok(c),
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
        run_text_timeout("wl-paste", &args).map(Content::Text)
    }

    fn read_xclip(primary: bool) -> Option<Content> {
        let sel = if primary { "primary" } else { "clipboard" };
        for (target, ext) in [("image/png", "png"), ("image/jpeg", "jpg")] {
            if let Some(data) =
                run_bytes_timeout("xclip", &["-selection", sel, "-target", target, "-o"])
            {
                return Some(Content::Image { data, ext });
            }
        }
        run_text_timeout("xclip", &["-selection", sel, "-o"]).map(Content::Text)
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
// macOS: read from clipboard
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    pub fn read() -> Result<Content> {
        // Read from clipboard. User must manually copy selection (Cmd+C) before saving.
        let arboard_content = read_arboard();
        let pb_text = run_text_timeout("pbpaste", &[]);
        choose_clipboard_content(arboard_content, pb_text)
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
/// On macOS reads clipboard (`arboard` + `pbpaste` for text), on other platforms via `arboard`.
pub fn read() -> Result<Content> {
    tracing::debug!("reading clipboard");
    let result = platform::read();
    match &result {
        Ok(Content::Text(t)) => {
            tracing::debug!(
                content_type = "text",
                len = t.len(),
                "clipboard read success"
            );
        }
        Ok(Content::Image { data, ext }) => {
            tracing::debug!(
                content_type = "image",
                ext = ext,
                size = data.len(),
                "clipboard read success"
            );
        }
        Err(e) => {
            tracing::debug!(error = %e, "clipboard read failed");
        }
    }
    result
}

#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod clipboard_tests;
