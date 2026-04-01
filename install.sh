#!/usr/bin/env bash
set -euo pipefail

REPO="syn7xx/scoria"
INSTALL_DIR="${HOME}/.local/bin"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"

OS="$(uname -s)"
ARCH="$(uname -m)"

die() {
  echo "Error: $*" >&2
  exit 1
}

sha256_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print tolower($1)}'
    return
  fi
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print tolower($1)}'
    return
  fi
  die "Neither sha256sum nor shasum is available."
}

normalize_arch() {
  case "$1" in
    x86_64)        echo "x86_64" ;;
    arm64|aarch64) echo "aarch64" ;;
    *)             die "Unsupported architecture: $1" ;;
  esac
}

normalize_platform() {
  case "$1" in
    Linux)   echo "linux" ;;
    Darwin)  echo "macos" ;;
    *)       die "Unsupported OS: $1" ;;
  esac
}

fetch_latest_tag() {
  local repo="$1"
  local latest
  latest="$(curl -sSfL "https://api.github.com/repos/${repo}/releases/latest" \
    | grep '"tag_name"' | head -1 | cut -d'"' -f4)"
  if [ -z "$latest" ]; then
    die "Could not fetch latest version."
  fi
  echo "$latest"
}

install_linux_desktop() {
  local scoria_bin="$1"
  local script_dir="$2"
  local repo="$3"
  local tag="$4"

  # Walker/Omarchy list applications from XDG desktop entries, not just binaries in $PATH.
  local xdg_data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
  local xdg_icon_home="${XDG_ICON_HOME:-$HOME/.local/share/icons}"

  local app_desktop_dir="${xdg_data_home}/applications"
  local app_desktop="${app_desktop_dir}/scoria.desktop"
  local icon_dir_svg="${xdg_icon_home}/hicolor/scalable/apps"
  local icon_dir_png="${xdg_icon_home}/hicolor/128x128/apps"
  local icon_file=""

  mkdir -p "$app_desktop_dir" "$icon_dir_svg" "$icon_dir_png"

  # Prefer a concrete icon path so KDE launcher can always resolve it.
  # First try local assets (repo checkout), then fall back to GitHub raw files.
  if [ -f "${script_dir}/assets/scoria.svg" ]; then
    cp -f "${script_dir}/assets/scoria.svg" "${icon_dir_svg}/scoria.svg"
    icon_file="${icon_dir_svg}/scoria.svg"
  fi
  if [ -f "${script_dir}/assets/scoria-128.png" ]; then
    cp -f "${script_dir}/assets/scoria-128.png" "${icon_dir_png}/scoria.png"
    icon_file="${icon_dir_png}/scoria.png"
  fi
  if [ -z "$icon_file" ]; then
    if curl -sLf "https://raw.githubusercontent.com/${repo}/${tag}/assets/scoria-128.png" \
      -o "${icon_dir_png}/scoria.png"; then
      icon_file="${icon_dir_png}/scoria.png"
    elif curl -sLf "https://raw.githubusercontent.com/${repo}/main/assets/scoria-128.png" \
      -o "${icon_dir_png}/scoria.png"; then
      icon_file="${icon_dir_png}/scoria.png"
    elif curl -sLf "https://raw.githubusercontent.com/${repo}/${tag}/assets/scoria.svg" \
      -o "${icon_dir_svg}/scoria.svg"; then
      icon_file="${icon_dir_svg}/scoria.svg"
    elif curl -sLf "https://raw.githubusercontent.com/${repo}/main/assets/scoria.svg" \
      -o "${icon_dir_svg}/scoria.svg"; then
      icon_file="${icon_dir_svg}/scoria.svg"
    fi
  fi
  if [ -z "$icon_file" ]; then
    icon_file="scoria"
  fi

  cat > "$app_desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Scoria
Comment=Save clipboard to Obsidian vault
Exec=${scoria_bin} run
Icon=${icon_file}
Terminal=false
Categories=Utility;
StartupNotify=false
EOF

  echo "Created desktop entry: ${app_desktop}."
}

