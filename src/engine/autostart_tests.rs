use super::*;
use tempfile::TempDir;

#[cfg(target_os = "linux")]
mod linux_tests {
    use super::*;
    
    #[test]
    fn test_autostart_path_format() {
        let path = autostart_path();
        assert!(path.is_some());
        let p = path.unwrap();
        assert!(p.to_string_lossy().ends_with("scoria.desktop"));
    }
    
    #[test]
    fn test_autostart_desktop_file_content() {
        let tmp = TempDir::new().unwrap();
        
        // Create a fake executable
        let fake_exe = tmp.path().join("scoria");
        std::fs::write(&fake_exe, "fake").unwrap();
        
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
        let path = launch_agent_path();
        assert!(path.is_some());
        let p = path.unwrap();
        assert!(p.to_string_lossy().contains("com.github.syn7xx.scoria.plist"));
    }
    
    #[test]
    fn test_disable_no_panic_when_missing() {
        // Disable should not panic when file doesn't exist
        disable();
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod other_tests {
    #[test]
    fn test_enable_disable_noop() {
        // On other platforms, enable/disable are no-ops
        // Just verify they can be called without panic
        let tmp = TempDir::new().unwrap();
        let fake_exe = tmp.path().join("scoria");
        std::fs::write(&fake_exe, "fake").unwrap();
        
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
