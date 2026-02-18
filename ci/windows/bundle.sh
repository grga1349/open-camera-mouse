#!/usr/bin/env bash
set -euo pipefail
EXE=$(ls -1 build/bin/*.exe | head -1)
[ -n "$EXE" ] || { echo "No .exe in build/bin"; exit 1; }

DIST="build/dist"
mkdir -p "$DIST"
cp "$EXE" "$DIST/"

# Use ntldd to copy only the MinGW DLLs this binary actually needs
ntldd -R "$EXE" \
  | grep -i '/mingw64/' \
  | awk '{print $NF}' \
  | while read -r dll_path; do
      [ -f "$dll_path" ] && cp -n "$dll_path" "$DIST/" || true
    done

# Qt platform plugin (required by OpenCV HighGUI; not captured by ntldd)
PLUGINS_SRC="/mingw64/share/qt6/plugins"
mkdir -p "$DIST/platforms"
cp "$PLUGINS_SRC/platforms/qwindows.dll" "$DIST/platforms/"

# Additional Qt plugin categories used by OpenCV
for dir in imageformats iconengines styles; do
  if [ -d "$PLUGINS_SRC/$dir" ]; then
    mkdir -p "$DIST/$dir"
    cp "$PLUGINS_SRC/$dir/"*.dll "$DIST/$dir/" 2>/dev/null || true
  fi
done

# Tell Qt where to find the plugins relative to the exe
printf '[Paths]\nPlugins = .\n' > "$DIST/qt.conf"

(cd "$DIST" && zip -r "../open-camera-mouse_${GITHUB_REF_NAME}_Windows.zip" .)
