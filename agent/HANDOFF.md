# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 6 ACTIVE, D0–D1 done)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Task `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. **D1 done** (gates green locally). Work rhythm (user): **one slice per session**,
then hand off.

**Done this session (D1):**
- `crates/pointer-core/src/smoothing.rs`: dt-aware EMA (`retain = smoothing^(dt/70 ms)`), FPS test.
- `crates/pointer-core/src/limits.rs`: px insets on all edges incl. desktop origin (E/S fix),
  offset drift ≈ 14.3 image px/s while pinned.
- `crates/pointer-core/src/dwell.rs`: fires once, re-arm on leave or refractory, ineligible resets.
- `map_raw` (unclamped) in `pointer-core`; `Controller::drive` composes
  map → smooth → limits → drift → dwell. `Effect::Click(Point2)`, `View::dwell_progress`.
- 7 new controller tests + 17 pointer-core unit tests.

**Provisional choices (in task 0008 Progress, re-check in D7):** dwell radius = fraction of the
desktop's shorter side; a `Held` frame resets the dwell.

**Next session:** D2 — ADR-0012 for the Phase 6 crates (re-query versions/licences: camera
backend candidate from D3, cursor crates, `global-hotkey`, `serde` + `toml` + `directories`;
state which crates `app-core` may take; keep `scripts/check-core-deps.sh` in step) and
settings persistence outside the pure cores (corrupt / newer-schema file → defaults + visible
notice, with tests).

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed F11/F12; recenter = white centre circle + 5 s
countdown badge; resume from pause goes through recenter; Recenter ends a pause.

**Decided:** preview is **colour**, not grayscale (user, 2026-11-12; ADR-0008 amendment) — D3 adapter delivers colour + luma.
**Still open:** font (default egui). D6 only.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only.

**Blockers:** none.
