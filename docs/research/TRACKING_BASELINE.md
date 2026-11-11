# Tracking baseline (Phase 3)

First measured numbers for the two ADR-0004 comparators on the BEHAVIOR_SPEC §8 synthetic
corpus. Task 0005. **These are synthetic-fixture results only.** No recorded real head-motion
sequence exists yet (needs the Phase 6 camera adapter), so nothing here proves real-world
superiority. A row is a claim only about that fixture.

## How to reproduce

```sh
# accuracy tables (~5 min on an M1; one scene: BASELINE_SCENE=head_sway)
cargo test -p tracking-core --release --test baseline -- --ignored --nocapture
# timing + allocations
cargo bench -p tracking-core
# fast correctness subset (runs in normal CI)
cargo test -p tracking-core --test fixtures
```

Code: `crates/tracking-core/src/{lk,ncc,image,pyramid}.rs`; generator and metrics in
`crates/tracking-core/tests/support/mod.rs`; harness `tests/baseline.rs`; benches
`benches/tracking.rs`.

## Comparators

| | B-LK18 (`LkTracker::default()`) | B-NCC (`NccTracker::default()`) |
|---|---|---|
| Algorithm | 1-point pyramidal LK (Bouguet), bilinear, Scharr | Fixed-template ZNCC (`TM_CCOEFF_NORMED`), exhaustive search |
| Preset | 21×21 window, levels 0..=3, 20 iter / ε 0.03, λ_min/|W| ≥ 1.0, guard 35/640·W, edge margin 10/640·W (ADR-0009) | template 45 px, search ±90 px, threshold 0.68 (legacy) |
| Init | from the selection frame (R-T3) | template cut from the selection frame, never updated |
| On failure | 2018 policy: near edge & moving out → `Held`; else `Lost`, reference dropped until re-select | per-frame `Lost(LowConfidence)`, keeps template, retries next frame (legacy) |
| Output | sub-pixel | integer pixel centres |

## Corpus and metrics

Generator: seeded fractal value-noise textures. A 45-unit-radius textured target disc
(the "head") over a textured background. The selected point is 12/−8 units off the disc
centre, so rotation and scale move it. Units are 1 = 1 px at 640 wide, so each fixture is
the same motion as a fraction of width at every resolution. 1280×720 is box-downsampled to
640×360 before tracking (ADR-0008), and its ground truth is mapped the same way. Sensor noise
σ = 2 unless stated. Ground truth comes from the scene transform only.
`generator_truth_matches_rendered_pixels` checks it against the rendered pixels for
translation.

Metric definitions (`support::metrics`; EPE in tracker pixels):
- **ok**: visible frames with status `Tracking` and EPE ≤ 2 % of width, over visible frames.
- **EPE med/p95/max**: over visible frames whose status is `Tracking`.
- **wrong-lock**: visible frames reported `Tracking` but > 2 % of width off. This is the
  dangerous case: confident and wrong.
- **false-track**: frames where the point is occluded/out of frame but reported `Tracking`.
- **detect**: frames from first invisibility to first non-`Tracking` status.
- **recover**: frames from visibility returning to `Tracking` within tolerance (— = never).
- **final EPE**: EPE on the last frame if `Tracking`, else `lost`.

## Results

Run 2026-11-12, Apple M1, release build. Deterministic: rows for unchanged scenes were
byte-identical across two full runs.

