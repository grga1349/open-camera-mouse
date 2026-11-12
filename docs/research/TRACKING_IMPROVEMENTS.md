# Tracking improvements (Phase 4)

Ablations against B-LK18 (`LkTracker::default()`) on the Phase 3 corpus. Task 0006.

**Outcome (ADR-0011):** the recommended tracker is `LkParams::recommended()` = B-LK18 + E2 (ZNCC ≥ 0.7)
+ E3 + E4 (τ 0.9, δ 0.1, K 16, level 2, r 0.25 w), with E1 off. On 70 runs it has 89.4 % mean ok (B-LK18
73.4 %), 0 wrong-lock and 0 false-track frames, and no gate-scene regression beyond E3's
+0.070 px on `rotate_15` 1280. A tracked step is 0.29–0.31 ms and a searching step 0.58 ms at 640 (M1).
E5 and E6 were not attempted (see below).
Baseline numbers, corpus and metric definitions: `TRACKING_BASELINE.md`. Every experiment is
kept or rejected on measured deltas from the same corpus. Synthetic-only caveats from Phase 3
still apply.

## How to reproduce

```sh
# exact LK output fingerprint (must be unchanged by "no behavior change" work)
cargo test -p tracking-core --release --test baseline lk_fingerprint -- --ignored --nocapture
# accuracy tables (B-LK18 vs B-NCC, ~5 min)
cargo test -p tracking-core --release --test baseline baseline_tables -- --ignored --nocapture
# ablation: every LK variant in `variants()` vs B-LK18 (~2 min; filters BASELINE_SCENE, ABLATION)
cargo test -p tracking-core --release --test baseline ablation_tables -- --ignored --nocapture
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

## E1 — forward-backward validation: KEPT as a component, default off (provisional)

**Change.** `LkParams::fb_max_error: Option<f32>` (default `None` = B-LK18). When set, after
`a → b` passes the motion guard, `b` is tracked back through the same two pyramids to `a'`.
The step is rejected when `|a − a'|` > threshold (tracker px, absolute), or when the backward
solve fails. A rejected step goes through the unchanged 2018 loss policy: near an edge and
moving out → `Held(NearEdgeMovingOut)`, otherwise `Lost(Inconsistent)` (new `LossReason`)
and the reference is dropped. `Diagnostics::fb_error` records the error whenever the check
ran. Default output is unchanged: `lk_fingerprint` identical on all 61 runs.

**Results** (`ablation_tables`, 2026-11-12, M1). Summary over all 61 scene × resolution runs:

| alg | mean ok | Σ wrong-lock | Σ false-track | worst good-scene Δp95 | more wrong-lock than LK18 |
|---|---|---|---|---|---|
| LK18 | 75.4% | 474 | 103 | +0.00 | none |
| E1 fb≤0.25 | 72.0% | 6 | 49 | +0.00 | none |
| E1 fb≤0.5 | 72.8% | 62 | 46 | +0.00 | none |
| E1 fb≤1 | 72.8% | 95 | 46 | +0.00 | none |
| E1 fb≤2 | 75.3% | 224 | 55 | +0.00 | none |

Every "LK good" gate scene row is byte-identical to LK18 at every threshold. Rows that
changed (ok / wrong-lock / false-track / final EPE):

| scene | input | LK18 | fb≤0.25 | fb≤0.5 | fb≤1 | fb≤2 |
|---|---|---|---|---|---|---|
| fast_40px | 640×480 | 37%/18/0/185.80 | 37%/0/0/lost | 37%/0/0/lost | 37%/0/0/lost | 37%/0/0/lost |
| brightness_step | 320×240 | 100%/0/0/0.90 | **50%**/0/0/lost | 100%/0/0/0.90 | 100%/0/0/0.90 | 100%/0/0/0.90 |
| contrast_drop | 320×240 | 100%/0/0/0.23 | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost | 100%/0/0/0.23 |
| contrast_drop | 640×480 | 100%/0/0/1.55 | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost | 100%/0/0/1.55 |
| contrast_drop | 1280×720 | 100%/0/0/1.53 | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost | 100%/0/0/1.53 |
| occlusion_full | 320×240 | 60%/20/20/7.55 | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost |
| occlusion_full | 640×480 | 60%/20/19/20.23 | 60%/0/19/lost | 60%/0/19/lost | 60%/20/19/20.23 | 60%/20/19/20.23 |
| occlusion_full | 1280×720 | 60%/20/19/20.30 | 60%/0/19/lost | 60%/0/19/lost | 60%/0/19/lost | 60%/20/19/20.30 |
| edge_left | 640×480 | 50%/37/2/26.08 | 50%/4/2/lost | 50%/31/2/lost | 50%/37/2/26.08 | 50%/37/2/26.08 |
| edge_right | 640×480 | 49%/35/6/52.36 | 49%/0/4/lost | 49%/0/1/lost | 49%/0/1/lost | 49%/0/1/lost |
| edge_top | 640×480 | 49%/35/6/29.59 | 49%/0/1/lost | 49%/0/1/lost | 49%/0/1/lost | 49%/0/1/lost |
| edge_bottom | 640×480 | 50%/37/6/31.24 | 50%/0/0/lost | 50%/0/0/lost | 50%/0/0/lost | 50%/0/0/lost |

