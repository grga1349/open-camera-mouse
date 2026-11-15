# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6 ACTIVE, D0–D5 done)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Task `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. **D5 done**: committed `9f04007`, pushed, CI run 43725986748 green (macOS,
Windows, Linux + cargo-deny): the first Linux build of the hotkey code. Work rhythm (user): **one
slice per session**, then hand off.

**Done this session (D5):**
- `global-hotkey` 0.10.0 adopted (ADR-0012 §"Global hotkeys"; 3 new lockfile crates, permissive).
- Measured on the dev Mac: F11 registration **refused** (Show Desktop, P13 VERIFIED); F10, F12 OK.
- **User decision:** start/pause = **F10 on macOS**, F11 on Windows/Linux; F12 recenter
  everywhere; still fixed. ADR-0006 hotkey row + BEHAVIOR_SPEC R-A4 amended.
- `platform::hotkeys`: `Hotkey` (per-OS key, `from_key_name`), `Registration`, `GlobalHotkeys`
  (create in the eframe creator closure), `KeyRouter` (300 ms cross-source dedup). Desktop
  skeleton shows one line per key.

**Next session:** D6 — desktop UI per the approved lab (see CURRENT_STATE "Immediate next
action"). Use `Hotkey::key_name()` for every key label (F10 vs F11 differs per OS).

**Carry to D7:** runtime order observe `position()` → controller → `move_to`; poll `access()`
every few s (never per frame); manual QA: F10/F12 pressed with another app focused (only
registration was verified), `Fn` on Mac laptops, dwell click on device, Accessibility grant for
a `.app`, camera unplug → `CameraDisconnected`, camera LED off after close (nokhwa #224),
camera permission denial in a `.app` (P2).

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed (F11/F12; F10/F12 on macOS); recenter =
white centre circle + 5 s countdown badge; resume from pause goes through recenter; Recenter ends
a pause. Preview is colour. **Still open:** font (default egui). D6 only. Save-on-change wiring: D6.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only. No `gh` CLI
here: check CI with `curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
(public API), then `/actions/runs/<id>/jobs`.

**Blockers:** none.
