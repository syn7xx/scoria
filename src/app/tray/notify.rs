#[cfg(target_os = "windows")]
fn notify_windows_native(summary: &str, body: &str) -> Result<(), String> {
    use winrt_notification::{Duration, Sound, Toast};

    Toast::new(Toast::POWERSHELL_APP_ID)
        .title(summary)
        .text1(body)
        .sound(Some(Sound::Default))
        .duration(Duration::Short)
        .show()
        .map_err(|e| e.to_string())
}

pub(crate) fn notify(summary: &str, body: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("notify-send")
            .args(["-a", "Scoria", "-i", "scoria", "-t", "3000", summary, body])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display notification \"{}\" with title \"{}\"",
            body.replace('\\', "\\\\").replace('"', "\\\""),
            summary.replace('\\', "\\\\").replace('"', "\\\""),
        );
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        if let Err(native_err) = notify_windows_native(summary, body) {
            tracing::warn!(error = %native_err, "native windows notification failed");
        }
    }
}
