# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6 ACTIVE, D0–D6b done, D7 next)

**Branch:** `rewrite/rust`. Task `tasks/active/0008-phase-6-desktop-app.md`. **D6b done** in the
working tree (uncommitted until the user says so): widgets restyled to the lab's CSS and
checked by screenshots against headless-Chrome lab captures. Details in the task file Progress
entry for D6b. Gates green locally: fmt, clippy `-D warnings`, workspace tests (desktop 53),
`check-core-deps.sh`. Work rhythm (user): **one slice per session**, then hand off.

**Font:** user decided 2026-11-15 to leave it for later (egui default; buttons faux-bold).
Candidate when revisited: Inter Regular + SemiBold embedded (OFL).

**Next session:** D7 (runtime integration and QA). Hook point `Shell::input` in
`apps/desktop/src/app.rs`.

**Screenshots without clicks (debug builds):** `OCM_DEV_SETTINGS=<file>` (use a copy, not the
real settings) and `OCM_DEV_OPTIONS=pointer|clicking|camera|general`; bounds via `osascript`
System Events for the process's windows, then `screencapture -x -R x,y,w,h`. Lab:
`Google Chrome --headless=new --screenshot … design-lab.html#state=first-run&tab=…&theme=…`.
A real click needs a posted CGEvent; ask first, it moves the user's cursor.

**Carry to D7:** runtime order observe `position()` → controller → `move_to`; poll `access()`
every few s (never per frame); manual QA: F10/F12 pressed with another app focused (only
registration was verified), `Fn` on Mac laptops, dwell click on device, Accessibility grant for
a `.app`, camera unplug → `CameraDisconnected`, camera LED off after close (nokhwa #224),
camera permission denial in a `.app` (P2), camera-settings deep link for the permission card
(none in `platform` yet), reduced motion for the searching animation, look at running states
on screen (only Stopped + Options were screenshot-checked).

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed (F11/F12; F10/F12 on macOS); recenter =
white centre circle + 5 s countdown badge; resume from pause goes through recenter; Recenter ends
a pause. Preview is colour. Font: later.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only. No `gh` CLI
here: check CI with `curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
(public API), then `/actions/runs/<id>/jobs`.

**Blockers:** none.
