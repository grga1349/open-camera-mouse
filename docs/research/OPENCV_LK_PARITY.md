# Rust LK vs OpenCV: numerical parity research

Phase 6d, part 3, slice 4 (task 0011, brief quoted there). Date: 2026-11-16.
Research only: **no production code changed.** Evidence labels follow `RE_EVIDENCE.md`:
VERIFIED (read in source or reproduced by experiment), MEASURED (number from the harness
below), INFERRED, HYPOTHESIS.

## Conclusion

**Our pyramidal LK is competitive with OpenCV 4.12.** Given identical frames and points, and
OpenCV's default `minEigThreshold` mapped into our units, the two solvers produce the same
status on every real pair (0 disagreements out of 3,164 pairs). Their positions differ by a
median of 0.0012 px (p95 0.006, max 0.046 px). These differences are fully explained:
an experimental fork that emulates OpenCV's u8 pyramid and fixed-point sampling brings the
max difference to 0.026 px and makes 556 pairs bit-equal.

On well-posed input the ground-truth accuracy is identical to about four decimals. On
ill-posed input (repeated texture, textureless, contrast ≤ 8 grey levels) both solvers are
wrong, and ours is less wrong. There the recommended tracker refuses to report Tracking.

OpenCV is faster per frame: 181 µs vs 289 µs single-threaded on an M1. The difference is
almost all in pyramid construction, where OpenCV uses NEON code from KleidiCV. Both are under
1% of a 33 ms frame budget. Ours makes zero allocations per frame, compared with OpenCV's 35.

No parity defect was found that needs fixing. None of OpenCV's extra mechanisms showed a
measurable benefit for our use: oscillation half-step, u8/fixed-point arithmetic, and
reflect-101 borders.

## 1. Sources

| Ref | Source | Notes |
|---|---|---|
| **CV4** | OpenCV 4.12.0 `modules/video/src/lkpyramid.cpp`, `modules/imgproc/src/pyramids.cpp`, `modules/video/include/opencv2/video/tracking.hpp` (GitHub tag `4.12.0`) | Byte-identical to 4.13.0 for `lkpyramid.cpp` and `pyramids.cpp` (VERIFIED, diffed), so the 4.13.0 binary below runs the 4.12.0 generic code paths |
| **CV21** | OpenCV 2.1.0 `src/cv/cvlkpyramid.cpp`, `cvsamplers.cpp`, `cvpyramids.cpp` from SourceForge `OpenCV-2.1.0.tar.bz2` (sha256 `26061fd5…397f`). The 2.1 tag is not on GitHub | 2018 shipped `cv210.dll` (RE_EVIDENCE / camera-mouse-reverse `02-tracking-algorithm.md`) |
| **HAL** | Homebrew OpenCV 4.13.0 arm64: carotene `hal_tegra_hal.hpp`, KleidiCV 0.7.0 `kleidicv_hal.{h,cpp}` | Carotene's LK and Scharr hooks are armv7-only, so they are not active here. KleidiCV replaces `pyrDown` and `ScharrDeriv` |
| **RS** | `crates/tracking-core/src/{lk,pyramid,image}.rs` at `8303dd7` | Production solver |

Key line references (CV4 = `lkpyramid.cpp` unless noted):

