# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-19 (Phase 7 ACTIVE, task 0012, slice 3 done)

**Branch:** `rewrite/rust`. CI last green at `2ae2a5c` (44604964205); slice 3 commit not yet
pushed/CI-checked when this was written — check the run first.

**This session:** slice 3 — macOS `.app` + dmg (`cd apps/desktop && cargo packager
--release`; `cargo-packager` 0.11.8 installed locally). Version 0.4.0, Info.plist with
`NSCameraUsageDescription`, camera entitlement (hardened runtime is forced), runtime app/dock
icon (`app::app_icon`, test), `LICENSE` as resource. On-device bug found + fixed: ad-hoc
grants were keyed to the build's cdhash (lost every rebuild; System Settings toggle can't
repair; tccd "Failed to match existing code requirement") → `scripts/macos-adhoc-sign.sh`
signs with `designated => identifier "io.github.grga1349.OpenCameraMouse"`; and
`CGPreflightPostEventAccess` is stale within a run → access = `AXIsProcessTrusted || preflight`.
ADR-0021 amendment; MANUAL_QA rows 19, 20, 22 pass. Diagnose TCC with
`/usr/bin/log show --predicate 'process == "tccd"'` (zsh `log` is a builtin).

**Next:** slice 4 — release workflow (see CURRENT_STATE "Immediate next action"). #7: read
the annotation on the next failure.

**User decisions (Phase 7):** macOS Apple Silicon only, ad-hoc signed dmg, notarization hook
behind secrets (off); Windows unsigned NSIS per-user installer; Linux tar.gz + AppImage, X11
only, Wayland documented; GitHub Releases; version 0.4.0; icon = F weight everywhere.

**Rules (ADR-0019/-0020):** measure first; changed defaults need the user's OK + ADR
amendment; every fix gets a CI test / replay metric / `OCM_PERF` + device check; recordings
diagnosis-only; one slice per session.

**Machine load:** one heavy command at a time, scoped (`-p`, test filter), say so first;
release builds: ask first.

**How the user runs the app:** `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (dev build), or the `.app` from `target/packager/`.

**Known, not bugs:** Searching animation ignores Reduce Motion (Phase 8); font egui default;
F10 not F11 on macOS; re-detection off by default; no auto-recenter; 16 px icon slightly
muddy; Gatekeeper rejects the ad-hoc app (documented in slice 8).

**User direction (don't drift):** start from 2018 but improve; new behaviour as options, off by
default unless approved; monochrome UI, neon green ring marker, yellow primary buttons,
Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on macOS); automated checks for every
fix; real numbers on controls.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI via the GitHub API with curl
(`.../actions/runs/<run>/jobs`, `.../check-runs/<job id>/annotations`).

**Blockers:** none. Windows/Linux on-device slices need machines — ask the user.
