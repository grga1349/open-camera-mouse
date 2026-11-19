# Current state

Last updated: 2026-11-19 (Phase 7 ACTIVE — task 0012 written; slice 1 = CI #7 failure annotations)

## Branch / commit

Branch: `rewrite/rust`. Last code: `687a9fc`, CI 43797400148 green on 3 OS. Docs-only close
`461d381` (run 43797804357) failed Windows `test (no webcam)` exit 1 = flake #7 again.
Task 0012 + CI failure-annotation step (slice 1) committed and pushed 2026-11-19; first CI run with the new step not yet checked.

## Milestone

Phases 0–6, 6b, 6c, 6d DONE. **Phase 7 ACTIVE: `tasks/active/0012-phase-7-multiplatform-release.md`**
(8 slices; user decisions 2026-11-19: macOS arm64 only, ad-hoc dmg, notarization hook off;
Windows unsigned + installer; Linux tar.gz + AppImage, X11; GitHub Releases; app icon = user's
yellow camera/CM image, sized per OS in slice 2).
Pointer defaults: gain 20/20, smoothing 0.20, steady zone automatic 0.2 cam px with the
drift capped at the limits, Main monitor = OS primary (`MainMonitor::Primary`).

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
  `show_viewport_immediate`. 33 kittest tests drive every control by label. D6b: widgets
  painted to the lab CSS (`widgets::Button` with key chips, switch, slider, tabs, segmented,
  readout); `theme::primary_colors` is the only yellow. The runtime is
  missing: controller `Effect`s are dropped, so a manual run stops at "Starting camera…".
- Engine (ADR-0016, slice 2a; replaced D7's `runtime.rs`): `apps/desktop/src/engine.rs` —
  `ThreadedCamera` worker (newest-wins frame slot, session numbers) → `Engine` (controller,
  pointer, recorder, perf; sync `apply`/`step`/`snapshot`) run by `EngineThread` (wakes on
  frame/command/100 ms). Frames handled at capture time (clamped monotonic). UI = `Shell` over
  `dyn EngineHandle`: sends `EngineCmd`, paints `Arc<Snapshot>` (view, displays, preview
  `Arc<ColorImage>`), keeps its own validated settings and saves them. UI tests use
  `InlineEngine`. `recorder.rs` (`OCM_RECORD=<dir>`) runs on the engine thread.
- Appearance gate anchored update (ADR-0018, slice 2d): `LkParams::anchored`, on in
  `recommended()`: gate vs the window accepted 10 steps earlier (≥ 0.7) + selection floor 0.3.
  `turn_extreme` 90/90; slow-occluder scenes `occluder_slide`/`occluder_creep` pinned.
- Pointer tuning (6c): `Settings` has no `steady_px`/`text_scale` any more (old files load,
  unknown fields ignored). `app_core::STEADY_ZONE_CAM_PX` = 0.2 (6d #13; was 0.5); offset drift capped at the head's overshoot; `Controller::set_steady_zone_cam_px`
  for tests and the debug-only `OCM_DEV_STEADY=<cam px>` (engine.rs). Options tabs: Pointer ·
  Clicking · Camera · Screen · General. Rest test `the_cursor_does_not_move_at_rest_*` covers
  gains 1–60 in four parallel parts (synthetic, *not yet calibrated*).
- Dev profile: deps opt 2, `app-core`/`pointer-core`/`platform` opt 3, `tracking-core` 3
  (root `Cargo.toml`). Benches: `app-core --bench frame`, `platform --bench decode`.

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Slice 1 of task 0012: after the CI step is pushed, wait for a `test (no webcam)` failure,
read `https://api.github.com/repos/grga1349/open-camera-mouse/check-runs/<job id>/annotations`
for the failing test, make it deterministic. Otherwise slice 2 (packaging ADR + icon).

## Open risks

- #12 closed (user: no jumps). Near a limit the zone holds less toward it (creep, no shake).
- #14: macOS checked on device; Linux RandR primary and the Windows origin rule are compile/CI-checked only (Phase 7 on device).
- #7: CI `test (no webcam)` flaky on macOS (exit 101) and Windows (exit 1, run 43779616732).

- Synthetic pointer tests passed while the device still jittered: not calibrated (ADR-0020).

- ADR-0018 limit: a change slower than the gate allows in 10 frames is followed down to the
  selection floor (0.3); a surface correlating ≥ 0.3 with the selection could be followed
  indefinitely (very slow hand creep). Re-measure on Phase 8 recordings.

- Synthetic-only tracker evidence; re-measure all thresholds on `OCM_RECORD` recordings (Phase 8).
- Re-acquisition → control handover: implemented in D0 (R-P4 bound 0.05 × shorter desktop side, provisional).
- `nokhwa` macOS bindings pull `block` 0.1.6 (future-incompat warning); MJPEG-only cameras unsupported (ADR-0012).
- Re-detection self-test rarely arms on smooth texture (task 0008 Progress); re-measure in D7.
- Hotkeys: F10/F12 from a background app verified on the dev Mac (QA 2026-11-15); whether `Fn`
  is needed on a Mac laptop keyboard not noted; Windows/X11 on device Phase 7.
- Camera crate panics on external webcams (P1): caught, but untested on device until Phase 7; Wayland pointer and window placement (P10/P11).
- `CGPreflightPostEventAccess` ≈10 ms: never call per frame (runtime polls it every 3 s).
- Pointer now moves from the engine thread (CGEvent posting, `CGGetActiveDisplayList` off the
  main thread): documented thread-safe, `pointer_device` test already runs off-main; confirm
  in the 2a device check (pointer, handover, dwell). Minimised: full rate now (was ~10 Hz).
- The desktop is the displays' bounding box: with uneven monitors the cursor sticks to display
  edges where the box has dead zones (snap). Fine for v1; revisit if users find it odd.
- UI: D6b screenshot-checked Options (all tabs, Paper, Larger controls) and Tracking (Stopped)
  only; running states passed manual QA row 18. Searching animation ignores reduced motion. Font =
  egui default, no bold (faux-bold buttons); revisit later (Inter candidate). Debug-only
  `OCM_DEV_SETTINGS` / `OCM_DEV_OPTIONS` env vars exist for screenshot checks.
