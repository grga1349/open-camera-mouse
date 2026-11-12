# Tracking improvements (Phase 4)

Ablations against B-LK18 (`LkTracker::default()`) on the Phase 3 corpus. Task 0006.
Baseline numbers, corpus and metric definitions: `TRACKING_BASELINE.md`. Every experiment is
kept or rejected on measured deltas from the same corpus. Synthetic-only caveats from Phase 3
still apply.

## How to reproduce

```sh
# exact LK output fingerprint (must be unchanged by "no behavior change" work)
cargo test -p tracking-core --release --test baseline lk_fingerprint -- --ignored --nocapture
# accuracy tables (B-LK18 vs B-NCC, ~5 min)
cargo test -p tracking-core --release --test baseline baseline_tables -- --ignored --nocapture
# ablation: every LK variant in `variants()` vs B-LK18 (~1.5 min; filters BASELINE_SCENE, ABLATION)
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

## E3–E6

Not started. See `tasks/active/0006-phase-4-tracking-improvements.md`.