| scene | input | alg | ok | EPE med | EPE p95 | EPE max | wrong-lock | false-track | detect | recover | final EPE |
|---|---|---|---|---|---|---|---|---|---|---|---|
| translate_int | 320×240 | LK18 | 100% | 0.02 | 0.04 | 0.05 | 0 | 0 | — | — | 0.03 |
| translate_int | 320×240 | NCC | 100% | 0.50 | 0.50 | 0.50 | 0 | 0 | — | — | 0.50 |
| translate_int | 640×480 | LK18 | 100% | 0.03 | 0.05 | 0.05 | 0 | 0 | — | — | 0.03 |
| translate_int | 640×480 | NCC | 100% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | 0.00 |
| translate_int | 1280×720 | LK18 | 100% | 0.02 | 0.03 | 0.04 | 0 | 0 | — | — | 0.01 |
| translate_int | 1280×720 | NCC | 100% | 0.35 | 0.35 | 0.35 | 0 | 0 | — | — | 0.35 |
| translate_subpx | 320×240 | LK18 | 100% | 0.05 | 0.09 | 0.10 | 0 | 0 | — | — | 0.08 |
| translate_subpx | 320×240 | NCC | 100% | 0.38 | 0.59 | 0.64 | 0 | 0 | — | — | 0.23 |
| translate_subpx | 640×480 | LK18 | 100% | 0.05 | 0.08 | 0.10 | 0 | 0 | — | — | 0.06 |
| translate_subpx | 640×480 | NCC | 100% | 0.42 | 0.57 | 0.71 | 0 | 0 | — | — | 0.46 |
| translate_subpx | 1280×720 | LK18 | 100% | 0.02 | 0.04 | 0.04 | 0 | 0 | — | — | 0.04 |
| translate_subpx | 1280×720 | NCC | 100% | 0.51 | 0.92 | 1.06 | 0 | 0 | — | — | 0.80 |
| head_sway | 320×240 | LK18 | 100% | 0.02 | 0.04 | 0.05 | 0 | 0 | — | — | 0.01 |
| head_sway | 320×240 | NCC | 99% | 0.38 | 0.62 | 0.69 | 0 | 0 | — | — | 0.08 |
| head_sway | 640×480 | LK18 | 100% | 0.03 | 0.05 | 0.05 | 0 | 0 | — | — | 0.02 |
| head_sway | 640×480 | NCC | 100% | 0.40 | 0.62 | 0.69 | 0 | 0 | — | — | 0.15 |
| head_sway | 1280×720 | LK18 | 100% | 0.02 | 0.04 | 0.05 | 0 | 0 | — | — | 0.03 |
| head_sway | 1280×720 | NCC | 100% | 0.54 | 0.78 | 0.84 | 0 | 0 | — | — | 0.27 |
| fast_40px | 320×240 | LK18 | 37% | 95.42 | 95.52 | 95.56 | 18 | 0 | — | — | 95.48 |
| fast_40px | 320×240 | NCC | 100% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | 0.00 |
| fast_40px | 640×480 | LK18 | 37% | 185.52 | 185.80 | 185.83 | 18 | 0 | — | — | 185.80 |
| fast_40px | 640×480 | NCC | 100% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | 0.00 |
| fast_40px | 1280×720 | LK18 | 37% | 189.22 | 189.31 | 189.34 | 18 | 0 | — | — | 189.28 |
| fast_40px | 1280×720 | NCC | 100% | 0.35 | 0.35 | 0.35 | 0 | 0 | — | — | 0.35 |
| brightness_step | 320×240 | LK18 | 100% | 0.87 | 0.93 | 0.94 | 0 | 0 | — | — | 0.90 |
| brightness_step | 320×240 | NCC | 100% | 0.43 | 0.59 | 0.69 | 0 | 0 | — | — | 0.43 |
| brightness_step | 640×480 | LK18 | 98% | 0.06 | 6.43 | 6.44 | 0 | 0 | — | — | 6.41 |
| brightness_step | 640×480 | NCC | 100% | 0.39 | 0.58 | 0.62 | 0 | 0 | — | — | 0.32 |
| brightness_step | 1280×720 | LK18 | 98% | 0.03 | 6.38 | 6.40 | 0 | 0 | — | — | 6.38 |
| brightness_step | 1280×720 | NCC | 100% | 0.56 | 0.76 | 0.84 | 0 | 0 | — | — | 0.67 |
| brightness_ramp | 320×240 | LK18 | 100% | 0.25 | 0.43 | 0.46 | 0 | 0 | — | — | 0.46 |
| brightness_ramp | 320×240 | NCC | 100% | 0.43 | 0.59 | 0.69 | 0 | 0 | — | — | 0.43 |
| brightness_ramp | 640×480 | LK18 | 100% | 0.93 | 1.56 | 1.64 | 0 | 0 | — | — | 1.64 |
| brightness_ramp | 640×480 | NCC | 100% | 0.39 | 0.58 | 0.62 | 0 | 0 | — | — | 0.32 |
| brightness_ramp | 1280×720 | LK18 | 100% | 1.00 | 1.65 | 1.71 | 0 | 0 | — | — | 1.71 |
| brightness_ramp | 1280×720 | NCC | 100% | 0.56 | 0.76 | 0.84 | 0 | 0 | — | — | 0.67 |
| contrast_drop | 320×240 | LK18 | 100% | 0.18 | 0.25 | 0.26 | 0 | 0 | — | — | 0.23 |
| contrast_drop | 320×240 | NCC | 100% | 0.43 | 0.59 | 0.69 | 0 | 0 | — | — | 0.43 |
| contrast_drop | 640×480 | LK18 | 100% | 1.46 | 1.59 | 1.61 | 0 | 0 | — | — | 1.55 |
| contrast_drop | 640×480 | NCC | 100% | 0.39 | 0.58 | 0.62 | 0 | 0 | — | — | 0.32 |
| contrast_drop | 1280×720 | LK18 | 100% | 1.52 | 1.57 | 1.59 | 0 | 0 | — | — | 1.53 |
| contrast_drop | 1280×720 | NCC | 100% | 0.56 | 0.76 | 0.84 | 0 | 0 | — | — | 0.67 |
| occlusion_partial | 320×240 | LK18 | 100% | 0.13 | 0.33 | 0.34 | 0 | 0 | — | — | 0.32 |
| occlusion_partial | 320×240 | NCC | 100% | 0.25 | 0.50 | 0.50 | 0 | 0 | — | — | 0.25 |
| occlusion_partial | 640×480 | LK18 | 100% | 0.05 | 0.07 | 0.08 | 0 | 0 | — | — | 0.04 |
| occlusion_partial | 640×480 | NCC | 100% | 0.50 | 0.50 | 0.50 | 0 | 0 | — | — | 0.50 |
| occlusion_partial | 1280×720 | LK18 | 100% | 0.01 | 0.03 | 0.03 | 0 | 0 | — | — | 0.02 |
| occlusion_partial | 1280×720 | NCC | 100% | 0.35 | 0.79 | 0.79 | 0 | 0 | — | — | 0.35 |
| occlusion_full | 320×240 | LK18 | 60% | 0.02 | 7.57 | 7.59 | 20 | 20 | — | — | 7.55 |
| occlusion_full | 320×240 | NCC | 100% | 0.25 | 0.50 | 0.50 | 0 | 0 | 0 | 0 | 0.25 |
| occlusion_full | 640×480 | LK18 | 60% | 0.06 | 20.24 | 20.27 | 20 | 19 | 0 | — | 20.23 |
| occlusion_full | 640×480 | NCC | 60% | 0.50 | 0.50 | 0.50 | 0 | 14 | 0 | — | lost |
| occlusion_full | 1280×720 | LK18 | 60% | 0.03 | 20.30 | 20.31 | 20 | 19 | 0 | — | 20.30 |
| occlusion_full | 1280×720 | NCC | 60% | 0.35 | 0.79 | 0.79 | 0 | 15 | 0 | — | lost |
| rotate_15 | 320×240 | LK18 | 100% | 0.33 | 0.46 | 0.47 | 0 | 0 | — | — | 0.11 |
| rotate_15 | 320×240 | NCC | 63% | 0.93 | 1.28 | 1.38 | 0 | 0 | — | — | 0.20 |
| rotate_15 | 640×480 | LK18 | 100% | 0.68 | 1.04 | 1.07 | 0 | 0 | — | — | 0.13 |
| rotate_15 | 640×480 | NCC | 100% | 0.79 | 1.81 | 1.85 | 0 | 0 | — | — | 0.39 |
| rotate_15 | 1280×720 | LK18 | 100% | 0.69 | 1.08 | 1.11 | 0 | 0 | — | — | 0.13 |
| rotate_15 | 1280×720 | NCC | 100% | 1.11 | 1.72 | 1.76 | 0 | 0 | — | — | 0.75 |
| scale_10 | 320×240 | LK18 | 100% | 0.03 | 0.06 | 0.08 | 0 | 0 | — | — | 0.01 |
| scale_10 | 320×240 | NCC | 100% | 0.83 | 0.85 | 0.85 | 0 | 0 | — | — | 0.08 |
| scale_10 | 640×480 | LK18 | 100% | 0.30 | 0.42 | 0.48 | 0 | 0 | — | — | 0.05 |
| scale_10 | 640×480 | NCC | 100% | 0.31 | 0.61 | 0.61 | 0 | 0 | — | — | 0.15 |
| scale_10 | 1280×720 | LK18 | 100% | 0.30 | 0.46 | 0.47 | 0 | 0 | — | — | 0.04 |
| scale_10 | 1280×720 | NCC | 100% | 0.51 | 0.92 | 0.95 | 0 | 0 | — | — | 0.41 |
| edge_left | 320×240 | LK18 | 50% | 9.64 | 10.09 | 10.33 | 37 | 4 | 0 | — | 9.64 |
| edge_left | 320×240 | NCC | 89% | 0.00 | 0.50 | 0.50 | 0 | 0 | 0 | 4 | 0.00 |
| edge_left | 640×480 | LK18 | 50% | 25.73 | 26.52 | 26.87 | 37 | 2 | 0 | — | 26.08 |
| edge_left | 640×480 | NCC | 95% | 0.00 | 0.00 | 0.00 | 0 | 0 | 0 | 2 | 0.00 |
| edge_left | 1280×720 | LK18 | 50% | 25.67 | 26.44 | 26.85 | 37 | 2 | 0 | — | 25.97 |
| edge_left | 1280×720 | NCC | 95% | 0.35 | 0.35 | 0.35 | 0 | 0 | 0 | 2 | 0.35 |
| edge_right | 320×240 | LK18 | 49% | 25.20 | 29.44 | 30.68 | 35 | 5 | 0 | — | 25.20 |
| edge_right | 320×240 | NCC | 86% | 0.50 | 0.50 | 0.50 | 0 | 0 | 0 | 5 | 0.00 |
| edge_right | 640×480 | LK18 | 49% | 52.36 | 54.59 | 54.83 | 35 | 6 | 1 | — | 52.36 |
| edge_right | 640×480 | NCC | 94% | 0.00 | 0.00 | 3.00 | 0 | 0 | 0 | 2 | 0.00 |
| edge_right | 1280×720 | LK18 | 49% | 0.01 | 0.02 | 0.03 | 0 | 0 | 0 | — | lost |
| edge_right | 1280×720 | NCC | 94% | 0.35 | 0.35 | 2.76 | 0 | 0 | 0 | 2 | 0.35 |
| edge_top | 320×240 | LK18 | 50% | 14.09 | 14.28 | 14.30 | 35 | 5 | 0 | — | 14.17 |
| edge_top | 320×240 | NCC | 77% | 0.50 | 0.50 | 1.00 | 0 | 0 | 0 | 8 | 0.00 |
| edge_top | 640×480 | LK18 | 49% | 29.54 | 29.59 | 29.61 | 35 | 6 | 1 | — | 29.59 |
| edge_top | 640×480 | NCC | 91% | 0.00 | 0.00 | 5.00 | 0 | 0 | 0 | 3 | 0.00 |
| edge_top | 1280×720 | LK18 | 49% | 29.57 | 29.60 | 29.60 | 35 | 6 | 1 | — | 29.60 |
| edge_top | 1280×720 | NCC | 91% | 0.35 | 0.35 | 5.26 | 0 | 0 | 0 | 3 | 0.35 |
| edge_bottom | 320×240 | LK18 | 49% | 13.52 | 18.08 | 19.42 | 37 | 3 | 0 | — | 13.52 |
| edge_bottom | 320×240 | NCC | 86% | 0.25 | 0.50 | 0.75 | 0 | 0 | 0 | 5 | 0.00 |
| edge_bottom | 640×480 | LK18 | 50% | 29.18 | 31.95 | 38.85 | 37 | 6 | 3 | — | 31.24 |
| edge_bottom | 640×480 | NCC | 95% | 0.50 | 0.50 | 4.00 | 0 | 0 | 0 | 2 | 0.00 |
| edge_bottom | 1280×720 | LK18 | 50% | 0.01 | 0.03 | 0.88 | 0 | 0 | 0 | — | lost |
| edge_bottom | 1280×720 | NCC | 91% | 0.35 | 0.79 | 4.76 | 0 | 0 | 0 | 2 | 0.35 |
| stall | 320×240 | LK18 | 100% | 0.02 | 0.04 | 0.05 | 0 | 0 | — | — | 0.02 |
| stall | 320×240 | NCC | 100% | 0.43 | 0.62 | 0.69 | 0 | 0 | — | — | 0.32 |
| stall | 640×480 | LK18 | 100% | 0.03 | 0.05 | 0.05 | 0 | 0 | — | — | 0.02 |
| stall | 640×480 | NCC | 100% | 0.42 | 0.57 | 0.62 | 0 | 0 | — | — | 0.40 |
| stall | 1280×720 | LK18 | 100% | 0.02 | 0.04 | 0.04 | 0 | 0 | — | — | 0.04 |
| stall | 1280×720 | NCC | 100% | 0.44 | 0.76 | 0.84 | 0 | 0 | — | — | 0.18 |
| static_60s | 640×480 | LK18 | 100% | 0.19 | 0.27 | 0.31 | 0 | 0 | — | — | 0.22 |
| static_60s | 640×480 | NCC | 100% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | 0.00 |
| motion_blur | 320×240 | LK18 | 100% | 0.11 | 0.17 | 0.20 | 0 | 0 | — | — | 0.20 |
| motion_blur | 320×240 | NCC | 93% | 0.36 | 0.67 | 0.70 | 0 | 0 | — | — | lost |
| motion_blur | 640×480 | LK18 | 100% | 0.42 | 0.82 | 1.00 | 0 | 0 | — | — | 1.00 |
| motion_blur | 640×480 | NCC | 100% | 0.40 | 0.69 | 0.75 | 0 | 0 | — | — | 0.69 |
| motion_blur | 1280×720 | LK18 | 100% | 0.49 | 0.89 | 1.03 | 0 | 0 | — | — | 1.03 |
| motion_blur | 1280×720 | NCC | 100% | 0.43 | 0.76 | 1.04 | 0 | 0 | — | — | 0.34 |
| sensor_noise | 320×240 | LK18 | 100% | 0.11 | 0.23 | 0.33 | 0 | 0 | — | — | 0.12 |
| sensor_noise | 320×240 | NCC | 93% | 0.38 | 0.66 | 0.74 | 0 | 0 | — | — | lost |
| sensor_noise | 640×480 | LK18 | 100% | 0.24 | 0.44 | 0.57 | 0 | 0 | — | — | 0.35 |
| sensor_noise | 640×480 | NCC | 100% | 0.40 | 0.65 | 0.70 | 0 | 0 | — | — | 0.15 |
| sensor_noise | 1280×720 | LK18 | 100% | 0.07 | 0.15 | 0.18 | 0 | 0 | — | — | 0.09 |
| sensor_noise | 1280×720 | NCC | 100% | 0.45 | 0.77 | 0.84 | 0 | 0 | — | — | 0.27 |
| textureless | 320×240 | LK18 | 3% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | lost |
| textureless | 320×240 | NCC | 3% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | lost |
| textureless | 640×480 | LK18 | 3% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | lost |
| textureless | 640×480 | NCC | 3% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | lost |
| textureless | 1280×720 | LK18 | 3% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | lost |
| textureless | 1280×720 | NCC | 3% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | lost |
| repeated_texture | 320×240 | LK18 | 3% | 0.21 | 0.21 | 0.21 | 0 | 0 | — | — | lost |
| repeated_texture | 320×240 | NCC | 52% | 0.25 | 0.75 | 0.75 | 0 | 0 | — | — | lost |
| repeated_texture | 640×480 | LK18 | 3% | 0.34 | 0.34 | 0.34 | 0 | 0 | — | — | lost |
| repeated_texture | 640×480 | NCC | 7% | 168.75 | 303.29 | 334.01 | 56 | 0 | — | — | 146.46 |
| repeated_texture | 1280×720 | LK18 | 2% | 0.00 | 0.00 | 0.00 | 0 | 0 | — | — | lost |
| repeated_texture | 1280×720 | NCC | 12% | 89.55 | 146.08 | 198.75 | 53 | 0 | — | — | 89.46 |