| Topic | CV4 | CV21 (`cvlkpyramid.cpp`) | RS |
|---|---|---|---|
| Defaults: `pyrBorder=REFLECT_101`, `derivBorder=CONSTANT`, `minEigThreshold=1e-4` | `tracking.hpp:126-127, 186` | — (no min-eigen test) | `lk.rs:78-119` (`recommended()`, `default()`) |
| Pyramid downsample | `pyramids.cpp:51` (`FixPtCast`, `(s+128)>>8`), `:1266, :1297` | `cvpyramids.cpp:214, 239` (reflect-101) | `pyramid.rs:26` (f32, reflect-101) |
| Pyramid padding (image, derivs) | `:811` (reflect-101), `:828` (const 0) | none; window clipped (`intersect`, `:612`) | none; clamp-to-edge sampling (`image.rs:52-69`) |
| Scharr derivative | `:89-150` (raw `short`, ×32 of intensity/px) | smoothing `(3,10,3)/32 × [-1,0,1]` on the resampled patch (`:571`) | per-window Scharr ÷32 (intensity/px) |
| Window bounds test | `:245, :520` (top-left `< -winSize` or `>= cols`) | clip + `isz < 3` (`:617`) | `lk.rs:222, 292, 338` |
| Bilinear weights | `:259` (`W_BITS=14`, rounded) | `cvsamplers.cpp:149` (float, `a=max(a,1e-4)`) | `image.rs:60` (f32) |
| Fixed-point I and J | `:463, :663` (`CV_DESCALE`, I at 1/32 intensity) | float | f32 |
| G, min-eigen, det | `:492-510` (`FLT_SCALE=2^-20`, `minEig < thr \|\| D < FLT_EPSILON`) | `D < DBL_EPSILON` (`:701`) | `lk.rs:264` |
| Convergence | `:694` (`δ² <= ε²`), `ε²` at `:1210` | `:723` (`< ε²`), `ε²` at `:412` | `lk.rs:324` (`< ε²`) |
| Oscillation half-step | `:697-700` | `:726` | none |
| Failure scope | level 0 only; coarse levels are skipped | **any level is terminal** (`:604`) | level 0 only; coarse levels are skipped |
| Patch size | `winSize` = full size | `patchSize = winSize*2+1` (`:575`, also `:944`) | `half_window=10` → 21×21 |

No OpenCV code was copied into this repository. The harness only *links* against OpenCV.

## 2. Method

A standalone harness, kept outside the workspace and not shipped, handles both solvers.

**Rust side** (`lkparity`, depends on `tracking-core` by path, opt-level 3):
- Runs production `track_point` and the complete `LkTracker`.
- Includes an experimental fork of the pure solver with switchable OpenCV behaviours:
  - `osc`: oscillation half-step
  - `u8pyr`: u8 pyramid with `(s+128)>>8` rounding
  - `fixed`: 14-bit weights, I at 1/32 intensity, `FLT_SCALE` accumulation
  - `cvborder`: reflect-101 image padding, const-0 derivative padding, OpenCV bounds test
- **With no flags set, the fork is bit-identical to production on all 6,243 pairs.**

**OpenCV side** (`cvlk.cpp`, links only `core`, `imgproc` and `video`):
- Calls `buildOpticalFlowPyramid` with derivatives, then `calcOpticalFlowPyrLK`.
- Settings: 21×21 window, `maxLevel 3`, criteria `COUNT|EPS 20/0.03`, `minEigThreshold 1e-4`.
- `hal_check` **verified the KleidiCV pyramid and Scharr output bit-equal** to plain reference
  implementations at all four levels, on synthetic and real frames. So the HAL does not
  change numerics.

**Fixtures:**
- 49 synthetic sequences with exact ground truth:
  - the `tracking-core` test corpus at 640×480 (motion, rotation, lighting, occlusion,
    distractors, borders, static)
  - sweeps: sub-pixel motion clean/noisy, fast 12 px/frame, contrast 4–64 grey levels,
    border distance 0–25 px
- 5 real segments from local recordings S1 and S2 (3,164 pairs). Their reference points are
  the recommended tracker's track.

**Recordings stayed local.** Only aggregate numbers appear here.

**Variants:**

| Variant | What |
|---|---|
| `cv` | OpenCV 4.13.0 (= 4.12.0 code) |
| `rs_eq` | production pure LK, `min_eigen 0.1024` (= OpenCV 1e-4, see §3.2) |
| `rs_lk18` | production `LkParams::default()` (B-LK18, `min_eigen 1.0`) |
| `rs_gb` | `rs_eq` + gain/bias compensation |
| `x_osc`, `x_u8`, `x_border`, `x_fixed` (u8+fixed), `x_cvemu` (all flags) | fork experiments |
| `sys_lk18`, `sys_rec` | complete `LkTracker` (B-LK18 preset / recommended preset, ADR-0011) |

