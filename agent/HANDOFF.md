# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6 ACTIVE, D0–D6 done)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Task `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. **D6 done, uncommitted** at the time of writing (check `git log`; the user is
asked whether to commit). Gates green locally: fmt, clippy `-D warnings`, `cargo test
--workspace` (desktop 53 tests), `check-core-deps.sh`. Work rhythm (user): **one slice per
session**, then hand off.

**Done this session (D6):** the desktop UI per the approved lab: Tracking + Options windows,
Carbon/Paper, colour preview texture, click-to-select through the drawing rect, all marker
variants, recenter ring + badge, status line per state, four Options tabs, save on change.
Details, test list, found-and-fixed bugs and **choices made without the lab** are in the task
file Progress entry for D6. Tell the user about those choices if they haven't seen them.

**Owed from D6:** a visual review by the user (`cargo run -p open-camera-mouse`: the Stopped
card, Options tabs, both themes, Larger controls, Text size). Only Stopped/Starting can be seen
until D7 wires the camera.

**Next session:** D7 — runtime integration and QA (CURRENT_STATE "Immediate next action").

**Carry to D7:** runtime order observe `position()` → controller → `move_to`; poll `access()`
every few s (never per frame); manual QA: F10/F12 pressed with another app focused (only
registration was verified), `Fn` on Mac laptops, dwell click on device, Accessibility grant for
a `.app`, camera unplug → `CameraDisconnected`, camera LED off after close (nokhwa #224),
camera permission denial in a `.app` (P2), camera-settings deep link for the permission card
(none in `platform` yet), reduced motion for the searching animation.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed (F11/F12; F10/F12 on macOS); recenter =
white centre circle + 5 s countdown badge; resume from pause goes through recenter; Recenter ends
a pause. Preview is colour. **Still open:** font (default egui; it lacks `→`).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only. No `gh` CLI
here: check CI with `curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
(public API), then `/actions/runs/<id>/jobs`. Screen Recording is granted to the terminal (2026-11-15):
`screencapture -x -R x,y,w,h` with bounds from `osascript` (System Events) works.

**Blockers:** none.
