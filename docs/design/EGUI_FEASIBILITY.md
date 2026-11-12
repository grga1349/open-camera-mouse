# egui feasibility — approved Phase 5 UI reference

Reference: `docs/design/design-lab.html` v3, approved by the user on 2026-11-12 (task 0007),
including the later marker-colour change to neon green the same day.
Target: `eframe`/`egui` **0.37.0** (workspace `Cargo.toml`). The API names below were checked in
the 0.37.0 registry sources, not recalled from memory. Nothing here is implemented yet; this is the
Phase 6 input.

## Approved reference in one paragraph

Monochrome UI in two themes, **Carbon** (dark, default) and **Paper** (light), switchable in
Options → General → Appearance. Two colours only: the target marker is a **thin neon-green ring**
(`#00ff00`, same in both themes, because it sits on the camera image), and primary buttons are
**yellow** (Carbon `#ffd60a`, Paper `#f5c518`, black text). Two **fixed-size** native windows:
**Tracking** (560 × 620 pt: preview, status text line, Pause/Start F11, Recenter F12, Options) and
**Options** (400 × 620 pt: Pointer / Clicking / Camera / General tabs, Restore defaults, Close). No
auto-recenter. Status is an icon + text line, never colour alone.

Marker colour was chosen by measurement (2026-11-12, lab with a colour camera stand-in and four
skin tones). The smallest CIELAB ΔE76 vs skin (light/olive/medium/dark + shadows) was: neon green
110, magenta 109, electric blue 112 (but weak on dark skin in the screenshots), mint `#3ddc84` 70,
yellow 58 (near-invisible on light skin, WCAG luminance ratio 1.10). Neon green separates by hue,
not luminance (ratio 1.14 vs light skin), so it must stay fully saturated. Known residual risk: a
green background or clothing next to the face.

## Per affordance

