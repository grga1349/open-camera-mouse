#!/usr/bin/env bash
set -euo pipefail
ZIP="build/open-camera-mouse_${GITHUB_REF_NAME}_macOS.zip"
[ -f "$ZIP" ] || { echo "Missing: $ZIP"; exit 1; }

PKG_DIR="$(mktemp -d)"
unzip -q "$ZIP" -d "$PKG_DIR"
APP=$(find "$PKG_DIR" -maxdepth 2 -name "*.app" -print -quit)
[ -n "$APP" ] || { echo "No .app in zip"; exit 1; }
BIN=$(find "$APP/Contents/MacOS" -maxdepth 1 -type f -perm -111 -print -quit)
[ -n "$BIN" ] || { echo "No binary in .app"; exit 1; }

"$BIN" --smoke-test

# Verify no absolute Homebrew paths remain in binary or bundled dylibs
# (delocate rewrites to @loader_path/... so only that and system paths are expected)
check_no_homebrew() {
  local f="$1"
  if otool -L "$f" | grep -qE '/opt/homebrew|/usr/local'; then
    echo "ERROR: absolute Homebrew path in $f"
    otool -L "$f"
    exit 1
  fi
}

check_no_homebrew "$BIN"
find "$APP/Contents/Frameworks" -name "*.dylib" -print | while read -r lib; do
  check_no_homebrew "$lib"
done
