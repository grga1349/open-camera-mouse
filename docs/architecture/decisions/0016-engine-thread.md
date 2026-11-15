# ADR-0016: Engine thread and camera-time frame handling

Status: Accepted
Date: 2026-11-15

## Context

Test drive issue #6 (`docs/qa/TEST_DRIVE.md`): the preview and cursor lag. `OCM_PERF` (dev
build) measured the main thread spending ≈ 31 of 33 ms per frame (`track` 19 ms + preview
`upload` 11.6 ms), frames waiting 42 ms (p50) before the controller saw them, the cursor
≈ 60 ms behind the camera, and UI hiccups (118 ms seen) stalling the cursor.

Task 0009 slice 2a measured before changing anything (`cargo bench -p app-core --bench frame
--profile dev`): 17.8 of the 18.4 ms were the colour preview's horizontal mirror at
opt-level 0. A pixel-wise `reverse` plus dev-profile opt-levels (dependencies 2, `app-core`,
`pointer-core`, `platform` 3; `Cargo.toml`) brought `handle(Frame)` to 0.67 ms. That fixes the
saturation but not the structure:

1. The controller and the pointer ran inside `eframe::App::logic`, so the cursor moved only
   when egui ran a pass: a slow UI pass, a modal OS event loop, or eframe's 100 ms throttle
   while minimised all delayed or slowed the pointer.
2. Each frame was handled at the UI loop's time, not when it was captured, so UI jitter
   became `dt` jitter in smoothing, dwell and steady-state timers.
3. Two queues (`LatestSlot` → mpsc with an in-flight cap of 3) kept *old* frames when the
   consumer fell behind and dropped the new ones.

## Options considered

- **Keep everything on the main thread, optimise only.** Cheapest; leaves 1–3 in place.
- **Engine thread owning controller + camera link + pointer; UI reads snapshots.** The pointer
  rate stops depending on repaints; the UI can't stall it. Costs a message boundary and a
  snapshot copy per frame (a `View` and an `Arc` to the preview image).
- **Controller behind a `Mutex` shared by UI and a pointer thread.** Fewer types, but every UI
  read contends with the frame path, and effects/inputs ordering gets split across threads.

## Decision

An engine thread (`apps/desktop/src/engine.rs`):

- `Engine` owns the `Controller`, the camera link, the pointer, the recorder and perf stats.
  It is synchronous (`apply(cmd)`, `step()`, `snapshot()`), so tests drive it inline with
  fakes. `EngineThread` runs it on a thread that it *creates there* from a factory (the macOS
  pointer isn't `Send`; it needn't be), waking on a camera frame, a UI command, or a timeout
  (100 ms; with no frame for 100 ms a step observes the cursor and ticks on the clock, so
  polls and deadlines fire with no frames and no UI). No busy loop.
- UI → engine: `EngineCmd` (controller inputs incl. commands, validated settings, "check
  pointer access again"). Engine → UI: an `Arc<Snapshot>` (`View`, displays, the preview as an
  `Arc<ColorImage>` built off the UI thread) published after every step that changed it, then
  `request_repaint`. The UI uploads that image and paints; it no longer ticks the controller.
- The UI keeps its own copy of the settings: it validates them with `Settings::validated`
  (pure, `app-core`) before sending, so sliders never wait for a round trip and saving stays
  on the UI side.
- Main-thread-only calls stay on the main thread: hotkey registration and draining, the macOS
  Accessibility prompt and opening System Settings.
- The camera worker hands frames over in a newest-wins slot (no in-flight cap): a stalled
  engine gets the newest frame next. Status messages (opened, failed) keep their order around
  it. The capture thread inside `NokhwaCamera` → worker hop remains; both hops are
  newest-wins and their consumers are parked waiting, so neither queues.
- **Camera time.** A frame is handled at its capture timestamp, clamped to be monotonic, and
  the cursor observation before it uses the same time. Commands and idle ticks use the clock;
  polls (access, displays) use the controller's current time, so they don't push frames' times
  around. Smoothing, dwell and steady-state `dt` then follow the camera, not the UI.
- One pointer move per processed frame (≤ 30 Hz). Higher-rate output with interpolation or
  prediction is a slice 2c experiment, not part of this ADR.

## Consequences

- The pointer keeps its rate while the window is minimised (was ~10 Hz, a "known, not bug"
  item) and while Options is open.
- `OCM_PERF` timings move to the engine: `in` (capture → engine), `track`, `move`, `age`
  (capture → pointer move done), plus `upload` and `ui` from the UI thread.
- Device risk: CGEvent posting and `CGGetActiveDisplayList` now run on a non-main thread
  (documented thread-safe in CoreGraphics); verified with the `pointer_device` test and the
  manual QA rows for pointer, handover and dwell.
- Tests: the old `Runtime` tests run against `Engine` inline; the UI tests use an inline engine
  through the same `EngineHandle` trait as the app.
