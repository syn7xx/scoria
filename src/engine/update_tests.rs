use super::*;

#[test]
fn test_current_version() {
    let ver = current_version();
    assert!(!ver.is_empty());
    // Version should be in format like "0.1.0"
    assert!(ver.contains('.'));
}

#[test]
fn test_version_newer() {
    // 1.0.0 vs 0.9.0
    assert!(version_newer("1.0.0", "0.9.0"));

    // 0.10.0 vs 0.9.0
    assert!(version_newer("0.10.0", "0.9.0"));

    // 0.9.0 vs 0.9.0 (equal, not newer)
    assert!(!version_newer("0.9.0", "0.9.0"));

    // 0.8.0 vs 0.9.0 (older)
    assert!(!version_newer("0.8.0", "0.9.0"));
}

#[test]
fn test_version_newer_with_v_prefix() {
    // `version_newer` expects normalized versions; callers should strip `v` first.
    assert!(version_newer(strip_v("v1.0.0"), "0.9.0"));
    assert!(version_newer(strip_v("v0.10.0"), strip_v("v0.9.0")));
}

#[test]
fn test_version_newer_patch_version() {
    // 1.0.1 vs 1.0.0
    assert!(version_newer("1.0.1", "1.0.0"));

    // 1.0.0 vs 1.0.1
    assert!(!version_newer("1.0.0", "1.0.1"));
}

#[test]
fn test_version_newer_complex() {
    // 2.0.0 vs 1.99.99
    assert!(version_newer("2.0.0", "1.99.99"));

    // 1.0.0 vs 0.99.99
    assert!(version_newer("1.0.0", "0.99.99"));
}

#[test]
fn test_parse_version() {
    assert_eq!(parse_version("1.2.3"), vec![1, 2, 3]);
    assert_eq!(parse_version("0.0.0"), vec![0, 0, 0]);
    assert_eq!(parse_version("10.20.30"), vec![10, 20, 30]);
}

#[test]
fn test_parse_version_unusual() {
    // Extra dots - all parts are parsed
    let parts = parse_version("1.2.3.4.5");
    assert_eq!(parts, vec![1, 2, 3, 4, 5]);

    // Non-numeric parts are ignored
    let parts = parse_version("1.2.beta");
    assert_eq!(parts, vec![1, 2]);

    // Empty string
    let parts = parse_version("");
    assert!(parts.is_empty());
}

#[test]
fn test_strip_v() {
    assert_eq!(strip_v("v1.0.0"), "1.0.0");
    assert_eq!(strip_v("v0.1.3"), "0.1.3");
    assert_eq!(strip_v("1.0.0"), "1.0.0"); // no v prefix
                                           // "v".strip_prefix('v') returns Some(""), so result is empty string
    assert_eq!(strip_v("v"), "");
    assert_eq!(strip_v(""), "");
}

#[test]
fn test_asset_name_linux() {
    #[cfg(target_os = "linux")]
    {
        let name = asset_name();
        assert!(name.starts_with("scoria-linux-"));
        assert!(name.ends_with(".tar.gz"));
        assert!(name.contains("x86_64") || name.contains("aarch64"));
    }
}

#[test]
fn test_asset_name_macos() {
    #[cfg(target_os = "macos")]
    {
        let name = asset_name();
        assert!(name.starts_with("scoria-macos-"));
        assert!(name.ends_with(".tar.gz"));
        assert!(name.contains("x86_64") || name.contains("aarch64"));
    }
}

#[test]
fn test_asset_name_windows() {
    #[cfg(target_os = "windows")]
    {
        let name = asset_name();
        assert!(name.starts_with("scoria-windows-"));
        assert!(name.ends_with(".zip"));
        assert!(name.contains("x86_64") || name.contains("aarch64"));
    }
}

#[test]
fn test_asset_name_other_platform() {
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let name = asset_name();
        assert!(name.starts_with("scoria-linux-"));
        assert!(name.ends_with(".tar.gz"));
    }
}

#[test]
fn test_asset_archive_filename() {
    let archive = asset_archive_filename();
    #[cfg(target_os = "windows")]
    assert_eq!(archive, "scoria.zip");
    #[cfg(not(target_os = "windows"))]
    assert_eq!(archive, "scoria.tar.gz");
}

#[test]
fn test_check_result_enum() {
    // Just verify the enum variants exist and can be constructed
    let _ = CheckResult::UpToDate;
    let _ = CheckResult::Unreachable;
    let _ = CheckResult::UpdateAvailable("v1.0.0".to_string());
}

#[cfg(target_os = "windows")]
#[test]
fn test_apply_windows_returns_manual_update_error() {
    let err = apply("v1.2.3").unwrap_err().to_string();
    assert!(err.contains("not supported for Windows MSI installs"));
}

#[test]
fn test_cached_tag_initially_none() {
    // Initially no cached tag
    assert!(cached_tag().is_none());
}

#[test]
fn test_version_comparison_edge_cases() {
    // Major version difference
    assert!(version_newer("10.0.0", "9.0.0"));
    assert!(version_newer("2.0.0", "1.0.0"));

    // Alpha/beta suffixes (parsed as 0)
    assert!(version_newer("1.0.0", "1.0.0alpha"));

    // Very long version numbers
    assert!(version_newer("1.2.3.4.5.6.7.8.9", "1.2.3"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_replace_binary_atomic_replaces_contents() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let src = tmp.path().join("new-bin");
    let dst = tmp.path().join("scoria");

    std::fs::write(&src, b"new").expect("write source binary");
    std::fs::write(&dst, b"old").expect("write destination binary");

    replace_binary_atomic(&src, &dst).expect("replace binary atomically");
    assert_eq!(std::fs::read(&dst).expect("read replaced binary"), b"new");
}
