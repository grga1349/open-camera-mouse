# macOS: CoreGraphics posting and what AppKit/winit make of it

Sources: SDK headers `CoreGraphics.framework/Headers/{CGEventSource,CGEvent,CGEventTypes,
CGRemoteOperation}.h` (quoted as "header"), objc2-core-graphics 0.3.2 bindings, winit
0.30.13 `platform_impl/macos/view.rs`, OCM `crates/platform/src/pointer/macos.rs`.

## Event sources and state tables (header, CGEventSource.h)

- A source state ID names a global table of modifier, key and **mouse button state "placed
  in effect by posting events with associated sources"**.
- `CombinedSessionState` (0): "combined state of all event sources posting to the current
  user login session. If your program is posting events from within a login session, you
  should use this source state".
- `HIDSystemState` (1): "combined state of all hardware event sources posting from the HID
  system" — advised for daemons / user-space drivers.
- `Private` (-1): independent table owned by the source (remote-control programs).
- OCM creates its source with `HIDSystemState` and reads `button_state(HIDSystemState, …)`
  as "physically held" (`held_buttons`). Since posting with a source updates that source's
  table, OCM's own posted down (between its down and up) may read as held. Unverified on
  device; a test of held-button logic must not assume the reading is physical-only. Changing the
  source state ID changes suppression behaviour that was measured (see below): measure
  before and after, don't switch on header advice alone.

## Local events suppression

- Header: default suppression interval after posting is 0.25 s; a filter per suppression
  state (`SuppressionInterval`, `RemoteMouseDrag`) chooses which local hardware event
  classes pass. `CGRemoteOperation.h`: after posting a left mouse down "with remote mouse
  drag suppressing hardware mouse move events", a warning is logged after 60 s
  (`kCGMouseDownEventMaskingDeadSwitchTimeout`).
- OCM sets the interval to 0 (`SystemPointer::new`). Measured: with 0, physical movement
  still shows while the camera posts at 30 Hz (89/163 samples); `CGWarpMouseCursorPosition`
  ("without generating events", header) gave 0/162 and was rejected (ARCHITECTURE.md P6).

## Event types, fields, posting

- `CGEventCreateMouseEvent(source, type, pos, button)`: `pos` in global coordinates;
  `button` "is ignored unless mouseType is one of kCGEventOtherMouse{Down,Dragged,Up}"
  (header). Left/right are implied by the type.
- Moves with a button down should be that button's `…MouseDragged`: a `MouseMoved` tells the
  window server the button is up (OCM `move_event`, device check 2026-11-25 for other apps).
  winit itself treats moved and dragged the same (`mouse_motion`), so for OCM's own windows
  the type matters only for routing (dragged → window that got the mouse-down; AppKit
  guide, not re-checked).
- `kCGMouseEventClickState` (field 1): 1 single, 2 double, 3 triple (header). OCM posts
  down/up without setting it (zero-gap synthetic down/up); whether AppKit apps need it for a single click is
  unverified.
- `kCGMouseEventNumber` (0): matching down/up share a number. `kCGMouseEventDeltaX/Y`
  (4/5): delta since the last movement event; OCM does not set them (synthetic drag events carry no deltas; unverified impact).
- `kCGEventSourceUserData` (42, settable per source with `CGEventSourceSetUserData`),
  `kCGEventSourceUnixProcessID` (41), `kCGEventSourceStateID` (45) identify posted
  events — readable by an event tap or from `NSEvent.CGEvent`, **not through winit**.
- `CGEventPost(tap, event)` "posts the specified event immediately before any event taps
  instantiated for that location". Locations: `HIDEventTap` (0, OCM), `SessionEventTap`,
  `AnnotatedSessionEventTap`. Posting at HID level makes the event look like hardware to
  every later tap and app.
- Landing is asynchronous: 0.3–0.9 ms, rarely ~38 ms (measured, `geometry.rs`).

## Reading back

- Cursor position: `CGEventCreate(NULL)` + `CGEventGetLocation` (OCM `raw_position`),
  global display coordinates in points, origin top-left of the main display, y down,
  secondary displays may be negative. Same space as `CGDisplayBounds` and posted events.
- winit/egui see `locationInWindow` → view (flipped) → physical px → egui points; on macOS
  1 egui point = 1 CG point at zoom 1.0.

## Permissions

- Posting needs Accessibility; without it events are dropped silently.
  `CGPreflightPostEventAccess` / `CGRequestPostEventAccess` (10.15+, header) and
  `AXIsProcessTrusted`. Measured (task 0012): `AXIsProcessTrusted` follows the switch live,
  preflight keeps its launch-time answer → OCM uses either.
- Listening (event taps) is a separate grant (`CGPreflightListenEventAccess`); OCM has none.
  Any design that distinguishes physical from synthetic via a tap needs a new permission:
  a product decision, ask first.

## Windows, focus, first mouse

- winit `accepts_first_mouse` defaults to true: a click on an inactive OCM window both
  activates it and is delivered. `mouseMoved:` go to the key window; outside the view they
  are dropped unless `pressedMouseButtons != 0` (winit), so after a physical press anywhere
  in an OCM window, camera moves become `CursorMoved` far outside it (OCM_INPUT logs:
  x −753 … 954 in the Options window).
- Global hotkeys: global-hotkey 0.10 uses Carbon `RegisterEventHotKey` (CGEventTap only for
  media keys) on the main thread's run loop. F11 is taken by Show Desktop, so start/pause is
  F10 (P13). Whether the focused OCM window also receives the key is unverified;
  `KeyRouter` (`crates/platform/src/hotkeys.rs`) de-duplicates either way.
