# 0006 — Phase 4: tracking improvements

Status: DONE (2026-11-12)
Owner/branch: `rewrite/rust`
Depends on: 0005 (done)

## Context

Phase 4 per `docs/ROADMAP.md`. Inputs: `docs/research/TRACKING_BASELINE.md` (§Findings,
§Performance), ADR-0009 (LK preset semantics, Q-R1), ADR-0006 (favor 2018; deviations cite
defect / unknown / platform / prior user decision), ADR-0004 (candidates), BEHAVIOR_SPEC §7
R-T1..R-T6 and §8. Skills: `computer-vision`, `tracking-validation`, `benchmarking`,
`rust-testing`.

B-LK18 stays the presumptive baseline. Every experiment is an **ablation against B-LK18 on
the same corpus**. Each one is kept or rejected on measured deltas. A rejected experiment
gets its numbers recorded, not deleted silently.

## Objective (one experiment per item, in this order)

- **E0 — Performance, no behavior change.** Pyramid build is ~98 % of the 4.1 ms LK step at
  640. Compute Scharr gradients only where LK reads them (the reference pyramid, around the
  window), and use an interior fast path without per-pixel reflect-101. Gate: baseline table
  rows **byte-identical** to Phase 3, `lk_step` at 640 median < 2 ms (PROPOSED §8 target),
  zero steady-state allocations.
- **E1 — Forward-backward validation.** Track b→a back and reject when `|a − a'|` exceeds a
  threshold. Targets finding 3 (occlusion false-track) without hurting finding 1 precision.
- **E2 — Photometric residual / confidence gating.** Gate `Tracking` on a normalised residual
  (ZNCC of the final window, or MAE/σ). Targets finding 3 (LK follows occluder). Also targets
  finding 7, keeping LK's refusal on repeated texture.
- **E3 — Gain/bias-compensated LK.** Estimate per-window gain and bias, or use a mean-normalised
  residual. Targets finding 5 (6 px shift on a 0.7 gain step; contrast_drop p95 1.6 px).
- **E4 — Loss recovery / re-detection.** Keep a reference template from the selection frame.
  On `Held`/`Lost`, search for it again (coarse-to-fine ZNCC, not the 33 ms naive search), and
  resume LK only when the match is confident *and* unambiguous (peak-to-second-peak ratio).
  Targets findings 2 and 4 (`fast_40px`, `edge_*`). It must not inherit NCC's repeated-texture
  wrong lock (finding 7). This one changes the 2018 loss policy (invalidate → re-select),
  so it needs an ADR citing reason (a) defect.
- **E5 — Adaptive search / guard.** Only if E4 doesn't already fix `fast_40px`. Re-tune or
  replace the 35/640 guard (ADR-0009 allows re-tuning on measurements).
- **E6 — Multi-point consensus.** Optional. Only if it keeps the user-selected point's
  semantics (R-T3) and E1–E4 leave a measured gap.
- Mapping/smoothing refinements (`pointer-core`) are out of scope here unless an experiment
  above needs them. Note them as a follow-up instead.

## Method

- A combined tracker (working name `LkTracker` + options, or `LkPlus`) with each experiment
  behind a param flag. B-LK18 = all flags off = `LkParams::default()` stays bit-identical.
- Extend `tests/baseline.rs` with an ablation mode: B-LK18, each experiment alone, and the
  combined candidate, on the same corpus. Same metrics. Add metric columns only if an
  experiment needs them (e.g. a held-but-recoverable count).
- Add fixtures only for gaps the experiments expose (e.g. a distractor of similar texture for
  E4). The generator stays independent of the tracker.
- A failing fixture test that documents a real remaining gap is a valid deliverable.

## Non-goals

Real recorded sequences (Phase 6 recorder). app-core/UI wiring. SIMD intrinsics or new
crates without an ADR. No tracking-cli. Retuning B-NCC (it stays a frozen comparator).

## Files/scope

`crates/tracking-core/` (src, tests, benches), `docs/research/TRACKING_BASELINE.md` (or a
new `TRACKING_IMPROVEMENTS.md` next to it), new ADR(s) for E4 and any default change,
`BEHAVIOR_SPEC` §9 if an open question is resolved.

## Acceptance criteria (ROADMAP Phase 4 gate)

- Each experiment E0–E4 (E5/E6 if attempted) has a measured result on the full corpus vs
  B-LK18, and a keep/reject decision with its numbers.
- Kept experiments produce no regression > 0.1 px EPE p95 on the Phase 3 "LK good" scenes
  (translate_*, head_sway, stall, scale_10, rotate_15, sensor_noise, static_60s), and no new
  wrong-lock frames on any scene.
- The recommended default tracker and its deviations from 2018 are recorded in an ADR.
- Latency stays within the §8 budget on the bench machine, measured, not assumed.
- `tracking-core` still passes `check-core-deps.sh`.

## Verification

`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`;
`cargo test --workspace --locked`; `scripts/check-core-deps.sh`;
`cargo test -p tracking-core --release --test baseline -- --ignored --nocapture`;
`cargo bench -p tracking-core`; CI green on 3 OS.

## Progress

- 2026-11-12 **E0 KEPT** (`f68f042`, CI 43121280464 green). Patch gradients (`image::GradPatch`, `scharr_at`),
  pyramid without gradient maps, `pyr_down_into` interior fast paths. `lk_step` at 640
  4.13 → 0.30 ms. `lk_fingerprint` (new ignored test in `tests/baseline.rs`) identical on all
  61 runs. fmt/clippy/`cargo test --workspace --locked` (46 passed, 0 failed, 2 ignored)/
  check-core-deps ok. Results: `docs/research/TRACKING_IMPROVEMENTS.md`.
