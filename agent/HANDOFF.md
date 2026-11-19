# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-19 (Phase 7 ACTIVE, task 0012, slice 2 done)

**Branch:** `rewrite/rust`. Last code `2ae2a5c`: CI 44604964205 green on 3 OS; the slice-1
failure-summary step was skipped as intended. Flake #7 not seen since.

**This session:** slice 2 — ADR-0021 (Accepted): `cargo-packager` 0.11.8 (macOS app+dmg,
Windows NSIS per-user, Linux AppImage; plain tar.gz by a workflow step), `cargo-about` 0.9.3 →
`THIRD_PARTY_LICENSES.html` in every package (generated, never committed), app id
`io.github.grga1349.OpenCameraMouse`, first release 0.4.0 / tag `v0.4.0`. Icon: the user's
design redrawn as geometry by `scripts/render-icon.py` with 1.6× line weight at every size (the
user picked this over a two-weight set) → `assets/icon/png/icon-{16…1024}.png` and
`assets/icon/macos/…` (824/1024 rounded square). Original in `assets/icon/source.png`.
Re-render: venv + `pip install pillow==11.3.0`, then run the script.

**Next:** slice 3 — macOS `.app` + dmg: `[package.metadata.packager]` in
`apps/desktop/Cargo.toml`, workspace version 0.4.0, `NSCameraUsageDescription`, ad-hoc
signing; verify the INFERRED tool facts in ADR-0021 (icns from PNGs, `-` identity); eframe
window icon (check eframe's PNG helper before adding a crate); MANUAL_QA rows 19–20 on the
bundle, including whether permissions survive a rebuild. Installing `cargo-packager` and the
release build are heavy: ask first. #7: read the annotation on the next failure.

**User decisions (Phase 7):** macOS Apple Silicon only, ad-hoc signed dmg, notarization hook
behind secrets (off); Windows unsigned NSIS per-user installer; Linux tar.gz + AppImage, X11
only, Wayland documented; GitHub Releases; version 0.4.0; icon = F weight everywhere.

**Rules (ADR-0019/-0020):** measure first; changed defaults need the user's OK + ADR
amendment; every fix gets a CI test / replay metric / `OCM_PERF` + device check; recordings
diagnosis-only; one slice per session.

**Machine load:** one heavy command at a time, scoped (`-p`, test filter), say so first;
release builds: ask first.

**How the user runs the app:** `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (dev build).

**Known, not bugs:** Searching animation ignores Reduce Motion (Phase 8); font egui default;
F10 not F11 on macOS; re-detection off by default; no auto-recenter; 16 px icon slightly muddy.

**User direction (don't drift):** start from 2018 but improve; new behaviour as options, off by
default unless approved; monochrome UI, neon green ring marker, yellow primary buttons,
Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on macOS); automated checks for every
fix; real numbers on controls.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI via the GitHub API with curl
(`.../actions/runs/<run>/jobs`, `.../check-runs/<job id>/annotations`).

**Blockers:** none. Windows/Linux on-device slices need machines — ask the user.
