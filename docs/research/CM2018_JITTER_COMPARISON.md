# Camera Mouse 2018 vs Rust rewrite — jitter, jumps and precision in the pointer path

Phase 6d part 3, task 0011 queue item 0 (user, 2026-11-16). Two questions:

- **(a)** Does the rewrite do something differently from 2018 that could cause jitter (#5/#11),
  jumps (#12) or imprecision (#13)?
- **(b)** Does 2018 have its own jitter solution?

Research only, no production change. The tracker core is already ruled out:
`OPENCV_LK_PARITY.md` shows our LK matches OpenCV in jitter and drift. This document is about
the path from the tracked point to the cursor.

**Sources.** 2018 behaviour comes from `BEHAVIOR_SPEC.md` §2 (re-verified CIL) and
`../camera-mouse-reverse/docs/research/CAMERA_MOUSE_2018_TRACKING_STABILITY_RE.md` (static
analysis of the shipped .NET build: tokens, RVAs, REA evidence IDs are there, and no decompiled
code is reproduced here). The rewrite's behaviour comes from `crates/pointer-core` and
`Controller::drive` (`crates/app-core/src/controller.rs`).

**Labels.** VERIFIED means read from 2018 CIL evidence or from our source. MEASURED means
replayed on recordings S1 (sitting still) and S2 (head turns), 2026-11-15. These recordings were
made on the user's desktop: a 1440×900 laptop plus a 2560×1440 external display, a 4000×1440
bounding box, with the laptop as Main monitor. INFERRED means reasoned from the above.

## 1. Method (MEASURED rows)

The local replay tool (`~/ocm-recordings/_tools/replay-tool`, test `replay_2018`) does the
following. It is not kept in the repo because it reads recordings.

1. It runs our controller on the recording with today's defaults to get the **tracker's marker
   path** (camera px).
2. It replays that marker through an **emulation of the 2018 cursor worker**. Every 70 ms the
   worker takes the newest point, maps it with gain 6 × bounding-desktop / frame size around
   the desktop centre, and blends `c = 0.95·m + 0.05·c_prev`. It clamps to the desktop, and
   while the cursor is pinned the offset drifts 1 camera px per tick (BEHAVIOR_SPEC §2.5–2.6).
   There is no 1€ filter, no steady zone and no time term.
3. It compares that output with our real pipeline (1€ → map → dt-aware smoothing 0.20 →
   limits/drift → steady zone 0.5 camera px → `approach_edge`).

Metrics are in **screen px**, with time-based windows so that a 14 Hz path and a 30 Hz path
compare fairly:

- moves/s
- direction reversals/s (axis reversals within 250 ms)
- jitter rms (distance from a ±0.5 s moving mean)
- step p95/max (within continuous control)
- lag (the shift that best matches the cursor's 100 ms displacements to the raw marker's)
- residual (bias-free rms distance between cursor and raw marker at that lag, in camera px)

S1 uses the still window only (selection + 5.5 s … 23.5 s). S2 lag is not reported: the
turns pin the cursor at edges often, and offset drift changes the bias mid-run, so this method
gives no stable S2 lag. Use `replay_metrics` for S2 lag instead.

Caveat: the emulation feeds 2018's rule with **our** tracker. That is representative because
LK parity is shown in `OPENCV_LK_PARITY.md`. 2018's real capture resolution is unknown
(Q-2018-6). The mapping divides by frame size, so sensitivity per degree of head turn does not
depend on resolution. Tracker noise as a fraction of the field of view would be larger at
320×240.

## 2. Results

### S1 — sitting still (18 s)

| Pointer path | moves/s | reversals/s | jitter rms px | step p95 px | span px | lag ms | residual cam px |
|---|---|---|---|---|---|---|---|
| **2018** (gain 6 × bounding desktop, 70 ms tick) | 14.3 | **6.8** | 6.9 | 8.8 | 116 | 40 | 0.45 |
| 2018 rule ticked at 30 Hz | 29.8 | 16.4 | 6.8 | 4.5 | 117 | 10 | 0.48 |
| 2018 rule at our scale (gain 22 × Main) | 14.3 | 6.0 | 10.6 | 12.0 | 153 | 20 | 0.28 |
| **ours, defaults (app: zone 0.5)** | 10.2 | **0.1** | 4.0 | 3.5 | 87 | **110** | **0.45** |
| ours, defaults, zone 0 | 30.0 | 3.9 | 7.5 | 4.4 | 135 | 70 | 0.12 |
| ours, gain 6 (Main), zone 0.5 | 10.2 | 0.1 | 1.1 | 1.0 | 24 | 110 | 0.45 |

Scale (screen px per camera px, x×y): 2018 **37.5×18.0**, ours **49.5×41.2** on this desktop.

### S2 — head turns (largest steps)

| Pointer path | step p95 px | step max px | where |
|---|---|---|---|
| 2018 (70 ms tick) | 93 | 293 | 34.28 s |
| 2018 rule at our scale | 120 | 387 | 34.28 s |
| ours, defaults (zone 0.5) | 67 | 184 | 34.22 s |
| ours, defaults, zone 0 | 67 | 184 | 34.22 s |

All paths make their largest step at the same moment, **34.2 s**. That moment is the fastest
head turn: the tracker marker steps up to 4.45 camera px per frame there (`replay_metrics`).
2018's coarser 70 ms tick makes that step **larger** than ours.

## 3. (a) What we do differently, and whether it matters

| # | Stage | 2018 (VERIFIED) | Rewrite (VERIFIED) | Effect on jitter / jumps / precision |
|---|---|---|---|---|
| D1 | **Sensitivity** | gain 6, scale = **bounding desktop** / frame | gain **22**, scale = **Main monitor** / frame (ADR-0017) | MEASURED: on the user's desktop ours is 1.3× (x) and 2.3× (y) 2018. On a single 1920×1080 monitor it is **3.7×** (18×13.5 vs 66×49.5 px per camera px). Jitter and imprecision in screen px grow linearly with this. Our camera-px metrics do not change with gain, but what the user sees does. This is the user's own choice (ADR-0017 amendments), not a defect. |
| D2 | Cursor update rate | separate worker, **70 ms sleep** (≤ 14 Hz), samples the newest point | every camera frame (30 Hz) | MEASURED: the same rule at 30 Hz has the same jitter rms (6.8 vs 6.9 px), but 2.4× the reversals/s with half the step size. The 14 Hz tick is **not** a jitter fix: it trades smoothness for 30–40 ms of extra lag and bigger steps. INFERRED: 30 fps tracking with 14 Hz sampling also repeats or skips points unevenly (beats). We don't have that. |
| D3 | Smoothing | one-pole blend per tick, **0.95 on the new point** (τ ≈ 23 ms): almost none | 1€ filter (camera space) + dt-aware EMA **0.20** per 70 ms | MEASURED: removes most shake (reversals/s 6.8 → 3.9 with zone 0), at a cost of about 30 ms more lag than the 2018 path. |
| D4 | **Dead zone** | **none** | steady zone **0.5 camera px**, rubber band (ADR-0015/-0017) | MEASURED: shake 3.9 → **0.1** reversals/s. The cost: lag 70 → **110 ms**, and the cursor sits up to 0.5 camera px from where the head points (residual 0.12 → **0.45** camera px; 25 screen px at gain 22 on the laptop, 44 px with a 2560 px Main). **The main #13 (imprecise) candidate.** |
| D5 | Offset drift at limits | 1 camera px per 70 ms tick while pinned | `DRIFT_IMAGE_PX_PER_S` = 1/70 ms, time-based | Same rate. Both shift where a head position lands after the cursor was held at an edge. 2018 has this too, so it is not a difference, but INFERRED it can feel like imprecision ("centre moved") after edge use. |
| D6 | Edge arrival | hard clamp | `approach_edge` (ADR-0017 amendment, #12) | Fixed in part 2; no snap. |
| D7 | Camera takeover | after 4 s of mouse inactivity (or forced start): head → desktop centre, **cursor jumps to centre** (`AutoCenterScheme` 0) | same (R-A2, ADR-0006) | Same as 2018, and a deliberate jump. INFERRED #12 candidate: a false move-stop (OS cursor > 5 px from the commanded position) would hand control to the mouse, and 4 s later the cursor would jump to the centre. The S1 recording has one Camera → Handover at 22.35 s, during the "still" phase. It may be the user reaching for the mouse; the recording can't tell. |
| D8 | Outlier guard | step > 35 px → hold the old point, no loss | motion bound 35/640 frame width (ADR-0009) + appearance gate | Equivalent bound. Not a jump source. |
| D9 | Output | absolute, normalised to 0..65535, primary-screen size only | float CGEvent / `SetCursorPos` / XTest per OS | No quantisation jitter in either. |
| D10 | Timing | no timestamps; per-tick blend | capture timestamps, dt-aware | Ours is frame-rate-independent; 2018 isn't. |

**Answer (a).** Nothing in the pointer path makes us *more* jittery than 2018. With the same
input, 2018's rule shakes about 70× more often at rest. The differences that matter are
deliberate trade-offs:

- **D1, sensitivity:** 1.3–3.7× 2018, so every camera-px error shows up 1.3–3.7× larger on
  screen.
- **D4, the steady zone:** it buys zero shake at the cost of +40 ms lag and about 0.45 camera px
  of hold error. This is the most likely cause of "imprecise" (#13).

For #12, the largest S2 step is a real fast head turn that 2018 would also show, and larger. A
takeover-to-centre after a false move-stop (D7) is the remaining plausible jump that isn't head
motion.

## 4. (b) Does 2018 have a jitter solution?

**No, not in the inspected path** (VERIFIED absence, bounded to the decoded `Process`,
`MainForm.b`/`T` and `m.thread_task` CIL; the input library and OS were not inspected). There
is no dead zone, no velocity-adaptive filter, no prediction and no time term. The 0.95 damping
keeps 95% of the *new* point, so it smooths almost nothing. The 35 px guard is an outlier hold,
not a jitter filter.

Emulated on our tracker (MEASURED), 2018 shakes **6.8 reversals/s with 6.9 px rms** at rest on
the user's desktop. Its only mitigations are structural:

- the low default gain (6);
- coarser 14 Hz cursor updates. These make the shake slower, not smaller.

There is nothing to adopt from 2018 for jitter. Our 1€ filter + zone is already well beyond it.

## 5. Proposed slices (ADR-0020 rules: measure, CI test, device check, user OK for defaults)

1. **#13 imprecise → reduce the zone's hold error (D4).** Candidates, measured with this
   replay (residual, lag, reversals/s) and `replay_metrics`:
   - a shake-only rule: hold only against direction *reversals*, and follow a sustained
     one-direction move with no radius (ADR-0020 §5);
   - a smaller zone (0.25 camera px) plus a lower 1€ `min_cutoff`;
   - zone release on settle.

   Target: reversals/s ≤ 0.2 at rest, with residual and lag close to zone 0
   (≤ 0.15 camera px, ≤ 80 ms).
2. **#12 jumps → instrument takeover (D7).** Add a jump-cause tag for "takeover recentre", and
   a CI test that a cursor echo within 5 px never triggers move-stop. Ask the user whether the
   jumps follow touching the mouse or trackpad.
3. **D1 is information for the user, not a fix.** Default gain 22 is 3.7× 2018 on a single
   1080p monitor. If precision still feels poor after slice 1, a lower default gain is the
   cheapest lever (the user's call).
