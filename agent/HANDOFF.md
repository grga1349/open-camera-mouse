# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6b closed; next session = start Phase 6c, tuning round)

**Branch:** `rewrite/rust`. Last code `df4c670` (slice 2d, ADR-0018); 6b close-out and this
handoff after it. **First: verify CI** for the latest push.

**Where we are:** Phase 6b (test drive, task 0009 → `tasks/done/`) is DONE: issues #1–#6 fixed
and device-checked (ADR-0014 main monitor, -0016 engine thread/lag, -0017 zero-jitter
defaults, -0018 anchored appearance gate; the user: "turns hold much better now"). The user
wants **one more round of tests, tuning and fixes before Phase 7**: Phase 6c (ADR-0019, ROADMAP
§6c).

**Next session — start 6c:**
1. Verify CI (`curl` below).
2. Write `tasks/active/0010-phase-6c-tuning-round.md` from ADR-0019 + ROADMAP §6c (context,
   objective, non-goals, gate, verification, progress) and set 0010 ACTIVE in `tasks/BACKLOG.md`.
   Use task 0009 (`tasks/done/`) as the format model. Don't pre-plan slices the user hasn't
   asked for.
3. Ask the user what to tune or what bothered them, log each item in `docs/qa/TEST_DRIVE.md`
   under "Phase 6c" (#7 on), then one item (or small group) per session.

**Carried into 6c:** #4 main-monitor recenter on-device check; minimised pointer rate and dwell
not re-checked on device since the engine thread (ADR-0016; pointer now runs at full rate when
minimised — tell the user); `ui` ≈ 15/17 ms while Options is open (likely vsync wait, cursor
unaffected); ADR-0018 limit (very slow hand creep followed down to the 0.3 floor). Unused 6b
plan items, only if tuning calls for them: 2c comparison rows (Kalman / alpha-beta /
velocity-dependent gain / precision mode), 2e research report, virtual-anchor feature cloud.

**Tuning rules (ADR-0019):** measure first; a changed default needs the user's OK plus an
amendment to the ADR that owns the value (ADR-0006 2018 defaults, -0011 tracker, -0015 steady
zone, -0017 pointer filter/scale, -0018 gate); every change gets an automated check (CI test,
local replay metric, or `OCM_PERF`) and a device check.

**Current defaults the user tunes against:** tracker `LkParams::recommended()` (LK floor 0.25 +
ratio 0.05, gate 0.7 anchored lag 10 / floor 0.3, gain/bias, re-detection when "Find the target
again" is on); 1€ filter (1 Hz, β 0.15, d 4 Hz) in camera space; scale by the Main monitor;
steady zone 5 px. Their settings (`~/Library/Application Support/Open Camera
Mouse/settings.toml`): `steady_px = 5`, `recenter_display = 0`, gain 6, `loss_recovery = true`.

**Local diagnosis (not in repo):** `~/ocm-recordings/S1-still-2026-11-15`,
`S2-turns-2026-11-15` (may be deleted; new ones via `scripts/record-scenario.sh`, keep asks
≈ 2 min). Replay tool: `cd ~/ocm-recordings/_tools/replay-tool && OCM_REPLAY=<dir> cargo test
-q --test replay replay_tune -- --ignored --nocapture` (variant table + each loss);
`replay_recording` + `../an.py` / `../jit.py` for per-frame CSV, loss runs, jitter.

**Machine load:** one heavy command at a time, scoped (`-p`, test filter), say so first;
release builds: ask first.

**How the user runs the app:** `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (dev build, never `--release` for test drives).

**Known, not bugs (don't "fix" without the user):** Searching animation ignores Reduce Motion
(Phase 8); font is egui default; F10 not F11 on macOS; re-detection off by default; no
auto-recenter.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it; new behaviour
as options, off by default unless the user approves a default; monochrome UI, neon green ring
marker, yellow primary buttons, Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on
macOS); automated checks for every fix; recordings diagnosis-only.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI via
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`.

**Blockers:** none.
