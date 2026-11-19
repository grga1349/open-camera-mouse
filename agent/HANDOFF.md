# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-19 (Phase 7 ACTIVE, task 0012, slice 4 done)

**Branch:** `rewrite/rust`. Last pushed code `2688256` (slice 3, CI green on 3 OS).

**This session:** slice 4 — `.github/workflows/release.yml` (tag `v*` → macOS arm64 dmg,
Windows NSIS, Linux AppImage + tar.gz → draft GitHub Release with `SHA256SUMS`; signing
hooks behind secrets), `cargo-about` notices shipped in every package (`about.toml`,
`apps/desktop/packaging/about.hbs`, CI `deny` job checks them), Windows `.ico`, CI also on
`main`. macOS packaging verified locally (`cargo packager --release`; `cargo-about` 0.9.3
installed with `--features cli`). Findings in the ADR-0021 slice-4 amendment.

**Next:** cleanup (see CURRENT_STATE "Immediate next action"), then the **user** merges to
`main`, tags `v0.4.0`; the release run is the first time the Windows/Linux packaging runs
(user: no separate dry run) — check its jobs via the API and fix what fails. Merge conflict
expected on `.github/workflows/release.yml` (legacy Go one on `main`): take the Rust one.

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

**Blockers:** none. No Windows/Linux machines (slices 5–6 deferred, user 2026-11-19).
