# Scoria

[Русский](README_RU.md)

Save clipboard content — text **and images** — into an [Obsidian](https://obsidian.md/) vault.
Works as a **system tray** app on Linux, macOS, and Windows, and as a **CLI** tool.

Scoria is smart about what to save: on Linux it first checks for **selected text** (primary selection), then falls back to the **clipboard**. On macOS it reads the system clipboard. Linux: select or copy, then save. macOS: copy first (`Cmd+C`), then save.

## Installation

### From source (all platforms)

```bash
git clone https://github.com/syn7xx/scoria.git
cd scoria
make deps      # installs system libraries (auto-detects pacman / dnf / apt / brew)
make install   # builds and installs to ~/.local/bin (+ icons & .desktop entry on Linux)
```

Make sure `~/.local/bin` is in your `PATH`.

On Linux, `make install` also installs the app icon under `~/.local/share/icons/hicolor/...` and `assets/scoria.desktop` into `~/.local/share/applications/` so launchers (KDE, GNOME, etc.) can show Scoria with an icon.

### Pre-built binary (recommended)

Linux/macOS one-command install or update:

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/install.sh | bash
```

The Linux path of `install.sh` downloads the release binary, writes `~/.local/share/applications/scoria.desktop`, and installs icons into `~/.local/share/icons/hicolor/` (PNG and/or SVG). The `.desktop` file uses an **absolute `Icon=` path** when possible so KDE Application Launcher and similar pick up the icon even if the themed icon name `scoria` is not installed system-wide.

Or download manually from the [Releases](https://github.com/syn7xx/scoria/releases) page.

#### Windows

Use the `scoria-windows-x86_64.msi` asset from [Releases](https://github.com/syn7xx/scoria/releases) for the easiest install.
The wizard asks for an installation folder, then a page with a **desktop shortcut** checkbox, then the summary. After install, the finish screen can offer **Launch Scoria now**. **Start menu** and optional **desktop** shortcuts use the standard per-user shell folders (so they appear correctly even if you install to a non-system drive). There is no separate license step.

Portable option: use `scoria-windows-x86_64.zip`, extract `scoria.exe`, and add its folder to `PATH`.
`install.sh` / `uninstall.sh` are Unix-only.

Or install portable ZIP with one PowerShell command:

```powershell
irm https://github.com/syn7xx/scoria/raw/main/install.ps1 | iex
```

For maintainers: winget manifests (portable **ZIP** installer) can be generated from the ZIP’s SHA256:

```powershell
powershell -File scripts/windows/gen-winget-manifests.ps1 -Version 0.2.5 -Sha256 "<sha256 of scoria-windows-x86_64.zip>"
```

Use the version you are releasing (see `version` in `Cargo.toml`). The script strips a leading `v` from `-Version` if present.

### With cargo

```bash
cargo install --git https://github.com/syn7xx/scoria.git
```

This installs only the binary. On Linux you'll need system libraries installed first (see below).

### System dependencies

#### Linux

Build-time and runtime libraries (GTK is used for **Settings**; the tray uses **StatusNotifierItem** via D-Bus, not `libappindicator`):

| Distro | Command |
|--------|---------|
| **Arch / Omarchy** | `sudo pacman -S --needed base-devel rust gtk3 xdotool wl-clipboard` |
| **Fedora** | `sudo dnf install gcc make rust cargo gtk3-devel xdotool wl-clipboard` |
| **Ubuntu / Debian** | `sudo apt install build-essential rustc cargo libgtk-3-dev libxdo-dev xdotool wl-clipboard` |

> **GNOME**: for the tray entry to appear in the top bar, the [AppIndicator](https://extensions.gnome.org/extension/615/appindicator-support/) extension (`gnome-shell-extension-appindicator`) is usually required. Without it, `scoria save` and `scoria settings-gui` still work; only tray integration may be missing.

> **Linux tray icon**: The tray uses the StatusNotifierItem protocol with an embedded **pixmap** icon so KDE, Hyprland, and other SNI hosts show the app icon reliably (including cases where a themed icon name would not resolve).

#### macOS

No extra dependencies — everything uses native APIs.

## Uninstall

Removes the binary under `~/.local/bin`, Linux menu icons / `.desktop` entries, config (`~/.config/scoria` on Linux, `~/Library/Application Support/scoria` on macOS), and autostart entries. Stops a running `scoria` if possible.

From the repository clone:

```bash
./uninstall.sh
```

Linux/macOS one command (same as `install.sh` style):

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/uninstall.sh | bash
```

On Windows (portable ZIP install):

```powershell
irm https://github.com/syn7xx/scoria/raw/main/uninstall.ps1 | iex
```

On Windows (MSI install): uninstall from **Apps & Features** (recommended), or run `msiexec /x` with the **full path** to the MSI you installed (or the cached copy), e.g.:

```powershell
msiexec /x "$env:USERPROFILE\Downloads\scoria-windows-x86_64.msi"
```

If you installed with `make install`, you can also run `make uninstall` (binary + icons + launcher only; use the script above to drop config and autostart).

## Quick start

```bash
scoria        # starts the tray app (default)
scoria save   # one-shot: save selection / clipboard to Obsidian
```

On macOS you can also open the same settings window from a terminal (starts a short-lived helper process):

```bash
scoria settings-gui
```

On Linux you can open the GTK settings from a terminal (works even when the tray is not visible):

```bash
scoria settings-gui
```

On Windows, `scoria settings-gui` opens the native settings window; layout and font scale with **display DPI** (e.g. 150%). Starting the tray from the Start menu or autostart does **not** open a console window.

To edit the raw TOML instead, use **Open config file…** in the tray menu.

On first launch Scoria will:
- create the config file (`~/.config/scoria/config.toml` on Linux, `~/Library/Application Support/scoria/config.toml` on macOS, `%APPDATA%\\scoria\\config.toml` on Windows)
- auto-detect your Obsidian vault
- pick the **interface language** from your system locale (override anytime in **Settings…**)
- show a tray icon (when using `scoria` / `scoria run`)

## Usage

### Tray menu

Labels follow your [interface language](#interface-language) setting (defaults to the system locale). Typical entries:

| Item | Action |
|------|--------|
| **Save to Obsidian** | Save selected text or clipboard content |
| **Settings…** | GTK settings dialog (Linux) / native AppKit window (macOS) / native Win32 window (Windows) |
| **Open config file…** | Open config in your default editor |
| **Check for updates** / **Checking for updates…** | While a check runs, the menu shows a disabled “checking” label. Linux/macOS: download and install the latest version when an update exists; Windows: notification and open the latest Releases page for MSI/winget |
| **Quit** | Exit |

### What gets saved

| Content | Result in vault |
|---------|-----------------|
| **Text** | Markdown file in `folder/` (or appended to `append_file`) |
| **Image** (PNG, JPEG, WebP, GIF, BMP, SVG) | Image in `folder/attachments/`, plus a `.md` note with `![[...]]` embed |

### Keyboard shortcuts

Bind `scoria save` to a hotkey in your desktop environment:

**Hyprland / Sway (Wayland)**

```ini
bind = $mainMod, V, exec, scoria save
```

**GNOME**

Settings → Keyboard → Custom Shortcuts:
- Name: `Scoria save`
- Command: `scoria save`
- Shortcut: your choice (e.g. `Super+V`)

**macOS**

Use [Hammerspoon](https://www.hammerspoon.org/), [skhd](https://github.com/koekeishiya/skhd), or System Settings → Keyboard → Shortcuts:

```lua
-- Hammerspoon example
hs.hotkey.bind({"cmd", "shift"}, "V", function()
  os.execute("scoria save")
end)
```

**Windows**

Use PowerToys Keyboard Manager, AutoHotkey, or your preferred launcher to bind:
- Command: `scoria save`

**X11 (built-in)**

Set `hotkey` in config (e.g. `hotkey = "Ctrl+Shift+S"`) — Scoria registers it while the tray runs. Modifiers: `ctrl`, `alt`, `shift`, `super`; keys: `a`-`z`, `0`-`9`, `F1`-`F12`, `Space`, etc.

## Configuration

Edit via **Settings…** in the tray menu, or directly in the config file.

| Field | Default | Meaning |
|-------|---------|---------|
| `vault_path` | *(auto-detected)* | Absolute path to vault root |
| `target` | `new_file_in_folder` | `new_file_in_folder` or `append_to_file` |
| `folder` | `scoria` | Subfolder for new files |
| `append_file` | `Scoria.md` | Vault-relative path when appending |
| `filename_template` | `clip-%Y-%m-%d-%H%M%S.md` | strftime template for new files |
| `prepend_timestamp_header` | `true` | Add `## timestamp` header |
| `hotkey` | *(none)* | Global shortcut (X11 and Windows tray mode) |
| `autostart` | `false` | Start Scoria on login |
| `auto_update` | `false` | Automatically check for updates on tray startup |
| `language` | *(empty)* | UI language: empty = auto-detect **English** or **Russian** from the OS (`LANG` / `LC_*`, and on Windows the UI locale); or set `en` / `ru` explicitly |

### Interface language

Scoria ships with **English** and **Russian** UI strings (tray menu, notifications, settings dialogs).

- In **Settings…**, choose **English** or **Русский**. If `language` in the config is empty, the settings dialog opens with the locale Scoria inferred from the system.
- In the config file, `language = ""` enables automatic detection; `language = "en"` or `language = "ru"` pins the UI language.

### Autostart

Enable **"Start Scoria on login"** in Settings (or set `autostart = true` in the config file). Scoria will:

- **Linux**: create a `.desktop` entry in `~/.config/autostart/`
- **macOS**: create a LaunchAgent in `~/Library/LaunchAgents/`
- **Windows**: add/remove `Scoria` in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

Disabling the checkbox removes the autostart entry.

## Updating

Scoria can check for updates automatically on tray startup when `auto_update = true` (disabled by default). You can always run a manual check from the tray menu.

Linux/macOS: click **"Check for updates"** in the tray menu — Scoria will download and replace the binary automatically. Restart to apply.

Windows: update via MSI/winget (in-app binary replacement is disabled for MSI-safe behavior). The tray command **"Check for updates"** shows a notification and opens the latest [Releases](https://github.com/syn7xx/scoria/releases/latest) page.

Or re-run the install script:

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/install.sh | bash
```

## Development

```bash
make build    # cargo build --release
make check    # cargo clippy
make fmt      # cargo fmt
make clean    # cargo clean
```

CI runs `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked`.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT OR Apache-2.0
