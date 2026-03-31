use std::sync::OnceLock;

use anyhow::{Context, Result, ensure};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_API: &str = "https://api.github.com/repos/syn7xx/scoria/releases/latest";

static LATEST_TAG: OnceLock<String> = OnceLock::new();

pub enum CheckResult {
    UpdateAvailable(String),
    UpToDate,
    Unreachable,
}

pub fn check() -> CheckResult {
    tracing::debug!("checking for updates");
    match fetch_latest_tag() {
        Some(tag) if version_newer(strip_v(&tag), CURRENT_VERSION) => {
            let _ = LATEST_TAG.set(tag.clone());
            CheckResult::UpdateAvailable(tag)
        }
        Some(_) => CheckResult::UpToDate,
        None => CheckResult::Unreachable,
    }
}

pub fn cached_tag() -> Option<&'static String> {
    LATEST_TAG.get()
}

pub fn current_version() -> &'static str {
    CURRENT_VERSION
}

pub fn apply(tag: &str) -> Result<()> {
    let exe = std::env::current_exe().context("could not find exe path")?;
    let url = format!(
        "https://github.com/syn7xx/scoria/releases/download/{tag}/{}",
        asset_name()
    );

    tracing::info!(url = %url, "downloading update");

    let tmp = std::env::temp_dir().join("scoria-update");
    let _ = std::fs::create_dir_all(&tmp);
    let tarball = tmp.join("scoria.tar.gz");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("create HTTP client")?;

    let response = client
        .get(&url)
        .send()
        .context("download failed")?;

    ensure!(response.status().is_success(), "HTTP error: {}", response.status());

    let bytes = response.bytes().context("read response body")?;
    std::fs::write(&tarball, &bytes).context("write tarball")?;

    let extract = std::process::Command::new("tar")
        .args(["xzf"])
        .arg(&tarball)
        .arg("-C")
        .arg(&tmp)
        .status()
        .context("tar")?;

    ensure!(extract.success(), "extraction failed");

    std::fs::copy(tmp.join("scoria"), &exe).context("replace binary")?;

    let _ = std::fs::remove_dir_all(&tmp);

    tracing::info!(tag = %tag, "update applied successfully");
    Ok(())
}

fn fetch_latest_tag() -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client
        .get(RELEASES_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let json: serde_json::Value = match response.json() {
        Ok(v) => v,
        Err(_) => return None,
    };

    json.get("tag_name")
        .and_then(|v: &serde_json::Value| v.as_str())
        .map(String::from)
}

fn version_newer(latest: &str, current: &str) -> bool {
    parse_version(latest) > parse_version(current)
}

fn parse_version(s: &str) -> Vec<u32> {
    s.split('.').filter_map(|part| part.parse().ok()).collect()
}

fn strip_v(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

fn asset_name() -> String {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };

    format!("scoria-{os}-{arch}.tar.gz")
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod update_tests;
