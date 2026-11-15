# 0009 — Phase 6b: test drive

Status: ACTIVE (2026-11-15)
Owner/branch: `rewrite/rust`
Depends on: 0008 (done)

## Context

Phase 6b per `docs/ROADMAP.md` and ADR-0013: before Phase 7, the user uses the app on the dev
Mac for real work and we sort out what comes up. Build under test starts at `fc884e5`
(code identical to `7e9a2bb`, CI 43746097717 green).

Inputs: `docs/qa/MANUAL_QA.md` (rows 1–18 pass), ADR-0006 (2018 defaults), ADR-0012
(platform adapters), `agent/HANDOFF.md` §"Known, not bugs". Skills: `rust-testing`,
`egui-development`, `platform-integration`, `tracking-validation`, `code-review`.

## Objective

- Log every issue the user reports in `docs/qa/TEST_DRIVE.md` (date, build, what happened,
  severity, triage, fix commit or deferral).
- Per issue: get exact steps; reproduce (headless test with fakes in
  `apps/desktop/src/runtime_tests.rs` / `app-core` tests where possible, `OCM_RECORD` for
  tracking problems, screenshots for UI); fix with a regression test, or defer to a named phase
  with a reason.
- One issue or a small related group per session.

## Non-goals

- New features. A wish for new behaviour is asked about first; if the user says yes, it ships as
  an option, off by default, with its own decision record if non-trivial.
- The items listed as "Known, not bugs" in `agent/HANDOFF.md`, unless the user asks.
- Windows/Linux on-device validation and `.app` packaging (Phase 7).

## Acceptance criteria (gate)

- Every row in `docs/qa/TEST_DRIVE.md` is fixed (commit + test, or a stated reason it can't be
  tested headless) or deferred to a named phase with a reason.
- Quality gates green locally and in CI on 3 OS.
- The user says the app is ready for Phase 7.

## Verification

```
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/check-core-deps.sh
```

## Progress

- 2026-11-15: task file and `docs/qa/TEST_DRIVE.md` created; waiting on the user's first reports.
- 2026-11-15 part 1: #1–#5 logged. Done: #1 rounded preview, #4 "Main monitor" (ADR-0014),
  #3 re-detection fix (ADR-0011 amendment). Partial: #5 steady zone (ADR-0015) — user says it
  doesn't help. Open: #2 losses, #5 jitter (top priority, part 2; see `agent/HANDOFF.md`).
  Replay harness `crates/app-core/tests/replay.rs` added.
