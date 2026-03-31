use std::sync::OnceLock;

use anyhow::Result;
#[cfg(not(target_os = "windows"))]
use anyhow::{ensure, Context};

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

#[cfg(target_os = "windows")]
pub fn apply(_tag: &str) -> Result<()> {
    anyhow::bail!(
        "in-app update is not supported for Windows MSI installs; update via winget/MSI installer"
    );
}

#[cfg(not(target_os = "windows"))]
pub fn apply(tag: &str) -> Result<()> {
    let exe = std::env::current_exe().context("could not find exe path")?;
    let url = format!(
        "https://github.com/syn7xx/scoria/releases/download/{tag}/{}",
        asset_name()
    );

    tracing::info!(url = %url, "downloading update");

    let tmp = std::env::temp_dir().join("scoria-update");
    let _ = std::fs::create_dir_all(&tmp);
    let archive = tmp.join(asset_archive_filename());

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("create HTTP client")?;

    let response = client.get(&url).send().context("download failed")?;

    ensure!(
        response.status().is_success(),
        "HTTP error: {}",
        response.status()
    );

    let bytes = response.bytes().context("read response body")?;
    std::fs::write(&archive, &bytes).context("write update archive")?;

    apply_unix_archive(&archive, &tmp, &exe)?;

    let _ = std::fs::remove_dir_all(&tmp);

    tracing::info!(tag = %tag, "update applied successfully");
    Ok(())
}

fn fetch_latest_tag() -> Option<String> {
    let user_agent = format!("scoria/{CURRENT_VERSION}");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client
        .get(RELEASES_API)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", user_agent)
        .send()
        .ok()?;

    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "update check failed");
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

#[cfg(not(target_os = "windows"))]
fn asset_name() -> String {
    let (os, arch) = current_asset_triplet();

    let ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };

    format!("scoria-{os}-{arch}.{ext}")
}

#[cfg(not(target_os = "windows"))]
fn asset_archive_filename() -> &'static str {
    if cfg!(target_os = "windows") {
        "scoria.zip"
    } else {
        "scoria.tar.gz"
    }
}

#[cfg(not(target_os = "windows"))]
fn current_asset_triplet() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };

    (os, arch)
}

#[cfg(not(target_os = "windows"))]
fn apply_unix_archive(
    archive: &std::path::Path,
    tmp: &std::path::Path,
    exe: &std::path::Path,
) -> Result<()> {
    let extract = std::process::Command::new("tar")
        .args(["xzf"])
        .arg(archive)
        .arg("-C")
        .arg(tmp)
        .status()
        .context("tar")?;

    ensure!(extract.success(), "extraction failed");
    replace_binary_atomic(&tmp.join("scoria"), exe)?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn replace_binary_atomic(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    use anyhow::bail;

    let parent = dst
        .parent()
        .context("executable path has no parent directory")?;
    let staging = parent.join(format!(".scoria-update-{}", std::process::id()));
    std::fs::copy(src, &staging)
        .with_context(|| format!("stage update at {}", staging.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(dst)
            .map(|m| m.permissions())
            .unwrap_or_else(|_| std::fs::Permissions::from_mode(0o755));
        std::fs::set_permissions(&staging, perms)
            .with_context(|| format!("set permissions for {}", staging.display()))?;
    }

    if let Err(e) = std::fs::rename(&staging, dst) {
        let _ = std::fs::remove_file(&staging);
        bail!("replace binary at {} failed: {e}", dst.display());
    }

    Ok(())
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod update_tests;
