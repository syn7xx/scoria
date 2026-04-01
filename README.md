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

### Pre-built binary (recommended)

Linux/macOS one-command install or update:

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/install.sh | bash
```

Or download manually from the [Releases](https://github.com/syn7xx/scoria/releases) page.

#### Windows

Use the `scoria-windows-x86_64.msi` asset from [Releases](https://github.com/syn7xx/scoria/releases) for the easiest install.
The wizard asks for an installation folder and whether to add a **desktop** shortcut; a **Start menu** shortcut is always installed. There is no separate license step.

Portable option: use `scoria-windows-x86_64.zip`, extract `scoria.exe`, and add its folder to `PATH`.
`install.sh` / `uninstall.sh` are Unix-only.

Or install portable ZIP with one PowerShell command:

```powershell
irm https://github.com/syn7xx/scoria/raw/main/install.ps1 | iex
```

For maintainers: winget manifests (portable **ZIP** installer) can be generated from the ZIP’s SHA256:

```powershell
powershell -File scripts/windows/gen-winget-manifests.ps1 -Version 0.2.3 -Sha256 "<sha256 of scoria-windows-x86_64.zip>"
```

Use the version you are releasing (see `version` in `Cargo.toml`). The script strips a leading `v` from `-Version` if present.

### With cargo

```bash
cargo install --git https://github.com/syn7xx/scoria.git
```

This installs only the binary. On Linux you'll need system libraries installed first (see below).

### System dependencies

#### Linux

| Distro | Command |
|--------|---------|
| **Arch / Omarchy** | `sudo pacman -S --needed gtk3 libappindicator-gtk3 xdotool wl-clipboard` |
| **Fedora** | `sudo dnf install gtk3-devel libappindicator-gtk3-devel xdotool wl-clipboard` |
| **Ubuntu / Debian** | `sudo apt install libgtk-3-dev libappindicator3-dev libxdo-dev xdotool wl-clipboard` |

> **GNOME**: the tray icon requires the [AppIndicator](https://extensions.gnome.org/extension/615/appindicator-support/) extension (package `gnome-shell-extension-appindicator`). Without it `scoria save` still works via a keyboard shortcut.

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

On Linux (including GNOME without AppIndicator) you can open the GTK settings from a terminal:

```bash
scoria settings-gui
```

On Windows, `scoria settings-gui` opens the native settings window. Starting the tray from the Start menu or autostart does **not** open a console window.

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
| **Check for updates** | Linux/macOS: download and install the latest version; Windows: show an update notice and open the latest Releases page for manual MSI/winget update |
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
| `language` | *(empty)* | UI language: empty string = auto from `LANG` / `LC_MESSAGES` / `LC_ALL`, or `en`, `ru` |

### Interface language

Scoria ships with **English** and **Russian** UI strings (tray menu, notifications, settings dialogs).

- In **Settings…**, use **Interface language**: *Auto* (follows your OS locale), *English*, or *Русский*. Saving applies the language immediately in that window; the tray app updates menu text and tooltip shortly after (it watches `config.toml`).
- In the config file, set `language = ""` for auto, `language = "en"`, or `language = "ru"`.

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

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT OR Apache-2.0