| Affordance | egui approach (0.37) | Feasible | Notes / risk |
|---|---|---|---|
| Two native windows | Tracking = root viewport. Options = `Context::show_viewport_deferred` (own repaint, state shared via `Arc<Mutex<…>>` / channel to `app-core`) or `show_viewport_immediate` (simpler borrow, repaints with parent). Recommend **immediate**: Options is low-frequency and must read the same settings struct | Yes | Multi-viewport is supported on the native winit backends (macOS, Windows, X11, Wayland). Wayland can't position windows, so "open next to the tracking window" is best-effort there (PLATFORM_RISKS P10/P11 class) |
| Fixed size, not resizable | `ViewportBuilder::with_inner_size([560., 620.])`, `.with_resizable(false)`, `.with_maximize_button(false)`; Options also `.with_minimize_button(false)` | Yes | Sizes are logical points, so high-DPI scales automatically. At text scale 130 % / larger controls the content may not fit. Phase 6 must either grow the fixed size per scale step or scroll the status/options area (`ScrollArea`), which is what the lab does |
| Themes Carbon / Paper | Two `egui::Visuals` built from the tokens below, applied with `Context::set_visuals` (or `set_theme` + per-theme style). Persist the choice in settings (R-A5) | Yes | Start from `Visuals::dark()` / `light()` and override every colour; don't rely on egui defaults (they're bluish) |
| Yellow primary buttons | Custom `Button::new(...).fill(yellow)` with black `RichText`, or a small `primary_button(ui, …)` helper | Yes | Keep one helper so the "only yellow" rule is enforced in one place |
| Camera preview | Grayscale frame → `ColorImage::from_gray([w, h], &luma)`. One `TextureHandle` created once with `Context::load_texture`, then updated each frame with `TextureHandle::set(image, TextureOptions::LINEAR)`. Draw with `ui.image` at 4:3 inside the window | Yes | 640 × 480 × 4 B ≈ 1.2 MB upload per frame at 30 fps; fine for wgpu/glow. `from_gray` expands to RGBA on the CPU. Measure it in Phase 6 (§8 budget); if it shows up, request repaint only on a new frame (`request_repaint` from the capture thread), never in a busy loop |
| Click to select target | `ui.image(...).interact(Sense::click())` → map the pointer position through the **same** rect used to draw (R-C2) → frame coords | Yes | Hover preview ring = the same painter call with a dashed stroke |
| Marker: thin neon-green ring | `Painter::circle_stroke(center, r, Stroke::new(2.0, Color32::from_rgb(0, 255, 0)))` | Yes | The lab also has an optional 1 px dark shadow under the ring. Not adopted. Revisit if Phase 6 recordings show low visibility |
| Marker variants (held dashed, searching growing ring, lost grey ring + X) | Dashed ring = `Shape::dashed_line` over a polyline approximating the circle (no dashed-circle primitive). Growing ring = radius/alpha from `ctx.input(|i| i.time)` + `request_repaint`. X = two `line_segment`s | Yes | Honour reduced motion: freeze the animation when the OS requests it (read via the platform crate; egui has no reduced-motion flag) |
| Labels on the preview ("Holding", "Searching", "Click to select") | `Painter::rect_filled` + `Painter::text` | Yes | — |
| Overlays on the preview (permission card, no camera, disconnected, loading spinner, welcome) | Draw in an `egui::Area` / child `Ui` placed over the preview rect, or `ui.put(rect, …)`. Spinner = `egui::Spinner` | Yes | The scrim is a `rect_filled` with alpha |
| Recenter countdown (white box 3-2-1) | `rect_filled` white + large `Painter::text`, driven by `app-core` `Recentering` state with a deadline (no UI timers) | Yes | Countdown time comes from `app-core`, not the widget, so it stays testable |
| Status line (icon + title + pointer owner + body + optional actions/hint) | Plain `ui.horizontal` with an icon, `RichText::strong` title, weak text; buttons only for states that have actions | Yes | AccessKit: give the line a live-region style label so screen readers announce changes (`Response::widget_info` / label text) |
| Icons | **Paint them** with `Painter` primitives (circles, rects, lines). ~15 simple shapes, all from the lab sprite. No SVG | Yes | SVG would need `egui_extras` + an SVG loader (new crate deps → ADR, cargo-deny). The lab's cog is the Feather path (MIT). Painting a simplified cog avoids both the dependency and the licence notice |
| Tabs | Row of `selectable_value` labels with an underline painted for the selected one; arrow keys handled explicitly | Yes | egui has no tab widget; ~40 lines |
| Sliders with value + default note | `egui::Slider` (`show_value(false)`, value drawn in the label row in monospace), help text as weak label | Yes | Slider range enforcement satisfies R-P2 "invalid values impossible" at the UI layer. `app-core` still validates (R-A5) |
| Toggle switch with ON/OFF text | Custom widget (egui demo `toggle_switch` pattern) + the ON/OFF label | Yes | Must set `WidgetInfo::selected` for AccessKit |
| Appearance segmented control | Two `selectable_value`s in a framed row | Yes | — |
| Screen limits (4 numeric fields) | `egui::DragValue` ×4 in a `Grid`, `range(0..=…)` | Yes | — |
| Camera selector | `egui::ComboBox` | Yes | — |
| Capture format readout | Monospace label in a frame | Yes | — |
| Diagnostics overlay | Monospace text in a translucent rect over the preview | Yes | Values come from `tracking-core` diagnostics (ZNCC, status, `redetect_armed()`) |
| Text scale | `Context::set_zoom_factor` | Yes | It scales everything incl. the fixed window contents. See "fixed size" row |
| Larger controls (48 px targets) | `Style::spacing.interact_size` / `button_padding` + thicker `visuals.widgets.*.bg_stroke` | Yes | Accessibility option, not a theme |
| F11 / F12 | In-window: `ctx.input(|i| i.key_pressed(Key::F11))`. **Global** (app unfocused, which is the normal case while the pointer drives other apps) needs an OS hotkey hook in `platform` | In-window yes; global = platform work | Global hotkeys are a Phase 6/7 `platform` adapter (legacy used `golang.design/x/hotkey`). Choosing a Rust crate needs an ADR. macOS F11 may be bound to "Show Desktop" by the OS |
| Focus ring / keyboard navigation | egui Tab focus traversal is built in. Focus stroke = `visuals.selection.stroke` / `widgets.*.fg_stroke`. Set it to the theme text colour, 2 px (4 px with larger controls) | Yes | Verify every custom widget (tabs, switch, segmented) takes focus. `egui_kittest` tests drive them by label |
| AccessKit | eframe 0.37 default features include `accesskit` | Yes | Every interactive widget needs a label (sliders get theirs from the label row via `labelled_by`) |
| Fonts | egui `default_fonts` (Ubuntu-Light + Hack) works offline. For the lab's system-UI look, embed one open font (e.g. Inter, OFL) with `Context::set_fonts` + `include_bytes!` | Yes | A font file is an asset, not a crate. Record the licence in the release licence bundle (Phase 7). Decide in Phase 6, default fonts are acceptable for a first build |
| Window title bar | Native decorations (keep), so traffic lights and close are OS-native | Yes | The lab's greyed zoom button is what macOS shows for `with_resizable(false)` |

