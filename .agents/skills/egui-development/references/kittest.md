# egui_kittest 0.37 in OCM

Paths under `~/.cargo/registry/src/*/egui_kittest-0.37.0/src/`.

## Harness

- OCM builds `Harness::builder().with_size(..).build_ui_state(|ui, shell| .., shell)` and
  calls the UI function directly (`app_tests.rs:150`, and :161 in helper `options_window`). Because it is
  a plain `Ui`, not eframe, `show_viewport_immediate` is not exercised: with no immediate
  renderer registered it embeds the viewport as a window (`embed_viewports` defaults to true, `context.rs:797`; branch at `context.rs:4670-4686`). Two-window
  routing is not testable here.
- `run_steps(n)` = `step()` n times. `run()` repeats until settled (animations done, no
  repaint requested) and panics past `max_steps`; the accessibility check runs only at the end
  of `run`/`try_run` (`lib.rs:519`), not after `run_steps`.
- DPI: `builder.with_pixels_per_point(x)` (`builder.rs:106`); default 1.0. Tests are in logical
  points; physical-pixel behaviour (rounding at 125 % / 130 %) needs an explicit ppp.
- `with_step_dt` sets the simulated time per pass (`builder.rs:166`); time-based behaviour
  (click duration, animations) follows it, not the wall clock.

## Event delivery: the important part

`Harness::step` (`lib.rs:371-380`): it takes all queued events and runs ONE PASS PER EVENT,
pushing one event into `RawInput` each time. So `h.event(press); h.event(release);
h.run_steps(1)` is two passes, and `Node::click()` (`node.rs:61-78`: hover, press, release)
is several passes. That is why `a_click_on_a_slider_rail_sets_the_value_there_in_one_pass_or_two`
(`app_tests.rs:1016`) passes for `same_pass = true`: both branches deliver press and release
in separate passes. It covers "press, release in a later pass" and "press, a pass, release" but
not a batched `RawInput`. Not a failing test; a gap in what the name claims.

To deliver both events in ONE pass, bypass the queue:

```rust
let ev = |pressed| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary,
    pressed, modifiers: egui::Modifiers::NONE };
h.event(egui::Event::PointerMoved(at));   // hover first, own pass
h.run_steps(2);
h.input_mut().events.push(ev(true));      // lib.rs:602 input_mut -> next RawInput
h.input_mut().events.push(ev(false));
h.step();                                  // queue is empty -> one pass with both events
h.run_steps(3);
```

Measured on 0.37.0 with an `egui::Slider` (scratch crate, 2026-11-25): queued `event()` pair ->
value set; `input_mut` pair -> value unchanged; with a `Sense::click` region -> clicked once.
A test that expects a slider to move on a same-pass click will fail today; write it as a
regression test only once the product decision on slider tap semantics is made (AGENTS.md:
ask before changing behaviour).

## Queries and actions

- `get_by_label`, `get_by_label_contains`, `query_*`, `get_by_role_and_label(Role::Tab, ..)`
  (used in `app_tests.rs:652`): match the AccessKit label; `get_*` panics on zero or several
  matches. Unlabelled widgets are invisible to queries: use `labelled_by`.
- `Node::rect()` is in logical points; compute positions from it (`rect.min.x + rect.width()
  * 0.25`) rather than hard-coding.
- `click_accesskit`, `increment_accesskit`, `set_value_accesskit` (`node.rs:103-135`) act through
  the accessibility action path (no pointer): they prove labelling and action wiring, not
  pointer behaviour. Use them in addition to pointer-driven tests, not instead.
- Accessibility check: after `run`, an input widget without a name panics with the widget
  list. `HarnessBuilder::with_accessibility_check(false)` disables it (`builder.rs:156`); only
  for tests of something deliberately unnamed (a scratch experiment), not to silence an
  OCM widget.

## What kittest cannot show

Real OS windows and focus, the two-viewport input split, a physical vs injected click,
frame batching by winit, cursor position, timing under load. For those: a by-hand check with
`OCM_INPUT=1` (`docs/TESTING.md`), and say so in the report. Image snapshots need the `snapshot`
and `wgpu` features and human review; OCM does not rely on them.

## Test hygiene

State assertions on `Shell`/`Settings` (`settings(&h).gain_x`), never on pixels; one behaviour
per test; names say the behaviour. A regression test must fail before the fix: run it first
and read the failure. Keep the Shell constructors that avoid camera, files and threads
(`stopped()`, `stopped_with(..)`).

Provenance: structure (harness choice, query table, accessibility check) informed by
gregorycarnegie/egui-agent-skills `skills/egui-kittest` (MIT, commit 82b329557a2921dd9c9455de1a87e76aea75abed, 0.36-era);
all API statements here were re-verified against 0.37.0 source. No text copied.
