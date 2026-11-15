# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6 ACTIVE, D0–D4 done)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Task `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. **D4 done in the working tree, NOT committed yet** (waiting for the user's yes).
After committing, push and check CI: the Linux pointer module was only clippy-checked in a
scratch crate, because `platform` can't cross-build for Linux on the Mac (nokhwa bindgen).
Work rhythm (user): **one slice per session**, then hand off.

**Done this session (D4):**
- Measured on the dev Mac (two displays, second at (1440, −540); Accessibility granted to the
  terminal mid-session): warp needs no permission but kills the physical mouse at 30 Hz →
  rejected. Posted events + source suppression 0 → physical moves show through → adopted.
  `CGPreflightPostEventAccess` ≈10.6 ms/call. Full table: ADR-0012 §"Pointer adapter".
- `PointerSink` gains `access()`, `click()`, `desktop()`. `platform::pointer`: `geometry`
  (bounding box, snap, `Echo`), `SystemPointer` for macOS / Windows / Linux X11; Wayland →
  `Unsupported`. No new crates in `Cargo.lock`. `unsafe` only for FFI (ADR-0007), commented.
- On-device: `cargo test -p platform --test pointer_device -- --ignored --nocapture --test-threads=1`.

**Next session:** D5 — global hotkeys F11/F12 via `global-hotkey` (0.10.0 candidate) in
`platform`, mapped to `ToggleStart`/`Recenter`; registration failure surfaced (P13: macOS F11 =
Show Desktop); in-window F11/F12 always work. Update ADR-0012 hotkey row, check licences.

**Carry to D7:** runtime order observe `position()` → controller → `move_to`; poll `access()`
every few s (never per frame); manual QA: dwell click on device, Accessibility grant for a
`.app`, camera unplug → `CameraDisconnected`, camera LED off after close (nokhwa #224),
camera permission denial in a `.app` (P2).

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
