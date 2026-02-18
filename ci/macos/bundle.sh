#!/usr/bin/env bash
set -euo pipefail

APP=$(find build/bin -maxdepth 1 -name "*.app" -print -quit)
[ -n "$APP" ] || { echo "No .app in build/bin"; ls build/bin; exit 1; }
BIN=$(find "$APP/Contents/MacOS" -maxdepth 1 -type f -perm -111 -print -quit)
[ -n "$BIN" ] || { echo "No executable in $APP/Contents/MacOS"; exit 1; }

echo "App: $APP"
echo "Binary: $BIN"

pip3 install --quiet delocate

# delocate-path walks the entire .app, collects all non-system dylib deps
# (including transitive ones like libgfortran → libquadmath), copies them
# into Frameworks, and rewrites load paths using @loader_path — which
# rewrites the full Mach-O header rather than calling install_name_tool,
# avoiding the headerpad-too-small failure from dylibbundler.
delocate-path "$APP" -L "$APP/Contents/Frameworks"

codesign --force --deep --sign - "$APP"
