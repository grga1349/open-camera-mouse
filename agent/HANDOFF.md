# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6d part 2 done: edge-jump fix committed; next = part 3)

**Branch:** `rewrite/rust`. Edge-jump fix committed and pushed (see `git log`); verify its CI.

**User decision:** no new recordings; corpus = `~/ocm-recordings/S1-still-2026-11-15` and
`S2-turns-2026-11-15`.

**Done (part 2):** #12 edge snap fixed — `pointer_core::steady::approach_edge` (`EDGE_TRAIL`
0.25) + `limits::bounds`, ADR-0017 amendment, CI test `a_slow_move_onto_an_edge_does_not_jump`.
User device check: edges fine, but **not sure the jumping is fixed**.

**Next session = part 3 ("trying to fix this stuff"), queued in task 0011 § "Queued for part 3":**
1. #13 the mouse is imprecise — triage first (zone trail/creep, filter lag, tracker noise).
2. #12 still open — other jump causes (fast-move filter catch-up S2 29–34 s, handover,
   display snap). Add a jump reference against the zone-off cursor (marker step over-flags
   filter catch-up).
3. #14 "Main monitor" default = the OS primary monitor (ADR-0014 amendment, user OK; platform
   must report which display is primary).
4. Research: Rust LK vs OpenCV parity → `docs/research/OPENCV_LK_PARITY.md` (brief verbatim in
   task 0011; no production changes during research, OpenCV only outside the shipped app).
Then step 3 sweep / step 4 calibration as before. One slice per session — ask which first.

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
