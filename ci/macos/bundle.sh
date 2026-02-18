#!/usr/bin/env bash
set -euo pipefail
APP=$(find build/bin -maxdepth 1 -name "*.app" -print -quit)
[ -n "$APP" ] || { echo "No .app in build/bin"; ls build/bin; exit 1; }
BIN=$(find "$APP/Contents/MacOS" -maxdepth 1 -type f -perm -111 -print -quit)
[ -n "$BIN" ] || { echo "No executable in $APP/Contents/MacOS"; exit 1; }

FW="$APP/Contents/Frameworks"
mkdir -p "$FW"
OPENCV_LIB="$(brew --prefix opencv)/lib"

# Seed Frameworks with OpenCV dylibs
find "$OPENCV_LIB" -maxdepth 1 -name "libopencv*.dylib" -exec cp -a {} "$FW/" \;
[ "$(find "$FW" -maxdepth 1 -name "libopencv*.dylib" | wc -l | tr -d ' ')" -gt 0 ] \
  || { echo "No OpenCV dylibs copied"; exit 1; }

# Bundle full closure for the binary and each OpenCV dylib
dylibbundler -od -b \
  -x "$BIN" \
  -d "$FW" \
  -p "@executable_path/../Frameworks" \
  -s "$OPENCV_LIB" \
  -s "$(brew --prefix)/lib"

find "$FW" -maxdepth 1 -name "libopencv*.dylib" -print | while read -r lib; do
  dylibbundler -od -b \
    -x "$lib" \
    -d "$FW" \
    -p "@executable_path/../Frameworks" \
    -s "$OPENCV_LIB" \
    -s "$(brew --prefix)/lib" || true
done

# Fix install names for everything in Frameworks
find "$FW" -type f -name "*.dylib" -print | while read -r lib; do
  base="$(basename "$lib")"
  install_name_tool -id "@executable_path/../Frameworks/$base" "$lib" || true
done

# Rewrite any remaining absolute or @rpath references
{ echo "$BIN"; find "$FW" -type f -name "*.dylib"; } | while read -r f; do
  otool -L "$f" | awk 'NR>1{print $1}' | while read -r dep; do
    case "$dep" in
      /opt/homebrew/*|/usr/local/*)
        dep_base="$(basename "$dep")"
        [ -e "$FW/$dep_base" ] && install_name_tool -change "$dep" "@executable_path/../Frameworks/$dep_base" "$f" || true
        ;;
      @rpath/*)
        dep_base="$(basename "$dep")"
        [ -e "$FW/$dep_base" ] && install_name_tool -change "$dep" "@executable_path/../Frameworks/$dep_base" "$f" || true
        ;;
    esac
  done
done

codesign --force --deep --sign - "$APP"