## Findings

Each finding cites rows above. "Better" means a measured delta on the same fixture.

1. **Smooth motion, sub-pixel accuracy: LK is clearly more accurate.** On `translate_subpx`,
   `head_sway`, `stall`, `scale_10` and `sensor_noise`, LK's EPE p95 is ≤ 0.46 px, against
   NCC's 0.57–0.92 px. NCC is limited by integer output (≥ 0.25–0.5 px quantisation). Both
   track 100 % (NCC drops a few frames at 320 under noise).
2. **Fast motion beyond the guard: LK-2018 fails, NCC doesn't.** `fast_40px` (40 units/frame
   for 5 frames): LK's 35/640 guard holds the point and it never re-catches the target. That
   gives 37 % ok, 18 wrong-lock frames and a final error of ~185 px at 640. NCC's ±90 px search
   absorbs the jump (100 %). That is the 2018 guard working as designed and failing.
   Phase 4: adaptive search / loss recovery.
3. **Full occlusion: both fail, differently.** `occlusion_full` at 640: LK follows the
   occluder (19 false-track frames, ~20 px final error). It has no photometric check.
   NCC detects the occlusion immediately (detect 0) but then locks onto background during
   the occlusion (14 false-track frames) and never recovers. At 320 it survives (100 % ok). *Unverified guess:* the 45 px template is
   twice as large relative to a 320-wide frame, so it covers more of the target disc. Phase 4: residual/confidence gating.