(`fast_40px` and `edge_*` at 320/1280 show the same pattern, with smaller wrong-lock counts at 320; `repeated_texture` drops from 3 % to
2 % ok because one more frame is rejected. The full per-scene output is reproducible with the
command above.)

**Reading.**
- E1 turns most *silent* wrong-locks into an explicit `Lost(Inconsistent)`. That covers
  `fast_40px`, all four edges, and the post-occlusion return. It recovers nothing: ok % is
  unchanged there, because the 2018 policy then drops the reference. Recovery is E4.
- Finding 3 is only half fixed. During `occlusion_full` the occluder moves rigidly, so
  forward and backward flow agree and 19 false-track frames remain at 640/1280. Catching
  that needs a photometric check against the *selection* appearance (E2), not flow consistency.
- The cost is photometric: at fb ≤ 0.5 and fb ≤ 1, the contrast halving in `contrast_drop`
  makes plain LK's FB error exceed the threshold, and the track is dropped (100 % → 50 %).
  That is finding 5's bias showing up as inconsistency. fb ≤ 2 keeps `contrast_drop` but
  lets `edge_left` and the occlusion return through.
- fb ≤ 0.5 dominates fb ≤ 1 (same losses, fewer wrong-locks). fb ≤ 0.25 additionally loses
  `brightness_step` at 320.

**Decision.** Keep the mechanism; the default stays off for now. Candidate threshold:
0.5 px. Whether it's enabled by default, and at what threshold, gets re-measured on top of E3
(gain/bias-compensated LK), which targets the `contrast_drop` loss. Then it goes into the
default-tracker ADR. It passes the task gate (no good-scene regression, no new wrong-lock),
but dropping a track on a lighting change is a usability regression that shouldn't ship on its own.

**Timing** (M1, divan, medians, `lk_fb_step` vs `lk_step`): 320: 164 vs 108 µs; 640: 330 vs
286 µs; 1280→640: 264 vs 223 µs. That's +1 LK solve (~45 µs), no extra pyramid, zero
steady-state allocations. Within the §8 budget.

## E2 — appearance gate (ZNCC vs the selection window): KEPT as a component, default off (provisional)

**Change.** `LkParams::min_ref_zncc: Option<f32>` (default `None` = B-LK18). When set,
`select` stores the `(2·hw + 1)²` level-0 window around the selected point (R-T3: the
selection frame), zero-mean and unit-norm. After a step passes the motion guard (and the E1
check, if on), the window at the new position in the current frame is scored by ZNCC
against that reference. Below the threshold, the step goes through the unchanged 2018 loss
policy, the same way as E1: `Held(NearEdgeMovingOut)` or `Lost(AppearanceChanged)` (new
`LossReason`). `Diagnostics::score` records the ZNCC whenever the gate ran. If the selection
window is flat, there is no reference and the gate is skipped (LK's own texture test refuses
that case anyway). It is the *selection* reference, not the previous frame, so a slowly
sliding occluder is caught as well as a sudden one. The fixture only has the sudden case.

ZNCC is gain/bias invariant, so a lighting change on the right surface passes. A different
surface (occluder, background after an edge or a fast jump) does not. Default output is
unchanged: `lk_fingerprint` identical on all 61 runs (diffed against `cac424f` on the same
toolchain).

**Results** (`ablation_tables`, 2026-11-12, M1). Summary over all 61 runs. The new column,
"worst good-scene ok drop", was added because Δp95 only covers frames still `Tracking` and
so can't see a gate scene losing its track:

| alg | mean ok | Σ wrong-lock | Σ false-track | worst good-scene Δp95 | worst good-scene ok drop | more wrong-lock than LK18 |
|---|---|---|---|---|---|---|
| LK18 | 75.4% | 474 | 103 | +0.00 | none | none |
| E1 fb≤0.5 | 72.8% | 62 | 46 | +0.00 | none | none |
| E2 zncc≥0.5 | 73.8% | 0 | 4 | +0.00 | none | none |
| E2 zncc≥0.6 | 73.8% | 0 | 2 | +0.00 | none | none |
| **E2 zncc≥0.7** | **73.7%** | **0** | **0** | +0.00 | none | none |
| E2 zncc≥0.8 | 72.4% | 0 | 0 | +0.00 | 82 pp (rotate_15 320) | none |
| E2 zncc≥0.9 | 72.2% | 0 | 0 | +0.00 | 90 pp (rotate_15 320) | none |
| E1+E2 fb≤0.5 zncc≥0.6 | 71.2% | 0 | 2 | +0.00 | none | none |
| E1+E2 fb≤0.5 zncc≥0.7 | 71.2% | 0 | 0 | +0.00 | none | none |
| E1+E2 fb≤0.5 zncc≥0.8 | 69.9% | 0 | 0 | +0.00 | 82 pp (rotate_15 320) | none |

Rows that changed (ok / wrong-lock / false-track / final EPE):

| scene | input | LK18 | zncc≥0.5 | zncc≥0.6 | zncc≥0.7 | zncc≥0.8 | fb≤0.5 + zncc≥0.7 |
|---|---|---|---|---|---|---|---|
| fast_40px | 640×480 | 37%/18/0/185.80 | 37%/0/0/lost | 37%/0/0/lost | 37%/0/0/lost | 37%/0/0/lost | 37%/0/0/lost |
| brightness_step | 640×480 | 98%/0/0/6.41 | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost |
| brightness_step | 1280×720 | 98%/0/0/6.38 | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost | **50%**/0/0/lost |
| contrast_drop | 640×480 | 100%/0/0/1.55 | 100%/0/0/1.55 | 100%/0/0/1.55 | 100%/0/0/1.55 | 100%/0/0/1.55 | **50%**/0/0/lost |
| occlusion_full | 320×240 | 60%/20/20/7.55 | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost |
| occlusion_full | 640×480 | 60%/20/19/20.23 | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost |
| occlusion_full | 1280×720 | 60%/20/19/20.30 | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost |
| rotate_15 | 320×240 | 100%/0/0/0.11 | 100%/0/0/0.11 | 100%/0/0/0.11 | 100%/0/0/0.11 | **18%**/0/0/lost | 100%/0/0/0.11 |
| edge_left | 640×480 | 50%/37/2/26.08 | 50%/0/0/lost | 50%/0/0/lost | 50%/0/0/lost | 50%/0/0/lost | 50%/0/0/lost |
| edge_right | 640×480 | 49%/35/6/52.36 | 49%/0/0/lost | 49%/0/0/lost | 49%/0/0/lost | 49%/0/0/lost | 49%/0/0/lost |
| edge_top | 640×480 | 49%/35/6/29.59 | 49%/0/1/lost | 49%/0/1/lost | 49%/0/0/lost | 49%/0/0/lost | 49%/0/0/lost |
| edge_bottom | 640×480 | 50%/37/6/31.24 | 50%/0/2/lost | 50%/0/0/lost | 50%/0/0/lost | 50%/0/0/lost | 50%/0/0/lost |

(`fast_40px` and `edge_*` at 320/1280 follow the same pattern. At zncc ≥ 0.7 a few
`edge_*` runs at 320 drop one more ok frame (50 % → 49 %). `contrast_drop` at 320/1280 behaves
like 640. `repeated_texture` is unchanged by E2 alone, and with E1 it drops 3 % → 2 % as in §E1.)

**Per-frame probe** (temporary diagnostic, not committed). Plain LK with the gate scoring
every step but never rejecting:
- `brightness_step` 640: ZNCC 1.00 up to frame 29. At the gain step (frame 30), LK's raw
  estimate exceeds the 35 px guard, so the frame is `Held(MotionOutlier)`. From frame 31 on, LK
  tracks a point a constant 6.4 px off the target, with ZNCC ≈ −0.02 against the selection.
  The baseline's 98 % ok there was a *silent 6.4 px offset* under the 2 % (12.8 px) tolerance.
  E2 reports it. The root cause is finding 5 (brightness constancy), which is E3's target.
- `rotate_15`: ZNCC falls with roll angle. At ±15°, the minimum is 0.74 at 320 and 0.91 at
  640, with EPE ≤ 0.5 / ≤ 1.1 px. So this is real appearance change, not tracker error. That
  is why ≥ 0.8 drops 320.

**Reading.**
- E2 closes finding 3. `occlusion_full` false-track goes 19–20 → 0 at every resolution,
  which E1 could not do. It also turns every silent wrong-lock in the corpus into an explicit
  loss (Σ 474 → 0, false-track 103 → 0 at ≥ 0.7). Finding 7 is unaffected: LK still refuses
  `repeated_texture` (no new wrong-lock).
- E2 dominates E1. At ≥ 0.7 it has fewer wrong-locks (0 vs 62), fewer false-tracks (0 vs 46)
  and keeps `contrast_drop`. Adding E1 on top only adds E1's `contrast_drop` loss. Unless E3
  changes that, E1 is redundant with E2.
- Threshold: 0.6 leaves 2 false-track frames (`edge_top` 640/1280). 0.8 breaks `rotate_15` at
  320. **0.7** is the only tested value with zero silent errors and no gate-scene loss.
- **Risk (HYPOTHESIS, synthetic only):** at 320 the margin is thin (0.74 vs 0.7 at 15° roll).
  Real head roll past ~15°, or out-of-plane rotation and expression change against a fixed
  selection template, could drop a valid track. Re-measure on recorded sequences (Phase 6).
  E4's re-detection shares this reference and inherits the same risk.
- Cost: dropping a lit-but-offset track (`brightness_step` 98 % → 50 %) is correct as a
  report but bad for usability under the 2018 loss policy (re-select). Like E1, it should be
  re-measured on top of E3.

**Decision.** Keep the mechanism; the default stays off for now. Candidate threshold: 0.7. It passes
the task gate (no good-scene regression, no ok drop on gate scenes, no new wrong-lock).
Whether it's enabled by default goes into the default-tracker ADR after E3 (re-measure
`brightness_step` and the 320 rotation margin). E4 should reuse the same reference window.

**Timing** (M1, divan, medians, `lk_zncc_step` vs `lk_step`): 320: 106.8 vs 106.2 µs; 640:
286.5 vs 291.1 µs; 1280→640: 228.3 vs 228.8 µs. One extra 21×21 bilinear pass, so the
difference is inside run-to-run noise. Zero steady-state allocations (the reference is
allocated in `select`).

## E3 — gain/bias-compensated LK: KEPT (default off until the default-tracker ADR)

**Change.** `LkParams::gain_bias: bool` (default `false` = B-LK18). At each pyramid level,
before iterating, the current-frame window `J` at the level's initial guess fixes a
photometric map `J ↦ α·J + β`. The map matches `J`'s mean and standard deviation to the
reference window `I`: `α = σ_I/σ_J`, `β = μ_I − α·μ_J`. A flat `J` (σ < 1e-3) gets bias-only,
`α = 1`. The map stays fixed while that level iterates, so each level is still a plain
translation LK solve against the same `G`, and only the residual `I − (α·J + β)` changes. The
level-0 `Diagnostics::residual` (mean |residual|) uses the map re-estimated at the final
position. E1's backward pass uses the same params, so it is compensated as well. Default
output is unchanged: `lk_fingerprint` is identical on all 61 runs (diffed against `6352603`
on the same toolchain).

**Variant rejected first: re-estimating `α, β` every iteration.** Lighting scenes measured the
same as the kept version (p95 ≤ 0.08 px). But `rotate_15` got worse: Δp95 vs LK18 was **+0.09998 px** at
1280, +0.06 at 640. That technically passes the 0.1 px gate, with no margin. Changing α while the
level iterates changes the objective halfway through the solve. Under rotation the window's
moments move with the misalignment, and that couples into the step. With the map fixed per
level, Δp95 is +0.070 / +0.04, and the kept version is also cheaper (one moment pass per level,
not one per iteration).

**Results** (`ablation_tables`, 2026-11-12, M1). Summary over all 61 runs. Δp95 is now
printed to 3 decimals, because the E3 row sits close to the gate:

| alg | mean ok | Σ wrong-lock | Σ false-track | worst good-scene Δp95 | worst good-scene ok drop | more wrong-lock than LK18 |
|---|---|---|---|---|---|---|
| LK18 | 75.4% | 474 | 103 | +0.000 | none | none |
| E2 zncc≥0.7 | 73.7% | 0 | 0 | +0.000 | none | none |
| E3 gain/bias | 75.4% | 474 | 106 | +0.070 (rotate_15 1280) | none | none |
| E3+E1 fb≤0.5 | 75.3% | 180 | 80 | +0.070 (rotate_15 1280) | none | none |
| **E3+E2 zncc≥0.7** | **75.4%** | **0** | **0** | +0.070 (rotate_15 1280) | none | none |
| E3+E2 zncc≥0.8 | 74.0% | 0 | 0 | +0.070 (rotate_15 1280) | 82 pp (rotate_15 320) | none |
| E3+E1+E2 fb≤0.5 zncc≥0.7 | 75.3% | 0 | 0 | +0.070 (rotate_15 1280) | none | none |

Selected rows (ok / EPE p95 / wrong-lock / false-track / final EPE):

| scene | input | LK18 | E2 zncc≥0.7 | E3 | E3+E2 zncc≥0.7 |
|---|---|---|---|---|---|
| brightness_step | 640×480 | 98%/6.43/0/0/6.41 | **50%**/0.06/0/0/lost | 100%/0.05/0/0/0.02 | **100%/0.05/0/0/0.02** |
| brightness_step | 1280×720 | 98%/6.38/0/0/6.38 | **50%**/0.02/0/0/lost | 100%/0.03/0/0/0.02 | **100%/0.03/0/0/0.02** |
| brightness_ramp | 640×480 | 100%/1.56/0/0/1.64 | 100%/1.56/0/0/1.64 | 100%/0.04/0/0/0.02 | 100%/0.04/0/0/0.02 |
| contrast_drop | 640×480 | 100%/1.59/0/0/1.55 | 100%/1.59/0/0/1.55 | 100%/0.07/0/0/0.05 | 100%/0.07/0/0/0.05 |
| contrast_drop | 1280×720 | 100%/1.57/0/0/1.53 | 100%/1.57/0/0/1.53 | 100%/0.05/0/0/0.03 | 100%/0.05/0/0/0.03 |
| occlusion_full | 640×480 | 60%/20.24/20/19/20.23 | 60%/0.06/0/0/lost | 60%/28.15/20/19/28.15 | 60%/0.07/0/0/lost |
| rotate_15 | 640×480 | 100%/1.04/0/0/0.13 | same | 100%/1.08/0/0/0.13 | 100%/1.08/0/0/0.13 |
| rotate_15 | 1280×720 | 100%/1.08/0/0/0.13 | same | 100%/1.15/0/0/0.14 | 100%/1.15/0/0/0.14 |
| scale_10 | 640×480 | 100%/0.42/0/0/0.05 | same | 100%/0.30/0/0/0.05 | 100%/0.30/0/0/0.05 |
| motion_blur | 640×480 | 100%/0.82/0/0/1.00 | same | 100%/0.59/0/0/0.76 | 100%/0.59/0/0/0.76 |
| sensor_noise | 640×480 | 100%/0.44/0/0/0.35 | same | 100%/0.38/0/0/0.36 | 100%/0.38/0/0/0.36 |

(`brightness_*`/`contrast_drop` at 320 also go to p95 ≤ 0.06. `rotate_15` at 320 is unchanged
(0.46). `fast_40px`, `edge_*` and `repeated_texture` behave as in §E2 under E3+E2. E3 alone leaves the
silent errors in place. It follows the occluder further: final EPE 20 → 28 px at 640. E3 is not a gate.)

**Reading.**
- E3 closes finding 5. The lighting scenes go from p95 1.6–6.4 px to ≤ 0.08 px at every
  resolution. The `brightness_step` silent 6.4 px offset is gone, because LK no longer
  overshoots the guard at the gain step.
- It also fixes the cost E2 paid for finding 5: E3+E2 keeps `brightness_step` at 100 %. So E3+E2
  ≥ 0.7 is the only tested row with the highest mean ok (75.4 %, = LK18) **and** zero wrong-lock
  / false-track frames.
