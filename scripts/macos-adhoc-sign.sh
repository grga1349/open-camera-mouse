#!/bin/sh
# Ad-hoc signs the macOS .app with a designated requirement that names only the bundle id
# (ADR-0021 amendment 2026-11-19). Run by cargo-packager as `before-each-package-command`
# (apps/desktop/Cargo.toml); acts only before the dmg step, when the .app exists.
#
# Why: an ad-hoc signature's default requirement is the build's cdhash, and macOS keys the
# Camera / Accessibility grants to it, so every update silently loses them (measured on the
# dev Mac: tccd "Failed to match existing code requirement", and toggling the switch in System
# Settings does not repair it). With `identifier "<id>"` the grants survive rebuilds.
# Trade-off: any ad-hoc binary claiming this id would match too; a Developer ID signature
# (team-anchored) replaces this once an account exists, and then this script does nothing.
set -eu

[ "${CARGO_PACKAGER_FORMAT:-}" = "dmg" ] || exit 0

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP="$ROOT/target/packager/Open Camera Mouse.app"
ID="io.github.grga1349.OpenCameraMouse"
ENTITLEMENTS="$ROOT/apps/desktop/packaging/macos/entitlements.plist"

[ -d "$APP" ] || { echo "macos-adhoc-sign: $APP not found" >&2; exit 1; }

# A real (Developer ID) signature from cargo-packager already has a stable requirement.
if codesign -dv "$APP" 2>&1 | grep -q '^Authority='; then
    echo "macos-adhoc-sign: $APP has a certificate signature; leaving it"
    exit 0
fi

# Hardened runtime as cargo-packager would sign (notarization needs it); the camera
# entitlement is what lets a hardened app use the camera at all.
codesign --force --sign - --options runtime --entitlements "$ENTITLEMENTS" \
    -r="designated => identifier \"$ID\"" "$APP"
codesign --verify --strict "$APP"
codesign -d -r- "$APP" 2>&1 | grep -q "designated => identifier \"$ID\"" || {
    echo "macos-adhoc-sign: designated requirement not applied" >&2
    exit 1
}
echo "macos-adhoc-sign: signed $APP (designated => identifier \"$ID\")"
