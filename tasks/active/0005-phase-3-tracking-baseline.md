# 0005 — Phase 3: tracking baseline

Status: ACTIVE (started 2026-11-12)
Owner/branch: `rewrite/rust`
Depends on: 0004 (done)

## Context

Phase 3 per `docs/ROADMAP.md`. Inputs: BEHAVIOR_SPEC §2.4 (2018 pipeline + loss policy),
§3.4 (legacy NCC), §7 R-T1..R-T6, §8 (fixtures, metrics, performance); ADR-0004 (both
candidates, measured), ADR-0006 (LK presumptive baseline; 35 px guard; symmetric edge
predicate; init from selection frame), ADR-0008 (canonical width ≤ 640). Skills:
`computer-vision`, `tracking-validation`, `benchmarking`, `rust-testing`.

## Objective

1. `tracking-core` image primitives: f32 image, bilinear sampling (pixel centres at integer
   coordinates, clamp-to-edge), Gaussian pyramid (5-tap binomial, 2× decimation), Scharr
   gradients. Unit tests with hand-computed values; no NaN/Inf on degenerate input.
2. **B-LK18**: pure-Rust pyramidal LK, one point, 2018 preset (window 10 = half-size,
   INFERRED from OpenCV 2.1 C API; max level 3; 20 iter / ε 0.03), initialised from the
   selection frame (R-T3), 2018 loss policy with the *symmetric* edge predicate (R-T4),
   outlier guard → `Held(MotionOutlier)`, explicit statuses (R-T2).
3. **B-NCC**: pure-Rust fixed-template ZNCC (TM_CCOEFF_NORMED), legacy preset: template 45,
   search ±2×template around the last good centre, threshold 0.68, integer output, template
   never updated, per-frame `Lost(LowConfidence)` while keeping the template (legacy §3.4).
4. Synthetic fixture generator (seeded, own PRNG, ground truth from the transform, never
   from a tracker) for the §8 corpus at 320×240, 640×480, 1280×720 (downsampled to 640 per
   ADR-0008 where relevant).
5. Metrics harness (EPE mean/median/p95/max px and % width, % tracked, false-`Tracking`
   during occlusion, frames to detect loss, drift) run via `cargo test`, results recorded in
   `docs/research/TRACKING_BASELINE.md`.
6. Timing harness: `divan` benches (with its allocation profiler) for gray→f32, pyramid,
   per-point LK, NCC search, end-to-end step at 3 resolutions; results + device metadata in
   the same doc.
7. Decide Q-R1 (units of motion bounds) in an ADR.

## Non-goals

Phase 4 improvements (forward-backward, adaptive search, re-detection, multi-point). No
recorded real head-motion sequences yet (needs camera adapter; note as gap). No app-core
wiring beyond what tests need. No tracking-cli.

## Files/scope

`crates/tracking-core/` (src, tests, benches), workspace `Cargo.toml` (dev-deps),
`docs/research/TRACKING_BASELINE.md`, new ADR, BEHAVIOR_SPEC §9 Q-R1, `DEPENDENCIES.md`.

## Acceptance criteria (ROADMAP Phase 3 gate)

- Reproducible baseline metrics for both comparators on the same corpus, from one command.
- Correctness tests (unit + fixture) pass; expected failures (textureless, full occlusion)
  are asserted as failures, not hidden.
- No superiority claim without a measured delta on the same corpus; limitations stated.
- Q-R1 decided. `tracking-core` still passes `check-core-deps.sh` (dev-deps only).

## Verification

`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`;
`cargo test --workspace`; `cargo test -p tracking-core --release --test baseline -- --ignored --nocapture`
(prints the metric tables); `cargo bench -p tracking-core`; CI green on 3 OS.

## Progress (2026-11-12)

Done locally, uncommitted:
- `tracking-core`: `image` (f32, bilinear, binomial pyrDown, Scharr), `pyramid`, `lk`
  (B-LK18), `ncc` (B-NCC), `frame::downsample_to_width` (ADR-0008).
- `tests/support` (seeded generator, 21 scenes, metrics), `tests/fixtures.rs` (8 correctness
  tests incl. expected failures), `tests/baseline.rs` (ignored; full tables), `benches/tracking.rs`.
- ADR-0009 (Q-R1: fraction of width; LK preset semantics; `min_eigen` deviation).
- `docs/research/TRACKING_BASELINE.md`: tables, 9 findings, performance, limitations.
- Local evidence (M1): fmt ok; clippy `-D warnings` ok; `cargo test --workspace --locked`
  43 passed / 0 failed / 1 ignored; `check-core-deps.sh` ok; full baseline ran twice,
  unchanged rows byte-identical; `cargo bench -p tracking-core` ran.
- Two generator bugs found from implausible rows and fixed (edge_bottom reach on 4:3;
  motion-blur centring). LK unit-test pattern had no low frequencies; replaced.

Gate status: met except "CI green on 3 OS" (needs push). Not met (recorded, not a gate):
LK p95 < 2 ms at 640 (4.1 ms, pyramid-bound) — PROPOSED target, carried to Phase 4.
