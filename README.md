# Scoria

[Русский](README_RU.md)

Save clipboard content — text **and images** — into an [Obsidian](https://obsidian.md/) vault.
Works as a **system tray** app on Linux and macOS and as a **CLI** tool.

Scoria is smart about what to save: on Linux it first checks for **selected text** (primary selection), then falls back to the **clipboard**. On macOS it reads the system clipboard. Just select or copy, then save — one action.

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

One command to install or update:

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/install.sh | bash
```

Or download manually from the [Releases](https://github.com/syn7xx/scoria/releases) page.

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

Or one command (same as `install.sh` style):

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/uninstall.sh | bash
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

To edit the raw TOML instead, use **Open config file…** in the tray menu or open  
`~/Library/Application Support/scoria/config.toml` in any editor (e.g. `open -e` for TextEdit).

On first launch Scoria will:
- create the config file (`~/.config/scoria/config.toml` on Linux, `~/Library/Application Support/scoria/config.toml` on macOS)
- auto-detect your Obsidian vault
- pick the **interface language** from your system locale (override anytime in **Settings…**)
- show a tray icon (when using `scoria` / `scoria run`)

## Usage

### Tray menu

Labels follow your [interface language](#interface-language) setting (defaults to the system locale). Typical entries:

| Item | Action |
|------|--------|
| **Save to Obsidian** | Save selected text or clipboard content |
| **Settings…** | GTK settings dialog (Linux) / native AppKit window (macOS) |
| **Open config file…** | Open config in your default editor |
| **Check for updates** | Download and install the latest version |
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
| `hotkey` | *(none)* | Global shortcut (X11 only) |
| `autostart` | `false` | Start Scoria on login |
| `language` | *(empty)* | UI language: empty string = auto from `LANG` / `LC_MESSAGES` / `LC_ALL`, or `en`, `ru` |

### Interface language

Scoria ships with **English** and **Russian** UI strings (tray menu, notifications, settings dialogs).

- In **Settings…**, use **Interface language**: *Auto* (follows your OS locale), *English*, or *Русский*. Saving applies the language immediately in that window; the tray app updates menu text and tooltip shortly after (it watches `config.toml`).
- In the config file, set `language = ""` for auto, `language = "en"`, or `language = "ru"`.

### Autostart

Enable **"Start Scoria on login"** in Settings (or set `autostart = true` in the config file). Scoria will:

- **Linux**: create a `.desktop` entry in `~/.config/autostart/`
- **macOS**: create a LaunchAgent in `~/Library/LaunchAgents/`

Disabling the checkbox removes the autostart entry.

## Updating

Scoria checks for updates automatically on tray startup. When a new version is available, you'll see a notification.

To update: click **"Check for updates"** in the tray menu — Scoria will download and replace the binary automatically. Restart to apply.

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

## License

MIT OR Apache-2.0
