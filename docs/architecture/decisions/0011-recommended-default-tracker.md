# ADR-0011: Recommended default tracker (end of Phase 4)

Status: Accepted
Date: 2026-11-12
Resolves: ADR-0004 (tracking candidates). Task 0006 acceptance criterion "recommended default
tracker and its deviations from 2018 recorded in an ADR".

## Context

Phase 3 measured B-LK18 (pyramidal LK, 2018 preset) against B-NCC (legacy template matching)
and kept LK as the presumptive baseline (TRACKING_BASELINE). Phase 4 ran E0–E4 as ablations
on the same synthetic corpus (TRACKING_IMPROVEMENTS). Each experiment sits behind an
`LkParams` flag, and `LkParams::default()` stays bit-identical to B-LK18. App-core (Phase 6)
needs one tracker configuration to ship. The ablation suite still needs B-LK18 as a fixed
baseline.

## Options considered

1. **Ship B-LK18 unchanged.** 73.4 % mean ok on 70 runs, 654 silent wrong-lock and 277
   false-track frames. Every loss means re-selecting. Rejected on measurements.
2. **E2 + E3** (appearance gate + gain/bias). 0 silent errors, but losses are still permanent
   (73.4 % mean ok).
3. **E2 + E3 + E4** (+ re-detection, ADR-0010). 89.4 % mean ok, 0 wrong-lock, 0 false-track,
   no gate-scene regression.
4. **Option 3 + E1** (forward-backward). Measured redundant on top of E2 + E3 (TRACKING_IMPROVEMENTS
   §E3): no fewer errors, one more dropped frame, and one more LK solve per step.

Exposure:
- (a) change `LkParams::default()` to the recommendation;
- (b) add `LkParams::recommended()` and keep `Default` = B-LK18.

## Decision

**Option 3, exposed as (b).** `LkParams::recommended()` is the product tracker. App-core
constructs it with `LkTracker::new(LkParams::recommended())`. `Default` stays B-LK18, so
`lk_fingerprint` and every ablation row stay reproducible against the Phase 3 baseline.
A fixture test (`recommended_params_are_the_measured_candidate`) pins
`recommended()` to the exact configuration measured as the E4 candidate.

Configuration and its relation to 2018 (ADR-0006 reasons):

| Aspect | Value | 2018? | Reason / record |
|---|---|---|---|
| Algorithm | one-point pyramidal LK (Bouguet) | yes | ADR-0004, TRACKING_BASELINE |
| Window, levels, iterations | 21×21, levels 0..=3, 20 iter / ε 0.03 | yes (INFERRED semantics) | ADR-0009 |
| Outlier guard, edge margin | 35/640, 10/640 of frame width | yes, re-expressed | (b), ADR-0009 |
| Near-edge-moving-out → hold | kept, symmetric predicate | deviates | (a), ADR-0006 / R-T4 |
| Reference = selection frame | kept | deviates | (a), ADR-0006 / R-T3 |
| Texture test `λ_min/|W| ≥ 1.0` | on | deviates | (a), ADR-0009 |
| E2 appearance gate, ZNCC ≥ 0.7 vs selection window | on | deviates | (a): 2018 silently follows occluders/background; R-T2 forbids silent errors |
| E3 gain/bias compensation | on | deviates | (a): brightness constancy gave a silent 6.4 px offset on a gain step (finding 5) |
| E4 re-detection, τ 0.9 / δ 0.1 / K 16 / level 2 / r 0.25 w | on in `recommended()`; **product default off** (amended 2026-11-12) | deviates | (a), ADR-0010; amendment below |
| E1 forward-backward | off | — | redundant (measured) |

**Latency** (Apple M1, divan, medians over 100 samples, `lk_recommended_step` /
`lk_searching_step`): a tracked step takes 0.29–0.31 ms at 640×480 (slowest clean sample 0.39–0.85 ms).
A step while searching takes 0.58 ms (slowest 0.78 ms). Both are inside the §8 budget (p95 < 2 ms at
640). There are zero steady-state allocations. One run showed isolated multi-ms samples on every bench,
including plain `lk_step`. Re-runs came in clean, so that is machine noise, not a tracker cost.

## Amendment 2026-11-12 (user, task 0008)

Re-detection becomes a user **option, off by default** in the app: Options → Camera → "Find the
target again" (setting `loss_recovery`, shown in `docs/design/design-lab.html`).
`LkParams::recommended()` is unchanged and stays the measured, pinned candidate. `app-core` builds
the tracker from it and sets `redetect = None` when the option is off. With the option off, a
loss follows the 2018 policy: control goes back to the mouse, and the user re-selects or recenters.
With it on, `Lost(Searching)` → `Tracking` hands control back only within the R-P4 jump bound.
Reason: the thresholds are synthetic-only evidence (see below), so automatic recovery stays opt-in
until Phase 6 recordings re-measure them. User direction: improve on 2018 and take the best of
2018 and legacy, but ship well-founded improvements as options until real data backs them.

## Consequences

- **App-core / UI (Phases 5–6)** must handle `Lost(Searching)` and the `Lost → Tracking`
  re-acquisition. The 2018 policy releases control to the mouse on loss. Whether
  re-acquisition hands control back immediately or via the §2.7 handover rules is a Phase 6
  decision. `LkTracker::redetect_armed()` tells the UI when a selection can't be recovered
  automatically (a lookalike in view at selection).
- **Every threshold is synthetic-corpus evidence only.** These include ZNCC 0.7 (the margin is 0.74
  under 15° roll at 320), τ/δ, and r (bracketed only to [0.15, 0.32) w). Phase 6 recordings must
  re-measure them before release (Phase 8). Changing a value means updating `recommended()`, the
  pinned candidate, and this ADR together.
- **Known gaps carried forward:** a lookalike that appears within r of the loss point; a target
  that returns farther than r away; a 1.0–1.4 px re-acquisition offset at 320 wide (candidate fix:
  LK refine against the selection pyramid); a fixed template versus pose change.
- ADR-0004 is resolved: LK with the Phase 4 extensions is the engine. B-NCC stays a frozen
  comparator for the test suite only.
