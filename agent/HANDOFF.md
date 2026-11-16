# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6d part 3 queue done: #13 gain 20, #12 closed, #14 primary monitor)

**Branch:** `rewrite/rust`, pushed. CI: check the run for this commit (last known green `304de65`).

**Done this session:**
- **#13 default gain 22 → 20** (`replay_gain_sweep`, `ZONE_VARIANTS.md` §6; user tried 22/18/16
  on device across two screens, 20 because most users have one monitor). ADR-0017 amendment.
  `a_slow_move_onto_an_edge_does_not_jump` now derives its end point from the scale.
- **#12 closed:** user reports no jumps any more.
- **#14 Main monitor = OS primary:** `app_core::MainMonitor { Primary (default), All, Display(n) }`
  (TOML `"primary"` / `"all"` / index; pre-#14 files → Primary). `Desktop::primary` from the
  adapters: macOS `CGMainDisplayID`, Windows monitor at (0, 0), X11 RandR primary. Controller
  `primary_display()` / `main_display()`, origin fallback, then centre of all. Options →
  Screen → "Primary monitor" / "Centre of all monitors" / each monitor. ADR-0014 amendment.
  Rest tests pin `MainMonitor::All` (largest scale = strictest).
- Gates: app-core, platform lib, desktop tests green; fmt; clippy (app-core, platform, desktop;
  platform also for the Windows target); core-deps OK. Linux can't be checked locally
  (v4l2 headers) → CI.

**Next session:**
1. If CI is red (esp. Linux `pointer/linux.rs` RandR change), fix first.
2. **Device check #14** (2 monitors): fresh/default settings → recenter lands on the menu-bar
   display; Options shows "Primary monitor" and "· primary" on the right monitor. The user's own
   saved file keeps gain 22 and their old Main monitor choice.
3. Then task 0011 **step 3 sweep / step 4 calibration** (the phase objective). One slice per session.

**Corpus (user decision, no new recordings):** `~/ocm-recordings/S1-still-2026-11-15`,
`S2-turns-2026-11-15`. Replay tool `~/ocm-recordings/_tools/replay-tool`; run with
`OCM_REPLAY_DIRS=<S1>:<S2> cargo test --test replay <name> -- --ignored --nocapture`. Tests:
`replay_metrics`, `replay_2018`, `replay_nakf`, `replay_zone_variants`, `replay_hold_bound`,
`replay_zone_sweep` (+ `visible_shake`);
harness `emulate_ours` / `emulate_chain` (swap the zone stage).

**Carried in:** #7 macOS CI flake (need the test name from run 43773727473's log); #4
main-monitor check, minimised rate, dwell on device (from 6b), `ui` 15/17 ms with Options open,
ADR-0018 slow-creep limit.

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
