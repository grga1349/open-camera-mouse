# 0011 — Phase 6d: calibrated pointer behaviour

Status: ACTIVE (started 2026-11-16)
Owner/branch: `rewrite/rust`
Depends on: 0010 (done)

## Context

Phase 6d per `docs/ROADMAP.md` and ADR-0020. Build under test starts at `d7728b5` (6c
close-out). Two open problems: jitter at rest vs the cost of the steady zone (#11, carried in),
and the cursor jumping (#12). Synthetic pointer tests passed while the device still jittered,
because their noise is hand-picked, not fitted to real heads.

Inputs: ADR-0020 (this phase), -0015 (steady zone), -0017 (filter/scale, defaults gain 22 /
smoothing 0.20), -0016 (engine thread), -0018 (anchored gate); `docs/qa/TEST_DRIVE.md` (#7,
#11, #12); `docs/qa/RECORDING_PROTOCOL.md`; `scripts/record-scenario.sh`; the local replay
tool `~/ocm-recordings/_tools/replay-tool` (outside the repo). Skills: `tracking-validation`,
`rust-testing`, `benchmarking`, `code-review`.

Evidence carried in (S1 replay, 14.5 s still, moves / reversals by camera-px zone, gain 24):
no zone 311/6, 0.2 → 169/5, 0.5 → 108/2, 1.0 → 59/1; same pattern at gain 8–60. Shake is
already low after the 1€ filter; most moves are slow drift (1–2.5 cam px over seconds).

Carried in: #7 macOS CI flake (failing test name needs the job log of run 43773727473, which
the unauthenticated API doesn't serve — ask the user or catch the next occurrence); #4
main-monitor check, minimised rate, dwell on device (from 6b); `ui` 15/17 ms with Options
open; ADR-0018 slow-creep limit.

## Objective (ADR-0020 order; slices planned one at a time)

1. **Corpus.** Fresh recordings of S0–S3, plus new scenarios if needed (precise slow moves,
   jump reproduction), ≈ 2 min per ask. Replay tool updated first (`steady_px` → 
   `Controller::set_steady_zone_cam_px`).
2. **Metrics.** One definition each for shake, creep, jumps, lag, precision, computed the same
   way on replays and synthetic runs (shared code in the repo; the replay tool calls it).
3. **Settings sweep.** Every recording × a grid (gain, smoothing, steady zone, 1€ params,
   candidate fixes); table in this file.
4. **Calibrated synthetic signals.** A few fitted numbers (tremor amplitude/spectrum, drift
   rate, tracker noise, jump events) reproducing the recorded metrics within a stated
   tolerance; CI tests at every gain 1–60.
5. **Fixes.** Jitter with the smallest dead zone (possibly separating shake from drift) and
   the jump causes; each with a CI test, a replay metric and a device check.

## Non-goals

- New features unless asked; then an option, off by default unless approved.
- Recordings or anything derived from them other than fitted numbers in the repo.
- Phase 7/8 work (packaging, Windows/Linux on device, reduced motion, threshold re-measure).

## Acceptance criteria (gate)

- Shake 0 at rest and no unexplained jumps on the recordings and on calibrated CI tests at
  every gain 1–60.
- The chosen steady zone is the smallest that achieves it, with the sweep table below.
- Every 6d row in `docs/qa/TEST_DRIVE.md` fixed or deferred with a reason; changed defaults
  have the user's OK and an ADR amendment.
- Quality gates green locally and in CI on 3 OS.
- The user says the app is ready for Phase 7.

## Verification

```
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/check-core-deps.sh
```

## Metrics (step 2)

`pointer_core::metrics` (pure, unit-tested): `rest` → creep (`moves`, `span`) and shake
(`reversals` within 250 ms, `hf_rms` after a 15-frame centred mean), `unexplained_jumps` /
`max_unexplained` (cursor step beyond the tracker's own step), `jumps` / `step_stats` (raw
steps), `lag_ms` (velocity cross-correlation vs an unfiltered run), `reached` (precision probe
rule, ≥ 50 % of a deliberate move). All in camera px. The replay tool's `replay_metrics` and
the CI rest tests (`the_cursor_does_not_move_at_rest_*`) call them.

Baseline (replay, desktop as recorded: 1440×900 laptop Main + 2560×1440; camera-controlled
frames after selection + recenter; S1 cut at 23.5 s):

| rec | variant | steps | moves | reversals | hf rms | span | step max | max unexplained | lag ms |
|---|---|---|---|---|---|---|---|---|---|
| S1 | defaults, zone 0 | 299 | 299 | 39 | 0.064 | 2.73 | 0.12 | 0.09 | 0 |
| S1 | defaults, zone 0.5 | 299 | 102 | 1 | 0.036 | 1.76 | 0.11 | 0.07 | 47 |
| S1 | defaults, zone 1.0 | 299 | 57 | 0 | 0.025 | 1.00 | 0.09 | 0.05 | 57 |
| S2 | defaults, zone 0 | 1084 | 1084 | 168 | 0.683 | 50.75 | 3.73 | 0.83 | 19 |
| S2 | defaults, zone 0.5 | 1084 | 727 | 41 | 0.687 | 50.26 | 3.73 | 0.83 | 28 |
| S2 | defaults, zone 1.0 | 1084 | 646 | 30 | 0.691 | 49.76 | 3.73 | 1.42 | 37 |

Tracker marker alone: S1 step p99 0.25 / max 0.31 cam px, 176 reversals, hf 0.114, span
3.26; S2 step p99 2.26 / max 4.45. Reading: at 0.5 cam px shake is ≈ 0 (1 reversal) but creep
is 102/299 frames: drift, not shake. S2 note: the defaults run has 1084 controlled steps vs
1563 as recorded (smoothing 0.05, gain 6) — check why in step 3 (loss/handover difference?).

## Sweep table

(filled by step 3)

## Progress

- 2026-11-16: task file written. CI on `d7728b5`: macOS (no flake), cargo-deny green;
  Windows/Linux still running at writing. Replay tool ported to
  `Controller::set_steady_zone_cam_px` (`bench_run_zone`, sweep in camera px, skips the
  selection + initial recenter). S1 baseline (recorded smoothing 0.05, gain-invariant 4–60):
  0.5 cam px → 99/300 rest frames move; 0 moves needs 2.24 cam px (drift). Waiting on the
  user's #12 report and the recordings.
- 2026-11-16: **user decision: no new recordings — the corpus is the existing S1 (still) and
  S2 (turns) from 2026-11-15.** S0/S3 and jump/precise-move scenarios are not recorded; #12
  is looked for in S1/S2 replays, precision is measured on calibrated synthetic runs.
- 2026-11-16 step 2 (metrics): `pointer_core::metrics` + 8 unit tests; rest tests ported to
  it (`reached` = 50 %, was 40 %); replay tool `replay_metrics`, `replay_around`. Baseline
  above. #12 lead: steady-zone edge snap (TEST_DRIVE #12). `cargo test -p pointer-core -p
  app-core` green, fmt clean, clippy (`-p pointer-core -p app-core`) clean; committed.
  CI on `d7728b5`: Windows `test (no webcam)` failed, exit 1 (#7), others green.
