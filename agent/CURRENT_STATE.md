# Current state

Last updated: 2026-11-12 (Phase 3 pushed, CI pending)

## Branch / commit

Branch: `rewrite/rust`. Phase 2 = `ff0d1ab`. Task 0004 close-out + all Phase 3 work are in the
commit after it (see `git log`), pushed; CI result not yet checked at handoff.

## Milestone

Phases 0–2 DONE (Phase 2: CI run 43116199528 green on all 3 OS + cargo-deny). Phase 3
(tracking baseline) ACTIVE — `tasks/active/0005-phase-3-tracking-baseline.md`; everything
done locally except the CI run.

## What exists now

- Workspace per ADR-0007; mirror-at-ingest + 640×480@30 per ADR-0008.
- `tracking-core`: B-LK18 (`lk::LkTracker`) and B-NCC (`ncc::NccTracker`), image/pyramid
  primitives, downsample. Fixture generator + metrics in `tests/support`.
- Baseline results + findings: `docs/research/TRACKING_BASELINE.md`. ADR-0009 (Q-R1).
- `pointer-core` (absolute mapping), `app-core` (states/commands/ports), `platform`
  (placeholders), `apps/desktop` (empty egui window).

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Follow `agent/HANDOFF.md`: verify CI on the Phase 3 push, close 0005, then start Phase 4.

## Open risks

- Baseline is synthetic only; no real head-motion recordings until a camera adapter exists.
- LK step 4.1 ms at 640 (pyramid-bound) vs PROPOSED 2 ms; NCC 33 ms (naive search).
- Known tracker failure modes (TRACKING_BASELINE findings 2–5, 7) are Phase 4 inputs.
- Camera crate macOS panics (PLATFORM_RISKS P1); Wayland pointer (P10/P11).
