# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6b: slice 2b done and pushed; next session = slice 2d)

**Branch:** `rewrite/rust`, task `tasks/active/0009-phase-6b-test-drive.md`. Last commits:
`54386f4` (slice 2b, ADR-0017) and `460de0a` (handoff). **First, check that CI is green for
both** (CI was in progress at handoff; the 2a run was green).

**Done in 2b:** recordings are **diagnosis-only** (user rule, see memory). The repo has no
recordings, no numbers from them and no code that reads them. Findings became synthetic tests.
Defaults (ADR-0017, user-approved): LK floor 0.25 + eigen ratio 0.05; 1€ filter in camera space;
mapping scale by the Main monitor; steady zone **5 px** (edges still reachable). On device the
user said "resolved jittering good part". Their settings: `steady_px = 5`,
`recenter_display = 0` (the 1440-px laptop, so 13.5 screen px per camera px), gain 6,
`loss_recovery = true`.

**Next: slice 2d, tracking continuity (#2 losses on head turns).** See the plan "2d" in the task
file; its `replay_bench` steps no longer apply. Check with synthetic CI tests and local
diagnosis only.
- What's left (S2 replay after the 2b fixes): every loss is `Lost(AppearanceChanged)` during a
  head turn. ZNCC vs the selection slides ≈ 0.85 → under 0.70 (`min_ref_zncc`). Recovery took
  25–72 frames. The gap in CI is `crates/tracking-core/tests/appearance.rs` `turn_extreme`
  (pinned at 31/90 tracked, no wrong lock). `turn_strong` and lighter turns hold at 100 %.
- Goal: hold the target through turns without wrong locks. Raise the `turn_extreme` pin
  (ideally 90/90), keep `wrong_lock` 0 / `false_tracking` 0 on the whole corpus and
  `recommended_tracker_never_tracks_flat_occluded_or_wrong_targets`.
- Candidates (task file 2d, in order): cheap first. That means hysteresis or a degraded "Held"
  state instead of an instant loss at the gate, or an anchored template update (refresh the
  reference only while ZNCC to the original stays high). After that, the virtual-anchor feature
  cloud (ADR before code). Stop condition and "ships as option" rule are in the task file. A
  new default needs the user's OK.
- Write the failing synthetic test before changing the tracker (memory: automated checks).

**Local diagnosis (not in repo):** recordings `~/ocm-recordings/S1-still-2026-11-15`,
`S2-turns-2026-11-15` (the user may delete them, so ask for a new ≈ 1 min turns recording if
they're gone: `scripts/record-scenario.sh S2`). Replay tool `~/ocm-recordings/_tools/replay-tool`
(scratch crate, path deps on this repo): `cd ~/ocm-recordings/_tools/replay-tool &&
OCM_REPLAY=<dir> OCM_REPLAY_OUT=<csv> cargo test --test replay replay_recording -- --ignored
--nocapture`; then `python3 ../an.py <csv>` (loss reasons, runs) and `../jit.py`.

**Open from 2a (not blocking):** `ui` pass ≈ 15/17 ms while Options is open (likely vsync wait
of the immediate viewport; cursor unaffected) — revisit only if the UI feels slow. Minimised
rate and dwell not re-checked on device. Tell the user if relevant: pointer now runs at full
rate while minimised (was ~10 Hz).

**Machine load (user complaint 2026-11-15):** one heavy command at a time, scoped (`-p`,
test filter), say so first; cap local replays with `OCM_REPLAY_FRAMES`. A release build is a "heavy"
command — ask first.

**How the user runs the app:** dev build, `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (never `--release` for the test drive). Their settings:
`~/Library/Application Support/Open Camera Mouse/settings.toml`.

**Known, not bugs (don't "fix" without the user):** Searching animation ignores Reduce Motion
(Phase 8); font is egui default; F10 not F11 on macOS; re-detection off by default; no
auto-recenter.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it; new behaviour
ships as options, off by default (ask before changing a default; ADR-0017's defaults were
user-approved); monochrome UI, neon green ring
marker, yellow primary buttons, Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on
macOS); wants as much as possible checkable by automated tests. Recordings: diagnosis only, keep asks
≈ 2 min.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI status via
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
(job logs need auth; failing step names via `.../actions/runs/<id>/jobs`).

**Blockers:** none.