ARCH="$(normalize_arch "$ARCH")"
PLATFORM="$(normalize_platform "$OS")"

ASSET="scoria-${PLATFORM}-${ARCH}.tar.gz"

LATEST="$(fetch_latest_tag "$REPO")"

URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
SHA_URL="${URL}.sha256"

echo "Installing scoria ${LATEST} (${PLATFORM}/${ARCH})..."
echo "  from: ${URL}"
echo "  to:   ${INSTALL_DIR}/scoria"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

ARCHIVE_PATH="${TMP}/${ASSET}"
SHA_PATH="${TMP}/${ASSET}.sha256"

curl -sSfL "$URL" -o "$ARCHIVE_PATH"
curl -sSfL "$SHA_URL" -o "$SHA_PATH"

EXPECTED_SHA="$(tr -d '\r\n' < "$SHA_PATH" | awk '{print tolower($1)}')"
ACTUAL_SHA="$(sha256_file "$ARCHIVE_PATH")"
if [ "$EXPECTED_SHA" != "$ACTUAL_SHA" ]; then
  die "Checksum mismatch for ${ASSET}"
fi

tar -xzf "$ARCHIVE_PATH" -C "$TMP"
mkdir -p "$INSTALL_DIR"
install -m 755 "$TMP/scoria" "$INSTALL_DIR/scoria"

SCORIA_BIN="${INSTALL_DIR}/scoria"

# User .app bundle so Spotlight / Launchpad can find "Scoria" (CLI-only install has no bundle metadata).
if [[ "$OS" == "Darwin" ]]; then
  APP="${HOME}/Applications/Scoria.app"
  VER="${LATEST#v}"
  mkdir -p "${APP}/Contents/MacOS"
  mkdir -p "${APP}/Contents/Resources"
  install -m 755 "$TMP/scoria" "${APP}/Contents/MacOS/scoria"
  # Icon from repo (release archives may not include .icns).
  if ! curl -sLf "https://raw.githubusercontent.com/${REPO}/${LATEST}/assets/macos/Resources/scoria.icns" \
      -o "${APP}/Contents/Resources/scoria.icns"
  then
    curl -sLf "https://raw.githubusercontent.com/${REPO}/main/assets/macos/Resources/scoria.icns" \
      -o "${APP}/Contents/Resources/scoria.icns" \
      || echo "Warning: could not download scoria.icns; app icon in Launchpad may be generic."
  fi
  cat > "${APP}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleExecutable</key>
	<string>scoria</string>
	<key>CFBundleIdentifier</key>
	<string>com.syn7xx.scoria</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>Scoria</string>
	<key>CFBundleDisplayName</key>
	<string>Scoria</string>
	<key>CFBundleIconFile</key>
	<string>scoria</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>${VER}</string>
	<key>CFBundleVersion</key>
	<string>2</string>
	<key>NSHighResolutionCapable</key>
	<true/>
	<key>LSUIElement</key>
	<true/>
</dict>
</plist>
EOF
  echo "Created app bundle: ${APP} (for Spotlight / Applications)."
elif [[ "$OS" == "Linux" ]]; then
  install_linux_desktop "$SCORIA_BIN" "$SCRIPT_DIR" "$REPO" "$LATEST"
fi

echo "Done. Run 'scoria --version' to verify."

if [[ "$OS" == "Linux" ]] && ! "$SCORIA_BIN" --version >/dev/null 2>&1; then
	echo ""
	echo "ERROR: scoria did not start. Missing shared library:"
	ldd "$SCORIA_BIN" 2>/dev/null | grep "not found" || true
	echo "Try rebuilding from source: git clone https://github.com/syn7xx/scoria.git && cd scoria && make deps && make install"
fi

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
  echo ""
  echo "Note: ${INSTALL_DIR} is not in your PATH."
  echo "Add to your shell profile:  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi
