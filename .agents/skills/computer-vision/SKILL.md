---
name: computer-vision
description: Grayscale conversion, image pyramids, interpolation, Lucas-Kanade optical flow and NCC template matching math, numerical stability. Use for tracking-core algorithm design/review. Not for turning math into idiomatic Rust structure (see rust-engineering) or for building the test fixtures that validate it (see tracking-validation).
---

## Objective

Get the math right and documented: pyramid construction, image gradients, bilinear sampling,
the LK normal-equations solve (and its degenerate/singular cases), and NCC scoring — whichever
of ADR-0004's candidates is being implemented. Document image coordinate conventions (origin,
axis direction, pixel-center-vs-corner) explicitly; this is where silent bugs hide.

## When to use

Designing or reviewing anything in `tracking-core`'s `image/` or `optical_flow/` modules
(once they exist), or evaluating which parts of `docs/research/RE_EVIDENCE.md`'s reported
parameters are safe to use as starting presets vs. need independent re-derivation.

## When not to use

Don't use this for cursor-space mapping/smoothing — that's `pointer-core`, covered by
`rust-engineering` + the pointer-mapping sections of `docs/research/RE_EVIDENCE.md`.

## Deliverables and gates

Equations written down with explicit coordinate-space assumptions; reference checks against
known-good values (e.g. a hand-computed small-window LK step); no NaN/Inf on degenerate input
(flat region, singular Hessian) — return a typed "low confidence" result instead of panicking
or returning garbage.
