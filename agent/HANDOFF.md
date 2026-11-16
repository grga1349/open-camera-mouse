# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6d part 3: NAKF vs dead zone done, NAKF loses; 3 slices left)

**Branch:** `rewrite/rust`. CI on `8303dd7` (edge-jump fix) green. Last commit `4f771ea` (docs).

**Done this session (part 3, item 0b):** `docs/research/NAKF_VS_STEADY_ZONE.md`. Noise-adaptive
CV Kalman (adaptive r from second differences, q raised by innovation NIS) prototyped in the
replay tool (`replay_nakf`). **No setting reaches ≤ 0.2 reversals/s at rest** (best 2.4/s,
residual then 0.50–0.55). Under zone 0.5 instead of 1€ + EMA: lag 70 vs 110 ms, residual 0.35
vs 0.45, but shake 0.30 vs 0.10/s and S2 step max 217 vs 184 px (CV overshoot). Cause: rest
noise is low-frequency head motion (adaptive r hits its floor 0.002 cam px²), so a hold is
needed. Recommendation: keep 1€ + EMA + zone, no NAKF option. `emulate_ours` in the replay
tool = today's chain outside the controller, validated (S1 zone 0.5: 0.10 rev/s, residual 0.43
vs controller 0.45), ready as the harness for zone variants. No production code changed.

**User decision:** no new recordings; corpus = `~/ocm-recordings/S1-still-2026-11-15` and
`S2-turns-2026-11-15`. Replay tool: `~/ocm-recordings/_tools/replay-tool` (tests
`replay_metrics`, `replay_2018`, `replay_nakf`, run with `OCM_REPLAY_DIRS=<S1>:<S2>`;
`OCM_NAKF_GRID="q,γ,thr[,r];…"` overrides the NAKF grid).

**Next session = #13 zone variants (user decision 2026-11-16):** variants from
`CM2018_JITTER_COMPARISON.md` §5.1 that cut the zone's hold error (0.45 cam px) while keeping
shake ≤ 0.2 reversals/s at rest, lag ≤ 80 ms, residual ≤ 0.15 cam px, S2 step max ≤ 184 px.
Measure in the replay tool's `emulate_ours` harness first (add variants beside `replay_nakf`),
then prototype in `pointer_core::steady` behind a test switch; default change = user OK +
ADR-0015/-0017 amendment + CI test at gains 1–60 + device check.

**After that (ask which first):**
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
