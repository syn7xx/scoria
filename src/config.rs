use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::i18n;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SaveTarget {
    #[default]
    NewFileInFolder,
    AppendToFile,
}

impl SaveTarget {
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::NewFileInFolder => "new_file_in_folder",
            Self::AppendToFile => "append_to_file",
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "new_file_in_folder" => Some(Self::NewFileInFolder),
            "append_to_file" => Some(Self::AppendToFile),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub vault_path: PathBuf,
    pub target: SaveTarget,
    pub folder: String,
    pub append_file: String,
    pub filename_template: String,
    pub prepend_timestamp_header: bool,
    pub hotkey: Option<String>,
    pub autostart: bool,
    /// Interface language: `""` = auto-detect, `"en"` = English, `"ru"` = Russian.
    pub language: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault_path: PathBuf::new(),
            target: SaveTarget::default(),
            folder: "scoria".into(),
            append_file: "Scoria.md".into(),
            filename_template: "clip-%Y-%m-%d-%H%M%S.md".into(),
            prepend_timestamp_header: true,
            hotkey: None,
            autostart: false,
            language: String::new(),
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().context("could not resolve config directory")?;
    Ok(dir.join("scoria").join("config.toml"))
}

fn ensure_config_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    Ok(())
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    ensure!(
        path.exists(),
        "config not found at {p}.\nRun `scoria` once to create a default, then set vault_path.",
        p = path.display()
    );

    let s = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&s).context("parse config.toml")
}

pub fn load_or_create() -> Result<Config> {
    let path = config_path()?;
    if path.exists() {
        return load();
    }

    ensure_config_dir(&path)?;
    let mut default = Config::default();
    if let Some(vault) = best_vault() {
        eprintln!("scoria: detected Obsidian vault at {}", vault.display());
        default.vault_path = vault;
    }
    let s = toml::to_string_pretty(&default).context("serialize default config")?;
    std::fs::write(&path, &s).with_context(|| format!("write {}", path.display()))?;
    eprintln!("Created default config at {}.", path.display());
    Ok(default)
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    ensure_config_dir(&path)?;
    let s = toml::to_string_pretty(config).context("serialize config")?;
    std::fs::write(&path, s).with_context(|| format!("write {}", path.display()))?;

    crate::autostart::apply(config.autostart);
    Ok(())
}

pub fn vault_ready(vault: &Path) -> Result<()> {
    if vault.as_os_str().is_empty() {
        let cfg_path = config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        bail!("{}", i18n::err_vault_path_empty(&cfg_path));
    }
    if !vault.exists() {
        bail!("{}", i18n::err_vault_not_found(&vault.display().to_string()));
    }
    if !vault.is_dir() {
        bail!("{}", i18n::err_vault_not_dir(&vault.display().to_string()));
    }
    Ok(())
}

pub fn open_in_editor() {
    if let Ok(p) = config_path() {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let _ = std::process::Command::new(opener)
            .arg(p.as_os_str())
            .spawn();
    }
}

#[derive(Debug, Clone)]
pub struct DetectedVault {
    pub path: PathBuf,
    pub open: bool,
}

/// Read the Obsidian config (`obsidian.json`) and extract known vault paths,
/// sorted by most-recently-used first.
pub fn detect_obsidian_vaults() -> Vec<DetectedVault> {
    let Some(cfg_dir) = dirs::config_dir() else {
        return vec![];
    };
    let json_path = cfg_dir.join("obsidian").join("obsidian.json");
    let Ok(data) = std::fs::read_to_string(&json_path) else {
        return vec![];
    };
    let Ok(root) = serde_json::from_str::<serde_json::Value>(&data) else {
        return vec![];
    };
    let Some(vaults) = root.get("vaults").and_then(|v| v.as_object()) else {
        return vec![];
    };

    let mut entries: Vec<(i64, DetectedVault)> = vaults
        .values()
        .filter_map(|v| {
            let path = PathBuf::from(v.get("path")?.as_str()?);
            if !path.is_dir() {
                return None;
            }
            let ts = v.get("ts").and_then(|t| t.as_i64()).unwrap_or(0);
            let open = v.get("open").and_then(|o| o.as_bool()).unwrap_or(false);
            Some((ts, DetectedVault { path, open }))
        })
        .collect();

    entries.sort_by(|a, b| b.0.cmp(&a.0));
    entries.into_iter().map(|(_, v)| v).collect()
}

/// Pick the best vault: prefer the one marked `open`, otherwise most recent.
pub fn best_vault() -> Option<PathBuf> {
    let vaults = detect_obsidian_vaults();
    vaults
        .iter()
        .find(|v| v.open)
        .or(vaults.first())
        .map(|v| v.path.clone())
}
