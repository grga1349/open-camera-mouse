# Behaviour requirements

The rules the code is held to. Code comments cite them by ID (`R-A1`, …). The rationale and
the Camera Mouse 2018 evidence behind them are in the ADRs (ADR-0006: start from 2018, fix
its defects). Changing one of these is a product decision: ask the user, then amend the ADR
that owns it.

## Tracking (`tracking-core`)

- **R-T1** A pure function of (frames, params, selection events): no clock, no I/O,
  bit-identical output for the same input on one platform.
- **R-T2** Output a sub-pixel position plus an explicit status (`Tracking`, `Held(reason)`,
  `Lost(reason)`) and diagnostics. No silent holds.
- **R-T3** The reference is built from the selection frame itself; no grace frame.
- **R-T4** Loss detection is symmetric on both axes and tested at all four edges.
- **R-T5** Motion and outlier bounds are fractions of the frame width (2018 values 35/640 and
  10/640, ADR-0009), so they mean the same at every resolution.
- **R-T6** The recommended tracker is `LkParams::recommended()` (ADR-0011, ADR-0018); B-NCC
  stays as a comparator in tests only.

## Coordinates

- **R-C1** One display space end to end: frames are mirrored at ingest (ADR-0008), and
  selection, overlay, tracking and pointer all use it. Head right moves the cursor right.
- **R-C2** A preview click maps to frame coordinates with the exact transform used to draw.

## Pointer (`pointer-core`)

- **R-P1** Absolute mapping with offset, screen limits and offset drift (2018), the
  desktop-origin defect fixed.
- **R-P2** Smoothing is time-based (FPS-independent); setting names match their polarity;
  invalid values can't be set.
- **R-P3** Per-axis gain, linked by default.
- **R-P4** Re-acquisition after a loss never jumps the pointer further than a bound
  (0.05 × the shorter desktop side); beyond it, control stays with the mouse.
- **R-P5** Multi-monitor: map into the whole virtual desktop, with negative origins handled.

## Clicking

- **R-K1** Dwell fires where the cursor is after the dwell time inside the radius; held still,
  it fires again every dwell time; leaving the radius starts a new dwell (ADR-0006
  amendment 2026-11-25).
- **R-K2** No dwell click while the target is lost or the camera doesn't own the pointer.

## Lifecycle (`app-core`)

- **R-A1** An explicit state machine, every state visible: `Stopped`, `Starting`, `NoCamera`,
  `PermissionDenied`, `NoTarget`, `Tracking`, `Lost`, `Recentering`, `Paused`,
  `CameraDisconnected`, `Error`. Pointer ownership is a separate sub-state `PointerOwner` =
  `Camera` | `Handover` | `Mouse` | `PermissionNeeded` | `Unsupported`. No silent stop.
- **R-A2** Handover (2018): a physical move over 5 px gives the mouse control; 4 s still gives
  it back to the camera.
- **R-A3** Recenter: white circle at the preview centre, 5-4-3-2-1 badge, then the point in the
  circle becomes the target. Resume from pause always recenters; Recenter also ends a pause.
- **R-A4** Hotkeys fixed: start/pause F10 on macOS, F11 elsewhere; F12 recenter. A key the OS
  refuses still works in the window, and the UI says which keys work globally.
- **R-A5** Settings validated on every update and on load; schema-versioned file; a corrupt
  file falls back to defaults and is kept aside; a newer schema is never overwritten; each
  case shows a notice.
- **R-A6** No unsynchronised shared state; a parameter change that invalidates the tracker
  re-initialises it.

## Fixture corpus and metrics

Synthetic, seeded fixtures with ground truth computed independently of the tracker
(`crates/tracking-core/tests/support`): integer and sub-pixel translation, fast motion,
brightness and contrast changes, partial and full occlusion, slow occluders, rotation ±15°,
scale ±10 %, blur, the four edges, textureless patches, distractors, dropped frames, noise; at
320×240, 640×480 and 1280×720.

Metrics: endpoint error (mean / median / p95 / max), % frames tracked, false `Tracking`
during occlusion, time to detect loss and to recover, drift, and for the pointer: shake at
rest, creep, unexplained jumps, lag (`pointer_core::metrics`).

Performance targets: no steady-state heap allocation per frame; tracker step p95 < 2 ms at
640×480 on the dev Mac; capture → cursor move p95 < one frame + 10 ms.
