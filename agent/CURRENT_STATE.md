# Current state

Last updated: 2026-11-12 (Phase 4: E0–E2 pushed, CI green)

## Branch / commit

Branch: `rewrite/rust`. Phase 3 = `4038fce` (CI 43119858155 green). Task 0005 close-out + E0 =
`f68f042`, plus `ad25a34` (task 0006 progress notes). Pushed; CI run 43121280464 green on 3 OS
+ cargo-deny.

## Milestone

Phases 0–3 DONE. Phase 4 (tracking improvements) ACTIVE:
`tasks/active/0006-phase-4-tracking-improvements.md`. E0 (perf, no behavior change) KEPT.
E1 (forward-backward, `LkParams::fb_max_error`) KEPT as a component, default off; enabling is
decided after E3. E1 = `cac424f`, CI 43124247853 green. E2 (appearance gate,
`LkParams::min_ref_zncc`, candidate 0.7) KEPT as a component, default off: Σ wrong-lock and
false-track → 0, no gate-scene change; enable decision after E3. E2 = `47fe4ea`, CI 43126229350 green.
E3–E6 not started.

## What exists now

- Workspace per ADR-0007; mirror-at-ingest + 640×480@30 per ADR-0008.
- `tracking-core`: B-LK18 (`lk::LkTracker`), B-NCC (`ncc::NccTracker`), image/pyramid
  primitives. LK computes window-only gradients (`image::GradPatch`); `lk_step` 0.30 ms at 640.
- Tests: `tests/fixtures.rs` (correctness), `tests/baseline.rs` (ignored: `baseline_tables`,
  `lk_fingerprint`, `ablation_tables` + `variants()`), benches in `benches/tracking.rs`.
- Docs: `docs/research/TRACKING_BASELINE.md` (Phase 3), `TRACKING_IMPROVEMENTS.md` (Phase 4).

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Phase 4 E3 (gain/bias-compensated LK), and re-measure E1/E2 on top
(`brightness_step`, `contrast_drop`, `rotate_15` 320 margin).

## Open risks

- Baseline is synthetic only; no real head-motion recordings until a camera adapter exists.
- Known tracker failure modes (TRACKING_BASELINE findings 2–5, 7) are the E1–E4 targets.
- E4 (re-detection) changes the 2018 loss policy and needs an ADR.
- Camera crate macOS panics (PLATFORM_RISKS P1); Wayland pointer (P10/P11).
