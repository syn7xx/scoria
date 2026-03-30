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
}
