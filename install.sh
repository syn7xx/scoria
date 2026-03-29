#!/usr/bin/env bash
set -euo pipefail

REPO="syn7xx/scoria"
INSTALL_DIR="${HOME}/.local/bin"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64)          ARCH="x86_64" ;;
  arm64|aarch64)   ARCH="aarch64" ;;
  *)               echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
  Linux)   PLATFORM="linux" ;;
  Darwin)  PLATFORM="macos" ;;
  *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

ASSET="scoria-${PLATFORM}-${ARCH}.tar.gz"

LATEST="$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | head -1 | cut -d'"' -f4)"

if [ -z "$LATEST" ]; then
  echo "Could not fetch latest version."
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"

echo "Installing scoria ${LATEST} (${PLATFORM}/${ARCH})..."
echo "  from: ${URL}"
echo "  to:   ${INSTALL_DIR}/scoria"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -sL "$URL" | tar xz -C "$TMP"
mkdir -p "$INSTALL_DIR"
install -m 755 "$TMP/scoria" "$INSTALL_DIR/scoria"

SCORIA_BIN="${INSTALL_DIR}/scoria"

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
