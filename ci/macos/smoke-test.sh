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

# Verify no Homebrew or @rpath leakage
if otool -L "$BIN" | grep -qE '/opt/homebrew|/usr/local|@rpath'; then
  echo "ERROR: unresolved dylib references in binary"
  otool -L "$BIN"
  exit 1
fi
find "$APP/Contents/Frameworks" -name "*.dylib" -print | while read -r lib; do
  if otool -L "$lib" | grep -qE '/opt/homebrew|/usr/local|@rpath'; then
    echo "ERROR: unresolved reference in $lib"
    otool -L "$lib"
    exit 1
  fi
done