- 2026-11-15 part 2 (planning session): #6 lag measured with the new `OCM_PERF=1` report
  (`apps/desktop/src/perf.rs`, numbers in `docs/qa/TEST_DRIVE.md` #6). Research inputs saved in
  `docs/research/HEAD_POINTER_STABILITY_INPUTS.md`. Plan below agreed for the next sessions.
- 2026-11-15 slice 2a DONE (device-checked, committed, pushed). User kept 2a lag-only (gain per
  display stays in 2c). Measured first: benches `crates/app-core/benches/frame.rs`,
  `crates/platform/benches/decode.rs` (dev profile): `handle(Frame)` 18.4 ms, of which the RGB
  mirror 17.8 ms; tracker ≈ 0.6 ms. Fix: pixel-wise `reverse` in `RgbFrame::mirror_horizontal`
  + `[profile.dev.package]` opt-levels → 0.67 ms; YUYV decode 1.8 ms, NV12 0.7 ms (capture
  thread). ADR-0016 engine thread: `apps/desktop/src/engine.rs` (`Engine` sync,
  `EngineThread`, `EngineHandle`, `InlineEngine` for UI tests; `runtime.rs` removed), frames on
  capture time, newest-wins frame slot, preview `Arc<ColorImage>` built by the engine, UI keeps
  its own validated settings. `OCM_PERF` stages now `in`/`track`/`move`/`age`/`upload`/`ui`.
  Tests: 14 runtime tests adapted, 5 new (no-UI moves on the thread, snapshot reads change
  nothing, jittered arrival = same path, deadlines without frames, stalled engine gets newest);
  33 UI tests on the inline engine; desktop 76 pass. Also fixed CI (red since `eb667e3`):
  `recommended_params_are_the_measured_candidate` didn't include the ADR-0011 2026-11-15
  amendment. Device run 1 (tracking, ~35 s): fps 29.6–30.0, `age` p95 ≤ 10.5 ms (target 25),
  `ui` p95 ≤ 0.4 ms (target 5), `replaced` 6 total — targets met. Run 2 with Options open: engine
  timings unchanged (target met); `ui` 15/17 ms p50/p95 while Options is open (likely vsync wait
  of the immediate viewport; misses the 5 ms UI target but can't reach the cursor; revisit only
  if the UI feels slow). Handover works. Not checked: minimised, dwell, CPU %.
- 2026-11-15 slice 2b DONE in code (uncommitted at handoff; device check pending). The user
  decided: **recordings are diagnosis-only**. No recordings, benchmark numbers or code that
  reads recordings go in the repo or CI; each finding becomes a fast synthetic test.
  `replay_bench`/`replay.rs`, the `toml` dev-dep and the BENCHMARK_* docs were dropped. The
  replay harness lives only in the session scratchpad. Kept: recorder thread +
  `desktop.csv`/`settings.toml` (`apps/desktop/src/recorder.rs`), `docs/qa/RECORDING_PROTOCOL.md`,
  `scripts/record-scenario.sh` (S0–S3), and the synthetic appearance corpus
  (`tracking-core/tests/appearance.rs`, pinned; `turn_extreme` 31/90 is the open gap). The user
  recorded S1 (still) and S2 (turns). Diagnosis: (a) #2 `LowConfidence` flapping on a faint
  texture; (b) #5 jitter = ≈ 0.1 px marker noise × whole-desktop scale. User: "do all three,
  make them default" and "jitter has to be 0". ADR-0017 records them: LK floor 0.25 + eigen
  ratio 0.05; 1€ filter (`pointer-core/src/one_euro.rs`); scale by the Main monitor, else the
  largest display (`scale_reference`, `map_raw_scaled`); steady zone default 5 px (3, raised by the user after the device check: "resolved jittering good part"), and an axis
  pinned at a screen limit reaches the edge. ADR-0011 and ADR-0015 amended. New tests:
  `a_faint_still_target_is_tracked_without_losses`,
  `the_cursor_does_not_move_at_rest_on_a_wide_desktop`, `one_euro` ×6,
  `scale_reference_*`, `scaled_mapping_*`,
  `recommended_tracker_never_tracks_flat_occluded_or_wrong_targets`. Five controller tests were
  restated because steady-zone trailing (≤ 5 px) and the 1€ settle (≈ 0.5 s) are by design.
  All gates green locally. This merges plan items 2c.1–2c.3 into 2b. Next: on-device check,
  then 2d (turn losses).

## Part 2 plan (slices 2a–2d, one per session)

Why this order: every later measurement depends on a frame path that isn't saturated and on
camera timestamps; the cursor work (2c) and the tracker work (2d) need the scripted recordings
and the metrics harness (2b) to be judged by numbers, not by feel. Each slice ends with gates,
the user's on-device check, a commit question, and a handoff (one slice per session).

Inputs: `docs/research/HEAD_POINTER_STABILITY_INPUTS.md` (A = online research, B = brief with
2018 RE facts) and `RE_EVIDENCE.md`. Section refs like "A §3" point into that file.

Every item says how it's checked. **CI** = runs in `cargo test` on 3 OS, no camera.
**Replay** = local diagnosis on the user's recordings, never committed; the original plan
below assumed a committed `replay_bench` and number tables. The user dropped that on 2026-11-15:
each finding becomes a synthetic CI test instead. **Device** = the user runs the app with `OCM_PERF=1`.

### 2a — Frame path and performance (#6), ADR-0016

Problem (measured, dev build): per frame the main thread spends `track` 19 ms + preview
`upload` 11.6 ms ≈ 31 of 33 ms, so the loop runs once per frame with no slack, frames wait
42 ms (p50) before the controller sees them, cursor ≈ 60 ms behind the camera, and any UI
hiccup (118 ms seen) stalls the cursor. Causes: (1) tracking + pointer run inside eframe
`logic`, paced by repaints; (2) only `tracking-core` is optimised in dev (decode, mirror,
`ColorImage::from_rgb`, egui at opt-level 0); (3) the controller's clock for a frame is the
UI-loop time, not the capture timestamp, so UI jitter becomes smoothing `dt` jitter;
(4) two hand-offs (`LatestSlot` → mpsc) and the in-flight cap keeps old frames over new ones.

1. **Measure first, keep measuring.** `OCM_PERF` moves with the code: stages timed on the
   engine thread, `age` becomes capture → pointer move. Add divan benches (compile-checked in
   CI, run locally): `platform` YUYV decode 640×480, `app-core` `Controller::handle(Frame)`
   with tracking on (synthetic textured frame), desktop preview image build. Find where the
   19 ms `track` goes before optimising anything (suspects: RGB mirror at opt 0, copies).
2. **Dev profile:** `[profile.dev.package."*"] opt-level = 2` (dependencies) and opt-level 3
   for `platform`, `app-core`, `pointer-core` (debug info kept). Not a workaround: per-pixel
   code at opt 0 is not representative of anything. Re-measure on device.
3. **Engine thread (ADR-0016 before code).** One `Engine` owns the controller, the camera
   (reading its `LatestSlot` directly: one hand-off, newest frame always wins) and the
   pointer (made on that thread by a factory, like the camera; `SystemPointer` isn't `Send`
   but needn't be). It wakes on a frame, a command, or the next controller deadline (no busy
   loop, no repaint dependency). UI → engine: commands (clicks, hotkeys, settings, access
   re-check). Engine → UI: a snapshot (`View`, settings, events) + the preview as an
   `Arc<ColorImage>` already converted (and mirrored) off the UI thread, then
   `request_repaint`. The UI only uploads that Arc and paints. Main-thread-only calls (macOS
   Accessibility prompt, opening System Settings, hotkey registration) stay on the main
   thread. Side effect: the pointer no longer drops to ~10 Hz while minimised (a "known, not
   bug" item — tell the user it changes).
4. **Camera time.** `Input::Frame` is handled at the frame's capture timestamp (clamped
   monotonic); ticks use the clock. Smoothing / dwell / steady `dt` then follow the camera.
5. **Update rate (B §4).** The engine moves the pointer once per processed frame (≤ 30 Hz;
   2018: ≤ 14 Hz, 70 ms sleep — not copied). Higher-rate output (60/120 Hz from the latest
   state, interpolation, short-horizon prediction) is *not* in 2a: interpolation adds no
   information and costs latency; it's a 2c experiment, judged by replay lag + device feel.

Checks:
- CI: `Engine` has a synchronous `step()`; the UI talks to an `EngineHandle` trait, so the 33
  kittest tests and 14 runtime tests run inline with fakes (adapt, don't delete). New tests:
  pointer moves happen with no UI pass at all; a slow UI changes neither the number nor the
  positions of moves; a stalled engine gets the *newest* frame next; the same frames with
  regular timestamps but irregular arrival give identical pointer paths; deadlines fire with
  no frames and no UI (minimised).
- Device (dev build, tracking on, 60 s, then again with Options open): fps ≥ 29, `dropped` 0;
  **proposed targets**: capture → move p95 ≤ 25 ms (was ≈ 60 p50); UI pass p95 ≤ 5 ms; opening
  Options doesn't change engine timings. Process CPU % before/after (`ps -o %cpu`).
- Device risk to check: CGEvent posting from a non-main thread on macOS (documented
  thread-safe; verify `pointer_device` test + manual QA rows for pointer/handover/dwell).
- Stop / fallback: if the engine refactor breaks more than it fixes inside the session, ship
  steps 1–2 + 4 alone (they don't need the thread) and hand off.

### 2b — Benchmark recordings and metrics harness (prerequisite for 2c, 2d)

1. `docs/qa/BENCHMARK_PROTOCOL.md` (2026-11-15: cut to S0 static, S1 still, S2 turns, S3 corners + return, ≈ 2 min; the list below is the original plan): one recording per scenario, stored **outside the repo and
   outside /tmp** (`~/ocm-recordings/<scenario>-<date>/`; the first recording was lost with
   /tmp). Scenarios: **S0 static target 30 s** (a printed photo or textured object in front of the camera,
nothing moves: pure measurement noise, A §1 distinction (1) vs (2)); S1 still 45 s looking at the main screen centre; S2 slow yaw ×5; S3 slow
   pitch ×5; S4 roll ×5; S5 four screen corners ×3; S6 fast comfortable moves; S7 talking
   30 s; S8 hand occlusion ×3; S9 return-to-start (look at centre, roam 20 s, return); S10
   optional 5 min normal use. Select the nose at the start of each. Recorder writes on its own
   thread so recording doesn't slow the engine.
2. `replay_bench` (`crates/app-core/tests/replay.rs`, `#[ignore]`, `OCM_BENCH=<root>`): runs
   every scenario through named variants (tracker × filter × mapping) on the user's real
   desktop geometry, prints one table, no ground truth needed: losses per minute and % time
   tracking (S2–S8), still wander screen px RMS / P95 / span (S1), anchor return error camera
   px (S9), lag of the filtered vs unfiltered cursor (cross-correlation, S2/S6), processing
   time p50/p95 (release). S0 vs S1 splits tracker noise from real head micro-movement, which
   decides whether 2c attacks measurement (2d) or motion (filter/gain). Results (numbers only) are committed under `docs/research/`.
3. CI: extend the synthetic corpus (`crates/tracking-core/tests/support`) with appearance-change
   scenes that have ground truth (scale + in-plane rotation + illumination gradient + local
   non-rigid warp, the 2D stand-in for out-of-plane turns) and pin today's tracker on them, so
   2d has regression gates.
- Needs the user: ~10 min of recording after 2a lands.

### 2c — Cursor stability (#5)

1. **Mapping reference** (Replay S1, S3): scale by desktop width (today, = 2018) vs the main
   display's width vs the display under the cursor. Expected: 2× less wander on 2 × 2560
   (only arithmetic so far). Changing the default departs from 2018 → ask the user with the
   numbers, ADR-0017 either way.
2. **1€ filter** in `pointer-core` (`one_euro.rs`, pure, timestamp `dt`), in camera space
   before mapping (screen space as a comparison). CI property tests: constant input → exact
   output after convergence; seeded-noise variance reduction at rest; ramp lag matches the
   formula; same response at 30 and 15 fps (dt invariance); no overshoot; reset semantics.
   Replay grid: `min_cutoff` 0.5 / 1.0 / 1.5 Hz, `beta` sweep, `d_cutoff` 1 Hz, vs the EMA.
3. Re-evaluate the steady zone (ADR-0015) on top, with hysteresis variants; dwell still
   clicks (existing dwell tests + one on filtered noise).
4. Comparison rows in the same replay table (A §3E, B §3): EMA (2018), 1€, constant-velocity
   Kalman, alpha-beta, double-exponential; then velocity-dependent gain
   `cursor_v = g(head_v)·head_v` (continuous, no discontinuity, tested for monotonicity) and a
   precision mode; 60 Hz output with interpolation vs per-frame (lag cost measured). Simplest
   that meets the targets wins.
5. Accessibility checks (B §6): dwell success / accidental clicks on S1 with each variant
   (replay counts dwell `Click` effects); no movement trapping at low speed (S2 slow onset).
- Proposed targets (Replay): S1 P95 wander ≤ 3 px on the user's desktop; S2 added lag ≤ 1
  frame vs unfiltered. Ships as an option ("Smoothing: 2018 / Adaptive"), off by default
  unless the user decides otherwise after trying it.
- Stop: if 1€ can't get S1 under ~5 px without > 2 frames lag on S6, try velocity-dependent
  gain / precision mode next; don't stack more filters.

### 2d — Tracking continuity (#2, and #3 "could be better")

1. Baselines in the `replay_bench` table first (cheap): **2018-like** (10×10 window, level 3,
   no appearance gate, 35 px displacement guard holding the previous point — B §RE; it should
   rarely "lose" but may drift: S9 return error tells), recommended (B-LK18), hw 15 + gate 0.6
   (9 vs 15 losses on the old recording), anchored template update (update the reference only
   while ZNCC to the original stays high; never replace it — Matthews 2004, A §2B), gate with
   hysteresis/degraded state instead of an instant loss (A §2C).
2. **Virtual-anchor feature cloud** (`tracking-core`, new tracker variant, ADR before code):
   24–32 Shi–Tomasi points with minimum spacing in a box around the clicked point; LK per point
   (existing kernel); forward–backward check, sweep 0.5–1.5 px (Kalal 2010); robust similarity
   with deterministic RANSAC (seeded, reprojection 1–2 px, ≥ 6–10 inliers, reject spatially
   clustered inlier sets) + least-squares refit on inliers; similarity vs affine compared;
   local subset near the anchor vs whole box (out-of-plane rotation breaks one 2D model, A §2A); the anchor moves with the transform;
   replenish lost points; confidence → existing statuses (`Tracking`, `Held` = degraded,
   pointer holds; `Lost`). The original patch becomes a drift check near the selection pose,
   not a hard per-frame gate. Must stay deterministic (R-T1) and pure.
3. Checks: CI on the 2b synthetic corpus (anchor error, losses, coherent-drift scene: all
   points slide together); Replay on S2–S9 vs baselines; release processing p95 ≤ 10 ms.
- **Stop condition:** no ≥ 50 % fewer losses on S2–S5 with S9 return error no worse than
  B-LK18 and p95 ≤ 10 ms → stop, keep the best cheap baseline, record why.
- Ships as an option, off by default (ask the user before changing the default tracker).

### Deferred, recorded so they aren't lost

- Face / landmark detection for recovery (PICO, NPD, small pure-Rust inference; A §2D): only if
  2d recovery is still weak; a face box alone can't restore the nose point.
- LK alternatives (inverse-compositional LK, ECC, direct patch alignment; B §5): only if 2d's
  per-point LK is the bottleneck in accuracy or CPU.
- Competitor source reading (eViacam GPL-3.0, GameFace Apache-2.0, archived 2025-09-05; A §7):
  behaviour only, never copy GPL code; worth one pass before finalising 2c defaults.
- Long-session drift and fatigue (B §6): S10 + the user's own use after 2d.
- Camera sensor-to-screen latency (A §5): `OCM_PERF` covers capture → move only; an end-to-end
  measurement (e.g. phone slow-motion video of an LED/screen) is Phase 8 latency work.

### After 2d

Optional 2e: the final research report (`HEAD_POINTER_STABILITY_RESEARCH.md`) only for what the
experiments leave open. Then the user test-drives again; Phase 7 when they say ready.