- E1 is now redundant. E3 removes E1's `contrast_drop` loss (E3+E1 stays at 100 % there). But on
  top of E3+E2, E1 adds nothing measurable except one dropped ok frame on `edge_left` 320 /
  `repeated_texture` (75.4 → 75.3 %). Alone, E3+E1 still lets 180 wrong-lock frames through
  (`edge_left` at 640/1280 pass forward-backward). Recommendation for the ADR: E2 + E3, no E1.
- Gate scenes: no ok drop. Δp95 improves on `scale_10` (−0.12/−0.14), `motion_blur`
  (−0.06/−0.23/−0.26), `sensor_noise`, `static_60s`. It regresses within the gate only on `rotate_15`
  640/1280 (+0.04/+0.070 px). HYPOTHESIS: under in-plane rotation the window moments at the
  propagated guess are slightly off, and the moment-matched α absorbs a little geometric error.
- E2's 320 `rotate_15` margin is not made worse. Positions at 320 are unchanged (p95 0.46 px),
  ≥ 0.7 still keeps it and ≥ 0.8 still drops it. The 0.74 minimum wasn't re-probed per frame.

**Decision.** KEEP. Default stays `false` in this commit (fingerprint discipline). The proposed
default tracker is **E2 (zncc ≥ 0.7) + E3**, with E1 off but kept available. That goes into the
default-tracker ADR, together with E4's outcome, since E4 changes the loss policy that E2's
rejections feed into.

**Timing** (M1, divan, medians, `lk_gain_bias_step` vs `lk_step`): 320: 114.2 vs 102.4 µs; 640:
291.7 vs 280.9 µs; 1280→640: 225.2 vs 219.2 µs. That's one extra window sample per iteration plus a
moment pass per level. Zero steady-state allocations (the `j` scratch is sized once).

## E4 — loss recovery by re-detection: KEPT (default off until the default-tracker ADR)

ADR-0010 (deviation from the 2018 loss policy, reason (a) defect).

**Change.** `LkParams::redetect: Option<RedetectParams>` (default `None` = B-LK18), with the
search in a new module, `crate::redetect`. At `select`, it stores one zero-mean, unit-norm
template per pyramid level from the selection frame. Level 0 is E2's window; level `l` uses
half-size `max(1, hw >> l)`. A step the 2018 policy turns into `Lost` now **keeps** the
reference and searches the same frame. Later frames report the new `Lost(Searching)` until a
match is accepted, and then `Tracking` resumes from it. `Held` statuses are unchanged: a
held point that ends up on background becomes a `Lost` through E2, so E4 is only measured on
top of E2 + E3.

Search per frame:
1. exhaustive integer ZNCC at level 2 (5×5 template);
2. 3×3 local maxima, greedy suppression within `2·hw_L`, top K = 16;
3. ±2 px refinement at each finer level;
4. separable parabolic sub-pixel fit at level 0.

Accept when the best level-0 score `z1 ≥ τ` **and** `z1 − z2 ≥ δ`, where `z2` is the best other
refined peak farther than `hw`. Two guards against wrong locks:
- **Selection-time self-test.** The selection frame must re-detect its own point (within
  1 px) under the same rule. Otherwise re-detection is disarmed for that selection, and
  `LkTracker::redetect_armed()` reports it. This catches lookalikes visible at selection,
  and gratings.
- **Spatial prior** `max_radius_frac`. While searching, only positions within `r·width` of the
  last known point are scored and accepted.

**New fixtures** (generator, `Distractor`: a static second disc, optional frame range). Each
one is `occlusion_full` (target fully occluded at frames 30–49) plus a disc 190 ref units to
the left:
- `distractor_twin`: an identical twin, visible throughout.
- `distractor_late_twin`: an identical twin that appears at frame 30, while the target is
  occluded. This is ADR-0010's predicted structural gap.
- `distractor_similar`: same texture statistics, other seed, visible throughout.

The existing 61 scene × resolution runs render the same as before. `lk_fingerprint` is
identical on all 61 shared lines (diffed against `8789d83` in a worktree with its own
`CARGO_TARGET_DIR`). From here on the corpus has **70 runs**, so the summary means are not
comparable to the 61-run tables above. The LK18 and E3+E2 rows are recomputed below.

**Results** (`ablation_tables`, 2026-11-12, M1, 70 runs):

