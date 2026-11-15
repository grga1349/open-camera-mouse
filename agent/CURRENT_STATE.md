# Current state

Last updated: 2026-11-15 (Phase 6 DONE, Phase 6b test drive NEXT)

## Branch / commit

Branch: `rewrite/rust`. Last code commit `7e9a2bb` (Phase 6 D7 part 1), CI run 43746097717
green on 3 OS + cargo-deny. Phase 6 close-out `2146984` pushed; the Phase 6b ADR + handoff may
be uncommitted: see `git status`.

## Milestone

Phases 0–6 DONE. Phase 6 (desktop app): `tasks/done/0008-phase-6-desktop-app.md`,
slices D0–D7. D0 (app-core controller), D1 (pointer-core behaviour), D2 (ADR-0012 + settings
persistence), D3 (camera adapter), D4 (pointer adapter), D5 (global hotkeys) and D6 (desktop
UI, headless-tested) and D6b (widgets restyled to the lab, screenshot-compared) done.
D7 done: runtime, end-to-end headless tests, recorder, and manual QA on the dev Mac with the
user (`docs/qa/MANUAL_QA.md` rows 1–18 pass, 2026-11-15). **Phase 6b (test drive, ADR-0013) is
NEXT: the user uses the app and we fix what comes up, before Phase 7. No task file yet.**

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
- Runtime (D7): `apps/desktop/src/runtime.rs` — `ThreadedCamera` worker (factory-made camera,
  session numbers, newest-frame coalescing) + `Runtime` on the main thread, pumped from
  `eframe::App::logic` (runs while minimised, ~10 Hz then). Access polled every 3 s, desktop
  every 1 s, cursor observed before each frame. `recorder.rs` (`OCM_RECORD=<dir>`). Fixed an
  `app-core` ordering bug (observation vs due takeover). 14 runtime tests with fakes.

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Start Phase 6b (test drive, ADR-0013) per `docs/ROADMAP.md`: write `tasks/active/0009-phase-6b-test-drive.md`
from its deliverables and gate, create `docs/qa/TEST_DRIVE.md` (issue log), then take the
user's reports one by one: reproduce, fix with a regression test or defer to a named phase.
Phase 7 follows when the user says the app is ready. Carry-overs unchanged: MANUAL_QA rows 19–20
in Phase 7; row 21 (reduced motion) and `OCM_RECORD` threshold re-measurement in Phase 8.
Font stays egui default (user: later).

## Open risks

- Synthetic-only tracker evidence; re-measure all thresholds on `OCM_RECORD` recordings (Phase 8).
- Re-acquisition → control handover: implemented in D0 (R-P4 bound 0.05 × shorter desktop side, provisional).
- `nokhwa` macOS bindings pull `block` 0.1.6 (future-incompat warning); MJPEG-only cameras unsupported (ADR-0012).
- Re-detection self-test rarely arms on smooth texture (task 0008 Progress); re-measure in D7.
- Hotkeys: F10/F12 from a background app verified on the dev Mac (QA 2026-11-15); whether `Fn`
  is needed on a Mac laptop keyboard not noted; Windows/X11 on device Phase 7.
- Camera crate panics on external webcams (P1): caught, but untested on device until Phase 7; Wayland pointer and window placement (P10/P11).
- `CGPreflightPostEventAccess` ≈10 ms: never call per frame (runtime polls it every 3 s).
- Minimised window: eframe throttles `logic` to 100 ms → pointer at ~10 Hz; judged usable in
  manual QA (row 13). Revisit in Phase 8 latency budgets if needed.
- The desktop is the displays' bounding box: with uneven monitors the cursor sticks to display
  edges where the box has dead zones (snap). Fine for v1; revisit if users find it odd.
- UI: D6b screenshot-checked Options (all tabs, Paper, Larger controls) and Tracking (Stopped)
  only; running states passed manual QA row 18. Searching animation ignores reduced motion. Font =
  egui default, no bold (faux-bold buttons); revisit later (Inter candidate). Debug-only
  `OCM_DEV_SETTINGS` / `OCM_DEV_OPTIONS` env vars exist for screenshot checks.
