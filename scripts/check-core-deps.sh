#!/usr/bin/env bash
# Fails if a core crate pulls in anything outside its allowlist (ADR-0007, ADR-0012).
# tracking-core / pointer-core must stay free of camera, OS, GUI and I/O crates.
# app-core may additionally take `serde` (derive only, for Settings); file I/O, OS and GUI
# crates stay out of it too.
set -euo pipefail

PURE='tracking-core|pointer-core|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident'
APP_CORE="$PURE|app-core|serde|serde_core|serde_derive"

status=0
check() {
  local crate=$1 allowed="^($2)$"
  deps=$(cargo tree -p "$crate" -e normal --prefix none --format '{p}' | awk '{print $1}' | sort -u)
  bad=$(printf '%s\n' "$deps" | grep -Ev "$allowed" || true)
  if [ -n "$bad" ]; then
    echo "error: $crate depends on crates outside its allowlist:" >&2
    printf '  %s\n' $bad >&2
    status=1
  else
    echo "ok: $crate deps = $(printf '%s ' $deps)"
  fi
}

check tracking-core "$PURE"
check pointer-core "$PURE"
check app-core "$APP_CORE"
exit $status
