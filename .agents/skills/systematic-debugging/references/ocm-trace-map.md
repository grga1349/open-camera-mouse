# OCM trace map: camera to cursor and back

Load for pointer, click, handover and settings-vs-cursor bugs. Verify file names against the
tree before relying on them; the code wins over this file.

## The loop, hop by hop

| # | Hop | Where | Log or check | Test layer |
|---|---|---|---|---|
| 1 | Camera worker captures a frame | `crates/platform/src/camera/` | `OCM_PERF` `in`, `age`; frame drops | none off-device; fake frames |
| 2 | Engine thread: observe, track, effects | `apps/desktop/src/engine.rs` | `OCM_PERF` track/loss lines; order of observe, frame, effects | `engine_tests.rs` with fake pointer |
| 3 | Tracker output (point, status) | `crates/tracking-core` | "Show diagnostics" scores; loss reason | synthetic fixture |
| 4 | Controller step: state, `drive`, dwell, takeover | `crates/app-core/src/controller.rs` (`drive`, `take_over`, `on_pointer_observed`) | state before and after, effects emitted | `crates/app-core/tests/controller.rs` |
| 5 | Pointer maths: map, smooth, limits, dwell, handover | `crates/pointer-core/src/` | inputs and outputs of the pure function | unit tests |
| 6 | Platform pointer adapter: move, click, echo filter | `crates/platform/src/pointer/{macos,windows,linux,geometry}.rs` | commanded vs. observed position; event type posted | unit tests on pure parts; OS call is device-only |
| 7 | OS delivers the event (cursor, focus, windows) | OS | `OCM_INPUT`; OS-specific (permissions, focus, other apps) | manual |
| 8 | winit and egui receive it per window | `winit` 0.30.13, `egui` 0.37.0 sources in `~/.cargo/registry/src/*/` | `OCM_INPUT` lines per window (`tracking`, `options`) | kittest replay (`app_tests.rs`) |
| 9 | UI widget acts (slider, button, preview click) | `apps/desktop/src/{options,widgets,preview,status}.rs` | the command it sends | kittest |
| 10 | Cursor observation re-enters the controller | `on_pointer_observed`, `last_commanded` | does the controller read its own move, or the user's? | controller rig with an echoing pointer |

Find the first hop whose output is wrong given a correct input; hops 8 to 10 close the loop,
so a bug can start at any hop and show up at another.

## Test-layer ladder (canonical)

Write the regression test at the lowest layer that shows the bug.

| Bug lives in | Test |
|---|---|
| Pure maths, dwell, handover, mapping | `pointer-core` unit test (`crates/pointer-core`) |
| State machine, effects, settings changes, takeover | controller rig `crates/app-core/tests/controller.rs` |
| Ordering of observe, frame, effect; threads | engine test with a fake pointer that echoes like the adapters (`apps/desktop/src/engine_tests.rs`) |
| Widgets, tabs, windows, event sequences | `egui_kittest` (`apps/desktop/src/app_tests.rs`); replay a logged event sequence |
| OS adapter geometry and event building | `platform` unit tests on pure functions; the OS call itself is device-only |
| Tracker behaviour | synthetic fixture with ground truth (`tracking-validation`) |

Input-specific rows (outside positions, same-pass RawInput): `desktop-input`.

## Traps specific to this loop

- **Self-observation.** What the controller commands, the OS echoes back (hop 6 to 10). A
  fake pointer that does not echo hides bugs; the engine tests use one that echoes like the
  adapters.
- **Settings that change the mapping** (gain, mirror, offset) move the cursor without
  camera motion; a widget reading the cursor can then change the setting again.
- **Synthetic vs. real input** (dwell clicks, posted drags) can be indistinguishable or
  differ per OS (click gap, delta fields, raw input). Say which OS was checked.
- **Two windows** (Tracking, Options): events, focus and first-click behaviour differ per
  window; log which one.
- **Handover timing** (mouse takes over for a few seconds): time-dependent; use injected
  timestamps (`now_us`) in tests, not sleeps.
- **Threads:** std threads, mpsc, Mutex and atomics (no async). Suspect ordering and stale
  snapshots before suspecting the maths; reproduce ordering with a deterministic test, not
  with sleeps.

## Experiments that change one variable

Disable dwell; fix the gain; freeze the camera input (fake frames); skip the effect at hop 6
(log instead of posting); swap the echoing fake for a non-echoing one; run on a second OS;
bisect commits between the last good tag and now. Record the predicted result before running.

## Turning a device log into a test

1. Extract the minimal event sequence (a handful of lines) that shows the symptom.
2. Re-create it with synthetic values at the lowest layer from the table; do not commit the
   log or recorded numbers.
3. Confirm it fails for the logged reason; if it passes, the repro is missing a variable
   (tab, window focus, OS event type): extend `OCM_INPUT` and ask for one more short run.
