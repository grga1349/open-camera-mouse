# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6 DONE, Phase 7 NEXT)

**Branch:** `rewrite/rust`. Task 0008 (Phase 6, desktop app) closed: `tasks/done/0008-…`.
Last code commit `7e9a2bb`, CI 43746097717 green. Manual QA on the dev Mac with the user:
`docs/qa/MANUAL_QA.md` rows 1–18 pass. The close-out is docs only (QA results, task move,
BACKLOG, ROADMAP, agent state). **Uncommitted unless the commit log says otherwise**.

**Next session:** start Phase 7 (multiplatform / release) from `docs/ROADMAP.md`: write its
task file in `tasks/active/` from the Phase 7 deliverables and gate (no pre-written task).
Phase 7 carry-over: MANUAL_QA rows 19–20 (`.app` permissions). Phase 8 carry-over (user
decision 2026-11-15): row 21 (reduced motion) and `OCM_RECORD` threshold re-measurement
(re-detection arming, R-P4 bound, dwell radius). Work rhythm (user): one slice per session.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed (F11/F12; F10/F12 on macOS); recenter =
white centre circle + 5 s countdown badge; resume from pause goes through recenter; Recenter ends
a pause. Preview is colour. Font: later.

**Screenshots without clicks (debug builds):** `OCM_DEV_SETTINGS=<file>` and
`OCM_DEV_OPTIONS=pointer|clicking|camera|general`; bounds via `osascript` System Events, then
`screencapture -x -R x,y,w,h`.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only. No `gh`
CLI: check CI with `curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
then `/actions/runs/<id>/jobs`.

**Blockers:** none.
