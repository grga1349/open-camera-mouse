#!/usr/bin/env bash
# Records one benchmark scenario (docs/qa/RECORDING_PROTOCOL.md) with the dev build.
# Usage: scripts/record-scenario.sh S1   (bash 3.2 compatible: macOS default shell)
set -euo pipefail

id="${1:-}"
case "$id" in
  S0) name=static ;; S1) name=still ;; S2) name=turns ;; S3) name=corners ;;
  *) echo "usage: $0 S0|S1|S2|S3  (see docs/qa/RECORDING_PROTOCOL.md)" >&2; exit 2 ;;
esac
dir="$HOME/ocm-recordings/$id-$name-$(date +%F)"
if [[ -e "$dir" ]]; then
  echo "$dir already exists; move or delete it first." >&2
  exit 1
fi
echo "Recording $id ($name) to $dir"
echo "F10 = start, click the nose tip, keep still until the cursor jumps (~4 s), do the scenario, quit."
export PATH="$HOME/.cargo/bin:$PATH"
OCM_RECORD="$dir" cargo run -p open-camera-mouse
