# 0003 — Phase 1: research and behavior specification

Status: READY (not started)
Owner/branch: `rewrite/rust`
Depends on: 0001 (done), 0002 (done)

## Context

Next phase per `docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md`. Phase 0 already
produced a first-pass synthesis (`docs/research/RE_EVIDENCE.md`) comparing Camera Mouse
2018's reported pipeline against `legacy/`'s actual NCC tracker. Phase 1 goes deeper: a full,
traceable behavior specification, not a synthesis of someone else's labels.

## Objective

Read `legacy/` source directly (already partially done for tracking/mouse/config — extend to
camera, preview, app runtime loop, hotkeys) and all of `../camera-mouse-reverse/research/*.md`
in full (done once already for this synthesis — re-verify claims rather than re-trusting the
synthesis). Chart end-to-end behavior and defaults for both systems. Mark confirmed vs.
inferred vs. unknown explicitly. Specify expected behavior and benchmark baselines for the
Rust rewrite, informed by but not bound to either source system.

## Non-goals

No Rust code, no Cargo workspace (that's Phase 2). No tracking algorithm decision beyond
recording candidates (ADR-0004 already does this at a proposal level).

## Files/scope

New content under `docs/research/` (e.g. a full `BEHAVIOR_SPEC.md`); no changes to `legacy/`
or `docs/archive/rust-rewrite-seed/`.

## Acceptance criteria

Per `docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md` Phase 1 gate: a traceable
behavior spec exists, with discrepancies between the two systems and explicit unanswered
questions called out — not papered over.

## Verification

Read-through review against `../camera-mouse-reverse/research/*.md` source files (addresses/
tokens/REA evidence IDs must trace back, not be restated from memory) and against actual
`legacy/` source line references.

## Next step

Unblocks Phase 2 (`docs/architecture/decisions/0003-proposed-crate-boundaries.md`,
`0004-proposed-tracking-baseline-candidates.md`) — Phase 2 should either ratify or revise
those ADRs once the Phase 1 spec exists.
