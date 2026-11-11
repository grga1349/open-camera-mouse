---
name: benchmarking
description: Performance measurement methodology — per-frame/sequence latency, allocations, p50/p95/p99, regression tracking across hardware. Use when adding a benches/ target or comparing tracking-algorithm candidates' speed. Not for correctness/error metrics (see tracking-validation).
---

## Objective

Record environment/hardware metadata with every result; report distributions (min/median/p95),
not single numbers; treat absolute timing as a trend to watch, not a hard CI gate across
heterogeneous machines. Measure image conversion, pyramid construction, LK iterations (or NCC
search), allocations and end-to-end latency separately, across resolutions.

## When to use

Comparing ADR-0004's tracking candidates, evaluating a proposed optimization (SIMD, buffer
reuse) only *after* profiling identifies a real cost, or tracking performance trend over time.

## When not to use

Don't use benchmark numbers as a proxy for tracking correctness — that's a different, measured
claim (see `tracking-validation`).

## Deliverables and gates

A baseline report with min/median/p95, input sizes and device metadata; comparisons stated as
measured deltas with the same fixture corpus on both sides, not cross-machine absolute
numbers. Tool: `divan` (or Criterion — pick one and stay consistent).