4. **Edges: NCC recovers, LK-2018 doesn't.** On `edge_*`, LK holds at the edge per the 2018
   policy. When the target comes back, the held point is on background, so ~35–38
   wrong-lock frames follow, or a `Lost` that needs re-selection. NCC reports lost and
   re-finds the target within 2–8 frames of it re-entering, because it keeps the template
   and searches. The symmetric edge predicate (R-T4) is unit-tested on all four edges.
5. **Photometric change hurts LK, not NCC.** `brightness_step` at 640: LK shifts by ~6 px
   when gain drops to 0.7, staying under the 2 % tolerance and so not counted as
   wrong-lock. At 640/1280, `brightness_ramp` and `contrast_drop` push LK's EPE p95 to 1.6 px. ZNCC is
   invariant to gain/bias by construction (p95 ≤ 0.76 px, max ≤ 0.84 px). Phase 4: gain/bias-compensated LK or a
   normalised residual.
6. **Textureless: both correctly refuse.** No `Tracking` after selection on a flat region
   (asserted in `fixtures.rs`). LK's `min_eigen` test is a deliberate deviation from 2018
   (ADR-0009).
7. **Repeated texture (1-D grating): LK refuses, NCC locks wrong.** LK reports
   `Lost(LowConfidence)` from frame 1 (singular G, the aperture problem). NCC reports
   `Tracking` while 90–170 px off for 53–56 frames at 640/1280. The silent wrong lock is the
   worse failure for a pointer.
