---
name: egui-development
description: egui/eframe 0.37 in apps/desktop - immediate-mode state, eframe lifecycle (logic/ui), the two viewports, per-pass input and widget interaction (press and release in one pass, Sense, hover/drag), repaint from the engine thread, points vs pixels, AccessKit labels, egui_kittest tests and their blind spots. Use for any change or test under apps/desktop. Not for app-core state (see rust-architecture), OS camera/cursor adapters (see platform-integration) or OS-level synthetic vs physical input (see desktop-input).
---

## Objective

Keep `apps/desktop` thin: it renders `app-core` state and sends `app-core` commands; no
tracking, pointer, camera or platform logic in the UI. Authority is the pinned source
(egui / eframe / egui_kittest 0.37.0, winit 0.30.13 in `~/.cargo/registry/src/*/`), not memory
and not third-party skills: they target other versions (the candidate
`gregorycarnegie/egui-agent-skills` documents 0.36). Grep the source before stating an API.

## When to use

UI code, widgets (`apps/desktop/src/widgets.rs`), the two windows, UI tests
(`apps/desktop/src/app_tests.rs`), any "the click/slider/hover did nothing" report.

## When not to use

State machine and ports: `rust-architecture`. Camera, cursor, hotkey adapters:
`platform-integration`. Whether the OS delivers a synthetic click like a physical one,
winit/AppKit event behaviour: `desktop-input`. Visual tokens: `docs/PRODUCT.md`.

## Core rules

- Immediate mode: the UI is a function of `Shell` (app-core state) each pass. Widgets keep no
  state of their own; anything a user can see changing belongs in app-core, or the test
  cannot assert it. egui's own per-widget memory is keyed by `Id` (label / `id_salt`): two
  widgets with the same text collide, and an id that changes between passes loses state.
- eframe 0.37 `App` has `logic(ctx, frame)` and `ui(ui, frame)`; there is no `update`
  (`eframe-0.37.0/src/epi.rs:170,185`). `logic` runs even when windows are hidden, so hotkeys
  and settings saves go there (`main.rs`); `ui` draws. `save`/`auto_save_interval` (30 s) and
  `persist_egui_memory` are the eframe persistence hooks (`epi.rs:215,237,262`); OCM persists
  its own settings file instead.
- Two windows = root viewport + `show_viewport_immediate("options")`, called every pass while
  open (`egui::Context::show_viewport_immediate`, `context.rs:4662`). The callback runs
  nested in the parent's pass but is its own pass: eframe takes that window's own input
  (`take_egui_input`, `wgpu_integration.rs:1251`) and runs `run_ui` for it. So each viewport
  has its own `ctx.input`, pointer, focus and interaction snapshot; read `ctx.input` only
  inside the closure whose window you mean, and expect a click to reach exactly one of them.
  Close is `input.viewport().close_requested()`.
- Interaction is decided at the start of a pass from the previous pass's widget rects. A
  widget that moves or appears this pass is hit-tested at its old place; a widget's
  `Response` can therefore lag one pass behind a layout change.
- Repaint: `ctx.request_repaint()` is safe from any thread (it wakes the UI thread, doc at
  `context.rs:2113-2128`); `request_repaint_after` keeps the smallest duration. The engine thread
  calls `request_repaint` on each published change (`main.rs`); never poll in `ui`.
- Units: coordinates in egui are logical points. `pixels_per_point = zoom_factor * native`
  (formula `context.rs:496`, getter `context.rs:2712`); `set_zoom_factor` applies next pass. Only convert at the platform
  boundary (cursor injection is the platform crate's job).
- Every interactive widget needs an AccessKit name: kittest finds by it, screen readers need
  it, and kittest 0.37 panics at the end of `run` for an unnamed input widget
  (`egui_kittest-0.37.0/src/accessibility.rs`). Label a bare widget with
  `.labelled_by(label_response.id)` (as `slider_row` does) or `on_hover_text`.

## The input trap to remember

Press and release in the same pass is handled differently per `Sense`: a `Sense::click`
widget (buttons) still gets `clicked()`, but `Sense::drag` (`egui::Slider`) sees nothing, so
a slider ignores a click whose press and release arrive in one frame. `egui_kittest`'s
`event()` delivers one pass per event, so it cannot show this. Measured on 0.37.0 (2026-11-25,
scratch crate): queued events -> slider value moved; one `RawInput` with both events -> value
unchanged; button clicked in both. Details, source lines and how to test it:
`references/input-and-interaction.md`. Relevant to dwell clicks, which inject press+release
back to back.

## References (load on demand)

- `references/input-and-interaction.md` - pass anatomy, `interaction.rs` walk-through,
  press+release in one pass, Sense table, hover suppression, focus, why OCM's same-pass slider
  test does not test the same pass, sliders and `slider_row`.
- `references/kittest.md` - Harness, event delivery, the same-pass recipe, queries, accessibility
  check, DPI, what kittest cannot see (real windows, OS, timing).

## Deliverables and gates

State-driven UI; a kittest per control and per bug (fails before, passes after);
labels on every interactive widget; `docs/PRODUCT.md` rules and tokens. Gates: fmt, clippy
`-D warnings`, `cargo test -p open-camera-mouse --locked` (scoped, one heavy command at a time). Anything only a
real window can show (OS focus, cursor, two-window input routing): say what to check by hand
with `OCM_INPUT=1`.
