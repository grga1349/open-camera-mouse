---
name: desktop-input
description: How OS pointer input, winit and egui interact in an app that both injects cursor moves/clicks and takes pointer input in its own windows. Covers implicit capture, coordinate spaces, synthetic vs physical events, button state during drags, focus and multi-viewport routing, feedback loops, global hotkeys. Use when diagnosing or changing click, drag or move behaviour under camera control, Echo/handover, OCM_INPUT, or anything where posted events reach OCM's own windows. Not for adapter plumbing, permissions or camera (see platform-integration) or widget/layout/kittest mechanics (see egui-development).
---

## Objective

OCM writes the system cursor (camera moves, dwell clicks) and reads it back twice: through
`PointerSink::position()` (handover) and through winit → egui in its own Tracking and
Options windows. The clicking bugs seen so far (slider swings, preview re-select, drag
released early) are interactions between those paths. This skill gives the event model per OS, the invariants an
injecting app must hold, and how to reproduce such bugs with tests before fixing them.

## When to use

- A click, drag or slider misbehaves while the camera drives the cursor (slider swings, drag ends early).
- Changing event types, timing or `Echo` in `crates/platform/src/pointer/*.rs`, or
  `apps/desktop` code that reacts to pointer input while the camera moves it.
- Physical vs synthetic events, held buttons, focus, hotkeys vs focus.

## When not to use

New OS adapters, camera, permissions, packaging: `platform-integration`. egui click/drag
rules, widgets, kittest: `egui-development`. The debugging method itself:
`systematic-debugging`. Pointer maths (`pointer-core`): `rust-engineering`; its tests: `rust-testing`.

## The loop to keep in mind

```
camera → controller → PointerSink::move_to/click → OS (CGEventPost / SetCursorPos+SendInput / XTest)
   ↑                                                     │ cursor position, button state
   │                                                     ├→ PointerSink::position() → Echo → handover
   └── settings change (gain, mirror, target) ← egui ← winit ← OS delivers events to OCM's windows
```

Any change to the controller's mapping that a pointer event in OCM's own window can trigger
closes the loop (a gain change rescales the mapping offset, moving the cursor under a held
slider, which reads the new x).

## Invariants

1. **Inside OCM's windows, posted and physical events are indistinguishable.** winit and
   egui carry no source flag. Decide by state OCM owns (what it commanded, `Echo`, held
   buttons), never by assuming an event in egui came from the user.
2. **A held button captures the pointer.** On all three OSes (macOS/Windows: winit source;
   X11: core protocol) the window that got the press
   keeps getting moves, in its own coordinates, far outside its rect (negative or larger than
   the window). egui sees `PointerGone` then more `PointerMoved`; a drag continues. Camera
   moves during a physical press therefore drive OCM's widgets.
3. **A setting changed by a pointer gesture must not move the cursor as a side effect**
   (gain scales the offset from the centre; the fix direction is decided in the fix session).
4. **Don't inject a button transition into a physical press**: a synthetic dwell down/up
   while a real button is held releases the user's drag. Needs held-button state from the OS, which
   `PointerSink` does not expose today; the exact rule is a product decision.
5. **One coordinate space per layer, converted once**: controller = OS desktop space
   (macOS points, Windows physical px, X11 root px); winit = window-relative physical px;
   egui = window-relative points. Don't compare across without the window origin and
   `pixels_per_point`.
6. **Posted moves land late** (macOS 0.3–0.9 ms, rarely ~38 ms, measured, `geometry.rs`).
   Any read-back within ~100 ms must go through `Echo`.
7. **Sent ≠ applied**: macOS drops events silently without Accessibility; on Windows neither
   `SendInput`'s return value nor `GetLastError` says a failure was UIPI (MS docs). Report
   state, don't assume.

## Failure scenarios and diagnostics

Symptom-to-mechanism table and `OCM_INPUT` extension list: `references/winit-egui.md`
(sections "Failure scenarios" and "Diagnostics"). Measure first, then turn the log into a test.

## Testing strategy

Lowest layer that shows the bug, failing before the fix; the full ladder (pure, controller
rig, engine, kittest, platform) is in `systematic-debugging/references/ocm-trace-map.md`.
Input-specific rows:
- **Engine + fake pointer** (`apps/desktop/src/engine_tests.rs`): a fake that echoes like the
  adapters (late landing, snapping, settable held button) for synthetic-click-in-drag rules.
- **kittest replay** (`apps/desktop/src/app_tests.rs`): the logged sequence via
  `Harness::event`, incl. outside positions and `PointerGone`; same-pass press+release via
  `input_mut` RawInput. Shows egui's reaction, not the OS's (no capture, no posted events).
- **Device check** for what only the OS decides (capture, event type, UIPI, XTest): exact
  steps for the user; say which OS was not run.

## References (load when needed)

- `references/winit-egui.md`: per-OS winit mouse events, capture, first mouse, egui-winit
  translation, eframe viewport routing. For anything seen in `OCM_INPUT`.
- `references/macos.md`: event sources, suppression, event types and fields, taps, button
  state, Accessibility, coordinates. For `pointer/macos.rs`.
- `references/windows.md`: `SetCursorPos` vs `SendInput`, raw input, capture, UIPI, DPI,
  injected flags. For `pointer/windows.rs`.
- `references/x11.md`: XTest, grabs, XI2 source devices, Wayland, global hotkeys per OS.
  For `pointer/linux.rs` or hotkeys.

## Deliverables and gates

A failing-then-passing test at the lowest layer for each fix (AGENTS.md); `OCM_INPUT`
evidence quoted when the bug came from a device; a by-hand check per OS listed as done or
not done; `docs/ARCHITECTURE.md` P-table and `docs/TESTING.md` updated when a platform
behaviour is newly measured. Claims about OS behaviour state their source (measured here,
official doc, upstream source line) or are marked unverified.