## Token → egui mapping

| Token | Carbon | Paper | egui target |
|---|---|---|---|
| `--bg` | `#000000` | `#ececec` | `Visuals::panel_fill`, `window_fill` |
| `--surface` | `#0a0a0a` | `#f8f8f8` | `extreme_bg_color` (text edits), `faint_bg_color` |
| `--surface-2` | `#151515` | `#ffffff` | `widgets.inactive.weak_bg_fill` / `bg_fill` (buttons, fields) |
| `--line` | `#262626` | `#d6d6d6` | `widgets.noninteractive.bg_stroke` (separators) |
| `--line-strong` | `#3f3f3f` | `#a8a8a8` | `widgets.inactive.bg_stroke` (control borders) |
| `--text` | `#fafafa` | `#111111` | `override_text_color` / `widgets.*.fg_stroke`; slider fill; selected-tab underline; switch on |
| `--muted` | `#8f8f8f` | `#5a5a5a` | `weak_text_color` (help text, pointer owner) |
| marker green | `#00ff00` | `#00ff00` | Marker stroke (all marker states except lost, which is grey). **Nowhere else** |
| `--yellow` | `#ffd60a` | `#f5c518` | Primary button fill. **Nowhere else** |
| primary text on yellow | `#111111` | `#111111` | Primary button `RichText` colour |
| `--radius` / `--radius-sm` | 8 / 6 | 8 / 6 | `CornerRadius` for frames / widgets |
| `--border-w` | 1 (2 large) | 1 (2 large) | widget stroke width |
| `--focus-w` | 2 (4 large) | 2 (4 large) | `selection.stroke` width |
| `--font-size` | 14 (16 large) | 14 (16 large) | `TextStyle::Body` size |
| `--hit` | 36 (48 large) | 36 (48 large) | `spacing.interact_size.y` |
| `--gap` / `--pad` | 12 / 16 | 12 / 16 | `spacing.item_spacing` / `window_margin` |
| marker ring | 2 px (3 large), r ≈ 15 frame px | same | `Stroke::new(2.0, green)` |

Preview colour is an open Phase 6 decision. The ports deliver luma only (ADR-0008), so the cheap path is a
grayscale preview, where the green ring stands out on every skin tone. A colour preview needs the
camera adapter to pass a colour frame for display alongside the luma, which costs one more conversion
and upload per frame. The marker colour was chosen to work on both.

## Phase 6 acceptance criteria carried from Phase 5

1. The only hues are the green marker and the yellow primary buttons. Every state is readable from icon + text with colour removed
   (kittest assertion on the status text per `AppState`).
2. Every interactive control is reachable with Tab, shows a visible focus ring and has an
   AccessKit label. One `egui_kittest` test per control, driven by its label.
3. Both windows are fixed size and not resizable. Contents fit or scroll at zoom 1.0–1.3 and with
   larger controls.
4. Preview click → frame coordinate uses the same rect as drawing (R-C2), tested.
5. Countdowns (recenter, handover) are driven by `app-core` state with deadlines, not UI timers.
6. Theme choice persists in settings and is validated on load (R-A5).

## Inputs for Phase 6 (not decided here)

- R-A1 has no state for "pointer permission needed" or "mouse in control (handover)". The lab
  shows both. Add them to `AppState` or model them as sub-states.
- The global F11/F12 hotkey adapter and its crate (ADR).
- Font: egui default vs an embedded OFL font.
- Preview: grayscale (luma) vs colour (see token section).
- Wording is draft-quality. Review it with real users in Phase 8.
