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
- 2026-11-16 step 5 (first fix, user: "fix the edge jump"; ahead of step 3 by request): user
  says the jumps were at the edges ("not 100% sure"). Synthetic repro
  `a_slow_move_onto_an_edge_does_not_jump` (slow creep onto the right edge at the defaults):
  33.0 px step = the zone radius before. A catch-up limited by the push past the edge was
  tried first and rejected: 2018 offset drift (≈ 0.47 cam px per frame) pulls the target off
  the edge at once, so the cursor stopped 29 px short. Fix: `steady::approach_edge` (trail ≤
  `EDGE_TRAIL` × target's distance to the limit), `limits::bounds`. Factor chosen on S2 58.54 s
  (last steps onto the edge, px): zone off 26/30/27/20; snap …/44; 1.0 26/30/32/39; 0.5
  26/31/41/29; 0.25 31/37/34/25. Replay after: 58.54 s no longer flagged; S1 unchanged. The
  30.88 s flag (very fast move onto the left edge, ~2000 px/s) is filter catch-up: 0.50 cam
  px even at zone 0, 0.79 at 0.5 (steps 72/71/38 vs zone-off 69/57/30). **Metric note for
  step 3:** `unexplained_jumps` compares against the marker step, which over-flags smoothing
  catch-up on fast moves; add a reference against the zone-off cursor step. Tests: pointer-core
  51, app-core 51 + 10 green; clippy (`-p pointer-core -p app-core`), fmt, core-deps clean.
  Device check: edges looked fine, but the user is not sure the jumping is fixed (#12 open).

## Queued for part 3 (user, 2026-11-16)

0. **DONE 2026-11-16 → `docs/research/CM2018_JITTER_COMPARISON.md`.** Verdict: 2018 has no
   jitter solution (0.95 damping ≈ none, no zone; emulated on S1 it shakes 6.8 reversals/s vs
   ours 0.1). Differences that matter: D1 sensitivity (gain 22 × Main = 1.3–3.7× 2018) and D4
   the steady zone (+40 ms lag, ≈ 0.45 cam px hold error = main #13 candidate). S2's biggest
   step (34.2 s) is a real fast turn, larger in 2018. #12 candidate left: takeover-to-centre
   after a false move-stop (D7). Proposed slices in §5 of the doc. Original brief: **2018 jitter comparison.** Go through
   `../camera-mouse-reverse/` and check (a) whether we do something differently from 2018 that
   could cause jitter or the other symptoms (#12 jumps, #13 imprecise) — tracking, offset
   drift, mapping, smoothing, dead zone, timing — and (b) whether 2018 has its own jitter
   solution. Behavioural research only: label findings VERIFIED/INFERRED, never paste
   decompiled code; write results into `docs/research/` (and RE_EVIDENCE where it fits) and
   turn any proposed fix into a measured slice (ADR-0020 rules).
0b. **DONE 2026-11-16 → `docs/research/NAKF_VS_STEADY_ZONE.md`. Verdict: NAKF loses.** No
   setting reaches ≤ 0.2 reversals/s at rest (best alone 2.4/s, with residual 0.50–0.55). As
   1€ + EMA replacement under zone 0.5: lag 70 vs 110 ms, residual 0.35 vs 0.45, but shake
   0.30 vs 0.10/s and S2 step max 217 vs 184 px (CV overshoot). Rest noise is low-frequency
   head motion (adaptive r hits its 0.002 floor); a hold is needed → #13 continues with zone
   variants (§5.1). Replay tool: `replay_nakf`, `emulate_ours` (validated vs controller).
   Original brief: **Noise-Adaptive Kalman Filter (NAKF) vs dead zone.** Evaluate a
   noise-adaptive Kalman filter (e.g. constant-velocity state, measurement/process noise
   estimated online from the tracker's innovations) as a replacement for the steady zone,
   and possibly for the 1€ filter too (one filter instead of 1€ + EMA + zone). Measure first
   on S1/S2 with the replay tool (`replay_2018` table + `replay_metrics`) against today's
   pipeline: reversals/s and jitter rms at rest, residual (hold error), lag, S2 step max.
   Target from `CM2018_JITTER_COMPARISON.md` §5: reversals/s ≤ 0.2 at rest, residual
   ≤ 0.15 cam px, lag ≤ 80 ms. Prototype in the replay tool / `pointer-core` behind a test
   switch; a default change needs the user's OK + ADR (new ADR or ADR-0017 amendment), a CI
   test on calibrated signals at every gain 1–60, and a device check. This supersedes the
   zone-only variants in §5.1 of the doc unless NAKF loses.
1. **#13 the mouse is imprecise** — triage first (zone trail/creep, filter lag, tracker noise).
2. **#12 not sure the jumping is fixed** — the edge snap is fixed; look for other causes
   (fast-move filter catch-up at S2 29–34 s, handover/re-acquisition, display snap).
3. **#14 "Main monitor" defaults to the primary monitor** (ADR-0014 amendment + user OK).
4. **DONE 2026-11-16 → `docs/research/OPENCV_LK_PARITY.md`.** Verdict: competitive, no
   parity defects, no production change recommended (min_eigen ×1024 mapping to document;
   optional perf experiments only). ADR-0009 21×21 now VERIFIED. **Research: Rust LK vs OpenCV numerical parity** — the user's brief, verbatim:

> # Research task — Rust LK vs OpenCV numerical parity
>
> We have a pure-Rust pyramidal Lucas–Kanade optical-flow implementation in `crates/tracking-core/`.
>
> Compare it against OpenCV 4.12.0 `modules/video/src/lkpyramid.cpp`, while distinguishing modern OpenCV behavior from OpenCV 2.1 used by Camera Mouse 2018.
>
> ## Investigate
>
> 1. Numerical accuracy of f32 bilinear interpolation versus OpenCV fixed-point interpolation.
> 2. Scharr derivative scaling and minimum-eigenvalue threshold normalization.
> 3. Gaussian pyramid differences, including rounding and border handling.
> 4. OpenCV's iterative oscillation detection and half-step correction. Determine whether our solver would benefit.
> 5. Boundary behavior and failure conditions.
> 6. Effects of our gain/bias compensation on LK convergence.
> 7. Potential SIMD or patch-level optimizations relevant to tracking only ONE point.
> 8. Whether the original OpenCV 2.1 C API's `winSize=(10,10)` is equivalent to our 21×21 patch. Verify from historical source, not assumptions.
>
> ## Benchmark requirements
>
> Compare on identical grayscale frames with identical selected points.
>
> Run both:
> - Pure LK baseline, with additional validation disabled.
> - Our recommended tracker, measured separately as a complete tracking system.
>
> Use synthetic scenes with ground truth and real diagnostic recordings when available. Real recordings must remain local.
>
> Report:
> - Median/P95/max endpoint error.
> - Static-point jitter and long-term drift.
> - Lost tracking and confident wrong tracking.
> - Performance p50/p95, CPU and allocations.
> - Sensitivity to texture, lighting, rotations, occlusion and image borders.
>
> Compare at equal window, pyramid and convergence settings wherever meaningful.
>
> ## Rules
>
> - No OpenCV dependency in the shipped Rust application.
> - Do not change production code during research.
> - Do not assume OpenCV is numerically superior.
> - Do not blindly copy OpenCV code.
> - Distinguish parity defects from intentional product-level improvements.
> - For every proposed improvement, require an isolated experiment and a measurable benefit.
>
> ## Deliverable
>
> `docs/research/OPENCV_LK_PARITY.md`
>
> Include exact source references, observed differences, experiment results, recommended actions ranked by impact, and a clear conclusion about whether our solver is competitive with OpenCV.
