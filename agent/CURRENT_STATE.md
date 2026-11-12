# Current state

Last updated: 2026-11-12 (Phase 4 DONE; Phase 5 next)

## Branch / commit

Branch: `rewrite/rust`. Phase 3 = `4038fce`. Phase 4: E0 `f68f042`, E1 `cac424f`, E2 `47fe4ea`, E3
`19a2f4a`, E4 `0074611`, then a close-out commit (ADR-0011, `LkParams::recommended()`). See `git log`
and the CI check in HANDOFF.

## Milestone

Phases 0–4 DONE. Phase 5 (design system) is NEXT per `docs/ROADMAP.md`. Its task file is not
written yet. The coordinator writes it from the roadmap when the phase starts.

## What exists now

- Workspace per ADR-0007; mirror-at-ingest + 640×480@30 per ADR-0008.
- `tracking-core`: B-LK18 (`lk::LkTracker`, `LkParams::default()`), the recommended product tracker
  `LkParams::recommended()` (ADR-0011: E2 appearance gate + E3 gain/bias + E4 re-detection via
  `crate::redetect`, ADR-0010), and B-NCC (`ncc::NccTracker`, frozen comparator).
- Statuses: `Tracking`, `Held(MotionOutlier | NearEdgeMovingOut)`, `Lost(… | AppearanceChanged |
  Searching | NoSelection)`.
- Tests: `tests/fixtures.rs`, `tests/baseline.rs` (ignored: `baseline_tables`, `lk_fingerprint`,
  `ablation_tables`), benches in `benches/tracking.rs`. The corpus is 70 scene × resolution runs (incl. 3
  distractor scenes).
- Docs: `TRACKING_BASELINE.md` (Phase 3), `TRACKING_IMPROVEMENTS.md` (Phase 4).

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Start Phase 5 (design system): write its task file from ROADMAP Phase 5 deliverables/gate. Inputs
for the UI states include `Lost(Searching)` and `redetect_armed()` (ADR-0011 consequences).

## Open risks

- Synthetic-only evidence. Every tracker threshold (ZNCC 0.7, τ/δ, r 0.25 w) must be re-measured on
  Phase 6 recordings.
- Known tracker gaps: a lookalike within r of the loss point; a target returning farther than r away;
  1.0–1.4 px re-acquisition offset at 320; a fixed template vs head roll (`rotate_15` 320 ZNCC 0.74).
- Control handover on re-acquisition (`Lost(Searching)` → `Tracking`) is undecided (Phase 6).
- Camera crate macOS panics (PLATFORM_RISKS P1); Wayland pointer (P10/P11).
