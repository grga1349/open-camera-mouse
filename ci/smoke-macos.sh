#!/usr/bin/env bash
set -euo pipefail

APP=$(ls -1 build/bin/*.app | head -1 || true)
if [ -z "$APP" ]; then
  echo "ERROR: No .app found in build/bin"
  exit 1
fi

APPNAME=$(basename "$APP" .app)
BIN="$APP/Contents/MacOS/$APPNAME"

echo "=== file BIN ==="
file "$BIN"

echo "=== otool -L BIN ==="
otool -L "$BIN"

if otool -L "$BIN" | grep -qE "/opt/homebrew|/usr/local"; then
  echo "ERROR: Found Homebrew paths in dylib references."
  exit 1
fi

if otool -L "$BIN" | grep -q "libopencv" && ! otool -L "$BIN" | grep -q "@executable_path/../Frameworks/libopencv"; then
  echo "ERROR: OpenCV dylibs are not rebased to @executable_path/../Frameworks."
  exit 1
fi

codesign --verify --deep --strict "$APP"

ZIP=$(ls -1 build/*-mac.zip 2>/dev/null | head -1 || true)
if [ -z "$ZIP" ]; then
  echo "ERROR: mac zip not found"
  exit 1
fi
echo "=== zip contents (head) ==="
unzip -l "$ZIP" | head -40
