# ADR-0021: Packaging — tool, installers, identity, icon, licence notices

Status: Accepted (user decisions 2026-11-19: icon, version, app id, packaging tool, Windows
installer format, licence-notice tool; platform targets recorded in task 0012).
Date: 2026-11-19
Task: `tasks/active/0012-phase-7-multiplatform-release.md` slice 2. Risks: PLATFORM_RISKS P8,
P18, P19.

## Context

Phase 7 ships the app as: macOS arm64 ad-hoc signed `.app` in a `.dmg` (Developer ID +
notarization hook off until an account exists); Windows unsigned **with an installer**
(signing hook off); Linux `.tar.gz` + AppImage, X11; all on GitHub Releases. Before
slices 3–4 build any of this we need: a packaging tool, the Windows installer format, the app
identity (name, id, version), the icon set, and how third-party licence notices are generated
and shipped. `deny.toml` already limits dependency licences to permissive ones plus OFL-1.1 /
Ubuntu-font-1.0 (egui's fonts); several of those require the licence text to ship with the
binary.

## Options considered

### Packaging tool

1. **`cargo-packager` 0.11.8** (CrabNebula, from Tauri's bundler; MIT OR Apache-2.0; latest
   release 2025-11-27). One config (`[package.metadata.packager]` or `Packager.toml`) for
   `.app`, `.dmg`, NSIS `.exe`, WiX `.msi`, `.deb`, AppImage. Has `before_packaging_command`,
   `resources`, `license_file`, macOS `signing_identity` / `entitlements` /
   `info_plist_path` / notarization credentials, NSIS `install_mode`. VERIFIED from docs.rs.
   Not yet verified (slice 3 checks on the dev Mac): PNG → `.icns`/`.ico` conversion (it
   depends on `tauri-icns` and `image`, so INFERRED yes), ad-hoc signing (`-` identity), and
   which formats build on which host (assume native runners: macOS for dmg, Windows for NSIS,
   Linux for AppImage).
2. **Hand scripts per OS** (`hdiutil` + `codesign`, `makensis`, `appimagetool`): no extra
   tool, full control, but three scripts to write and maintain, each re-solving icons, plist
   and signing hooks.
3. **`cargo-bundle`**: `.app`, `.deb`, `.msi`; no NSIS, no AppImage, little maintenance.
4. **`cargo-dist`**: aimed at CLI tools (shell/PowerShell installers, MSI); no `.app`/`.dmg`.

### Windows installer format

1. **NSIS `.exe`, per-user install** (`install_mode = currentUser`): no admin prompt, installs
   to `%LOCALAPPDATA%`, Start-menu entry and uninstaller, small, builds with
   `cargo-packager`. NSIS is zlib-licensed (its stub is inside the installer).
2. **WiX `.msi`**: suits enterprise/GPO deployment; per-machine needs admin; WiX toolset needed
   on the runner.
3. **Inno Setup**: good installer, but not supported by `cargo-packager` (hand script).

Per-user vs per-machine doesn't change P8: `SendInput` can't drive elevated windows unless
the app is signed, has `uiAccess`, and lives in a protected location (Program Files) — none
of which is in v1 scope. Documented as a known exception.

### Licence notices

1. **`cargo-about` 0.9.3** (Embark, same family as `cargo-deny`; MIT OR Apache-2.0; release
   2026-10-26): `about.toml` (accepted licences = the `deny.toml` allow list) + a Handlebars
   template → one `THIRD_PARTY_LICENSES.html` with the full text for every crate.
2. **`cargo-deny list`**: lists licences per crate but not the texts — not enough for MIT /
   BSD / OFL notice requirements.
3. Hand-maintained file: goes stale with every `cargo update`.

## Decision

**Identity (user, 2026-11-19).** Product name "Open Camera Mouse"; binary
`open-camera-mouse`; app id / macOS bundle id **`io.github.grga1349.OpenCameraMouse`**
(reverse DNS of the GitHub Pages domain; also the Linux `.desktop`/AppStream id). macOS camera
and Accessibility permissions are keyed to this id, so it must not change after release. First
Rust release **0.4.0** (continues legacy 0.3.0; 1.0 after Phase 8): the workspace version
moves from 0.0.0 to 0.4.0 in slice 3, and the release tag is `v0.4.0`.

**Icon (user, 2026-11-19).** The user's design (yellow square, black camera outline, lens
ring, "C" around "M"; original in `assets/icon/source.png`) is **redrawn as geometry** by
`scripts/render-icon.py` with lines 1.6× the source weight, the same at every size (user picked
this over a two-weight set; it's a little muddy at 16 px). The script writes
`assets/icon/png/icon-{16…1024}.png` (full-bleed square: Windows `.ico`, Linux, eframe window
icon) and `assets/icon/macos/icon-{16…1024}.png` (Apple grid: 824/1024 rounded square,
transparent margin → `.icns`). The PNGs are committed; the script is a dev tool (Python +
Pillow, not run in CI or the build). The source raster was AI-generated; it's the user's
asset, and the redraw is ours.

**Tool = `cargo-packager`**, pinned (`cargo install cargo-packager --version 0.11.8
--locked` in the release workflow), config in `apps/desktop/Cargo.toml`
`[package.metadata.packager]`. Formats: macOS `app` + `dmg`; Windows `nsis` per-user; Linux
`appimage`, plus a plain `.tar.gz` made by a short release-workflow step (binary, `.desktop`,
icon PNG, licence files), since `cargo-packager`'s `pacman` target is an Arch package, not a
plain tarball. If slice 3 finds that a needed feature is missing (icon conversion, ad-hoc
signing), the fallback for that one step is the OS tool (`iconutil` for `.icns`, `codesign -s
-` for the ad-hoc signature) as a `before`/after-packaging command, not a switch of tools.

**Notices = `cargo-about` 0.9.3**, pinned the same way. `about.toml` mirrors the
`deny.toml` allow list; output `THIRD_PARTY_LICENSES.html`, generated in the release workflow
(never committed, so it can't go stale) and shipped next to `LICENSE` in every package:
`.app` `Contents/Resources`, next to the exe for NSIS, inside the AppImage and tarball. CI
runs `cargo about generate` too, so a dependency with an unlisted licence fails before a
release does.

Signing hooks (off until secrets exist): macOS `APPLE_SIGNING_IDENTITY` + notarization
credentials → `cargo-packager`'s macOS signing/notarization; Windows a `signtool` step guarded
by a certificate secret. Without secrets: macOS ad-hoc (`-`), Windows unsigned.

## Consequences

- One config describes all packages; slices 3–4 wire it rather than write three bundlers.
- Two pinned dev/release tools (`cargo-packager`, `cargo-about`), installed in the workflow
  only — no new crate dependencies of the app, so `deny.toml` and
  `scripts/check-core-deps.sh` are unaffected. Python + Pillow is needed only to re-render the
  icon.
- Per-user NSIS means no admin prompt but also no `uiAccess`: elevated windows can't be
  driven (P8) — documented in the release notes (slice 8).
- Ad-hoc signatures change on every build, so macOS may ask for camera/Accessibility again
  after an update; slice 3 measures this and documents the reset (MANUAL_QA rows 19–20).
- The 16 px icon keeps the "C" and reads slightly muddy; a lighter-detail small-size variant
  can be added later without changing the id or packaging.
- Follow-ups: slice 3 verifies the INFERRED tool facts above on the dev Mac and records them
  here; the eframe window icon needs PNG decoding (check `eframe`'s icon helpers before
  adding any crate — an ADR if one is needed).
