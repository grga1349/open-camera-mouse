#!/usr/bin/env bash
set -euo pipefail

: "${ASSET_NAME:=open-camera-mouse_windows.zip}"

BIN_DIR="build/bin"
OUT_DIR="build"
mkdir -p "$OUT_DIR"

exe_file="$(ls -1 "$BIN_DIR"/*.exe 2>/dev/null | head -1 || true)"
if [ -z "$exe_file" ]; then
  echo "ERROR: No .exe found in $BIN_DIR"
  ls -la "$BIN_DIR" || true
  exit 1
fi

echo "== EXE =="
echo "$exe_file"

echo "== Dependency scan (objdump) =="
deps="$(objdump -p "$exe_file" | awk '/DLL Name:/{print $3}' | sort -u)"

if echo "$deps" | grep -qiE "opencv_highgui|Qt6"; then
  echo "ERROR: Qt/HighGUI dependency detected!"
  echo "$deps" | grep -iE "opencv_highgui|Qt6"
  exit 1
fi

system_re='^(KERNEL32|USER32|GDI32|ADVAPI32|SHELL32|OLE32|OLEAUT32|WS2_32|SHLWAPI|COMDLG32|IMM32|WINMM|UCRTBASE|VCRUNTIME140|VCRUNTIME140_1|MSVCP140|MSVCP140_1|MSVCP140_2|VERSION|COMCTL32|WINSPOOL|api-ms-win|bcrypt|CRYPT32|ntdll|RPCRT4|SETUPAPI|USERENV|UxTheme|dwmapi|iphlpapi|MPR|NETAPI32|Secur32|WINHTTP|urlmon)\.dll$'

missing=0
echo "== Copy required non-system DLLs from /mingw64/bin =="
while read -r dll; do
  [ -z "$dll" ] && continue
  if [[ "$dll" =~ $system_re ]]; then
    continue
  fi
  if [ -f "/mingw64/bin/$dll" ]; then
    cp -n "/mingw64/bin/$dll" "$BIN_DIR/" || true
  else
    echo "MISSING: /mingw64/bin/$dll"
    missing=1
  fi
done <<< "$deps"

for rt in libstdc++-6.dll libgcc_s_seh-1.dll libwinpthread-1.dll; do
  if [ -f "/mingw64/bin/$rt" ]; then
    cp -n "/mingw64/bin/$rt" "$BIN_DIR/" || true
  else
    echo "MISSING runtime: /mingw64/bin/$rt"
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  echo "ERROR: Missing required DLLs. Packaging aborted."
  exit 1
fi

echo "== Final contents =="
ls -la "$BIN_DIR"

echo "== Zip portable package =="
( cd "$BIN_DIR" && zip -r "../$(basename "$ASSET_NAME")" . )
mv -f "$BIN_DIR/$(basename "$ASSET_NAME")" "$OUT_DIR/$ASSET_NAME"

echo "OK: $OUT_DIR/$ASSET_NAME"
