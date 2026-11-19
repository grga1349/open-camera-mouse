---
name: rust-testing
description: Rust unit/integration/property/golden test design and quality gates across the workspace. Use when writing any cargo test target, choosing proptest/insta usage, or deciding what counts as sufficient coverage for a change. Not for defining tracking-specific fixtures/metrics (see tracking-validation).
---

## Objective

Table-driven unit tests; `proptest` for invariants (bounds, finite output, no panics on
arbitrary dimensions/strides); `insta` for compact structured golden snapshots (never raw
image buffers); integration tests that exercise the same `app-core` controller the desktop
app uses, with fake capture/pointer adapters — never move the real OS cursor in a test.

## When to use

Any time Rust code is added or changed. Define test cases and expected
failure behavior *before* writing the implementation where practical.

## When not to use

Tracker fixtures and metrics: `tracking-validation`. Timing: `benchmarking`.

## Deliverables and gates

A reproducible failing-test repro command and a meaningful assertion (not just "didn't
panic") for every bug fix; `cargo test --workspace` (or `cargo nextest run` if adopted) green
before reporting done; waits are counted or driven by an injected clock, never fixed sleeps; snapshot changes reviewed explicitly, never auto-accepted.
