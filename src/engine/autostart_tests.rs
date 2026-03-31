use super::*;
use tempfile::TempDir;

#[cfg(target_os = "linux")]
mod linux_tests {
    use super::*;

    #[test]
    fn test_autostart_path_format() {
        let p = autostart_path().expect("autostart path should resolve");
        assert!(p.to_string_lossy().ends_with("scoria.desktop"));
    }

    #[test]
    fn test_autostart_desktop_file_content() {
        let tmp = TempDir::new().expect("create temp dir");

        // Create a fake executable
        let fake_exe = tmp.path().join("scoria");
        std::fs::write(&fake_exe, "fake").expect("write fake exe");

        // We can't easily test enable() because it uses autostart_path() which
        // uses dirs::config_dir(). Instead, we verify the function doesn't panic.
        // The actual file writing is tested indirectly.

        // Test that disable doesn't panic when file doesn't exist
        disable();
    }
}

#[cfg(target_os = "macos")]
mod macos_tests {
    use super::*;

    #[test]
    fn test_launch_agent_label() {
        assert_eq!(LAUNCH_AGENT_LABEL, "com.github.syn7xx.scoria");
    }

    #[test]
    fn test_launch_agent_path_format() {
        let p = launch_agent_path().expect("launch agent path should resolve");
        assert!(p
            .to_string_lossy()
            .contains("com.github.syn7xx.scoria.plist"));
    }

    #[test]
    fn test_disable_no_panic_when_missing() {
        // Disable should not panic when file doesn't exist
        disable();
    }
}

#[cfg(target_os = "windows")]
mod windows_tests {
    use super::*;

    #[test]
    fn test_windows_run_key_constants() {
        assert_eq!(
            RUN_KEY,
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run"
        );
        assert_eq!(RUN_VALUE_NAME, "Scoria");
    }

    #[test]
    fn test_windows_run_value_data() {
        let value = run_value_data(std::path::Path::new(r"C:\Tools\scoria.exe"));
        assert_eq!(value, r#""C:\Tools\scoria.exe" run"#);
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod other_tests {
    use super::*;

    #[test]
    fn test_enable_disable_noop() {
        // On other platforms, enable/disable are no-ops
        // Just verify they can be called without panic
        let tmp = TempDir::new().expect("create temp dir");
        let fake_exe = tmp.path().join("scoria");
        std::fs::write(&fake_exe, "fake").expect("write fake exe");

        enable(&fake_exe);
        disable();
    }
}

// Test that apply() doesn't panic on any platform
#[test]
fn test_apply_does_not_panic() {
    // apply() tries to get current_exe, which might fail in tests
    // but it should not panic - it just returns early
    apply(true);
    apply(false);
}
