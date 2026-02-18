#!/usr/bin/env bash
set -euo pipefail

EXE=$(ls -1 build/bin/*.exe 2>/dev/null | head -1)
[ -n "$EXE" ] || { echo "ERROR: No .exe in build/bin"; ls -la build/bin/; exit 1; }
echo "Bundling: $EXE"

DIST="build/dist"
mkdir -p "$DIST"
cp "$EXE" "$DIST/"

# Recursively collect all MinGW DLLs using objdump (part of the toolchain, no extra package).
collect_dlls() {
  local target="$1"
  while IFS= read -r dll; do
    local src="/mingw64/bin/$dll"
    if [ -f "$src" ] && [ ! -f "$DIST/$dll" ]; then
      cp "$src" "$DIST/$dll"
      collect_dlls "$DIST/$dll"
    fi
  done < <(objdump -p "$target" 2>/dev/null | awk '/DLL Name:/{print $3}')
}

echo "Collecting DLL dependencies..."
collect_dlls "$EXE"

dll_count=$(find "$DIST" -maxdepth 1 -name "*.dll" | wc -l | tr -d ' ')
echo "DLLs copied: $dll_count"
[ "$dll_count" -gt 0 ] || { echo "ERROR: no MinGW DLLs found for $EXE"; exit 1; }

# Qt platform plugin (required by OpenCV HighGUI; loaded dynamically, not via import table)
echo "Locating Qt plugins directory..."
PLUGINS_SRC=""
for candidate in /mingw64/share/qt6/plugins /mingw64/lib/qt6/plugins /mingw64/plugins; do
  if [ -d "$candidate/platforms" ]; then
    PLUGINS_SRC="$candidate"
    break
  fi
done
if [ -z "$PLUGINS_SRC" ]; then
  qwindows=$(find /mingw64 -type f -name "qwindows.dll" 2>/dev/null | head -1 || true)
  [ -n "$qwindows" ] && PLUGINS_SRC="$(dirname "$(dirname "$qwindows")")"
fi
[ -n "$PLUGINS_SRC" ] || {
  echo "ERROR: Qt plugins directory not found"
  find /mingw64 -name "qwindows.dll" 2>/dev/null || true
  exit 1
}
echo "Qt plugins: $PLUGINS_SRC"

mkdir -p "$DIST/platforms"
cp "$PLUGINS_SRC/platforms/qwindows.dll" "$DIST/platforms/"

for dir in imageformats iconengines styles; do
  if [ -d "$PLUGINS_SRC/$dir" ]; then
    mkdir -p "$DIST/$dir"
    cp "$PLUGINS_SRC/$dir/"*.dll "$DIST/$dir/" 2>/dev/null || true
  fi
done

# Tell Qt where to find plugins/ relative to the exe
printf '[Paths]\nPlugins = .\n' > "$DIST/qt.conf"

echo "Creating zip..."
(cd "$DIST" && zip -r "../open-camera-mouse_${GITHUB_REF_NAME}_Windows.zip" .)
echo "Bundle complete"
