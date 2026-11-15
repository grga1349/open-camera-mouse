# Head-pointer stability — research inputs (2026-11-15)

**Status: INPUT, not a finished report.** Two documents the user brought back on 2026-11-15
(Phase 6b, task 0009, issues #2 losses and #5 jitter), kept here so later sessions don't depend
on chat history. The final report they ask for is
`docs/research/HEAD_POINTER_STABILITY_RESEARCH.md` (not written yet).

- **A — preliminary findings** of the online research agent run with
  `TEST_DRIVE_RESEARCH_PROMPT.md`. Hypotheses and starting values, explicitly *not* validated.
- **B — research brief update** adding the Camera Mouse 2018 reverse-engineering facts. Every
  2018 fact in B is already in `RE_EVIDENCE.md` / `../camera-mouse-reverse/research/05-*.md`
  (checked 2026-11-15: 10×10 window, level 3, 35 px guard, `0.95·new + 0.05·prev`, 70 ms
  worker sleep, desktop-size mapping). No conflict.

## Reconciliation with this repository (checked 2026-11-15)

| Claim in A/B | Our code / evidence |
|---|---|
| ≈ 48 screen px per camera px on 2 × 2560 | VERIFIED by formula: `map_raw` scales by desktop width (`pointer-core`), same as 2018 `MainForm.b`. 2018 has the same amplification on this desktop. |
| Fixed selection-frame ZNCC gate 0.7 causes the losses | VERIFIED on the one unscripted recording (`TEST_DRIVE.md` #2: every loss `AppearanceChanged`, ZNCC 0.97 → < 0.7 over ~1 s of head turn). |
| 2018 smoothing barely smooths | VERIFIED: our default `smoothing = 0.05` is 2018's `Damping 0.95` (weight 0.95 on the new position). |
| Smoothing must use timestamp `dt` | **Partly violated:** `pointer-core` smoothing is dt-aware, but the controller is given the UI-loop time (`Shell::now_us` at pump), not the camera timestamp. See the lag investigation in `TEST_DRIVE.md`. |
| Cursor update rate vs camera rate | We move the pointer once per processed camera frame (≤ 30 Hz), from the UI loop. 2018: ≤ 14 Hz worker. |

## What we take from it (proposed order, to be agreed with the user)

1. Fix the frame path first (lag investigation): no filter can be judged on top of a jittery,
   UI-coupled timeline.
2. Smallest experiments, on recordings via `crates/app-core/tests/replay.rs`, no UI change:
   (a) gain per display instead of desktop bounding box; (b) 1€ filter vs current EMA in
   camera space, timestamp `dt`; (c) multi-point feature cloud + forward–backward check +
   robust similarity → virtual anchor, compared with single-point LK on losses and drift.
3. Only then the full report, scoped to whatever the experiments leave open.

---

## A — Preliminary stability research handoff (verbatim)

**Date:** 2026-11-15 · **Status:** Preliminary findings and engineering hypotheses — NOT a
completed research report.

### 1. Current engineering observations

Pipeline at the time: 640×480 @ 30 FPS; single-point pyramidal LK (21×21, 4 levels, 20 iter,
eps 0.03); gain/bias compensation; fixed selection-frame ZNCC gate 0.7; coarse-to-fine template
recovery with 3-frame confirmation; absolute mapping gain 6; EMA + rubber-band dead zone; two
2560-px-wide monitors.

Cursor gain ≈ `5120 / 640 × 6 = 48` screen px per camera px:

| Camera-point error | Cursor error |
|---|---|
| 0.10 px | 4.8 px |
| 0.25 px | 12 px |
| 0.50 px | 24 px |
| 1.00 px | 48 px |

**Inference:** the observed 10–20 px instability may be only 0.21–0.42 camera px of position
variation. Filtering alone is unlikely to solve both stability and responsiveness.

Separate: (1) measurement noise, (2) true involuntary head micro-movement, (3) LK drift from the
selected anatomical point, (4) pose-dependent appearance change, (5) mapping amplification,
(6) filter lag and dead-zone behaviour. These need different solutions.

### 2. Problem 1 — tracking continuity

**A. Multi-point tracking with robust consensus.** Track ~20–40 well-distributed points around
the selected location, keep the selected point as a virtual anchor. Per frame: pyramidal LK;
forward–backward check; reject unreliable tracks; robust 2D similarity (RANSAC or similar);
transform the anchor; report confidence; replenish points. Candidate (not validated) values:
20–40 points, FB threshold 0.5–1.5 px, RANSAC reprojection 1–2 px, min inliers 6–10, reject
spatially clustered consensus. **Verified:** Kalal et al. 2010 — FB error is useful for failure
detection and complementary to appearance correlation
(https://doi.org/10.1109/ICPR.2010.675, https://cmp.felk.cvut.cz/~matas/papers/kalal-2010-fb_track-icpr.pdf).
**Inferred:** should beat isolated LK on weakly textured skin if the anchor is rigid relative
to the features. **Limitation:** a 2D similarity can't describe a rotating 3D face; features at
different depths move differently.

**B. Drift-resistant appearance adaptation.** Immutable original template + adaptively updated
recent template + pose keyframes; update only after strong geometric consensus; original as an
anchor, not an unconditional pose-invariant gate. **Verified:** Matthews, Ishikawa & Baker 2004,
*The Template Update Problem* (https://doi.org/10.1109/TPAMI.2004.16). **Inferred:** the fixed
ZNCC gate rejects legitimate rotations. Lowering 0.7 alone may trade false losses for silent
drift.

**C. Confidence states:** `TRACKING`, `DEGRADED` (constrain/suspend pointer), `RECOVERING`,
`LOST`. Confidence must include drift diagnostics; a confident biased track is not ground truth.

**D. Optional lightweight face detection** (PICO, NPD, small pure-Rust inference) for recovery.
A face box alone can't recover the nose tip precisely. Compare: box + offset, landmark
re-anchoring, local patch correspondences, no detector. Keep pure-Rust / no OpenCV / no
mandatory ML runtime.

### 3. Problem 2 — cursor stability

**A. 1€ filter** vs existing EMA on identical recorded trajectories.
`alpha = 1 / (1 + tau / dt)`, `tau = 1 / (2π · cutoff)`,
`cutoff = min_cutoff + beta · |filtered_velocity|`. **Verified:** Casiez, Roussel & Vogel 2012
(https://doi.org/10.1145/2207676.2208639, https://gery.casiez.net/1euro/,
https://github.com/casiez/OneEuroFilter); tuning starts at `min_cutoff ≈ 1 Hz`, `beta = 0`, then
raise beta for fast-motion lag. Experiment: min_cutoff 0.5 / 1.0 / 1.5 Hz, beta tuned in real
units, d_cutoff 1 Hz, **timestamp dt, never an assumed constant**, compare camera-space vs
screen-space filtering.

**B. Velocity-aware gain** `cursor_velocity = gain(head_velocity) · head_velocity`; compare
fixed gain, continuous speed-dependent gain, explicit precision mode, adaptive dead-zone
hysteresis, hybrid relative/absolute. Risk: nonlinearity, discontinuities, oscillation.

**C. Multi-feature noise reduction:** `1/sqrt(N)` only for independent unbiased noise; facial
errors are correlated. Don't present theory as measurement.

**D. Dead zone / hysteresis:** no jump on escape, no trapping, stable dwell, low latency for slow
intentional motion, predictable for motor-impaired users.

**E. Alternatives:** constant-velocity Kalman, alpha-beta, double-exponential, EMA, adaptive
dead-zone controllers. The most sophisticated filter isn't necessarily the most usable.

### 4. Architecture

Three independently testable layers: **tracking engine** (geometric motion + anchor +
confidence), **motion filter** (noise vs latency, irregular timestamps), **pointer controller**
(gain, precision, dead zones, hysteresis, screen limits, dwell). Keep cursor heuristics out of LK.

### 5. Performance targets (goals, not verified)

30 FPS without sustained backlog; P95 tracking-processing < 10 ms on reference hardware; P95
stationary cursor deviation < 3 screen px in a defined stillness test; no losses in a standard
head-rotation test; minimal added latency; recovery without a wrong jump; no accessibility
regression. Measure camera-to-screen latency separately from algorithm time.

### 6. Testing

Recordings: 30–60 s stillness; slow yaw; slow pitch and roll; fast comfortable motion; repeated
target acquisition; small precision corrections; hand occlusion; lighting/exposure change;
leaving and re-entering; long use. Metrics: losses per visible-face minute; wrong accepted tracks
per minute; anchor drift vs annotation; P50/P95/P99 position error; cursor RMS and P95 wander at
rest; endpoint error, task time; onset latency, settling time; recovery time and max recovery
jump; dwell success and accidental clicks; CPU, processing latency, dropped frames. Don't hide
delay with excessive smoothing.

### 7. Competitors

eViacam GPL-3.0 (https://github.com/cmauri/eviacam); Google Project GameFace Apache-2.0
(https://github.com/google/project-gameface, archived 2025-09-05, MediaPipe face models). Also to
check: Camera Mouse, HeadMouse, Precision Gaze Mouse, OptiKey, Nose Tracker, Talon, AsTeRICS,
TrackerPro, SmartNav — separate inspectable source from marketing claims.

### 8. Remaining research (final report)

At `docs/research/HEAD_POINTER_STABILITY_RESEARCH.md`: top-5 continuity techniques; top-5
stability techniques; math; parameter ranges with evidence; CPU estimate at 640×480/30 (measured
vs estimated); failure modes; verified OSS implementations + licences; competitors; first
experiment with concrete parameters; reproducible benchmark protocol; sources with dates and
verified/inferred labels. Do not implement.

---

## B — Research brief update with Camera Mouse 2018 RE findings (summary)

**2018 facts (static RE, not runtime measurement):** single-point `cvCalcOpticalFlowPyrLK`;
window 10×10, level 3, 20 iter, eps 0.03; status checked, error pointer null; displacement
> 35 camera px → previous point restored (not necessarily "lost"); no persistent selection-frame
appearance comparison / ZNCC gate / drift correction found; float coordinates through mapping;
absolute mapping, gain 6/6; `output = 0.95·new + 0.05·previous`; cursor worker sleeps 70 ms
(≤ ~14.3 updates/s); edge clamping modifies a camera-space offset. Unresolved: automatic
recovery, Windows input quantisation, capture scheduling, full multi-monitor behaviour.

**Updated priorities:**

1. **Virtual-anchor tracking as the primary candidate:** feature cloud around the clicked point,
   FB-LK validation, spatially distributed selection, RANSAC similarity/affine, local vs
   whole-face motion, preserving the anatomical point under yaw/pitch/roll, sufficiency of 2D
   transforms, cumulative drift, safe add/replace/remove of features. **Critically evaluate
   whether a cloud can drift coherently with excellent internal consensus.**
2. **Avoid needless loss without accepting wrong tracking:** distinguish genuine loss, pose
   change, LK drift, occlusion, large legitimate motion, illumination. Compare immutable,
   adaptive, multi-pose templates, geometric confidence, explicit re-identification. Don't just
   lower ZNCC without evidence.
3. **Precision at ~48× gain:** robust multi-point displacement, temporal filters, 1€, Kalman,
   adaptive gain, smooth precision modes, dead-zone hysteresis, sub-pixel accumulation;
   camera-space filter + separate screen-space controller together?
4. **Update rates and latency:** don't copy the 70 ms sleep without reason. Compare cursor at
   camera rate vs 60/120 Hz from the latest state, interpolation, short-horizon prediction,
   latest-frame-only capture queues. Interpolation is visual smoothness, not new information.
5. **Beyond LK:** inverse-compositional LK, direct patch alignment, ECC, sparse matching,
   lightweight face/landmark detection, hybrid detect + flow, small pure-Rust ML only if
   justified. Effort, licence, dependency size, CPU, accessibility benefit. No OpenCV.
6. **HCI / accessibility:** head-pointer transfer functions, tremor resistance, dwell
   stabilisation, target acquisition at high gain, intentional vs involuntary small movement,
   per-user calibration, drift over long sessions, fatigue, jump-free recovery. Don't assume
   users can hold still or correct quickly.

**Targets (to validate):** stable 640×480 @ 30; P95 processing < 10 ms; P95 stationary wander
< 3 px; no false losses in a standard moderate rotation sequence; low perceived latency; no jumps
on re-acquisition; reliable precision and dwell; consistent on large multi-monitor desktops.

**Critical question:** can estimating the motion of a stable facial feature cloud and
reconstructing the selected anchor beat tracking the selected skin patch directly? If yes, the
simplest reliable architecture; if no, why, and alternatives.

**Required decision output:** best architecture; top three smallest experiments; expected
improvement (what improves, may regress, is only theoretical); failure analysis (coherent drift,
out-of-plane rotation); implementation roadmap of independently testable Rust components by
benefit/risk; evidence quality VERIFIED/INFERRED/UNTESTED with URLs, dates, licences; stop
conditions. Be skeptical of complexity; the goal is daily use better than 2018, not synthetic
benchmarks.
