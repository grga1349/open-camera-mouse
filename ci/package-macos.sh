#!/usr/bin/env bash
set -euo pipefail

APP=$(ls -1 build/bin/*.app | head -1 || true)
if [ -z "$APP" ]; then
  echo "ERROR: No .app found in build/bin"
  exit 1
fi

APPNAME=$(basename "$APP" .app)
BIN="$APP/Contents/MacOS/$APPNAME"
FW="$APP/Contents/Frameworks"
TAG="${GITHUB_REF_NAME#v}"

echo "App: $APP"
echo "Binary: $BIN"

mkdir -p "$FW"
dylibbundler -od -b -x "$BIN" -d "$FW" -p "@executable_path/../Frameworks"

echo "=== otool -L (post-bundle) ==="
otool -L "$BIN"

if otool -L "$BIN" | grep -qE "/opt/homebrew|/usr/local"; then
  echo "ERROR: Found Homebrew paths in dylib references."
  exit 1
fi

if otool -L "$BIN" | grep -q "libopencv" && ! otool -L "$BIN" | grep -q "@executable_path/../Frameworks/libopencv"; then
  echo "ERROR: OpenCV dylibs are not rebased to @executable_path/../Frameworks."
  exit 1
fi

codesign --force --deep --sign - "$APP"
codesign --verify --deep --strict "$APP"

if command -v create-dmg >/dev/null 2>&1; then
  (cd build/bin && create-dmg "$APPNAME.app" --overwrite || true)
  DMG_FILE=$(ls build/bin/*.dmg 2>/dev/null | head -1 || true)
  if [ -n "$DMG_FILE" ]; then
    mv "$DMG_FILE" "build/open-camera-mouse_${TAG}_macOS.dmg"
  fi
fi

ditto -c -k --sequesterRsrc --keepParent "$APP" "build/${APPNAME}-mac.zip"