- 2026-11-12 **E1 KEPT as a component, default off (provisional).** `LkParams::fb_max_error`,
  `LossReason::Inconsistent`, `Diagnostics::fb_error`. New `ablation_tables` (ignored test,
  `variants()` list, gate summary) and `lk_fb_step` bench. Σ wrong-lock 474 → 62 at fb ≤ 0.5,
  with no good-scene change. But `contrast_drop` is dropped at 0.25–1 px, and occlusion
  false-track remains. Enable/threshold decision is deferred until it's measured on top of E3.
  `lk_fingerprint` identical. fmt/clippy/tests (48 passed)/check-core-deps ok.
- 2026-11-12 **E2 KEPT as a component, default off (provisional).** `LkParams::min_ref_zncc`
  (ZNCC of the level-0 window against the selection-frame window), `LossReason::AppearanceChanged`,
  `Diagnostics::score`. `ablation_tables` got a "worst good-scene ok drop" column, because Δp95 can't
  see a lost gate scene. At zncc ≥ 0.7: Σ wrong-lock 474 → 0, false-track 103 → 0 (occlusion
  fixed), no gate-scene change. Costs: `brightness_step` 640/1280 98 % → 50 %. That was a silent 6.4 px
  offset after a guard hold, i.e. finding 5. Also a thin margin on `rotate_15` at 320 (min ZNCC 0.74;
  ≥ 0.8 drops it to 18 %). E2 dominates E1. Enable decision after E3. `lk_fingerprint` identical.
  fmt/clippy/`cargo test --workspace --locked` (50 passed)/check-core-deps ok.
- 2026-11-12 **E3 KEPT, default off until the default-tracker ADR.** `LkParams::gain_bias`: per-level
  mean/σ-matched map `α·J + β`, fixed while the level iterates. The per-iteration re-estimate was
  measured and rejected (`rotate_15` 1280 Δp95 +0.09998, no margin). Lighting scenes p95 1.6–6.4 →
  ≤ 0.08 px. E3+E2 zncc ≥ 0.7: mean ok 75.4 % (= LK18), Σ wrong-lock 0, false-track 0,
  `brightness_step` back to 100 %. Worst gate Δp95 +0.070 (`rotate_15` 1280), no ok drop. E1 is
  redundant on top of E3+E2. Proposed default: E2 + E3, E1 off (ADR after E4). `lk_fingerprint`
  identical. fmt/clippy/`cargo test --workspace --locked` (52 passed)/check-core-deps ok.
  Bench `lk_gain_bias_step` +6–12 µs.
- 2026-11-12 **E4 KEPT, default off until the default-tracker ADR.** ADR-0010 (Accepted;
  deviation from the 2018 loss policy, reason (a)). `LkParams::redetect: Option<RedetectParams>`,
  new `crate::redetect` (coarse-to-fine ZNCC vs per-level selection templates, τ/δ acceptance,
  selection-time self-test, spatial prior `max_radius_frac`), `LossReason::Searching`,
  `LkTracker::redetect_armed()`. New fixtures `distractor_twin`, `distractor_late_twin`,
  `distractor_similar` (corpus now 70 runs; the existing 61 render unchanged).
  Candidate τ 0.9 / δ 0.1 / K 16 / L2 / r 0.25 w on E2+E3: mean ok 73.4 → 89.4 %, Σ wrong-lock 0,
  false-track 0. `fast_40px` 37 → 100 %, `edge_*` ≈ 50 → 93–99 %, `occlusion_full` 60 → 100 %. No
  gate-scene change vs E3+E2. Without the prior, `distractor_late_twin` locks wrong (60 / 57 frames),
  which is why the prior is in. Searching step 0.59 ms at 640 (0.92 full-frame), zero allocs.
  `lk_fingerprint` identical on all 61 shared runs vs `8789d83`. fmt/clippy/
  `cargo test --workspace --locked` (62 passed)/check-core-deps ok. New fixture test
  `e4_recovers_losses_without_silent_errors`, bench `lk_searching_step`.
- 2026-11-12 **E5 not attempted** (E4 already fixes `fast_40px`). **E6 not attempted** (no measured gap
  left on the corpus). Reasons are in TRACKING_IMPROVEMENTS §E5/§E6.
- 2026-11-12 **Default tracker: ADR-0011.** `LkParams::recommended()` = E2 0.7 + E3 + E4 candidate,
  E1 off. `Default` stays B-LK18. Pinned by the fixture test
  `recommended_params_are_the_measured_candidate`. New bench `lk_recommended_step`: 0.29–0.31 ms at
  640 (median), searching step 0.58 ms. ADR-0004 → Accepted.

## Close-out against the acceptance criteria

- E0–E4 each have full-corpus measurements and a keep/reject decision with numbers
  (TRACKING_IMPROVEMENTS §E0–§E4). E5/E6 not attempted, reasons recorded (§E5, §E6). ✔
- Kept experiments: worst gate-scene Δp95 +0.070 px (E3, `rotate_15` 1280) < 0.1. No gate-scene ok
  drop. No scene has more wrong-lock frames than LK18 (`ablation_tables` summary column "none"). ✔
- Recommended default and its deviations from 2018: ADR-0011 (with ADR-0009, ADR-0010). ✔
- Latency within §8 on the bench machine, measured: tracked 0.29–0.31 ms, searching 0.58 ms
  (full-frame worst case 0.92 ms) at 640 vs the < 2 ms target. ✔
- `scripts/check-core-deps.sh` ok. ✔
- Verification: fmt, clippy `-D warnings`, `cargo test --workspace --locked`, check-core-deps,
  `ablation_tables`, `lk_fingerprint` (identical vs `8789d83`), benches. All run locally on
  2026-11-12. CI green on 3 OS + cargo-deny: E4 `0074611` run 43131916361, close-out `66768c9` run 43132339284.
