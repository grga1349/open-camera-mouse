#!/usr/bin/env bash
set -euo pipefail

APP=$(find build/bin -maxdepth 1 -name "*.app" -print -quit || true)
if [ -z "$APP" ]; then
  echo "ERROR: No .app found in build/bin"
  ls -la build/bin
  exit 1
fi

BIN=$(find "$APP/Contents/MacOS" -maxdepth 1 -type f -perm -111 -print -quit || true)
if [ -z "$BIN" ]; then
  echo "ERROR: No executable found in $APP/Contents/MacOS"
  ls -la "$APP/Contents/MacOS"
  exit 1
fi
APPNAME=$(basename "$APP" .app)
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

npx --yes create-dmg "$APP" --overwrite
DMG_FILE=$(ls build/bin/*.dmg 2>/dev/null | head -1 || true)
if [ -z "$DMG_FILE" ]; then
  echo "ERROR: DMG not created"
  exit 1
fi
mv "$DMG_FILE" "build/open-camera-mouse_${TAG}_macOS.dmg"

ditto -c -k --sequesterRsrc --keepParent "$APP" "build/${APPNAME}-mac.zip"
