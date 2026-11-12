# Current state

Last updated: 2026-11-12 (Phase 5 DONE; Phase 6 next)

## Branch / commit

Branch: `rewrite/rust`. Phase 4 close-out `66768c9` (CI green). Phase 5 work is docs-only and may
be uncommitted — check `git status`.

## Milestone

Phases 0–5 DONE. Phase 6 (desktop app) is NEXT per `docs/ROADMAP.md`. Its task file is not
written yet. The coordinator writes it from the roadmap when the phase starts.

## What exists now

- Workspace per ADR-0007; mirror-at-ingest + 640×480@30 per ADR-0008.
- `tracking-core`: recommended tracker `LkParams::recommended()` (ADR-0011), B-LK18 default, B-NCC
  comparator. Statuses `Tracking`, `Held(…)`, `Lost(… | Searching | NoSelection)`.
- Design (task 0007): approved `docs/design/design-lab.html` v3 — monochrome, Carbon (dark, default) +
  Paper (light) themes switchable in Options, neon green `#00ff00` thin circle-ring marker, yellow primary
  buttons, two fixed-size windows (Tracking 560×620, Options 400×620), status as an icon + text line.
  `docs/design/EGUI_FEASIBILITY.md` = per-affordance egui 0.37 plan, token mapping, Phase 6 criteria.
- Auto-recenter dropped from the initial rewrite (ADR-0006 row amended 2026-11-12).
- `apps/desktop` is still the Phase 2 skeleton (one window, status label, start/stop button).

## Toolchain

`export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

## Immediate next action

Start Phase 6 (desktop app): write its task file from ROADMAP Phase 6 deliverables/gate, using
`EGUI_FEASIBILITY.md` §"Phase 6 acceptance criteria" and §"Inputs for Phase 6".

## Open risks

- Synthetic-only tracker evidence; re-measure all thresholds on Phase 6 recordings.
- Re-acquisition → control handover is undecided (Phase 6).
- Preview grayscale vs colour is undecided (EGUI_FEASIBILITY); the marker colour works on both.
- R-A1 lacks pointer-permission and mouse-handover states (needed by the approved UI).
- Global F11/F12 hotkeys need a `platform` adapter + crate ADR.
- Camera crate macOS panics (PLATFORM_RISKS P1); Wayland pointer and window placement (P10/P11).
