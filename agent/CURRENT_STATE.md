# Current state

Last updated: 2026-11-12 (Phase 4: E0–E4 done; E4 uncommitted at time of writing)

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
E3 (gain/bias-compensated LK, `LkParams::gain_bias`) KEPT, default off. E2+E3 at zncc ≥ 0.7 has 0
silent errors at LK18's mean ok. E3 = `19a2f4a`, CI 43128269741 green.
E4 (re-detection, `LkParams::redetect`, ADR-0010 Accepted) KEPT, default off. Candidate on E2+E3:
mean ok 89.4 % (70-run corpus, vs 73.4 %), 0 wrong-lock, 0 false-track. E5/E6 not attempted (reasons
in TRACKING_IMPROVEMENTS). Proposed default: E2 + E3 + E4 candidate, E1 off → default-tracker ADR.

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

Default-tracker ADR (ADR-0011), the final latency check, then the Phase 4 close-out (task 0006
acceptance criteria).

## Open risks

- Baseline is synthetic only; no real head-motion recordings until a camera adapter exists.
- Findings 2–5 are closed on the synthetic corpus by E2+E3+E4. Remaining (unmeasured on real faces): a
  lookalike appearing within 0.25 w of the loss point; a target returning farther than that;
  a fixed template vs real head roll (`rotate_15` 320 ZNCC 0.74 vs gate 0.7); 1–1.5 px
  re-acquisition offset at 320.
- Camera crate macOS panics (PLATFORM_RISKS P1); Wayland pointer (P10/P11).
