# ADR-0010: LK loss recovery by re-detection against the selection template

Status: Accepted (mechanism and parameters). Whether it's on by default is decided in the
default-tracker ADR. Phase 4 E4, task 0006. Evidence: `docs/research/TRACKING_IMPROVEMENTS.md` §E4.
Date: 2026-11-12

## Context

The 2018 loss policy (ADR-0006, BEHAVIOR_SPEC §2.4): near an edge and moving out → hold;
otherwise → invalidate the track point, release control to the mouse, and the user re-selects.
Phase 3 measured what this costs (TRACKING_BASELINE findings 2 and 4):

- `fast_40px`: one head motion faster than the 35/640 guard is held. The point stays on
  background and the target is never re-acquired (37 % ok).
- `edge_*`: the point is held at the edge, and when the target comes back it is on
  background (≈ 50 % ok, followed by a wrong lock or a `Lost`).

With E2 (appearance gate) these become explicit `Lost`s instead of silent wrong locks. They are
still unrecoverable without the user. A head tracker that loses the face for a moment, then
needs the user to re-select it through some other input, is a usability defect for the
product's users, who may not be able to use a mouse. Legacy NCC re-finds the target within 2–8
frames because it keeps its template (finding 4). But it also locks onto repeated texture
(finding 7), and that is worse than losing the track.

## Options considered

1. **Keep 2018 (invalidate → re-select).** No new risk, keeps the measured defect.
2. **Local re-search around the last position** (grow a window over time). Cheap, and a
   lookalike far away can't capture it. But `fast_40px` needs a > 3× guard radius, and the
   radius schedule is a new tuning surface.
3. **Full-frame coarse-to-fine ZNCC against the selection template**, resuming only on a
   match that is confident (ZNCC ≥ τ) **and** unambiguous (best − second-best distinct peak ≥ δ).
   Also a selection-time self-test: if the selection frame already contains a near-duplicate
   of the template, re-detection is disabled for that selection.
4. **Feature detector + descriptor re-detection** (ORB-like). More invariance, much more
   code and tuning. Not justified before real recordings exist (Phase 6).

## Decision

Option 3 plus option 2's spatial prior, behind `LkParams::redetect: Option<RedetectParams>` (default `None` = B-LK18,
bit-identical). Deviation from 2018 under ADR-0006 reason **(a) defect**: the 2018 intent,
"stop moving the cursor when the target is gone", is kept, because every frame without a
confident match is reported `Lost`. Only the "user must re-select" consequence is removed.

- The reference is the selection frame (R-T3). It is sampled the same way as E2's window at
  level 0, plus one downscaled window per pyramid level down to the search level.
- On a step that the 2018 policy turns into `Lost`, the reference is **kept**. The same frame
  is searched immediately. Later frames report `Lost(Searching)` until a match is accepted, and
  then `Tracking` resumes from the match.
- `Held` statuses (motion outlier, near-edge) are unchanged. A held point that ends up on
  background is turned into a `Lost` by the E2 gate, which is why E4 is measured on top of
  E2 + E3.
- Search: exhaustive integer ZNCC at a coarse level, the top-K distinct local maxima refined
  level by level (±2 px), a parabolic sub-pixel fit at level 0. Peaks closer than the
  half-window are the same peak.

- Spatial prior: while searching, only positions within `max_radius_frac · width` of the
  last known point are scored or accepted. The selection-time self-test still searches the
  whole frame.

Parameters, from the measured sweep (70 runs, TRACKING_IMPROVEMENTS §E4):
- **τ = 0.9.** 0.8–0.9 makes no ok / wrong-lock difference on the corpus. With the prior, the
  only changed row is `edge_top` 320: one frame later, and 1.4 px more precise. Background-only
  best matches reach 0.885–0.889, so 0.9 rejects them on score alone instead of relying on δ.
- **δ = 0.1.** 0.05 is the same. 0.2 refuses most true recoveries.
- **K = 16, search level 2.** K 8–32 and level 1 are identical. Level 3 misses targets.
- **r = 0.25 w.** Without the prior, `distractor_late_twin` locks onto the twin: 20 wrong-lock +
  19 false-track frames per resolution. Every r in 0.15–0.3 gives 0 / 0 and still recovers
  `fast_40px` (37 → 100 %), `edge_*` (≈ 50 → 93–99 %) and `occlusion_full` (60 → 100 %). The
  corpus only brackets r to [0.15, 0.32). 0.25 is the middle.

The candidate (E2 zncc ≥ 0.7 + E3 + this) has mean ok 89.4 %, vs 73.4 % for E2+E3 and for LK18. It
has 0 wrong-lock and 0 false-track frames, and no gate-scene regression. A searching step costs 0.59 ms at
640 (0.92 ms full-frame), with zero steady-state allocations.

## Consequences

- `LossReason::Searching` is added. App-core (Phase 5) must decide whether a re-acquired
  `Tracking` hands control back automatically, or waits for the 2018 handover rules (§2.7).
  That is not decided here.
- Re-detection inherits E2's fixed-template risk: real head roll and out-of-plane rotation
  lower the ZNCC of the true target. That makes re-acquisition fail, not lock wrong, and needs
  Phase 6 recordings.
- Remaining structural gap: a lookalike that is not visible at selection, but appears
  **within r of the loss point** while the target is lost, is accepted. It is the only peak. A
  target that comes back farther than r from where it was lost is not re-found. Both are
  the price of the prior, and Phase 6 recordings should measure them.
- At 320 wide, the recovered point can carry a 1.0–1.5 px offset (template includes some
  background). Follow-up candidate: refine the match with an LK solve against the selection
  pyramid.
- The search runs every frame while lost, so it must fit the §8 budget (step p95 < 2 ms at
  640) on its own.
