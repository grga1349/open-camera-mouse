#!/usr/bin/env bash
set -euo pipefail

cd build/bin

exe_file=$(ls -1 *.exe | head -1 || true)
if [ -z "$exe_file" ]; then
  echo "ERROR: No .exe found in build/bin"
  exit 1
fi

system_dll_regex='^(KERNEL32|USER32|GDI32|ADVAPI32|SHELL32|OLE32|OLEAUT32|WS2_32|SHLWAPI|COMDLG32|IMM32|WINMM|UCRTBASE|VCRUNTIME140|VCRUNTIME140_1|MSVCP140|MSVCP140_1|MSVCP140_2|MSVCP140_ATOMIC_WAIT|MSVCP140_CODECVT_IDS|VERSION|COMCTL32|WINSPOOL|api-ms-win|bcrypt|CRYPT32|ntdll|RPCRT4|SETUPAPI|USERENV|UxTheme|dwmapi|iphlpapi|MPR|NETAPI32|Secur32|WINHTTP|urlmon)\.dll$'

echo "=== Final dependency list (from objdump) ==="
objdump -p "$exe_file" | grep "DLL Name:" | awk '{print $3}' | sort -u > deps.txt
cat deps.txt

if grep -qiE "opencv_highgui|Qt6" deps.txt; then
  echo "ERROR: Qt/HighGUI dependency detected in dependency list"
  grep -iE "opencv_highgui|Qt6" deps.txt
  exit 1
fi

echo "=== Copying required DLLs ==="
missing=0
copied_list="copied-dlls.txt"
: > "$copied_list"

while read -r dll; do
  if [[ "$dll" =~ $system_dll_regex ]]; then
    continue
  fi
  if [ -f "/mingw64/bin/$dll" ]; then
    cp -n "/mingw64/bin/$dll" .
    echo "$dll" >> "$copied_list"
  else
    echo "ERROR: Required DLL not found in /mingw64/bin: $dll"
    missing=1
  fi
done < deps.txt

for rt in libstdc++-6.dll libgcc_s_seh-1.dll libwinpthread-1.dll; do
  if [ -f "/mingw64/bin/$rt" ]; then
    cp -n "/mingw64/bin/$rt" .
    echo "$rt" >> "$copied_list"
  else
    echo "ERROR: MinGW runtime DLL missing: $rt"
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  echo "ERROR: Missing required DLLs, cannot package."
  exit 1
fi

echo "=== Copied DLLs (final) ==="
sort -u "$copied_list"

echo "=== build/bin contents ==="
ls -la

zip -r "../${ASSET_NAME}" .
