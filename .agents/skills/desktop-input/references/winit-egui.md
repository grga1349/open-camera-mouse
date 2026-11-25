# winit 0.30.13 → egui-winit / eframe 0.37.0: the pointer path

Sources: `~/.cargo/registry/src/*/winit-0.30.13`, `egui-winit-0.37.0`, `eframe-0.37.0`,
`egui-0.37.0`. Line numbers are for those versions; re-check after any upgrade.

## winit: what reaches a window

**macOS** (`platform_impl/macos/view.rs`)
- `mouseMoved:`, `mouseDragged:`, `rightMouseDragged:`, `otherMouseDragged:` all call the
  same `mouse_motion` (≈625–643): winit makes no difference between moved and dragged.
- `mouse_motion` (≈1061): position = `locationInWindow` converted to the (flipped,
  top-left) view, logical → physical by `scale_factor`. If the point is outside the view it
  is dropped **unless `NSEvent::pressedMouseButtons() != 0`**: with any button down, moves
  far outside the window are delivered as `CursorMoved` in window coordinates (negative or
  beyond the size). Whether `pressedMouseButtons` includes posted CG button state:
  unverified.
- `mouseDown:`/`mouseUp:` (and right/other) first call `mouse_motion`, then emit
  `MouseInput` (≈582–620). `buttonNumber` 0/1/2/3/4 → Left/Right/Middle/Back/Forward.
- `CursorEntered`/`CursorLeft` come from a tracking rect over the view (`mouseEntered:` /
  `mouseExited:`, ≈645–660), not from position checks.
- `acceptsFirstMouse:` returns `accepts_first_mouse`, default `true`
  (`window_delegate.rs` ≈73); eframe/egui-winit don't change it (grep: no use). So the first
  click on an inactive OCM window is delivered as a click, and also activates it.
- `setAcceptsMouseMovedEvents(true)` on every window (`window_delegate.rs` ≈586). AppKit
  routes mouse-moved events to the key window and dragged events to the window that got the
  mouse-down (Cocoa event-handling guide; not re-checked this session).

**Windows** (`platform_impl/windows/event_loop.rs`)
- Every button down calls `SetCapture` (refcounted), every up `ReleaseCapture` (≈978–991,
  1781–1866). While captured, `WM_MOUSEMOVE` keeps arriving with client coordinates outside
  the client rect.
- `WM_MOUSEMOVE` (≈1643): `get_pointer_move_kind` compares to `GetClientRect` → emits
  `CursorEntered` (+ `TrackMouseEvent(TME_LEAVE)`) or `CursorLeft`, then `CursorMoved` only
  if the position changed (spurious-move filter). So during a captured drag you get
  `CursorLeft` followed by more `CursorMoved`.
- Raw input (`handle_raw_input`, ≈2559) emits `DeviceEvent::MouseMotion` only for
  `MOUSE_MOVE_RELATIVE` packets; absolute packets produce no motion event.
- Process DPI awareness: `become_dpi_aware()` at event-loop creation sets
  PER_MONITOR_AWARE_V2 (fallbacks V1, 8.1, Vista) (`dpi.rs` ≈20, `event_loop.rs` ≈199).
  OCM creates `SystemPointer` in the engine factory, after that point.

**X11** (`platform_impl/linux/x11/event_processor.rs`, XInput2)
- `XI_Motion` → `CursorMoved` with `event_x/event_y` (window-relative) when changed
  (≈1104); `XI_Leave` → `CursorLeft` (≈1227). The core protocol's automatic grab on
  ButtonPress keeps events going to the pressed window outside its bounds (X11 protocol,
  not re-checked here). `device_id` is the event's `deviceid` (master); the XTest slave
  is only visible as `sourceid`, which winit does not expose (unverified on device).
- Emulated pointer events from touch (`XIPointerEmulated`) are skipped for buttons (≈1063).

## egui-winit: winit event → `egui::Event`

`egui-winit-0.37.0/src/lib.rs`
- `CursorMoved` → `on_cursor_moved` (≈926): physical px / `pixels_per_point` → points,
  sets `pointer_pos_in_points`, pushes `PointerMoved`. No bounds check: outside positions
  pass through.
- `CursorLeft` (≈415) → `pointer_pos_in_points = None` + `PointerGone`.
- `MouseInput` → `on_mouse_button_input` (≈870): **pushes `PointerButton` only if
  `pointer_pos_in_points` is `Some`**. A press/release arriving after `CursorLeft` without a
  new `CursorMoved` is dropped (macOS always sends a motion first; Windows/X11 depend on
  ordering: unverified). A dropped release leaves egui thinking the button is down.
- `any_pointer_button_down` is only updated in touch-simulation mode (≈898–912); don't rely
  on it.
- `Focused` (≈483): on macOS uses `window.has_focus()` instead of the event value (winit
  issue 4371 workaround); pushes `WindowFocused`.
- `DeviceEvent::MouseMotion` → `on_mouse_motion` → `Event::MouseMoved(delta)` only if the
  pointer is in the window or (touch mode) a button is down (≈839).

## eframe: which viewport gets it

`eframe-0.37.0/src/native/wgpu_integration.rs`: each viewport owns its own
`egui_winit::State` (≈119), found by `viewport_from_window[window_id]` (≈90, 931). Window
events go only to the window winit addressed. `MouseMotion` (no window) goes to the focused
viewport if it accepts it, else the first that does (≈511). OCM: Tracking = root viewport,
Options = `show_viewport_immediate` (`apps/desktop/src/main.rs` ≈196); `input_log::log` is
called per viewport, so `OCM_INPUT` lines name the window that got the event.

## egui: what a widget concludes

Click/drag rules, same-pass press+release, `Sense`, slider internals and kittest blind spots
are in `egui-development` (`references/input-and-interaction.md`, `kittest.md`). The two
facts that matter for the OS loop: `PointerGone` is **not** a release, so a drag continues
while the OS keeps sending outside positions (`input_state/mod.rs` ≈1219); and
`Slider::value_from_position` uses `remap_clamp` (`slider_core.rs` ≈144), so a pointer far
outside the rail pins the value to min or max (a speed slider swinging 1 ↔ 60).

A kittest replay can include outside positions and `PointerGone`; it can't show OS capture,
routing between the two OS windows or posted-event timing.

## Failure scenarios (symptom → mechanism)

| Symptom | Mechanism |
|---|---|
| Slider swings min ↔ max during a real press | capture + camera moves → outside positions clamp; or a gain change rescales the mapping offset and moves the cursor under the held slider (feedback loop) |
| Preview click re-selects, "find again" loop | a dwell click on the preview while tracking is taken as user intent and re-selects the target |
| Drag ends early, orphan release | a synthetic dwell click inside a physical-button drag releases the drag |
| Handover fires on OCM's own move | read-back before landing, snap, rounding (`Echo`) |
| Stuck drag in egui | egui-winit drops `MouseInput` while its pointer pos is `None` |
| Other apps ignore camera drags | wrong event type, no deltas, no raw input, zero-gap synthetic down/up |

## Diagnostics

`OCM_INPUT=1` logs what egui received per window (`apps/desktop/src/input_log.rs`). When a
symptom needs more, extend it (diagnosis only, logs never committed) with: active Options
tab, posted move target + event type + held-button reading from the adapter, focused
viewport, and a shared clock so OS-side and egui-side lines interleave. Measure first, then
turn the log into a test.
