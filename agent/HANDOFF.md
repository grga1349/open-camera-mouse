# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6 ACTIVE, D7 part 1 done, D7 part 2 = manual QA next)

**Branch:** `rewrite/rust`. Task `tasks/active/0008-phase-6-desktop-app.md`. **D7 part 1 done**
(runtime + headless end-to-end tests + recorder + `docs/qa/MANUAL_QA.md`), details in the task
file Progress entry. Gates green locally: fmt, clippy `-D warnings` (+ Windows target), workspace
tests (desktop 68, app-core controller 37), `check-core-deps.sh`. **Uncommitted unless the
commit log says otherwise** — check `git status`. Work rhythm (user): one slice per session.

**Runtime shape:** `apps/desktop/src/runtime.rs`. Camera on a worker thread (`ThreadedCamera`),
controller + pointer on the main thread in `eframe::App::logic` (runs while minimised, 100 ms
throttle). An `app-core` bug was fixed on the way (observation vs due takeover, regression
test in `crates/app-core/tests/controller.rs`).

**Next session:** D7 part 2, needs the user at the Mac: run MANUAL_QA rows 1–18 and record
results with date; rows 19–20 need a `.app`: **user decided Phase 7** (record them `n/a: Phase 7`); a few
`OCM_RECORD=<dir>` sessions to re-measure re-detection arming / R-P4 bound / dwell radius.
Then close task 0008 against acceptance criteria 1–11 (CI run ID for the closing commit).

**Screenshots without clicks (debug builds):** `OCM_DEV_SETTINGS=<file>` and
`OCM_DEV_OPTIONS=pointer|clicking|camera|general`; bounds via `osascript` System Events, then
`screencapture -x -R x,y,w,h`. A real click needs a posted CGEvent; ask first, it moves the
user's cursor.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed (F11/F12; F10/F12 on macOS); recenter =
white centre circle + 5 s countdown badge; resume from pause goes through recenter; Recenter ends
a pause. Preview is colour. Font: later.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only (no new
crates in this slice). No `gh` CLI: check CI with `curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
then `/actions/runs/<id>/jobs`.

**Blockers:** none.
