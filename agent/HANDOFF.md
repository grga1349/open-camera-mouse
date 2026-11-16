# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6d part 3: #13 → steady zone 0.2 + drift cap, committed + pushed)

**Branch:** `rewrite/rust`. The #13 change is committed and pushed (the commit after `cf95cf0`).
**First thing next session: check CI on it** (curl below); the full workspace gate wasn't run
locally (only pointer-core, app-core, desktop tests + scoped clippy).

**Done this session (#13):** `docs/research/ZONE_VARIANTS.md` — zero shake with ≤ 0.15 cam px
hold error is infeasible even non-causally (S1 bound 0.25 at 0.2 reversals/s); user chose
"accept a little shake, closest to a real mouse". Sweep with a visible-shake metric (≥ 2 screen
px swings) → device check 0.2/0.25/0.35 → user picked **0.2**. Shipped:
`STEADY_ZONE_CAM_PX` 0.5 → 0.2; `pointer_core::limits::drift_offset(…, past)` caps the 2018
drift at the head's overshoot (the smaller zone exposed a 5–18 px saw off screen edges). ADR-0015
/-0017 amendments, TEST_DRIVE #13 "Improved". pointer-core / app-core / desktop tests green, fmt,
scoped clippy, core-deps clean. Full workspace gate + CI not yet run on this change.

**Next session — left over, in this order (ask the user which slice first):**
1. **CI check** of the #13 commit (3 OS); if `test (no webcam)` fails, it may be #7 — get the
   test name.
2. **#13 device follow-ups** (user, quick): the cursor stays put on an edge / menu bar while
   the head is held past it (drift cap); wobble at zone 0.2 at high gain (e.g. 40–60) and with
   the 2560 px monitor as Main (visible wobble scales with gain × Main width). If it shows:
   re-sweep with `replay_zone_sweep` at that scale.
3. **#13 open decision:** lower the default gain 22 (`ZONE_VARIANTS.md` §4.2; only lever that
   cuts on-screen error without shake) — user's call, ADR-0017 amendment if changed.
4. **#12 jumps:** tag takeover recentres as a jump cause, CI test that a ≤ 5 px echo never
   triggers move-stop; ask if jumps follow touching the mouse/trackpad.
5. **#14** "Main monitor" default = OS primary (ADR-0014 amendment, user OK; platform must
   report which display is primary).
6. Then step 3 sweep / step 4 calibration (task 0011 objective).
One slice per session.

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
