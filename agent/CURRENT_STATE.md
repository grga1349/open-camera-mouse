# Current state

Last updated: 2026-11-15 (Phase 6 ACTIVE, D0–D6 done)

## Branch / commit

Branch: `rewrite/rust`. Last pushed code commit `9f04007` (Phase 6 D5), CI run 43725986748
green. **D6 is uncommitted in the working tree** (gates green locally) unless a later commit
says otherwise: check `git log`.

## Milestone

Phases 0–5 DONE. Phase 6 (desktop app) is ACTIVE: `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. D0 (app-core controller), D1 (pointer-core behaviour), D2 (ADR-0012 + settings
persistence), D3 (camera adapter), D4 (pointer adapter), D5 (global hotkeys) and D6 (desktop
UI, headless-tested) done. D7 (runtime integration and QA) next.

## What exists now

- Workspace per ADR-0007; mirror-at-ingest + 640×480@30 per ADR-0008.
- `tracking-core`: recommended tracker `LkParams::recommended()` (ADR-0011), B-LK18 default, B-NCC
  comparator. Statuses `Tracking`, `Held(…)`, `Lost(… | Searching | NoSelection)`. Re-detection is
  a user option "Find the target again", off by default (ADR-0011 amendment 2026-11-12).
- Design (task 0007): approved `docs/design/design-lab.html` v3 — monochrome, Carbon (dark, default) +
  Paper (light) themes switchable in Options, neon green `#00ff00` thin circle-ring marker, yellow primary
  buttons, two fixed-size windows (Tracking 560×620, Options 400×620), status as an icon + text line.
  `docs/design/EGUI_FEASIBILITY.md` = per-affordance egui 0.37 plan, token mapping, Phase 6 criteria.
- Auto-recenter dropped from the initial rewrite (ADR-0006 row amended 2026-11-12).
- `app-core`: full controller (R-A1 + `Error`, `PointerOwner` sub-state, injected clock,
  `Step`/`View`, validated `Settings` with schema version). `pointer-core`: `map_raw`/`map_absolute`,
  `smoothing` (dt-aware), `limits` (+ offset drift as a rate), `dwell`, `handover` (move-stop,
  inactivity, R-P4 jump bound). Composed in `Controller::drive`; `Effect::Click` for dwell.
- Settings persistence (ADR-0012): `serde` derives on `app_core::Settings` (+ `repaired()`),
  `platform::settings_store` (TOML, `schema_version`, atomic write, never-failing load with a
  `SettingsNotice`; newer schema → read-only store). Config dir hand-written (`directories`
  rejected: MPL-2.0 `option-ext`). `check-core-deps.sh` also checks `app-core`.
- Camera (D3, ADR-0012): `platform::camera::NokhwaCamera` (`nokhwa` 0.10.11 `input-native`, one
  adapter for all OS), capture thread + latest-frame slot, luma + RGB per frame, failures mapped
  per PLATFORM_RISKS P1/P2/P3/P5. Measured on the dev Mac: 640×480 @ 30.09 fps. `app-core`
  carries the colour image through ingest (`RgbFrame`, `Controller::preview_rgb()`).
- Pointer (D4, ADR-0012 §"Pointer adapter"): `platform::pointer::SystemPointer` per OS behind
  the extended `PointerSink` (`access`, `move_to`, `click`, `position`, `desktop`). macOS =
  posted CGEvents from a source with suppression 0 (needs Accessibility; warp rejected because
  it blocks the physical mouse, measured). Windows `SetCursorPos`/`SendInput`, Linux X11 XTest
  (`x11rb` 0.13.2), Wayland → `Unsupported`. Shared pure `geometry`: bounding-box desktop,
  snap to nearest display, `Echo` so the adapter's own snapped/late moves don't look like
  handover. On-device test `cargo test -p platform --test pointer_device -- --ignored`.
- Hotkeys (D5, ADR-0012 §"Global hotkeys"): `platform::hotkeys` over `global-hotkey` 0.10.0.
  Start/pause **F10 on macOS** (macOS refuses F11: Show Desktop, measured; user decision), F11
  on Windows/Linux; F12 recenter. Per-key `Registration` shown in the window; in-window keys
  always work; `KeyRouter` dedups global + window presses (300 ms). Wayland / no `DISPLAY` →
  `Unsupported`. `GlobalHotkeys::register` must run in the eframe creator closure (main thread).
- Desktop UI (D6): `apps/desktop` = `app` (Shell: controller + clock, debounced save-on-change),
  `status` (pure View → status line / card), `preview` (one drawing rect for paint + click,
  markers, overlays), `options`, `widgets`, `icons` (painted), `theme` (Carbon/Paper; the only
  hue helpers are `theme::marker_green()` and `theme::primary()`). Two fixed windows, Options via
  `show_viewport_immediate`. 33 kittest tests drive every control by label. The runtime is
  missing: controller `Effect`s are dropped, so a manual run stops at "Starting camera…".

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Task 0008 slice D7: runtime integration and QA (task file §Objective D7). Camera thread →
`Input::Frame` → controller → execute `Effect`s (`OpenCamera`/`CloseCamera` on `NokhwaCamera`,
`MovePointer`/`Click` on `SystemPointer`), observe `position()` before each step, poll
`access()` every few s, `desktop()` about once a second, `request_repaint` on a new frame.
Hook point: `Shell::input` in `apps/desktop/src/app.rs` (it drops the `Step` today). Then the
fake-camera/fake-pointer end-to-end test, frame recorder, `docs/qa/MANUAL_QA.md`, `.app` bundle
for permission QA. First ask the user for the D6 visual review (see HANDOFF).

## Open risks

- Synthetic-only tracker evidence; re-measure all thresholds on Phase 6 recordings.
- Re-acquisition → control handover: implemented in D0 (R-P4 bound 0.05 × shorter desktop side, provisional).
- `nokhwa` macOS bindings pull `block` 0.1.6 (future-incompat warning); MJPEG-only cameras unsupported (ADR-0012).
- Re-detection self-test rarely arms on smooth texture (task 0008 Progress); re-measure in D7.
- Hotkeys: registration verified on the dev Mac, but a press from a background app was not
  observed yet (D7 manual QA); `Fn` needed on Mac laptop keyboards; Windows/X11 on device Phase 7.
- Camera crate panics on external webcams (P1): caught, but untested on device until Phase 7; Wayland pointer and window placement (P10/P11).
- `CGPreflightPostEventAccess` ≈10 ms: never call per frame (D7 runtime polls it every few s).
- The desktop is the displays' bounding box: with uneven monitors the cursor sticks to display
  edges where the box has dead zones (snap). Fine for v1; revisit if users find it odd.
- D6 UI: only the Tracking window (Stopped) was screenshot-reviewed. Searching animation ignores
  reduced motion. Default font lacks `→` (wording uses `>`); font decision still open.