| alg | mean ok | Σ wrong-lock | Σ false-track | worst good-scene Δp95 | worst good-scene ok drop | more wrong-lock than LK18 |
|---|---|---|---|---|---|---|
| LK18 | 73.4% | 654 | 277 | +0.000 | none | none |
| E3+E2 zncc≥0.7 | 73.4% | 0 | 0 | +0.070 (rotate_15 1280) | none | none |
| E4 τ.85 δ.1 K16 L2 (full frame) | 87.6% | 60 | 57 | +0.070 (rotate_15 1280) | none | none |
| E4 τ.8 / τ.9, same | 87.6% | 60 | 57 | +0.070 | none | none |
| E4 δ.05 | 87.7% | 60 | 57 | +0.070 | none | none |
| E4 δ.2 | 78.1% | 20 | 19 | +0.070 | none | none |
| E4 K8 / K32 / L1 | 87.6% | 60 | 57 | +0.070 | none | none |
| E4 L3 | 79.8% | 20 | 19 | +0.070 | none | none |
| E4 τ.85 r.15 | 89.5% | 0 | 0 | +0.070 | none | none |
| E4 τ.85 r.2 / r.25 / r.3 | 89.4% | 0 | 0 | +0.070 | none | none |
| **E4 τ.9 δ.1 K16 L2 r.25 (candidate)** | **89.4%** | **0** | **0** | +0.070 (rotate_15 1280) | none | none |

(E1/E2/E3 rows on 70 runs are in the test output. Their ordering is unchanged from §E1–§E3.)

Rows that changed, at 640×480 (ok / wrong-lock / false-track / final EPE):

| scene | LK18 | E3+E2 | E4 full frame | E4 candidate (r.25) |
|---|---|---|---|---|
| fast_40px | 37%/18/0/185.80 | 37%/0/0/lost | 100%/0/0/0.04 | 100%/0/0/0.04 |
| occlusion_full | 60%/20/19/20.23 | 60%/0/0/lost | 98%/0/0/0.09 | 100%/0/0/0.06 |
| edge_left | 50%/37/2/26.08 | 50%/0/0/lost | 99%/0/0/0.06 | 99%/0/0/0.06 |
| edge_right | 49%/35/6/52.36 | 49%/0/0/lost | 97%/0/0/0.05 | 97%/0/0/0.05 |
| edge_top | 49%/35/6/29.59 | 49%/0/0/lost | 94%/0/0/0.02 | 96%/0/0/0.02 |
| edge_bottom | 50%/37/6/31.24 | 50%/0/0/lost | 99%/0/0/0.55 | 99%/0/0/0.55 |
| distractor_twin | 60%/20/19/20.23 | 60%/0/0/lost | 60%/0/0/lost | 60%/0/0/lost |
| distractor_late_twin | 60%/20/19/20.23 | 60%/0/0/lost | **60%/20/19/224.48** | **100%/0/0/0.06** |
| distractor_similar | 60%/20/19/20.23 | 60%/0/0/lost | 98%/0/0/0.09 | 100%/0/0/0.06 |
| repeated_texture | 3%/0/0/lost | 3%/0/0/lost | 3%/0/0/lost | 3%/0/0/lost |

(320 and 1280 follow the same pattern. At 320, the candidate's `edge_right` / `edge_bottom`
recoveries come back with a 1.0 / 1.4 px residual offset (final EPE 0.98 / 1.36). That is within
tolerance; see Reading. Every gate-scene row is identical to E3+E2, because none of them
loses the track.)

**Reading.**
- **Findings 2 and 4 are closed.** `fast_40px` goes 37 % → 100 % at every resolution, and `edge_*`
  goes ≈ 50 % → 93–99 %, re-acquiring 1–4 frames after the target is back. Finding 3 gets its
  recovery: `occlusion_full` 60 % → 100 %, re-found on the first frame the target is visible
  (recover 0). E2 + E3 had already made these losses explicit. E4 turns them back into tracking.
- **Finding 7 stays fixed.** `repeated_texture` is refused twice: LK's texture test, and the
  self-test (the grating fails δ). The `distractor_twin` self-test disarms re-detection, so it
  behaves exactly like E3+E2: lost, not locked.
- **The predicted gap is real without the prior.** Full-frame re-detection accepts a lookalike
  that appears while the target is lost: `distractor_late_twin`, 20 wrong-lock + 19 false-track
  frames per resolution, i.e. all 60 / 57 frames in the full-frame rows. Margin can't help,
  because the twin is the only peak. δ = 0.2 happens to refuse it at 640/1280, but it also refuses
  most true recoveries (78.1 %).
