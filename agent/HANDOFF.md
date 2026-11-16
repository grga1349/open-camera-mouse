# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6d CLOSED → Phase 7 next)

**Branch:** `rewrite/rust`, pushed. Last code `687a9fc`; CI run 43797400148 was in progress at
close. **First thing: confirm it's green** (the Linux RandR change in `pointer/linux.rs` can't be
compiled on the Mac).

**Phase 6d closed (task 0011 → `tasks/done/`, ROADMAP 6d DONE):** zone 0.2 cam px + drift cap
(#13), default gain 20 (#13), #12 closed (user: no jumps), #14 Main monitor = OS primary
(device-checked OK by the user). User: ready for Phase 7; the planned 6d steps 3–4 (grid sweep,
calibrated synthetic signals) are obsolete — don't resurrect them.

**Next session: start Phase 7 — multiplatform / release** (`docs/ROADMAP.md` §7). Write the task
file (0012) from the roadmap's deliverables and gate; don't assume a pre-written one. Carry-overs:
MANUAL_QA rows 19–20 (`.app` permissions), TEST_DRIVE #7 CI flake (needs the job log, ask the
user), Windows/X11 pointer + primary-monitor detection on device, external webcams (P1), Wayland
limits (P10/P11). Phase 8 keeps: MANUAL_QA row 21 (reduced motion), `OCM_RECORD` threshold
re-measurement, ADR-0018 slow-creep limit.

**Corpus (6d, kept for reference; user: no new recordings):** `~/ocm-recordings/S1-still-2026-11-15`,
`S2-turns-2026-11-15`. Replay tool `~/ocm-recordings/_tools/replay-tool`; run with
`OCM_REPLAY_DIRS=<S1>:<S2> cargo test --test replay <name> -- --ignored --nocapture`. Tests:
`replay_metrics`, `replay_2018`, `replay_nakf`, `replay_zone_variants`, `replay_hold_bound`,
`replay_zone_sweep` (+ `visible_shake`), `replay_gain_sweep`;
harness `emulate_ours` / `emulate_chain` (swap the zone stage).

**Also open from 6b/6c (fold into Phase 7/8 as fits):** minimised rate and dwell on device,
`ui` 15/17 ms with Options open.

**Rules (ADR-0019/-0020):** measure first; changed defaults need the user's OK + ADR amendment;
every fix gets a CI test / replay metric / `OCM_PERF` + device check; recordings diagnosis-only,
only fitted numbers enter the repo; one slice per session.

**Machine load:** one heavy command at a time, scoped (`-p`, test filter), say so first;
release builds: ask first. `app-core` tests ≈ 12 s (rest test split in 4 parallel parts).

**How the user runs the app:** `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (dev build). Try a zone: `OCM_DEV_STEADY=0.25 cargo run -p open-camera-mouse`.

**Known, not bugs:** Searching animation ignores Reduce Motion (Phase 8); font egui default;
F10 not F11 on macOS; re-detection off by default; no auto-recenter.

**User direction (don't drift):** start from 2018 but improve; new behaviour as options, off by
default unless approved; monochrome UI, neon green ring marker, yellow primary buttons,
Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on macOS); automated checks for every
fix; recordings diagnosis-only; real numbers on controls (no hidden multipliers).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI via
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`.

**Blockers:** none.
