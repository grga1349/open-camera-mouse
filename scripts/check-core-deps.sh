#!/usr/bin/env bash
# Fails if a pure core crate pulls in anything outside its allowlist (ADR-0007).
# tracking-core / pointer-core must stay free of camera, OS, GUI and I/O crates.
set -euo pipefail

ALLOWED='^(tracking-core|pointer-core|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)$'
status=0
for crate in tracking-core pointer-core; do
  deps=$(cargo tree -p "$crate" -e normal --prefix none --format '{p}' | awk '{print $1}' | sort -u)
  bad=$(printf '%s\n' "$deps" | grep -Ev "$ALLOWED" || true)
  if [ -n "$bad" ]; then
    echo "error: $crate depends on crates outside its allowlist:" >&2
    printf '  %s\n' $bad >&2
    status=1
  else
    echo "ok: $crate deps = $(printf '%s ' $deps)"
  fi
done
exit $status
