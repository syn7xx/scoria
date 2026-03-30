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

    eprintln!("scoria: downloading {url}...");

    let tmp = std::env::temp_dir().join("scoria-update");
    let _ = std::fs::create_dir_all(&tmp);
    let tarball = tmp.join("scoria.tar.gz");

    let dl = std::process::Command::new("curl")
        .args(["-sL", "-o"])
        .arg(&tarball)
        .arg(&url)
        .status()
        .context("curl")?;

    ensure!(dl.success(), "download failed");

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

    Ok(())
}

fn fetch_latest_tag() -> Option<String> {
    let out = std::process::Command::new("curl")
        .args([
            "-sL",
            "-H",
            "Accept: application/vnd.github+json",
            RELEASES_API,
        ])
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;

    json.get("tag_name")
        .and_then(|v| v.as_str())
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