**Modes:**
- **pair**: every pair starts from the ground-truth point, with a backward track for FB error.
- **chain**: one long track; failure ends it.
- **system**: the complete tracker, with validation, redetection and loss policy.

**Environment:**
- Apple M1, 8 cores, 8 GB, macOS 14.7.3
- rustc 1.99.1
- Apple clang `-O2`
- Homebrew OpenCV 4.13.0 (TBB, 8 threads available)

## 3. Investigation items

### 3.1 f32 bilinear vs OpenCV fixed point: VERIFIED, MEASURED

- OpenCV rounds bilinear weights to 14 bits and stores I with 5 fractional bits (1/32 grey
  level). It accumulates G and b in float with `FLT_SCALE = 2^-20`.
- Ours is plain f32 throughout.

Agreement with `cv`, pair mode, |Δ| in px (both solvers report ok):

| Data | Variant | Median | p95 | p99 | Max | Bit-equal | Status disagreements |
|---|---|---|---|---|---|---|---|
| Real (3,164) | `rs_eq` | 0.00116 | 0.0060 | 0.011 | 0.046 | 1 | 0 |
| Real | `x_u8` | 0.00103 | 0.0045 | 0.008 | 0.029 | 0 | 0 |
| Real | `x_fixed` = `x_cvemu` | 0.00003 | 0.00053 | 0.0013 | 0.026 | 556 | 0 |
| Synthetic | `rs_eq` | 0.00019 | 0.024 | 0.34 | 23.2 | — | — |
| Synthetic | `x_cvemu` | 0.00003 | 0.00018 | 0.0013 | 0.117 | 931 | — |

- The fixed-point and u8 emulation accounts for nearly all of the difference.
- Accuracy against ground truth does not change: on the motion, rotation, occlusion,
  distractor and static groups the EPE of `cv` and `rs_eq` is identical to about four
  decimals (motion: median 0.023, p95 0.091 px).
- The quantisation is far below sensor noise.
- **f32 is not less accurate.** Verdict: an intentional difference with no defect.

### 3.2 Scharr scaling and min-eigenvalue normalisation: VERIFIED

OpenCV keeps Scharr output raw, at 32× intensity/px. Its I values are at 32× and
`FLT_SCALE = 2^-20`, so its `A` entries equal ours ÷ 1024. Its `minEig` is `λ_min / (w·h)`,
the same normalisation as ours.

> **Our `min_eigen` = OpenCV `minEigThreshold` × 1024.** OpenCV's default 1e-4 ≡ 0.1024.

This mapping is confirmed empirically: `rs_eq` and `cv` disagree on status on 0 of 3,164 real
pairs.

| Preset | `min_eigen` | Relation to OpenCV default |
|---|---|---|
| B-LK18 | 1.0 | about 10× stricter. Rejects 1,984 of the 3,164 real pairs that OpenCV accepts |
| Recommended | 0.25 + ratio 0.05 | Rejects none of them |

- OpenCV's determinant test is `D < FLT_EPSILON` on the ÷1024 scale, which is about 0.13 in
  our units vs our `f32::EPSILON`.
- This difference is irrelevant: the min-eigen test always dominates, because
  λ_min ≥ 0.1·441 ⇒ det ≳ 1,900.

### 3.3 Gaussian pyramid rounding and borders: VERIFIED, MEASURED

Both use a 5-tap `[1 4 6 4 1]/16` separable filter with reflect-101. OpenCV rounds each level
to u8 with `(s+128)>>8`; ours stays f32.

**On well-posed input:** no measurable effect.

**On ill-posed input:** the coarse-level rounding moves OpenCV to a *different* local
minimum. Ours is usually closer to the truth. Pair EPE in px:

| Sequence | `cv` p50 / p95 | `rs_eq` p50 / p95 | Note |
|---|---|---|---|
| repeated_texture | 1.06 / 7.82 | 0.84 / 2.84 | `x_u8` reproduces `cv` (1.01 / 7.66) |
| textureless | 1.66 / 4.15 | 1.00 / 2.79 | `x_u8` ≈ `cv` |
| contrast 4 grey levels | 1.53 / 3.04 | 1.54 / 3.02 | equal |
| contrast 16 | 0.33 / 0.90 | 0.33 / 0.90 | equal |
| contrast ≥ 32 | equal | equal | |

- On these inputs both solvers are wrong by more than 1 px.
- **The complete recommended tracker reports Tracking on 0 frames** for repeated_texture,
  textureless and contrast ≤ 16. That is the correct product behaviour.
- At contrast 32, `sys_rec` tracks 59/59 frames vs `sys_lk18` 48/59.
- Verdict: keep f32.

### 3.4 Oscillation detection and half-step correction: MEASURED

Both CV4 and CV21 apply this: if `j>0` and `|δ+δ_prev| < 0.01` on both axes, step back by
`δ/2` and stop. Our solver has no such test. Experiment `x_osc`:

- **Fires on 25 synthetic pairs**, all in repeated_texture, and **on 0 real pairs**.
- Mean iterations: 6.97 → 6.90 on the affected sequences.
- EPE changes are mixed, about ±0.07 px. Real-data agreement is unchanged (identical row
  to `rs_eq`).
- Production level-0 iterations: median 2, p95 4 (real p95 3–4).
  - Synthetic: 49 of 4,616 pairs hit the 20-iteration cap, all on ill-posed input.
  - Real: 2 of 3,164 pairs hit the cap.

Verdict: **no measurable benefit, so not adopted.**

### 3.5 Boundary behaviour and failure conditions: VERIFIED, MEASURED

**Where the conventions differ:**

| | OpenCV 4.x | Ours |
|---|---|---|
| Image padding | reflect-101 | clamp-to-edge sampling |
| Derivative padding | constant 0, so gradients vanish outside the image | — |
| Window outside the image | allowed up to `winSize` beyond the border | — |
| Final position check | none | `contains(est)` returns `OutOfFrame` |

Both treat a coarse level the same way: a degenerate or outside window skips that level, and
only level 0 fails. **OpenCV 2.1 differs:**
- Failure at any level is terminal.
- The window is clipped to the image rather than extrapolated, and G is recomputed for each
  clipped size.
- Its u-adjust centres I and J the same way, so it introduces no bias.

**Border sweep, pair mode, offset `d` px from the edge.** p95 EPE (`cv` vs `rs_eq`):

| d | `cv` | `rs_eq` |
|---|---|---|
| 0 | 0.096 | 0.152 |
| 1 | 0.078 | 0.135 |
| ≥ 8 | identical | identical |

`x_border` reproduces `cv` (p95 |Δ| 0.0056).

**The other side of that convention:**
- On `c_edge_right` t=34 (point x=629) OpenCV returned a **confident 11.9 px wrong** result,
  where ours reports `OutOfFrame`.
- In chain mode on the border group, `rs_eq` had 3 false-tracking frames and `cv` had 0.

The evidence is mixed and small. In the complete tracker (`sys_rec`) the border group shows
81.9% tracked-ok and 0 wrong frames, because the loss-policy edge margin
(`edge_margin_frac`) covers this zone. Verdict: no change.

### 3.6 Effect of gain/bias compensation on convergence: MEASURED

Level-0 iterations and FB error, `rs_eq` → `rs_gb`:

| Group | Mean iterations | Pairs at iteration cap | FB p95 (px) |
|---|---|---|---|
| Lighting | 1.68 → 1.28 | 1 → 0 | 0.0040 → 0.0037 |
| Motion / rotation | unchanged (±0.04) | — | — |
| Real | 2.32 → 2.22 (p95 4 → 3) | 2 → 0 | 0.069 → 0.060 |