8. **Drift**: `static_60s` (1800 frames, σ = 2): LK's EPE stays ≤ 0.31 px with no
   accumulation, NCC is exact. Neither drifts.
9. **Motion blur, rotation ±15°, scale ±10%**: both keep tracking. LK's EPE p95 is ≤ 1.08 px,
   NCC's is ≤ 1.81 px (rotation). NCC drops frames on rotation at 320 (63 % ok).

**Net:** LK-2018 is the better *precision* tracker on well-behaved motion. NCC-legacy is the
better *recovery* tracker (fast motion, edges, lighting). Each has a silent-wrong failure:
LK under occlusion, NCC on repeated texture and after occlusion. Per ADR-0006, LK stays the
presumptive baseline: NCC isn't "clearly better" overall, it fails differently. The Phase 4
experiments that follow from the data are residual/confidence gating (findings 3, 7),
re-detection or template recovery for LK (findings 2, 4), and photometric compensation
(finding 5).

## Performance

Apple M1, 8 GB, macOS 14.7.3, rustc 1.99.1, `cargo bench` (divan 0.1.21, release),
100 samples. Inputs are `head_sway` frames. "1280x720" means the downsampled 640×360 frame
(the downsample is benched separately).

| Bench | 320×240 | 640×480 | 1280×720 → 640×360 |
|---|---|---|---|
| downsample 1280→640 | — | — | 1.25 ms (1 alloc, output frame) |
| gray → f32 | 3.1 µs | 27.5 µs | 20.1 µs |
| pyramid build (4 levels + Scharr) | 1.02 ms | 4.08 ms | 3.11 ms |
| LK solve, one point (`track_point`) | 35.7 µs | 35.7 µs | 34.0 µs |
| **LK end-to-end step** | **1.06 ms** | **4.13 ms** | **3.15 ms** |
| **NCC end-to-end step** | **32.8 ms** | **33.6 ms** | **33.6 ms** |

