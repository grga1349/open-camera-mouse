#!/usr/bin/env bash
set -euo pipefail

EXE=$(ls -1 build/bin/*.exe 2>/dev/null | head -1)
[ -n "$EXE" ] || { echo "ERROR: No .exe in build/bin"; ls -la build/bin/; exit 1; }
echo "Bundling: $EXE"

DIST="build/dist"
mkdir -p "$DIST"
cp "$EXE" "$DIST/"

# ntldd -R output format: "  foo.dll => /mingw64/bin/foo.dll (0xADDR)"
# $3 is the resolved path; $NF is the trailing hex address — use $3
echo "Collecting DLL dependencies via ntldd..."
ntldd -R "$EXE" 2>/dev/null \
  | awk '$2 == "=>" && $3 ~ /[Mm]ingw/ { print $3 }' \
  | sort -u \
  | while read -r dll_path; do
      [ -f "$dll_path" ] && cp -n "$dll_path" "$DIST/" || true
    done

dll_count=$(find "$DIST" -maxdepth 1 -name "*.dll" | wc -l | tr -d ' ')
echo "DLLs copied: $dll_count"
[ "$dll_count" -gt 0 ] || { echo "ERROR: ntldd found no MinGW DLLs"; exit 1; }

# Qt plugins are loaded dynamically by OpenCV HighGUI — ntldd won't find them
echo "Locating Qt plugins directory..."
PLUGINS_SRC=""
for candidate in \
    /mingw64/share/qt6/plugins \
    /mingw64/lib/qt6/plugins \
    /mingw64/plugins; do
  if [ -d "$candidate/platforms" ]; then
    PLUGINS_SRC="$candidate"
    break
  fi
done
if [ -z "$PLUGINS_SRC" ]; then
  # Last-resort: search the mingw64 tree
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
