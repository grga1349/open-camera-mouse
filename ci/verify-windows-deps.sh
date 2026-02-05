#!/usr/bin/env bash
set -euo pipefail

# Usage (MSYS2 MINGW64):
#   bash ci/verify-windows-deps.sh build/bin/open-camera-mouse.exe
# If no argument is provided, defaults to build/bin/*.exe (first match).

exe_path="${1:-}"
if [ -z "$exe_path" ]; then
  exe_path=$(ls -1 build/bin/*.exe 2>/dev/null | head -1 || true)
fi

if [ -z "$exe_path" ] || [ ! -f "$exe_path" ]; then
  echo "ERROR: exe not found. Provide path: bash ci/verify-windows-deps.sh build/bin/<app>.exe"
  exit 1
fi

exe_dir=$(cd "$(dirname "$exe_path")" && pwd)

system_dll_regex='^(KERNEL32|USER32|GDI32|ADVAPI32|SHELL32|OLE32|OLEAUT32|WS2_32|SHLWAPI|COMDLG32|IMM32|WINMM|UCRTBASE|VCRUNTIME140|VCRUNTIME140_1|MSVCP140|MSVCP140_1|MSVCP140_2|MSVCP140_ATOMIC_WAIT|MSVCP140_CODECVT_IDS|VERSION|COMCTL32|WINSPOOL|api-ms-win|bcrypt|CRYPT32|ntdll|RPCRT4|SETUPAPI|USERENV|UxTheme|dwmapi|iphlpapi|MPR|NETAPI32|Secur32|WINHTTP|urlmon)\.dll$'

echo "=== Dependency list for: $exe_path ==="
deps=$(objdump -p "$exe_path" | grep "DLL Name:" | awk '{print $3}' | sort -u)
echo "$deps"

if echo "$deps" | grep -qiE "opencv_highgui|Qt6"; then
  echo "ERROR: Qt/HighGUI dependency detected"
  echo "$deps" | grep -iE "opencv_highgui|Qt6"
  exit 1
fi

missing=0
echo "=== Verifying non-system DLLs are present next to exe ==="
while read -r dll; do
  if [[ "$dll" =~ $system_dll_regex ]]; then
    continue
  fi
  if [ ! -f "$exe_dir/$dll" ]; then
    echo "MISSING: $dll"
    missing=1
  fi
done <<< "$deps"

if [ "$missing" -ne 0 ]; then
  echo "ERROR: Missing required DLLs next to exe."
  exit 1
fi

echo "OK"
