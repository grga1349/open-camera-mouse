# egui 0.37 input and interaction (verified against source)

Paths are under `~/.cargo/registry/src/*/egui-0.37.0/src/` unless stated.

## One pass

1. eframe collects winit events into `egui_input.events` (a `Vec`) and hands the whole batch
   to egui at the next redraw (`egui-winit-0.37.0/src/lib.rs:870-884` pushes
   `PointerButton`; `take_egui_input` drains it). Press and release that arrive between two
   redraws are therefore ONE `RawInput`. (Whether the OS/winit delivers an injected click as
   two events and when: `desktop-input`.)
2. `Context::begin_pass` (`context.rs:459`) builds `InputState`. `PointerState::begin_pass`
   walks the events in order and appends `PointerEvent::{Moved,Pressed,Released}`
   (`input_state/mod.rs:1140-1215`); `interact_pos` ends at the last event's position.
3. Hit test against `prev_pass.widgets` at `interact_pos` (`context.rs:~525`), then
   `interaction::interact(...)` (`interaction.rs:109`) produces the snapshot
   (`clicked`, `dragged`, `drag_started`, `drag_stopped`, `hovered`, `contains_pointer`) and
   updates `potential_click_id` / `potential_drag_id` in `memory.interaction`.
4. Your `ui` runs; each widget's `Response` is built by `get_response`
   (`context.rs:1714`) from that snapshot and the same pointer events.

## Press + release in one pass (the answer)

`interaction.rs` has the note "a press-release in the same frame is NOT considered a drag"
(`interaction.rs:157`). In the event loop: `Pressed` sets `potential_click_id` /
`potential_drag_id` from the hits; `Released` then uses `potential_click_id` to set `clicked`
(only if `click.is_some()` and not decidedly dragging) and resets both potentials and `dragged`.

- `Sense::click` / `click_and_drag` widget: `clicked` is set, `Response::clicked()` is true.
  Verified: a click-sensing region with press+release in one `RawInput` clicks once.
- `Sense::drag` only (egui `Slider`: `ui.allocate_response(.., Sense::drag())`,
  `widgets/slider.rs:594`): the "start drag" step runs after the loop, when
  `potential_drag_id` is already None, so `dragged`/`drag_started`/`drag_stopped` stay unset.
  `get_response` then has `IS_POINTER_BUTTON_DOWN_ON` false (reset by the `Released` branch,
  `context.rs:1820`), `clicked` None, `drag_stopped` false, so `is_interacted_with` is false and
  `interact_pointer_pos()` is None (`context.rs:1828-1837`). `Slider::slider_ui` only sets the
  value `if let Some(pos) = response.interact_pointer_pos()` (`slider.rs:532`), so the value does
  not change. Measured, not just read: see SKILL.md.
- If press and release are in different passes, the press pass marks the drag-only widget
  `dragged` immediately (`interaction.rs:~219`, drag-only branch) and the slider takes the
  value from the pointer position that pass.

Consequence for OCM: a real tap on a slider track works when the two events straddle a redraw
(normal mouse: milliseconds apart, usually different frames, but not guaranteed) and can be
lost when batched (dwell/injected press+release back to back, a busy frame). A fix belongs in
the UI layer only if it is an interaction fix (e.g. a widget that senses click+drag, or
reading `pointer.interact_pos()` on `clicked`); changing slider semantics is user-visible:
ask first, measure with the same-pass test in `kittest.md`.

## Sense

| Constructor | Click | Drag | Focusable | Notes |
|---|---|---|---|---|
| `Sense::click()` | yes | no | yes | `Sense::CLICK` is the non-focusable form (`sense.rs:57-69`) |
| `Sense::drag()` | no | yes | yes | drag starts on press, no latency |
| `Sense::click_and_drag()` | yes | yes | yes | decision postponed until drag or release (latency, `sense.rs:73-80`) |

For a click+drag widget the drag only starts once `is_decidedly_dragging()` (moved more than
`max_click_dist` or held longer than `max_click_duration`, `input_state/mod.rs:1539,1566`) or the
pointer leaves the widget.

## Hover, focus, capture

- `Response::hovered()` is false while any button is down and this widget is not the one being
  interacted with (`context.rs:1828-1830`); the interacted widget stays the only hovered one for
  the whole gesture, even outside its rect (`interaction.rs` hovered computation,
  `response.rs:322-333`). A hover highlight on another control during a drag is therefore
  absent by design, not a bug.
- Drag capture: `dragged` persists from `drag_started` until a `Released` (or Escape,
  `interaction.rs:138`), whatever the pointer does. `PointerGone` does not release
  (`input_state/mod.rs:1219`), so a slider drag survives leaving the window.
- Focus: `Sense::FOCUSABLE` widgets register for keyboard focus; a press elsewhere
  surrenders focus (`SurrenderFocusOn`, `context.rs:1840-1848`). Space/Enter act as a click
  on a focused click-sensing widget (`context.rs:1758-1765`); the slider handles arrows via
  `slider_core::keyboard_steps`.
- Disabled widgets (`ui.add_enabled`) are never hovered or dragged (`context.rs:1787`).
- Two viewports: pointer and focus are per viewport; the `InteractionSnapshot` is
  `viewport.interact_widgets`, so a press in the Options window cannot become a click in the
  tracking window.

## OCM's `slider_row` (`apps/desktop/src/widgets.rs`)

An invisible egui `Slider` (full width via `slider_width`, `show_value(false)`,
`SliderClamping::Always`) does pointer, keyboard and AccessKit set-value; `paint_slider`
draws the rail and thumb at the handle centre egui computes (`rect.height()/2.5` inset), so
clicks and paint agree. The label is attached with `labelled_by`. Value feedback hazard: the
displayed value is the same `f32` the engine reads; anything that rewrites that value while the
pointer is down (app-core re-clamp, or a gain change rescaling the mapping offset and moving
the cursor under a held slider) makes the slider fight the user. Keep the
slider writing only on `response.changed()` semantics and test with a held pointer.

## Debugging "the click did nothing"

Method: `systematic-debugging`; OS/winit side (no log line, capture, synthetic events):
`desktop-input`. egui-specific checks:

1. Was press+release in one pass and the widget `Sense::drag`-only? (above)
2. Was the widget disabled, covered by another layer, or at a different rect last pass?
3. Did the id change between passes (label text includes a value)?
