# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6d step 2 metrics committed; next session = testing + step 3 sweep)

**Branch:** `rewrite/rust`. Step 2 committed and pushed (see `git log`); verify its CI. CI on `d7728b5`: Windows
`test (no webcam)` failed with exit 1 (#7; log needs auth — ask the user for the test name).

**User decision:** no new recordings; corpus = `~/ocm-recordings/S1-still-2026-11-15` and
`S2-turns-2026-11-15`.

**Done:** task 0011 written; replay tool ported (`bench_run_zone`, `OCM_REPLAY_STEADY` in cam
px); metrics module `crates/pointer-core/src/metrics.rs` (creep, shake, unexplained jumps,
lag, precision rule); `the_cursor_does_not_move_at_rest_*` use it; replay tool
`replay_metrics` (`OCM_REPLAY_DIRS=<S1>:<S2>`) and `replay_around` (`OCM_AT_S`). Baseline
table in task 0011. **#12 lead:** at S2 58.54 s the cursor snaps 44 px to the bottom edge —
`Controller::drive` pins an axis at the limit past the steady zone's trailing radius.

**Next session (user: "2nd part, where we will test stuff"):**
0. Testing with the user on the dev build (`cargo run -p open-camera-mouse`): ask whether the
   jumps were at screen edges and reproduce #12 by moving the cursor to each edge at the
   default zone vs `OCM_DEV_STEADY=0` (expect: snap with zone, none without). Note results in
   TEST_DRIVE #12.
1. Step 3: sweep S1/S2 over gain, smoothing, zone, 1€ params (+ candidate edge-approach fix)
   with `replay_metrics`; table into task 0011. Also explain S2's 1084 vs 1563 controlled
   steps (defaults vs as-recorded).
2. Ask the user whether their jumps happened at screen edges.
3. Then step 4 (fit synthetic signals), step 5 (fixes).

**Evidence so far (S1 replay, recorded smoothing 0.05; 2026-11-16 re-run: 0.5 cam px → 99/300 rest frames move, 0 moves needs 2.24 cam px, identical at gain 4–60). Older numbers, moves / reversals by camera-px zone:** gain 24: no
zone 311/6, 0.2 → 169/5, 0.5 → 108/2, 1.0 → 59/1; same pattern at gain 8–60. Reversals (shake)
are already low after the 1€ filter; most moves are slow drift (1–2.5 cam px over seconds).
So a dead zone big enough for drift costs precision: separate shake from drift (ADR-0020 §5).

**Carried in:** #7 macOS CI flake (need the test name from run 43773727473's log); #4
main-monitor check, minimised rate, dwell on device (from 6b), `ui` 15/17 ms with Options open,
ADR-0018 slow-creep limit.

**Rules (ADR-0019/-0020):** measure first; changed defaults need the user's OK + ADR amendment;
every fix gets a CI test / replay metric / `OCM_PERF` + device check; recordings diagnosis-only,
only fitted numbers enter the repo; one slice per session.

**Machine load:** one heavy command at a time, scoped (`-p`, test filter), say so first;
release builds: ask first. `app-core` tests ≈ 12 s (rest test split in 4 parallel parts).

**How the user runs the app:** `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (dev build). Try a zone: `OCM_DEV_STEADY=0.75 cargo run -p open-camera-mouse`.

**Known, not bugs:** Searching animation ignores Reduce Motion (Phase 8); font egui default;
F10 not F11 on macOS; re-detection off by default; no auto-recenter.

**User direction (don't drift):** start from 2018 but improve; new behaviour as options, off by
default unless approved; monochrome UI, neon green ring marker, yellow primary buttons,
Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on macOS); automated checks for every
fix; recordings diagnosis-only; real numbers on controls (no hidden multipliers).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI via
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`.

**Blockers:** none.
