# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6c closed; next session = start Phase 6d)

**Branch:** `rewrite/rust`, 6c close-out commit pushed. **First: verify CI** (`curl` below).

**Where we are:** Phase 6c (task 0010 → `tasks/done/`) is DONE: #8 real gain 1–60, #9 Options
Screen tab, #10 Text size removed, #11 automatic steady zone 0.5 camera px. Defaults gain 22,
smoothing 0.20 (user, ADR-0017 amendments). The user wants **Phase 6d (ADR-0020)**: the
smallest dead zone that fixes jitter, a fix for jumps, a broad settings sweep on recordings,
and synthetic tests calibrated to the recordings.

**Next session — start 6d:**
1. Verify CI.
2. Write `tasks/active/0011-phase-6d-calibrated-pointer.md` from ADR-0020 + ROADMAP §6d (format:
   `tasks/done/0010-*`), set 0011 ACTIVE in `tasks/BACKLOG.md`. Don't pre-plan slices beyond
   ADR-0020's order.
3. Ask the user for #12 (jumps): when, how big, what they were doing; a recording if possible.
4. Fix the local replay tool first (`~/ocm-recordings/_tools/replay-tool`): it still sets
   `steady_px` (removed) — use `Controller::set_steady_zone_cam_px`. It has
   `replay_steady_sweep` and `replay_dump_path` (S1 path at gain 24 for offline zone sims).
5. Recording asks ≈ 2 min each (`scripts/record-scenario.sh`); add scenarios to
   `docs/qa/RECORDING_PROTOCOL.md` if needed (precise moves, jumps).

**Evidence so far (S1 replay, 14.5 s still, moves / reversals by camera-px zone):** gain 24: no
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
