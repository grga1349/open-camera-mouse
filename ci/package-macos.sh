#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="build"
BIN_ROOT="build/bin"
mkdir -p "$OUT_DIR"

APP="$(find "$BIN_ROOT" -maxdepth 1 -name '*.app' -print -quit)"
if [ -z "$APP" ]; then
  echo "ERROR: No .app found in $BIN_ROOT"
  ls -la "$BIN_ROOT" || true
  exit 1
fi

echo "== APP =="
echo "$APP"

MACOS_DIR="$APP/Contents/MacOS"
BIN="$(find "$MACOS_DIR" -maxdepth 1 -type f -perm -111 -print -quit)"
if [ -z "$BIN" ]; then
  echo "ERROR: No executable found in $MACOS_DIR"
  ls -la "$MACOS_DIR" || true
  exit 1
fi

echo "== BIN =="
echo "$BIN"
file "$BIN" || true

FW="$APP/Contents/Frameworks"
mkdir -p "$FW"

echo "== dylibbundler =="
dylibbundler -od -b -x "$BIN" -d "$FW" -p "@executable_path/../Frameworks"

echo "== otool -L (post-bundle) =="
otool -L "$BIN"

if otool -L "$BIN" | grep -qE "/opt/homebrew|/usr/local"; then
  echo "ERROR: Found Homebrew paths in dylib references."
  exit 1
fi

echo "== codesign (ad-hoc) =="
codesign --force --deep --sign - "$APP"
codesign --verify --deep --strict "$APP"

APP_BASENAME="$(basename "$APP")"
APPNAME="${APP_BASENAME%.app}"

ZIP_PATH="$OUT_DIR/${APPNAME}-mac.zip"
echo "== zip (ditto) =="
ditto -c -k --sequesterRsrc --keepParent "$APP" "$ZIP_PATH"
echo "OK: $ZIP_PATH"

echo "== dmg (create-dmg via npx) =="
(
  cd "$OUT_DIR"
  npx --yes create-dmg "../$APP" --overwrite
)

DMG_FOUND="$(ls -1 "$OUT_DIR"/*.dmg 2>/dev/null | head -1 || true)"
if [ -n "$DMG_FOUND" ]; then
  DMG_PATH="$OUT_DIR/${APPNAME}-mac.dmg"
  mv -f "$DMG_FOUND" "$DMG_PATH"
  echo "OK: $DMG_PATH"
else
  echo "ERROR: DMG not created."
  exit 1
fi
