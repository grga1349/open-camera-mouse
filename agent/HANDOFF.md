# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6b test drive, part 1 committed; part 2 NEXT)

**Branch:** `rewrite/rust`, task `tasks/active/0009-phase-6b-test-drive.md`, issue log
`docs/qa/TEST_DRIVE.md` (#1–#5). Part 1 is committed (see `git log`); **not pushed, and the
full gates were not re-run locally after the last changes** (the user stopped a heavy run).
First step next session: push only if the user says so, otherwise run the gates **one crate at a
time** (see "Machine load" below).

**Status of the issues:**
- #1 rounded preview corners: done.
- #4 "Main monitor" (recenter/pause on a chosen monitor, shown only with 2+ monitors): done,
  ADR-0014. Not yet confirmed on device by the user.
- #3 "Find the target again": fixed (ADR-0011 amendment: margin 0.03, 3 confirming frames,
  41×41 context check). User: "ok for now, could be better".
- **#2 losses and #5 jitter: the user's top priority. "Needs to improve a lot."** The steady
  zone (ADR-0015, off by default) did not help: the cursor still moves 10–20 px.

**Part 2 plan (agreed direction, not started):**
1. The user runs an online research agent with `docs/research/TEST_DRIVE_RESEARCH_PROMPT.md`;
   its report goes into `docs/research/` before implementing.
2. Read the 2018 RE notes in `../camera-mouse-reverse/research/` (04 recovery, 05 smoothing;
   behaviour only, never copy code). Known: 2018 has no long-term template (the LK reference
   advances every frame), a 35 px motion guard, damping per 70 ms tick, and mapping by
   desktop size.
3. **Strong lead for #5:** the mapping scales by the whole desktop
   (`map_raw`: gain × desktop_width / 640). The user has 2 × 2560 wide monitors → ≈ 48 screen px
   per camera px, so 0.2 px of tracker noise ≈ 10 px. Try scaling by the main / current display
   instead of the bounding box, then a 1€ filter, then re-tune the steady zone. Ask the user
   before changing the default behaviour.
4. #2: get a **scripted benchmark recording** from the user (`OCM_RECORD=<dir outside repo>`,
   about 60 s: look at the 4 screen corners, slow and fast turns, tilt, talk, never leave the
   frame), then tune with `replay_tune` (`crates/app-core/tests/replay.rs`). First pass on the
   unscripted recording: hw 15 + gate 0.6 = 9 losses vs 15 (recommended), no added drift.
   Ideas to test: anchored template update, multi-point median flow around the chosen point,
   larger windows.

**Replay tools** (`crates/app-core/tests/replay.rs`, all `#[ignore]`):
`OCM_REPLAY=<dir> cargo test --release -p app-core --test replay <name> -- --ignored --nocapture`.
`replay_recording` (full controller; `OCM_REPLAY_OUT` CSV trace, `OCM_REPLAY_STEADY`,
`OCM_REPLAY_RECOVERY`), `replay_tune` (tracker variants table; `OCM_REPLAY_FRAMES` cap),
`replay_redetect_scores`, `replay_redetect_best_positions`, `replay_context_zncc`.
The first recording is in this session's scratchpad (`/private/tmp/claude-501/…/scratchpad/rec-1`,
≈ 20k frames, contains the user's face) and will be lost when /tmp is cleaned. Don't rely on it.

**Machine load (user complaint 2026-11-15, "you almost burned my processor"):** never chain
workspace clippy + workspace tests + cross-checks + release replays back to back while the app
runs. Run one heavy command at a time, scoped (`-p <crate>`, a test filter), and say so first.
Cap replays with `OCM_REPLAY_FRAMES`.

**How the user runs the app:** dev build, `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (never `--release` for the test drive). Fix issues as they come
up. Their settings: `~/Library/Application Support/Open Camera Mouse/settings.toml`.

**Known, not bugs (don't "fix" without the user):** pointer ~10 Hz while minimised; Searching
animation ignores Reduce Motion (Phase 8); font is egui default; F10 not F11 on macOS;
re-detection off by default; no auto-recenter.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it; new behaviour
ships as options, off by default (ask before changing a default); monochrome UI, neon green ring
marker, yellow primary buttons, Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on macOS).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; check CI with
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`.

**Blockers:** none. Waiting on: the research report, a benchmark recording, and the user's
on-device check of #4.
