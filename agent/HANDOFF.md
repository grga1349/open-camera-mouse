# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6d part 3: 2018 jitter comparison done; 3 slices left)

**Branch:** `rewrite/rust`. CI on `8303dd7` (edge-jump fix) green. Last commit `2293676` (docs).

**Done this session (part 3, slice 0):** `docs/research/CM2018_JITTER_COMPARISON.md`.
2018 has **no jitter solution** in the inspected path (0.95 damping ≈ no smoothing, no zone, 14 Hz
tick). Emulated on our tracker (replay tool test `replay_2018`, local only), S1 at rest: 2018
6.8 reversals/s, 6.9 px rms; ours (app) 0.1 reversals/s, but lag 110 ms vs 70 ms zone-off and
residual 0.45 vs 0.12 cam px → **the steady zone's hold error is the main #13 candidate**.
Sensitivity: gain 22 × Main = 1.3× (x) / 2.3× (y) 2018 on the user's desktop, 3.7× on a single
1080p screen (D1, user's choice). S2's largest step (34.2 s) is a real fast head turn, larger
in 2018. #12 candidate left: takeover-to-centre after a false move-stop (D7, S1 has one
Camera→Handover at 22.35 s, cause unknown). Earlier: `OPENCV_LK_PARITY.md` (tracker = OpenCV).

**User decision:** no new recordings; corpus = `~/ocm-recordings/S1-still-2026-11-15` and
`S2-turns-2026-11-15`. Replay tool: `~/ocm-recordings/_tools/replay-tool` (tests
`replay_metrics`, `replay_2018`, run with `OCM_REPLAY_DIRS=<S1>:<S2>`).

**Next session = NAKF vs dead zone (user, task 0011 queue item 0b):** compare a
Noise-Adaptive Kalman Filter against the steady zone, and whether it can also replace the 1€
filter (1€ + EMA + zone → one filter). Measure first on S1/S2 with the replay tool
(`replay_2018` + `replay_metrics`), targets: reversals/s ≤ 0.2 at rest, residual ≤ 0.15 cam px,
lag ≤ 80 ms, no larger S2 steps. Prototype behind a test switch; default change = user OK +
ADR + CI test at gains 1–60 + device check. Research/measure slice first, then ask.

**After that (ask which first):**
1. #13 imprecise — if NAKF doesn't solve it: zone variants from the doc §5.1.
2. #12 jumps — tag takeover recentres as a jump cause, CI test that a ≤ 5 px echo never
   triggers move-stop; ask the user if jumps follow touching the mouse/trackpad.
3. #14 "Main monitor" default = the OS primary monitor (ADR-0014 amendment, user OK; platform
   must report which display is primary).
Then step 3 sweep / step 4 calibration as before. One slice per session.

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
