# Current state

Last updated: 2026-11-12 (Phase 6 ACTIVE, D0–D1 done)

## Branch / commit

Branch: `rewrite/rust`. Phase 5 committed and pushed (`3578620` design system + a follow-up commit
for the guided recenter flow, 5 s countdown). Docs-only; code unchanged since Phase 4 (`66768c9`, CI green).

## Milestone

Phases 0–5 DONE. Phase 6 (desktop app) is ACTIVE: `tasks/active/0008-phase-6-desktop-app.md`,
slices D0–D7. D0 (app-core controller) and D1 (pointer-core behaviour) done, gates green locally; D2 next.

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
- `apps/desktop` is still the Phase 2 skeleton (one window, status label, start/stop button),
  adapted to the new controller API.

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Task 0008 slice D2: dependencies ADR-0012 (re-query versions/licences) + settings persistence
(serde/toml/directories, outside the pure cores; corrupt/newer-schema → defaults + notice).

## Open risks

- Synthetic-only tracker evidence; re-measure all thresholds on Phase 6 recordings.
- Re-acquisition → control handover: implemented in D0 (R-P4 bound 0.05 × shorter desktop side, provisional).
- Preview is **colour** (user decision 2026-11-12): adapter must deliver a colour frame next to luma (D3), mirrored at ingest with it (ADR-0008 amendment).
- Re-detection self-test rarely arms on smooth texture (task 0008 Progress); re-measure in D7.
- Global F11/F12 hotkeys need a `platform` adapter + crate ADR.
- Camera crate macOS panics (PLATFORM_RISKS P1); Wayland pointer and window placement (P10/P11).
