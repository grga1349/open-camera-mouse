# 0003 — Phase 1: research and behavior specification

Status: REVIEW — deliverable written 2026-11-11, awaiting independent read-through review
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

## Outcome (2026-11-11)

- `docs/research/BEHAVIOR_SPEC.md` — full behavior spec: 2018 (§2), legacy (§3), discrepancy
  matrix (§4), corrections to prior research (§6), Rust expected behavior R-* (§7), benchmark
  baselines as procedures (§8), open questions Q-2018-*/Q-R* (§9).
- `docs/research/tools/dump_il.py` — stdlib script that reprints the CIL anchors behind any
  2018 citation from `../camera-mouse-reverse/research/rea-evidence.json`.
- `docs/research/RE_EVIDENCE.md` — smoothing-polarity error corrected; superseded-in-part note.

What was actually verified: every 2018 constant/branch cited was decoded from the REA export
(EV-FT/EV-MF/EV-CFG/EV-M/EV-CALL, see spec §1), not restated from the prose research; every
legacy claim was read from source with line refs. Spot-check reproduction via `dump_il.py`
ran successfully (outlier guard `0x0600010a`, `DefaultConfig`, inactivity delay).

Not done: no new REA runs against the binary (dwell `MouseClicker`, `CenterDetect`, native
LK internals stay UNRESOLVED, §9); no runtime measurement of either system.

To close (REVIEW → DONE): an independent reviewer re-runs a sample of `dump_il.py` citations
and legacy line refs against §2–§3 and confirms §6 corrections.
