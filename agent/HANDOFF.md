# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 6 ACTIVE, D0–D2 done)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Task `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. **D2 done** (gates green locally). Work rhythm (user): **one slice per session**,
then hand off.

**Done this session (D2):**
- ADR-0012 (`docs/architecture/decisions/0012-phase-6-dependencies.md`): crates re-queried
  2026-11-12; `app-core` may take `serde` derive (ADR-0007 amended); candidate table for D3–D5.
- `crates/app-core/src/settings.rs`: serde derives (`#[serde(default)]`, theme lowercase,
  `camera` omitted when `None`), `Settings::repaired()` (per-field reset).
- `crates/platform/src/settings_store.rs`: `SettingsStore` (`at`, `in_user_config_dir`, `load`,
  `save`), `SettingsNotice`, fallbacks table in the module doc. Hand-written config dir
  (`directories` rejected: `option-ext` is MPL-2.0 → would fail `cargo deny`).
- `scripts/check-core-deps.sh` checks `app-core` too. Desktop skeleton loads settings + shows notice.

**Next session:** D3 — camera adapter spike on the dev Mac (`nokhwa` `input-native` vs thin
`objc2-av-foundation`), measure open / 640×480 / real FPS from timestamps / release on close
(PLATFORM_RISKS P1), adapter delivers luma + colour, panics caught → `CameraError::Backend`,
bounded latest-frame channel, disconnect/permission errors. Record the pick in ADR-0012's table
and check every new transitive licence against `deny.toml` (cargo-deny is CI-only locally).

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed F11/F12; recenter = white centre circle + 5 s
countdown badge; resume from pause goes through recenter; Recenter ends a pause. Preview is colour.
**Still open:** font (default egui). D6 only. Save-on-change wiring: D6 (Options window).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only.

**Blockers:** none.
