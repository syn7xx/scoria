#!/usr/bin/env bash
set -euo pipefail

BIN="${1:-scoria}"

echo "[macos] smoke: version"
"$BIN" --version >/dev/null

echo "[macos] smoke: help"
"$BIN" --help >/dev/null

echo "[macos] smoke: command help"
"$BIN" save --help >/dev/null
"$BIN" settings-gui --help >/dev/null

echo "[macos] smoke: expected failure path (save without clipboard)"
set +e
SAVE_OUTPUT="$("$BIN" save 2>&1)"
SAVE_EXIT=$?
set -e
if [ "$SAVE_EXIT" -eq 0 ]; then
  echo "Expected non-zero exit for save without clipboard context"
  exit 1
fi
echo "$SAVE_OUTPUT" | grep -Eiq "nothing to save|нечего сохранять" || {
  echo "Expected user-facing empty clipboard error, got:"
  echo "$SAVE_OUTPUT"
  exit 1
}

echo "[macos] smoke: OK"
