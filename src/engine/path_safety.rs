use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

fn has_windows_drive_prefix(raw: &str) -> bool {
    let b = raw.as_bytes();
    b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic()
}

fn has_unc_or_device_prefix(raw: &str) -> bool {
    raw.starts_with("\\\\") || raw.starts_with("//")
}

/// Validate a user-provided relative path fragment (folder/append path).
/// Rejects traversal and absolute path forms across platforms.
pub fn validate_relative_fragment(raw: &str, field_name: &str) -> std::result::Result<(), String> {
    if raw.trim().is_empty() {
        return Err(format!("{field_name} cannot be empty"));
    }

    if has_windows_drive_prefix(raw) || has_unc_or_device_prefix(raw) {
        return Err(format!("{field_name} must be relative to vault"));
    }

    let normalized = raw.replace('\\', "/");
    let path = Path::new(&normalized);

    if path.is_absolute() {
        return Err(format!("{field_name} must be relative to vault"));
    }

    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(format!("{field_name} contains path traversal (..)"));
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(format!("{field_name} must be relative to vault"));
            }
            _ => {}
        }
    }

    Ok(())
}

fn closest_existing_ancestor(path: &Path) -> Option<&Path> {
    let mut cur = Some(path);
    while let Some(p) = cur {
        if p.exists() {
            return Some(p);
        }
        cur = p.parent();
    }
    None
}

/// Join `raw` relative path with `base` and ensure resulting path cannot escape `base`
/// (including via symlinked existing ancestors).
pub fn resolve_within_base(base: &Path, raw: &str, field_name: &str) -> Result<PathBuf> {
    validate_relative_fragment(raw, field_name)
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| format!("invalid {field_name}"))?;

    let base_canonical =
        std::fs::canonicalize(base).with_context(|| format!("canonicalize {}", base.display()))?;
    let joined = base.join(raw);

    if joined.exists() {
        let canonical = std::fs::canonicalize(&joined)
            .with_context(|| format!("canonicalize {}", joined.display()))?;
        if !canonical.starts_with(&base_canonical) {
            bail!("{field_name} escapes vault directory");
        }
        return Ok(canonical);
    }

    if let Some(existing) = closest_existing_ancestor(&joined) {
        let existing_canonical = std::fs::canonicalize(existing)
            .with_context(|| format!("canonicalize ancestor {}", existing.display()))?;
        if !existing_canonical.starts_with(&base_canonical) {
            bail!("{field_name} escapes vault directory");
        }
    }

    Ok(joined)
}
