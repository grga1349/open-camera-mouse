# ADR-0009: Motion bounds as a fraction of frame width; LK preset semantics

Status: Accepted
Date: 2026-11-12
Resolves: BEHAVIOR_SPEC Q-R1. Task 0005 (Phase 3).

## Context

2018's 35 px outlier guard and 10 px edge margin are absolute image pixels at a capture
resolution nobody knows (Q-2018-6). R-T5 says bounds SHOULD be relative or re-derived per
resolution. Implementing B-LK18 also forced three readings of the OpenCV 2.1 C call that the
spec leaves implicit: what `win = 10×10` means, what `level = 3` means, and when a point
"fails".

## Options considered

**Units (Q-R1).**
- *Pixels at the tracker resolution.* This is 2018-literal, but the meaning changes 4× between
  320 and 1280 wide.
- *Pixels at a canonical 640 width, scaled to the actual width.* Equivalent to a fraction
  of width, but carries a fake "reference resolution" through the API.
- *Fraction of frame width.* One number per bound and resolution-independent. With ADR-0008
  capping tracker input at ≤ 640 wide, the preset lands exactly on 2018's pixel value at 640.

**LK window.** OpenCV 2.1's C `cvCalcOpticalFlowPyrLK` treats `winSize` as a *half* size
(patch `2·win + 1`). Its later C++ API uses the full size. 2018 called the C API.

## Decision

1. Motion bounds are **fractions of frame width**: `max_motion_frac = 35/640` (outlier
   guard) and `edge_margin_frac = 10/640` (loss-policy edge margin). At 640 wide these are
   exactly 2018's 35 px / 10 px. At 320 they are 17.5 / 5 px. Both are preset values
   that Phase 4 can re-tune with measurements, not facts. (Reason (b), ADR-0006: the 2018
   resolution is unknown.)
2. `half_window = 10` → 21×21 window (INFERRED from the C API semantics above, reason (b)).
   `max_level = 3` → four levels (0..=3), OpenCV's convention. `max_iter = 20`,
   `epsilon = 0.03` on the update step length.
3. Failure test: `λ_min(G)/|W| ≥ min_eigen` with `min_eigen = 1.0` intensity²/px², plus
   `det G > ε`. A degenerate window at a coarse level skips that level (OpenCV behaviour).
   At level 0 it is `Lost(LowConfidence)`. This is **not** a 2018 value: 2018 rejected only a
   numerically singular `G`, so it would "track" sensor noise on a flat wall. Reason (a)
   defect. The flat-region fixture pins it (`textureless_target_is_never_reported_as_tracking`).
4. The B-NCC legacy preset stays in absolute pixels (template 45, search ±90). It is a
   comparator reproducing what legacy shipped, not a tuned design.

## Consequences

- `LkParams` exposes fractions. A UI that wants "pixels" converts them using the current
  frame width.
- The 35 px guard has a known failure mode: a fast head motion > 35/640 of the frame per
  frame is held, and the tracker can stay stuck on background. The `fast_40px` fixture
  measures it (see docs/research/TRACKING_BASELINE.md). Phase 4's adaptive search / loss
  recovery owns the fix.
