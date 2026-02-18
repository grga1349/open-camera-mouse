#!/usr/bin/env bash
set -euo pipefail
APP=$(find build/bin -maxdepth 1 -name "*.app" -print -quit)
[ -n "$APP" ] || { echo "No .app in build/bin"; exit 1; }
ditto -c -k --sequesterRsrc --keepParent "$APP" \
  "build/open-camera-mouse_${GITHUB_REF_NAME}_macOS.zip"
