---
name: tracking-validation
description: Build synthetic video fixtures with known ground truth, define error metrics and failure taxonomy for the tracker. Use when adding/changing fixtures under fixtures/ or defining what "correct" means for a tracking test. Not for the algorithm implementation itself (see computer-vision) or for perf measurement (see benchmarking).
---

## Objective

Deterministic frame-sequence generators (seeded, reproducible) with independently-derived
ground truth — never generate expected trajectories using the implementation under test.
Cover: integer and subpixel translation, brightness/contrast shift, partial occlusion,
rotation/scale, motion blur, near-edge motion, textureless regions, distractors, lost-frame
gaps. Define metrics: endpoint error (mean/median/p95/max), percent-frames-tracked,
recovery time after loss, drift over a long sequence, cursor jitter while stationary.

## When to use

Adding a new fixture, changing what a test asserts about tracker output, or defining the
corpus a new tracking candidate (ADR-0004) must pass before being compared to the others.

## When not to use

Not for benchmarking wall-clock/CPU/memory — that's `benchmarking`. Not for the tracker math
itself — that's `computer-vision`.

## Deliverables and gates

A `FrameSequence` fixture with `Frame`s, timestamps, initial target location and expected
trajectory/visibility, stored under `fixtures/synthetic/` or `fixtures/recorded/` with a
metadata file; a failing regression test is a valid deliverable when it documents a real gap.
