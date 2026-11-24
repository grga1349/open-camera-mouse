# ADR-0018: Anchored update of the appearance gate

Status: Accepted (user, 2026-11-16: default on, "Default on (Recommended)")
Date: 2026-11-16

## Context

Test drive part 2, slice 2d (task 0009, `docs/qa/TEST_DRIVE.md` #2). After ADR-0017 every
remaining loss on a local head-turn recording was `Lost(AppearanceChanged)`: the E2 gate
(ADR-0011) compares each new window with the *selection* window, and a turn slides that ZNCC
from ≈ 0.85 to below 0.70. Recovery then took 25–72 frames. In CI the same gap was
`turn_extreme` in `tracking-core/tests/appearance.rs` at 31/90 frames tracked.

Measured first (synthetic, 640×480):

- With the gate off, plain LK follows `turn_extreme` through the whole turn (EPE ≤ 4.2 px,
  tolerance 12.8 px) and returns to 0.2 px. The gate, not LK, causes the losses.
- In the turn the selection ZNCC falls by ≈ 0.01–0.03 per frame (down to 0.50), and the
  frame-to-frame ZNCC stays ≈ 1.00. An occluder edge sliding over the point drops the
  selection ZNCC by 0.08–0.15 per frame, a full occluder to ≈ −0.2 in one frame.

So rate tells the two apart. A previous-frame reference with the same 0.7 threshold lets a
slow occluder through (3–5 frames of false tracking on the new `occluder_slide` scene).

## Options considered

1. Lower the gate (0.6, 0.5): tried in 2b replays. Fewer losses, but it weakens occluder and
   lookalike rejection everywhere.
2. Hysteresis or a degraded `Held` state: the cursor would stop during every turn. That is
   the symptom, not a fix.
3. **Anchored update (Matthews et al. 2004 style): chosen.** Gate against the window accepted
   K steps earlier with the same threshold, plus a floor against the selection. The
   selection window is never replaced.
4. Virtual-anchor feature cloud (task file 2d.2): a new tracker. Not needed after 3 met the
   stop condition. Kept as an idea if real turns still lose the target.

## Decision

`LkParams::anchored: Option<AnchoredUpdate>`, `Some(AnchoredUpdate::default())` in
`recommended()`, `None` in `Default` (B-LK18). With it set, a step passes the appearance gate
when both hold:

- ZNCC(new window, accepted window `lag_frames` = **10** steps earlier) ≥ `min_ref_zncc`
  (0.7). A ring of the last 10 accepted windows. It is reset to the selection window on
  `select` and on a re-detection resume.
- ZNCC(new window, selection window) ≥ `min_selection_zncc` = **0.3** (backstop).

`Diagnostics::score` still reports the selection ZNCC; `Diagnostics::anchor_score` (added
v0.4.1) reports the lag-window ZNCC, so a loss shows which check rejected it. Cost per frame: one more 21×21 ZNCC and
one normalised window copy, no allocation after `select`.

Lag sweep 5 / 10 / 15 on the appearance corpus and the fixture corpus (320 and 640): 5 lets the
2.7× slower `occluder_creep` through (6–13 frames of false tracking). 15 costs on `rotate_15` at
320. 10 has no wrong lock or false tracking anywhere and no scene worse than the plain gate.
Floor 0.3 vs 0.4 vs 0.5: the same in synthetic. On the local turn recording, the turn went down
to selection ZNCC ≈ 0.38, and 0.5 cost one more loss. So 0.3: the lag check does the occluder
work.

Evidence (local recordings, diagnosis only, not committed): turns recording, losses 5 → 2
(−60 %), no drift by the replay's context check. Still recording: unchanged. The 2 remaining
losses are the lag check at the deepest point of a turn, not the floor.

## Consequences

- CI: `turn_extreme` pinned at 90/90 (was 31/90), EPE p95 4.15 px (the turn's own error at
  its peak, now included). New scenes `occluder_slide` (≤ 4.2 px/frame) and `occluder_creep`
  (≤ 1.6 px/frame) are pinned with 0 wrong locks and 0 false tracking. The adversarial
  fixture test runs both the recommended and the plain gate. `plain_gate_baseline_loses_the_extreme_turn`
  keeps the old number visible.
- Known limit, by construction: a change slower than the gate allows within 10 frames is
  followed until the selection floor. If the new surface happens to correlate ≥ 0.3 with the
  selection, it is followed indefinitely (unit test: a dissolve between two correlated
  synthetic textures never reached the floor; LK drifted 10 px). For a real head this means
  a very slow hand creeping over the nose. Re-measure on Phase 8 recordings.
- Re-detection (E4) is unchanged: it still searches for the selection templates.
- Departure from 2018 (ADR-0006): 2018 had no appearance gate at all.

## Amendment 2026-11-24 (v0.4.1 slice 1) — a short hold for borderline drops

Decision D4 (user): keep tracking through borderline appearance drops, but still declare a
loss when things clearly go wrong, so re-detection runs.

Evidence (macOS `OCM_PERF`, diagnosis only, not committed): every loss was
`AppearanceChanged` from this gate, in three groups. (1) Most: the lag check just under
0.7 while the selection score and the residual looked healthy. (2) Violent moves: both
scores low, high residual (real slips). (3) Rare: the selection floor on a far, smooth
turn. Rebuilt synthetically in `tests/support` `loss_groups()` (640×480).

Change: `AnchoredUpdate` gains `degraded_ref_zncc` = **0.6** and `max_degraded_frames` =
**3**. A step the anchored gate rejects with lag ZNCC ≥ 0.6 and the selection floor met is
`Held(AppearanceDegraded)` instead of `Lost`, for at most 3 consecutive steps; the next one
is lost. During the hold the tracker keeps following LK's estimate and records its window
as usual; the reported position stays the last tracked one. The floor (0.3) is unchanged, so
group 3 is still lost.

Measured first (synthetic):

- A hold, not `Tracking`: `occluder_slide` crosses the band (lag ZNCC 0.61) on its first
  covered frame; reporting it as tracking would be false tracking.
- Holding the old point (as the other holds do) fails on a quick turn: LK then has to cover
  two frames of motion and slips. Not recording the window only delays the loss.
- Budget: 3 is the largest without regressions. 5 or more let `occluder_creep` into the lag
  ring at 320 (7 frames of false tracking). A band of 0.55 or 0.65 gained nothing over 0.6.
- Result: `turn_borderline` 71 % → 94 % tracked (no loss, 4 held frames);
  `turn_borderline_long` is held 3 frames, then lost; `violent_slip` is lost at once and
  re-detected; `turn_far` is lost at the floor. Every corpus and appearance scene at 320,
  640 and 1280 is unchanged, with 0 wrong locks and 0 false tracking.

Consequences: the cursor stops for up to 3 frames (≈ 0.1 s at 30 fps) on such a dip, then
catches up through the smoother, instead of losing the point until re-detection. Dwell resets
on a hold (R-K2 treats every hold alike). The desktop status line shows "Tracking" for this
hold (it is too short to read); `OCM_PERF` counts it as `held look`.
