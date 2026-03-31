use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use chrono::Local;

use crate::engine::clipboard::Content;
use crate::engine::config::{Config, SaveTarget};
use crate::engine::path_safety;
use crate::i18n;

pub fn save(config: &Config, content: &Content) -> Result<PathBuf> {
    crate::engine::config::vault_ready(&config.vault_path)?;

    match content {
        Content::Text(text) => {
            let len = text.len();
            let result = save_text(config, text)?;
            tracing::debug!(target = ?config.target, text_len = len, "text saved");
            Ok(result)
        }
        Content::Image { data, ext } => {
            let len = data.len();
            let result = save_image(config, data, ext)?;
            tracing::debug!(ext = ext, size_bytes = len, "image saved");
            Ok(result)
        }
    }
}

fn save_text(config: &Config, text: &str) -> Result<PathBuf> {
    if text.is_empty() {
        tracing::warn!("attempted to save empty text");
        bail!("{}", i18n::err_text_empty());
    }

    let result = match config.target {
        SaveTarget::NewFileInFolder => save_new_file(config, text),
        SaveTarget::AppendToFile => append_to_file(config, text),
    };

    if let Ok(ref path) = result {
        tracing::debug!(target = ?config.target, path = %path.display(), "text save complete");
    }
    result
}

fn format_body(config: &Config, text: &str) -> String {
    if !config.prepend_timestamp_header {
        return text.to_string();
    }

    let stamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    format!("## {stamp}\n\n{text}")
}

fn save_new_file(config: &Config, text: &str) -> Result<PathBuf> {
    let folder = path_safety::resolve_within_base(&config.vault_path, &config.folder, "folder")?;

    std::fs::create_dir_all(&folder)
        .with_context(|| format!("create folder {}", folder.display()))?;

    let name = Local::now().format(&config.filename_template).to_string();
    // Security: validate filename doesn't contain path traversal
    if name.contains("..") || name.starts_with('/') {
        anyhow::bail!("filename contains invalid characters");
    }
    let path = folder.join(name);

    std::fs::write(&path, format_body(config, text))
        .with_context(|| format!("write {}", path.display()))?;

    Ok(path)
}

fn append_to_file(config: &Config, text: &str) -> Result<PathBuf> {
    let path =
        path_safety::resolve_within_base(&config.vault_path, &config.append_file, "append_file")?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open {}", path.display()))?;

    let needs_separator = file.metadata().map(|m| m.len() > 0).unwrap_or(false);

    if needs_separator {
        writeln!(file)?;
        writeln!(file)?;
    }

    write!(file, "{}", format_body(config, text))
        .with_context(|| format!("append {}", path.display()))?;

    Ok(path)
}

fn save_image(config: &Config, data: &[u8], ext: &str) -> Result<PathBuf> {
    if data.is_empty() {
        bail!("{}", i18n::err_image_empty());
    }

    // Security: validate extension
    if ext.contains("..") || ext.contains('/') || ext.contains('\\') {
        bail!("image extension contains invalid characters");
    }

    let now = Local::now();
    let folder = path_safety::resolve_within_base(&config.vault_path, &config.folder, "folder")?;
    let attachments = folder.join("attachments");

    std::fs::create_dir_all(&attachments)
        .with_context(|| format!("create {}", attachments.display()))?;

    let stamp = now.format("%Y-%m-%d-%H%M%S").to_string();
    let img_name = format!("clip-{stamp}.{ext}");
    let img_path = attachments.join(&img_name);

    std::fs::write(&img_path, data).with_context(|| format!("write {}", img_path.display()))?;

    let md_name = format!("clip-{stamp}.md");

    let md_path = folder.join(&md_name);
    let embed = format!("![[attachments/{img_name}]]");
    let body = if config.prepend_timestamp_header {
        format!("## {}\n\n{embed}\n", now.format("%Y-%m-%d %H:%M:%S"))
    } else {
        format!("{embed}\n")
    };

    std::fs::write(&md_path, body).with_context(|| format!("write {}", md_path.display()))?;

    Ok(md_path)
}

#[cfg(test)]
#[path = "vault_tests.rs"]
mod vault_tests;
