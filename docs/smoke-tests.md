# Smoke Tests by Platform

This checklist validates that a released binary is minimally healthy on each supported platform.

## 1) Automated CLI smoke

These scripts verify basic CLI behavior:

- version/help output works
- subcommand help works
- `save` fails gracefully when clipboard has no content

Scripts:

- Linux: `scripts/smoke/linux.sh`
- macOS: `scripts/smoke/macos.sh`
- Windows: `scripts/smoke/windows.ps1`

Examples:

```bash
scripts/smoke/linux.sh ./target/x86_64-unknown-linux-gnu/release/scoria
scripts/smoke/macos.sh ./target/x86_64-apple-darwin/release/scoria
```

```powershell
powershell -File scripts/smoke/windows.ps1 -Bin .\target\x86_64-pc-windows-msvc\release\scoria.exe
```

## 2) Manual tray/UI smoke (per platform)

### Linux

1. Start tray: `scoria run`
2. Verify tray icon visible and menu opens.
3. Open settings from tray and save config changes.
4. Trigger **Check for updates** and confirm user-facing notification.
5. Toggle autostart and verify `~/.config/autostart/scoria.desktop` created/removed.

### macOS

1. Start tray: `scoria run` (Accessory app, no Dock icon expected).
2. Verify tray menu actions work (`Save`, `Settings`, `Open config file`).
3. Open settings window (`scoria settings-gui`) and save.
4. Trigger **Check for updates** and verify notification path.
5. Toggle autostart and verify LaunchAgent is created/removed.

### Windows

1. Start tray: `scoria run`.
2. Verify native settings window opens from tray and from `scoria settings-gui`.
3. Trigger **Check for updates** and verify:
   - notification explains manual update path
   - Releases page opens.
4. Toggle autostart and verify Run key add/remove under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
5. Validate MSI install and portable ZIP install both run correctly.

## 3) Release artifact smoke

For every release tag:

1. Download each artifact (`.tar.gz`, `.zip`, `.msi`) and corresponding checksum file.
2. Verify checksums match.
3. Execute automated CLI smoke scripts against extracted binaries.
4. Execute manual tray/UI smoke checklist on real hosts/VMs.
