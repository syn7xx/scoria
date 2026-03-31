use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::Local;

use crate::engine::clipboard::Content;
use crate::engine::config::{Config, SaveTarget};
use crate::i18n;

pub fn save(config: &Config, content: &Content) -> Result<PathBuf> {
    crate::engine::config::vault_ready(&config.vault_path)?;

    match content {
        Content::Text(text) => save_text(config, text),
        Content::Image { data, ext } => save_image(config, data, ext),
    }
}

fn save_text(config: &Config, text: &str) -> Result<PathBuf> {
    if text.is_empty() {
        bail!("{}", i18n::err_text_empty());
    }

    match config.target {
        SaveTarget::NewFileInFolder => save_new_file(config, text),
        SaveTarget::AppendToFile => append_to_file(config, text),
    }
}

fn format_body(config: &Config, text: &str) -> String {
    if !config.prepend_timestamp_header {
        return text.to_string();
    }

    let stamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    format!("## {stamp}\n\n{text}")
}

fn save_new_file(config: &Config, text: &str) -> Result<PathBuf> {
    let folder = config.vault_path.join(&config.folder);

    std::fs::create_dir_all(&folder)
        .with_context(|| format!("create folder {}", folder.display()))?;

    let name = Local::now().format(&config.filename_template).to_string();
    let path = folder.join(name);

    std::fs::write(&path, format_body(config, text))
        .with_context(|| format!("write {}", path.display()))?;

    Ok(path)
}

fn append_to_file(config: &Config, text: &str) -> Result<PathBuf> {
    let path = config.vault_path.join(&config.append_file);

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

    let now = Local::now();
    let folder = config.vault_path.join(&config.folder);
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