- **brightness_step:** both `cv` and `rs_eq` make a 37 px blunder. `rs_gb` max EPE is 0.095.
- **Lighting group with the complete tracker:** EPE p95 0.243 (`sys_rec`) vs 6.4 (`sys_lk18`).
- **Positions on real data** move by a median of 0.054 px under gain/bias (max 5.9 px). This
  is a different model, not a parity gap.

Verdict: an intentional improvement over OpenCV. Keep it.

### 3.7 SIMD and single-point optimisations: MEASURED

Timings in µs on an M1. Rust is single-threaded; OpenCV was measured with 1 and 8 TBB
threads. Sequences: `c_head_sway` (synthetic) and `r_S2_2` (real); they show the same pattern.

| Stage | Rust p50 / p95 | OpenCV 1 thread p50 / p95 | OpenCV 8 threads p50 / p95 |
|---|---|---|---|
| Pyramid (4 levels, 640×480) | 247 / 252 (f32, no derivatives) | 177 / 197 (u8 + derivatives, KleidiCV NEON) | 164 / 270 |
| LK for one point | 41 / 47 | 6 / 8 (synthetic), 11 / 13 (real) | same |
| Per frame, pyramid reused | **289 / 299** | **181 / 188** | 207 / 330 (CPU/wall 7.6) |
| Per frame, `calcOpticalFlowPyrLK` builds both pyramids | — | 325 / 558 | 405 / 790 |
| Complete `LkTracker::step`, recommended preset | 304 / 314 | — | — |
| Allocations per frame (steady state) | **0** | 35 | 1,424 |

- **TBB threading makes p95 worse** for one 640×480 frame, at 7.6× the CPU.
- **Isolated experiments**, both result-equivalent and neither adopted:
  - **Constant-weight window sampler.** One integer base and one set of bilinear weights per
    window, and contiguous row reads when the window lies inside the level; this is what
    OpenCV does.
    - LK time: 41 → 22 µs (−46%).
    - Max position difference vs production: 3.4e-5 px. 0 status changes on either test sequence (35,600 + 17,300 timed calls).
  - **ROI pyramid.** The pyramid is 85% of frame time. Building it over a crop:

    | Crop | Pyramid build |
    |---|---|
    | 640×480 | 235 µs |
    | 384² | 116 µs |
    | 320² | 83 µs |
    | 256² | 55 µs |

    A tracker ROI would have to cover the motion bound plus window at every level, and the
    redetection search.
- All of these figures are under 1% of a 33 ms frame. Verdict: optional. Do them only if
  `OCM_PERF` ever shows the tracker on the critical path.

### 3.8 OpenCV 2.1 C API `winSize=(10,10)` vs our 21×21: VERIFIED

- CV21 `cvlkpyramid.cpp:575` (and `:944` in the affine variant) sets
  `patchSize = cvSize(winSize.width*2+1, winSize.height*2+1)`. So the C API treats `winSize`
  as a half-size, and 2018's `(10,10)` gives a **21×21** patch.
- The 2.1 C++ wrapper `cv::calcOpticalFlowPyrLK` uses `halfWin = (winSize-1)/2` (`:63`). That
  is the full-size convention that 4.x kept. 2018 used the C API (camera-mouse-reverse).
- ADR-0009 decision 2 can therefore move from INFERRED to VERIFIED. A note was added there.

Other 2.1 facts, recorded for completeness and **not** to be emulated:

| 2.1 behaviour | Our choice |
|---|---|
| No min-eigen test; only `D < DBL_EPSILON` in double | ADR-0009 reason (a) defect: it tracks noise on flat walls |
| Failure at any pyramid level is terminal | Ours skips coarse levels, like 4.x |
| Derivatives are computed on the resampled patch with `(3,10,3)/32` smoothing, rather than on the level | — |
| Bilinear weight floor `a = max(a, 1e-4)` (`cvsamplers.cpp`) | — |

## 4. Complete trackers (system mode, synthetic)

