#!/usr/bin/env bash
# Remove Scoria binary, desktop integration, config, and autostart (Linux / macOS).
# Safe to run multiple times; only Scoria-specific paths under ~/.local and standard config dirs.

set -euo pipefail

PREFIX="${HOME}/.local"
BINDIR="${PREFIX}/bin"
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
XDG_ICON_HOME="${XDG_ICON_HOME:-$HOME/.local/share/icons}"

rm_path() {
	if [[ -e "$1" ]] || [[ -L "$1" ]]; then
		rm -rf "$1"
		echo "removed: $1"
	fi
}

OS="$(uname -s)"

if command -v killall >/dev/null 2>&1; then
	killall scoria 2>/dev/null || true
elif command -v pkill >/dev/null 2>&1; then
	pkill -x scoria 2>/dev/null || true
fi

rm_path "${BINDIR}/scoria"

case "$OS" in
Linux)
	rm_path "${XDG_ICON_HOME}/hicolor/scalable/apps/scoria.svg"
	rm_path "${XDG_ICON_HOME}/hicolor/128x128/apps/scoria.png"
	rm_path "${XDG_DATA_HOME}/applications/scoria.desktop"
	rm_path "${HOME}/.config/autostart/scoria.desktop"
	rm_path "${HOME}/.config/scoria"
	;;
Darwin)
	rm_path "${HOME}/Applications/Scoria.app"
	rm_path "${HOME}/Library/Application Support/scoria"
	rm_path "${HOME}/Library/LaunchAgents/com.github.syn7xx.scoria.plist"
	;;
*)
	echo "Note: unsupported OS ($OS); removed ${BINDIR}/scoria only if present."
	;;
esac

echo "Scoria uninstall done."
