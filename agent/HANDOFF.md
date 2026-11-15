# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6 DONE, Phase 6b test drive NEXT)

**Branch:** `rewrite/rust`. Phase 6 (desktop app, task 0008) closed in `2146984`; last code
commit `7e9a2bb`, CI 43746097717 green. Manual QA rows 1–18 pass (`docs/qa/MANUAL_QA.md`).
Phase 6b was added by **ADR-0013** (user decision): before Phase 7, the user test-drives the app
and we sort out what comes up. **Check `git status`/`git log` for whether the ADR + this
handoff are committed.**

**Next session = Phase 6b, test drive.** The user will have used the app for a while and will
bring issues. Do this:
1. Write `tasks/active/0009-phase-6b-test-drive.md` from ROADMAP §6b (deliverables + gate),
   add it to `tasks/BACKLOG.md` as ACTIVE.
2. Create `docs/qa/TEST_DRIVE.md`: an issue table (#, date, what happened, severity, triage,
   fix commit or "deferred: Phase N, reason") plus the build each issue was seen on.
3. Take the issues one at a time: get exact steps from the user; reproduce (headless test with
   fakes in `apps/desktop/src/runtime_tests.rs` / app-core tests where possible, `OCM_RECORD`
   for tracking problems, screenshots for UI); fix with a regression test; gates green. One
   issue or a small related group per session (user rhythm).
4. Fixes only. A wish for new behaviour → ask the user; if yes, it ships as an option, off by
   default (feedback rule), and gets its own decision record if non-trivial.
5. Close 6b only when every logged issue is fixed or deferred and the user says ready for Phase 7.

**How the user runs it** (tell them if asked): `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run --release -p open-camera-mouse` (real settings; release for realistic speed).
`OCM_RECORD=<dir>` adds a frame recording (~9 MB/s, contains their face, keep it outside the
repo) when a tracking issue needs evidence. Permissions are attributed to the launching
terminal until the Phase 7 `.app`.

**Known, not bugs (don't "fix" without the user):** pointer ~10 Hz while minimised (judged
usable); Searching animation ignores Reduce Motion (Phase 8); font is egui default (later);
F10 not F11 on macOS (F11 = Show Desktop); re-detection off by default; no auto-recenter.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed (F11/F12; F10/F12 on macOS); recenter =
white centre circle + 5 s countdown badge; resume from pause goes through recenter; Recenter ends
a pause. Preview is colour.

**Carry-overs (unchanged):** MANUAL_QA rows 19–20 (`.app`) → Phase 7; row 21 (reduced motion)
and `OCM_RECORD` threshold re-measurement (re-detection arming, R-P4, dwell radius) → Phase 8.

**Screenshots without clicks (debug builds):** `OCM_DEV_SETTINGS=<file>` and
`OCM_DEV_OPTIONS=pointer|clicking|camera|general`; bounds via `osascript` System Events, then
`screencapture -x -R x,y,w,h`. A real click needs a posted CGEvent; ask first.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only. No `gh`
CLI: check CI with `curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
then `/actions/runs/<id>/jobs`.

**Blockers:** none.