(medians; min/max in the bench output were within ~10 % except gray→f32 640 max 46.8 µs)

- **Steady-state heap allocations: zero** for `lk_step`, `ncc_step`, `pyramid_build`,
  `track_point` (divan AllocProfiler reports no allocations after the first iteration).
  Meets the §8 target.
- **LK p95 < 2 ms at 640×480: not met** (4.1 ms). *Met in Phase 4 E0 (0.30 ms, identical
  outputs). See `TRACKING_IMPROVEMENTS.md`. The rest of this bullet is the Phase 3 analysis.* The profile is unambiguous: ~98 % is the
  pyramid build, which computes full-image Scharr gradients on every level with a
  per-pixel reflect-101 border, while LK reads a 21×21 window. Only the reference
  pyramid's gradients are ever used. Fix candidates for Phase 4: gradients only for the
  reference pyramid / window, and interior fast paths without border handling.
- **NCC is not real-time as written** (33 ms per frame, ~1 frame budget at 30 fps). This is
  a naive scalar exhaustive search (~18.5 k offsets × 2025 MACs). It says nothing about
  legacy's OpenCV speed (SIMD/DFT). If NCC survives as a re-detector in Phase 4, it needs a
  coarse-to-fine or FFT implementation.

## Limitations

- Synthetic only. Value-noise textures are not faces. Real sequences (Phase 6 recorder) are
  required before any product claim.
- The generator's pixel self-check covers translation only. Rotation/scale/blur ground
  truth relies on the transform code. Two generator bugs were found and fixed while
  producing these tables: `edge_bottom` never left a 4:3 frame, and motion blur was
  centred on the previous pose. Both were visible as implausible rows, which is how they
  were caught.
- One machine. Timings are a trend, not a gate (`benchmarking` skill).
- The legacy-via-GoCV parity run (§8 optional) was not done.
