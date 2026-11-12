# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 6 ACTIVE, D0 done)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Task `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. **D0 done and committed** (`918dcca`, not pushed), gates green locally.
Work rhythm (user): **one slice per session**, then hand off.

**Done this session (D0):**
- `crates/app-core/src/controller.rs`: full R-A1 state machine + `Error`; `PointerOwner`
  sub-state (decision written into BEHAVIOR_SPEC R-A1); `handle(Input, now_us) -> Step
  {events, effects}`, `view(now_us) -> View`, `update_settings` (R-A5/R-A6). Owns
  `LkTracker` (recommended, `redetect = None` unless `loss_recovery`), mirror + downsample at ingest.
- `crates/app-core/src/settings.rs`: validated/normalised settings, `SCHEMA_VERSION = 1`.
- `crates/pointer-core/src/handover.rs`: move-stop, inactivity timer, R-P4 jump bound
  (pulled forward from D1).
- `crates/app-core/tests/controller.rs`: 26 headless tests. Desktop skeleton adapted to the new API.

**Two findings for the user (in task 0008 Progress):**
1. Re-detection rarely arms on smooth texture (self-test margin). Open risk; re-measure in D7.
2. Smoothing polarity: **decided** (user) — "Smoothing", higher = steadier, default 0.05
   (`= 1 − 2018 damping`), range 0–0.95. Lab updated to match.

**Next session:** `git log -1` should show the D0 handoff commit. Then D1 (`pointer-core`): dt-aware EMA smoothing (α 0.95 per 70 ms equivalent,
FPS-independence test), W/E/N/S limits with E/S origin fix, offset drift as a rate, dwell
(fires once, never while lost / not in control). Then compose them in `Controller::map` /
`track_step` (currently plain `map_absolute`). Handover + jump bound already exist.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed F11/F12; recenter = white centre circle + 5 s
countdown badge; resume from pause goes through recenter; Recenter ends a pause.

**Still open:** grayscale vs colour preview (default grayscale), font (default egui). D6 only.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only.

**Blockers:** none.
