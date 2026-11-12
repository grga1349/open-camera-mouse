# ADR-0004: Tracking-algorithm baseline candidates for Phase 3

Status: Accepted — resolved by measurement: Phase 3 kept LK (TRACKING_BASELINE), and ADR-0011
records the shipped LK configuration. B-NCC stays a frozen test comparator.
Date: 2026-11-11

## Context

`docs/research/RE_EVIDENCE.md` establishes that the two available reference implementations
use *different* algorithms: Camera Mouse 2018 uses one-point pyramidal Lucas-Kanade optical
flow (`cvCalcOpticalFlowPyrLK`, VERIFIED call site); the current `legacy/` Go app uses
fixed-template normalized cross-correlation (`gocv.MatchTemplate` + `TmCcoeffNormed`). Phase 3
needs a starting point for the pure-Rust, no-OpenCV engine. This ADR records the candidates
and the evaluation plan, not a winner.

## Options considered

- **Pure-Rust pyramidal LK**, parameterized from the 2018 app's reported defaults (10×10
  window, 3 pyramid levels, 20 iterations / epsilon 0.03) as a *starting preset*, not a
  correctness requirement. Pro: matches the algorithm family with the best reputation for
  this exact product category. Con: those parameters are reverse-engineered reports about a
  specific OpenCV 2.1 build, not independently validated as optimal, and several native
  details (interpolation, exact gradient computation, first-frame init) are unresolved.
- **Pure-Rust NCC template matcher**, ported in spirit from `legacy/internal/tracking/tracking.go`.
  Pro: known to already work acceptably in a shipped product; simpler to implement and reason
  about; no drift from the original selected point since matching is template-based, not
  incremental. Con: fixed template never adapts to lighting/pose change; `legacy/`'s own
  0.68 score threshold and re-centering behavior would need re-validation, not copying as-is.
- **Both, compared head-to-head on the same fixture corpus before picking one.** This is what
  the seed's Phase 3 plan already calls for
  (`docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md`,
  `docs/archive/rust-rewrite-seed/03-research-and-evidence.md`) and what this ADR recommends.

## Decision (proposed)

Do not pick a winner in Phase 0 or Phase 1. Phase 3 implements both as independent, pure-Rust,
no-OpenCV baselines against the same deterministic synthetic+recorded fixture corpus
(`docs/archive/rust-rewrite-seed/10-rust-skills-architecture-testing-research.md`, section
"Core test design"), and measures endpoint error, tracking-success ratio, recovery time,
drift, and cursor jitter for each before Phase 4 improvements build on whichever is kept (or
a hybrid). The 35px outlier guard and 0.68 NCC threshold from the two source systems are
*candidate starting values for their respective algorithm*, each re-derived against fixtures,
not assumed correct and not swapped between algorithms.

## Consequences

- Phase 2's crate design (ADR-0003) must not hard-code assumptions that only fit one
  algorithm family (e.g. `tracking-core`'s public API should express "frame in, position +
  status + diagnostics out," not LK-specific or NCC-specific types).
- This explicitly defers, rather than silently drops, the open question in the seed's decision
  register: "What is the old tracker algorithm and quality vs binary-researched PyrLK?" —
  answered qualitatively here (they're different algorithms), with the quality comparison left
  to Phase 3 measurement.