- **The spatial prior closes the fixture's gap at no measured cost.** Every radius from 0.15 to
  0.3 of the width gives 0 wrong-lock / 0 false-track, and recovers the true target in
  `distractor_late_twin`. The twin sits ≈ 0.32 w from the loss point, so **the corpus only brackets r
  to [0.15, 0.32)**. r = 0.25 is the middle: 4.6× the 35/640 guard, which comfortably covers
  `fast_40px` (≈ 80–120 units from the held point). A lookalike within r of the loss point is
  still accepted. That is the remaining structural limit, unmeasured on real faces.
- **τ.** 0.8–0.9 makes no corpus difference. A probe with the target removed from a
  `head_sway` frame found background-only best scores of 0.885 / 0.889 (640 / 1280,
  full frame) and 0.84 / 0.88 within r.25. They were refused only by δ. **τ = 0.9** is chosen so
  that this background is rejected on score alone. The τ.9 r.25 rows equal τ.85 r.25 on 69 of 70
  runs. The exception is `edge_top` 320: τ.9 re-acquires one frame later (ok 94 → 93 %, recover 3 → 4)
  but lands precisely (final EPE 1.44 → 0.04 px). τ.85 accepted a partly-visible, lower-score
  match there.
- **K and level.** K 8–32 and level 1 or 2 are identical. Level 3 (3×3 template at 640) misses
  true targets. Level 2 is chosen for cost.
- **Re-acquisition precision.** At 640/1280 the re-acquired point is within ≤ 0.6 px. At 320,
  `edge_right` / `edge_bottom` keep a 1.0 / 1.4 px offset after recovery. HYPOTHESIS, partly
  supported by the τ.85 → τ.9 change on `edge_top`: at 320 the 21×21 window spans
  42 ref units around a point 14 units from the disc centre, so the template includes
  background that moved relative to the target. Under absolute mapping (ADR-0006) 1.5 px at 320
  is a visible cursor offset. A follow-up candidate: refine the match with one LK solve against
  the selection pyramid. Not done here.

**Decision.** KEEP. Candidate: τ = 0.9, δ = 0.1, K = 16, search level 2, r = 0.25 w
(`RedetectParams::default()` plus `max_radius_frac: Some(0.25)`, `support::e4_candidate()`). It
passes the task gate: worst gate-scene Δp95 +0.070 (from E3, unchanged), no gate-scene ok
drop, no new wrong-lock on any scene. The default stays off in this commit (fingerprint
discipline). The default-tracker ADR decides it.

**Timing** (M1, divan, medians, 100 samples). `lk_searching_step` is one step while lost:
pyramid build + search. That is the per-frame cost for as long as the target is away.
`lk_step` is shown for reference:

| Bench | 320×240 | 640×480 | 1280×720 → 640×360 |
|---|---|---|---|
| lk_step | 101 µs | 281 µs | 220 µs |
| searching, r = 0.25 (candidate) | 315 µs | 588 µs (max 613) | 528 µs |
| searching, full frame | 361 µs | 916 µs (max 1.02 ms) | 714 µs |

That is within the §8 budget (p95 < 2 ms at 640), even full-frame. There are zero steady-state
allocations (divan reports no allocation rows; scratch is sized in `select`). The
self-test adds one search to `select`. A tracked step costs the same as E3+E2: the search only
runs once the 2018 policy says `Lost`.

## E5 — adaptive search / guard: NOT ATTEMPTED

The task scopes E5 to "only if E4 doesn't already fix `fast_40px`". E4 takes it to 100 % at
every resolution with no guard change, so the 35/640 guard stays as the 2018 preset (ADR-0009).
The guard still produces one `Held(MotionOutlier)` frame per over-guard jump before
re-detection catches up. That is visible in `fast_40px` as the step pattern, not as an ok loss.

## E6 — multi-point consensus: NOT ATTEMPTED

Optional, "only if E1–E4 leave a measured gap". On the corpus the candidate leaves the
expected refusals (`textureless`, `repeated_texture`, `distractor_twin`), and nothing else below
93 % ok. The remaining risks (head roll vs a fixed template, lookalikes within the prior,
320 re-acquisition offset) need real recordings (Phase 6) before a multi-point design is justified.
