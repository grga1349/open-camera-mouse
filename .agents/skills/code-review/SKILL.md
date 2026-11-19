---
name: code-review
description: Independent, adversarial review focused on correctness, failure modes, scope and security — evidence over assertion. Use before merging non-trivial changes, ideally from an agent that did not write the change. Not a substitute for running the actual verification commands (see rust-testing).
---

## Objective

Review for real failure scenarios (concrete inputs/state that break it), not style
preferences. Report findings ranked by severity; don't mark something fixed without having
actually re-verified it. Keep claims taken from Camera Mouse 2018 or other
sources distinct from this project's own measurements — don't let an inferred claim get
upgraded to verified casually.

## When to use

Before merging a non-trivial change, especially anything touching
`tracking-core`/`pointer-core` correctness or crossing a crate boundary (ADR-0007).

## When not to use

Don't use this in place of running `cargo test`/`cargo clippy` yourself first — review is a
second pass after those gates pass, not instead of them.

## Deliverables and gates

Severity-ordered findings with a concrete failure scenario per finding; no unverified
assertions ("this might be slow" without a measurement, "this looks fine" without having
traced the actual code path).