| Group | `sys_rec` | `sys_lk18` |
|---|---|---|
| Occlusion | 93.5% tracked-ok, 0 wrong, 0 false | 77.4%, 52 wrong, 235 false |
| Border | 81.9%, 0 wrong | 77.0%, 144 wrong |
| Distractor | 83.7%, 0 wrong | 59.2%, 60 wrong |
| fast 40 px/frame | 0 wrong | 18 wrong |
| Lighting EPE p95 | 0.243 | 6.4 |

In chain mode, pure `fast_40px` is wrong in both `cv` and `rs_eq` (about 145 px). That
motion is beyond any 4-level, 21×21 pyramid. The validation layers of `sys_rec`, not the core
solver, are what turn this into 0 wrong frames. **OpenCV has no equivalent layers.**

**Static jitter and drift:**
- **`static_60s` (synthetic noise):**

  | Metric | `rs_eq` | `cv` |
  |---|---|---|
  | Step p50 / p95 | 0.025 / 0.054 px | 0.025 / 0.054 px |
  | Spread p95 | 0.122 px | 0.126 px |
  | End drift | 0.20 px | 0.22 px |

- **S1 real ("still" recording):** step p50 is 0.127 px and p95 1.36 px for both solvers. The
  head itself moved, with a spread of about 80 px. `sys_rec` reported loss at frame 592.

**Real FB error, pair mode:**

| Segment | `cv` median / p95 | `rs_eq` median / p95 |
|---|---|---|
| S1_0 | 0.0128 / 0.104 | 0.0124 / 0.100 |
| S2_0 | 0.0152 / 0.048 | 0.0143 / 0.047 |
| S2_2 | 0.0041 / 0.031 | 0.0040 / 0.030 |
| S2_3 | 0.0138 / 0.095 | 0.0136 / 0.095 |

B-LK18 fails 632/688 pairs on S1_0, 429/429 on S2_0 and 923/1,120 on S2_3.

## 5. Parity defects vs intentional differences

| Difference | Class | Evidence |
|---|---|---|
| f32 pyramid and sampling vs u8 / fixed point | Intentional; equal or better | §3.1, §3.3 |
| Clamp-to-edge vs reflect-101 + const-0 derivatives | Intentional; mixed within 3 px of the edge, covered by the edge margin | §3.5 |
| No oscillation half-step | Intentional; no measurable benefit | §3.4 |
| Convergence `<` vs `<=` ε² | Negligible (equality has measure zero) | — |
| Final `contains(est)` check | Intentional; stricter, avoids out-of-image results | §3.5 |
| `min_eigen` units (×1024) | Not a defect; a unit mapping, now documented | §3.2 |
| Gain/bias, ZNCC gate, ratio test, redetection | Intentional product improvements beyond OpenCV | §3.6, §4 |

**No parity defects found.**

## 6. Recommended actions, ranked by impact

1. **Keep the solver as is.** It agrees with OpenCV to 0.001 px median on real frames, and no
   OpenCV mechanism measurably improves it.
2. **Keep the recommended preset (ADR-0011) as the default.**
   - On real data, B-LK18's `min_eigen 1.0` rejects 63% of the pairs that OpenCV accepts.
   - The validation layers, not the core solver, are what beat OpenCV on occlusion,
     distractors, lighting and fast motion.
3. **Document the `min_eigen` ↔ `minEigThreshold` ×1024 mapping** in the `LkParams` docs when
   that file is next touched. This is a doc change only; it was not made here because of the
   research-only rule.
4. **Optional performance work, each as an isolated experiment** with a near-equality test
   (≤ 1e-4 px, 0 status changes) and an `OCM_PERF` before/after:
   - (a) a constant-weight window sampler: −19 µs/frame;
   - (b) an ROI pyramid: up to −150 µs/frame.

   Only worth doing if profiling ever shows the tracker matters, because today it costs under
   1% of the frame budget.
5. **Border convention: no action.** Revisit only with a dedicated near-edge fixture showing a
   system-level gain, because the reflect-101 variant also produced an 11.9 px confident-wrong
   case.
6. **Rejected:** oscillation half-step, u8 pyramid, and fixed-point arithmetic. None showed a
   benefit, and the u8 pyramid is worse on ill-posed input.
