#!/usr/bin/env bash
set -euo pipefail

APP=$(find build/bin -maxdepth 1 -name "*.app" -print -quit)
[ -n "$APP" ] || { echo "No .app in build/bin"; ls build/bin; exit 1; }
BIN=$(find "$APP/Contents/MacOS" -maxdepth 1 -type f -perm -111 -print -quit)
[ -n "$BIN" ] || { echo "No executable in $APP/Contents/MacOS"; exit 1; }

FW="$APP/Contents/Frameworks"
mkdir -p "$FW"
BREW="$(brew --prefix)"

echo "App: $APP"
echo "Binary: $BIN"

# Recursively copy all Homebrew dylibs into Frameworks.
# @rpath/ refs are resolved using the source file's own LC_RPATH entries.
collect() {
  local src="$1"
  local rpaths
  rpaths=$(otool -l "$src" 2>/dev/null \
    | awk '/cmd LC_RPATH/{found=1; next} found && /path /{print $2; found=0}')

  while IFS= read -r dep; do
    local real_path=""
    case "$dep" in
      "$BREW"/*) real_path="$dep" ;;
      @rpath/*)
        local base="${dep#@rpath/}"
        while IFS= read -r rp; do
          [ -f "$rp/$base" ] && { real_path="$rp/$base"; break; }
        done <<< "$rpaths"
        [ -z "$real_path" ] && [ -f "$BREW/lib/$base" ] && real_path="$BREW/lib/$base"
        ;;
    esac
    [ -z "$real_path" ] || [ ! -f "$real_path" ] && continue
    local base_name; base_name="$(basename "$real_path")"
    if [ ! -f "$FW/$base_name" ]; then
      echo "  + $base_name"
      cp -L "$real_path" "$FW/$base_name"
      collect "$FW/$base_name"
    fi
  done < <(otool -L "$src" 2>/dev/null | awk 'NR>1{print $1}')
}

echo "Collecting dependencies..."
collect "$BIN"

dylib_count=$(find "$FW" -maxdepth 1 -name "*.dylib" | wc -l | tr -d ' ')
echo "Dylibs collected: $dylib_count"
[ "$dylib_count" -gt 0 ] || { echo "ERROR: no dylibs collected"; exit 1; }

# Fix the binary.
# The binary is built with -headerpad_max_install_names so adding LC_RPATH is safe.
# All @rpath/ refs in transitively loaded dylibs resolve via this chain at runtime.
echo "Fixing binary load paths..."
install_name_tool -add_rpath "@executable_path/../Frameworks" "$BIN"
while IFS= read -r dep; do
  case "$dep" in
    "$BREW"/*)
      install_name_tool -change "$dep" "@rpath/$(basename "$dep")" "$BIN" || true
      ;;
  esac
done < <(otool -L "$BIN" | awk 'NR>1{print $1}')

# Fix each dylib in Frameworks.
# Only change absolute Homebrew paths -> @rpath/basename (always shorter, no headerpad issue).
# Leave existing @rpath/... LC_LOAD_DYLIBs alone: dyld resolves them via the
# binary's LC_RPATH chain (@executable_path/../Frameworks) at runtime.
echo "Fixing dylib load paths..."
while IFS= read -r lib; do
  base="$(basename "$lib")"
  install_name_tool -id "@rpath/$base" "$lib" || true
  while IFS= read -r dep; do
    case "$dep" in
      "$BREW"/*)
        install_name_tool -change "$dep" "@rpath/$(basename "$dep")" "$lib" || true
        ;;
    esac
  done < <(otool -L "$lib" | awk 'NR>1{print $1}')
done < <(find "$FW" -maxdepth 1 -name "*.dylib" -print)

echo "Signing bundle..."
codesign --force --deep --sign - "$APP"
echo "Bundle complete"
