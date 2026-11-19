# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-19 (Phase 7 ACTIVE, task 0012, slice 1)

**Branch:** `rewrite/rust`. Last code `687a9fc` CI green on 3 OS. `461d381` (docs only) failed
Windows `test (no webcam)` exit 1 → flake #7.

**This session:** wrote `tasks/active/0012-phase-7-multiplatform-release.md` (8 slices, gate,
user decisions). Slice 1: `.github/workflows/ci.yml` test step now `--no-fail-fast`, output
teed to `test.log`; on failure a `summarise test failures` step emits one `::error` annotation
with FAILED lines / panics / `error: test failed, to rerun pass -p …`. Read it without auth:
`curl -s https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs/<run>/jobs`
→ job id → `.../check-runs/<job id>/annotations`. Script dry-run locally; not yet run in CI.

**Next:** slice 1 at `2ae2a5c`: CI 44604964205 green on 3 OS, summary step skipped (doesn't break green runs; not yet seen on a failure). On the next flake read the annotation and fix the test
deterministically (TEST_DRIVE #7). Then slice 2: packaging ADR (tool, Windows installer
format, app id, version, licence notices) + icon from
`~/Downloads/Black Camera Outline with Concentric C and M.png` sized per OS — show the user
16/32 px renders before adopting (thin strokes).

**User decisions (Phase 7):** macOS Apple Silicon only, ad-hoc signed dmg, notarization hook
behind secrets (off); Windows unsigned with an installer; Linux tar.gz + AppImage, X11 only,
Wayland documented; GitHub Releases.

**Rules (ADR-0019/-0020):** measure first; changed defaults need the user's OK + ADR
amendment; every fix gets a CI test / replay metric / `OCM_PERF` + device check; recordings
diagnosis-only; one slice per session.

**Machine load:** one heavy command at a time, scoped (`-p`, test filter), say so first;
release builds: ask first.

**How the user runs the app:** `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (dev build).

**Known, not bugs:** Searching animation ignores Reduce Motion (Phase 8); font egui default;
F10 not F11 on macOS; re-detection off by default; no auto-recenter.

**User direction (don't drift):** start from 2018 but improve; new behaviour as options, off by
default unless approved; monochrome UI, neon green ring marker, yellow primary buttons,
Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on macOS); automated checks for every
fix; real numbers on controls.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI via the GitHub API with curl.

**Blockers:** none. Windows/Linux on-device slices need machines — ask the user.
