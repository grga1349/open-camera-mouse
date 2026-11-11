#!/usr/bin/env bash
set -euo pipefail
ZIP="build/open-camera-mouse_${GITHUB_REF_NAME}_Windows.zip"
[ -f "$ZIP" ] || { echo "Missing: $ZIP"; exit 1; }

PKG_DIR="$(mktemp -d)"
unzip -q "$ZIP" -d "$PKG_DIR"

PKG_EXE=$(ls -1 "$PKG_DIR"/*.exe 2>/dev/null | head -1)
[ -n "$PKG_EXE" ] || { echo "No .exe in package"; find "$PKG_DIR" -maxdepth 2; exit 1; }

# Run smoke test — exits 0 if all native libs loaded; panics/crashes otherwise
"$PKG_EXE" --smoke-test
