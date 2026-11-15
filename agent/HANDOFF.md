# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6 ACTIVE, D0–D3 done)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Task `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. **D3 done**: committed `7d39825`, pushed, CI run 43721527313 green (macOS, Windows,
Linux + cargo-deny). Work rhythm
(user): **one slice per session**, then hand off.

**Done this session (D3):**
- Camera backend picked by measurement: `nokhwa` 0.10.11 `input-native`, no `decoding`
  (ADR-0012 §"Camera backend": 640×480 YUYV, 30.09 fps from timestamps, close ~90 ms, re-open OK).
  `objc2-av-foundation` stays the fallback (needs `unsafe`).
- `app-core`: `color.rs` (`RgbFrame`, mirror/downsample identical to luma), `CameraFrame.rgb`,
  `Controller::preview_rgb()`. `GrayFrame`/`Point2`/`RgbFrame` re-exported from `ports`.
- `platform::camera`: `pixels` (YUYV/NV12/GRAY/RGB/BGR → luma + RGB), `latest` (latest-wins slot),
  `NokhwaCamera` (capture thread, panics → `Backend`, permission → `PermissionDenied`, 3 s frame
  timeout → `Disconnected`, 10 s open timeout), `list_devices()`. On-device test:
  `cargo test -p platform --test camera_device -- --ignored --nocapture`.
- CI Linux apt adds `libclang-dev` (bindgen in `v4l2-sys-mit`); nokhwa builds on all three OS in CI.

**Next session:** D4 — pointer adapter, macOS first: warp (`CGWarpMouseCursorPosition`) vs posted
move events measured on the dev Mac, incl. whether each needs Accessibility (P6) and warp's
local-event suppression vs handover; Accessibility check + System Settings deep link; `position()`;
virtual-desktop rect (R-P5); dwell click via posted events (P7); Windows `SendInput` + Linux X11
XTest compiled in CI; Wayland → `Unsupported`. Update ADR-0012's pointer rows; check licences.

**Carry to D7 manual QA:** camera unplug → `CameraDisconnected`; camera LED off after close
(nokhwa #224); permission denial in a `.app` (P2).

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed F11/F12; recenter = white centre circle + 5 s
countdown badge; resume from pause goes through recenter; Recenter ends a pause. Preview is colour.
**Still open:** font (default egui). D6 only. Save-on-change wiring: D6 (Options window).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only. No `gh` CLI
here: check CI with `curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
(public API), then `/actions/runs/<id>/jobs`.

**Blockers:** none.
