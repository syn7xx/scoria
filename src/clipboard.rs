use anyhow::{Result, bail};

pub enum Content {
    Text(String),
    Image { data: Vec<u8>, ext: &'static str },
}

fn run_bytes(cmd: &str, args: &[&str]) -> Option<Vec<u8>> {
    let out = std::process::Command::new(cmd).args(args).output().ok()?;
    if out.status.success() && !out.stdout.is_empty() {
        Some(out.stdout)
    } else {
        None
    }
}

fn run_text(cmd: &str, args: &[&str]) -> Option<String> {
    let bytes = run_bytes(cmd, args)?;
    let s = String::from_utf8_lossy(&bytes).into_owned();
    if s.is_empty() { None } else { Some(s) }
}

fn read_arboard() -> Option<Content> {
    let mut cb = arboard::Clipboard::new().ok()?;
    if let Ok(img) = cb.get_image()
        && let Some(data) = encode_rgba_png(img.width, img.height, &img.bytes)
    {
        return Some(Content::Image { data, ext: "png" });
    }
    if let Ok(text) = cb.get_text()
        && !text.is_empty()
    {
        return Some(Content::Text(text));
    }
    None
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
        run_text("wl-paste", &args)
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
            if let Some(data) = run_bytes("wl-paste", &args) {
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
            if let Some(data) = run_bytes("xclip", &["-selection", sel, "-target", target, "-o"]) {
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
                    run_text("xsel", &["--primary", "--output"]).map(Content::Text)
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

    pub fn read() -> Result<Content> {
        if let Some(c) = read_arboard() {
            return Ok(c);
        }

        if let Some(t) = run_text("pbpaste", &[]) {
            return Ok(Content::Text(t));
        }

        bail!("{}", i18n::err_nothing_to_save())
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
