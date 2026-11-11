# Current state

Last updated: 2026-11-12 (Phase 3 closed; Phase 4 E0 done, pushed, CI green)

## Branch / commit

Branch: `rewrite/rust`. Phase 3 = `4038fce` (CI 43119858155 green). Task 0005 close-out + E0 =
`f68f042`, plus `ad25a34` (task 0006 progress notes). Pushed; CI run 43121280464 green on 3 OS
+ cargo-deny.

## Milestone

Phases 0–3 DONE. Phase 4 (tracking improvements) ACTIVE:
`tasks/active/0006-phase-4-tracking-improvements.md`. E0 (perf, no behavior change) KEPT.
E1–E6 not started.

## What exists now

- Workspace per ADR-0007; mirror-at-ingest + 640×480@30 per ADR-0008.
- `tracking-core`: B-LK18 (`lk::LkTracker`), B-NCC (`ncc::NccTracker`), image/pyramid
  primitives. LK computes window-only gradients (`image::GradPatch`); `lk_step` 0.30 ms at 640.
- Tests: `tests/fixtures.rs` (correctness), `tests/baseline.rs` (ignored: `baseline_tables`,
  `lk_fingerprint`), benches in `benches/tracking.rs`.
- Docs: `docs/research/TRACKING_BASELINE.md` (Phase 3), `TRACKING_IMPROVEMENTS.md` (Phase 4).

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Phase 4 E1 (forward-backward validation). Add an ablation mode to `tests/baseline.rs` first.

## Open risks

- Baseline is synthetic only; no real head-motion recordings until a camera adapter exists.
- Known tracker failure modes (TRACKING_BASELINE findings 2–5, 7) are the E1–E4 targets.
- E4 (re-detection) changes the 2018 loss policy and needs an ADR.
- Camera crate macOS panics (PLATFORM_RISKS P1); Wayland pointer (P10/P11).
