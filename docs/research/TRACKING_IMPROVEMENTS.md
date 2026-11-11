# Tracking improvements (Phase 4)

Ablations against B-LK18 (`LkTracker::default()`) on the Phase 3 corpus. Task 0006.
Baseline numbers, corpus and metric definitions: `TRACKING_BASELINE.md`. Every experiment is
kept or rejected on measured deltas from the same corpus. Synthetic-only caveats from Phase 3
still apply.

## How to reproduce

```sh
# exact LK output fingerprint (must be unchanged by "no behavior change" work)
cargo test -p tracking-core --release --test baseline lk_fingerprint -- --ignored --nocapture
# accuracy tables
cargo test -p tracking-core --release --test baseline baseline_tables -- --ignored --nocapture
cargo bench -p tracking-core
```

The fingerprint hashes every step's position (f32 bits) and status per scene × resolution
with `std`'s `DefaultHasher`. It is stable across runs on one toolchain, not guaranteed
across Rust versions, so compare before/after on the same toolchain.

## E0 — performance, no behavior change: KEPT

**Change.** The pyramid no longer stores full-image Scharr gradient maps for every level.
LK only reads gradients in its window, and only from the previous pyramid. So `track_point`
now computes Scharr for a `(2·hw + 4)²` patch per level (`image::GradPatch`, `scharr_at`).
`pyr_down_into` got interior fast paths that skip reflect-101. Summation order is unchanged.

**Correctness evidence.**
- Unit tests assert bit equality (`to_bits`): `scharr_at` vs `scharr_into` on every pixel
  (including 1×1 and 2×3 images), `GradPatch::sample` vs bilinear sampling of the full maps
  (including windows hanging off a corner), and `pyr_down_into` vs an all-reflect-101 reference.
- `lk_fingerprint`: all 61 scene × resolution hashes identical before and after (M1,
  rustc 1.99.1). Baseline table rows are therefore unchanged.
- Debug-build fixture tests pass with the patch-bounds `debug_assert`s on.

**Timing** (Apple M1, release, divan, 100 samples, medians; Phase 3 numbers in brackets):

| Bench | 320×240 | 640×480 | 1280×720 → 640×360 |
|---|---|---|---|
| pyramid build | 65 µs [1.02 ms] | 248 µs [4.08 ms] | 186 µs [3.11 ms] |
| LK solve, one point | 45.6 µs [35.7 µs] | 45.4 µs [35.7 µs] | 43.6 µs [34.0 µs] |
| **LK end-to-end step** | **111 µs [1.06 ms]** | **296 µs [4.13 ms]** | **232 µs [3.15 ms]** |

The §8 PROPOSED target of LK p95 < 2 ms at 640 is now met with margin (slowest of 100
samples: 320 µs). The one-point solve got ~10 µs slower because it now owns its gradients.
That's the intended trade. Steady-state allocations are still zero: divan's AllocProfiler
reports no allocation rows.

## E1–E6

Not started. See `tasks/active/0006-phase-4-tracking-improvements.md`.
