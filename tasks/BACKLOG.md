# Task backlog

Single source of truth for task status — don't duplicate status elsewhere. Lifecycle:
`TRIAGE → READY → ACTIVE → REVIEW → DONE` (or `BLOCKED`). One coordinator updates this file
and moves task files between `active/`, `done/`, `blocked/` as status changes; a checklist is
not completion evidence, passing verification commands are.

## Active

| ID | Title | Status |
|---|---|---|
| 0008 | Phase 6 — desktop app | ACTIVE — `tasks/active/0008-phase-6-desktop-app.md`, slices D0–D7; D0–D2 done, next D3 (camera adapter spike, macOS) |

## Done

| ID | Title | Status |
|---|---|---|
| 0007 | Phase 5 — design system | DONE — lab v3 approved 2026-11-12 (Carbon/Paper, neon-green thin-ring marker, yellow primary buttons); `docs/design/EGUI_FEASIBILITY.md` |
| 0006 | Phase 4 — tracking improvements | DONE — E0–E4 (`f68f042`…`0074611`), E4 CI 43131916361 green, close-out `66768c9` CI 43132339284 green on 3 OS + cargo-deny; ADR-0010, -0011, `docs/research/TRACKING_IMPROVEMENTS.md` |
| 0005 | Phase 3 — tracking baseline | DONE — `4038fce`, CI run 43119858155 green on 3 OS + cargo-deny; ADR-0009, `docs/research/TRACKING_BASELINE.md` |
| 0004 | Phase 2 — Rust architecture | DONE — `ff0d1ab`, CI run 43116199528 green on macOS/Windows/Linux + cargo-deny; ADR-0007, -0008 |
| 0001 | Phase 0 — repository foundation | DONE — commit `a1e4f4e`, pushed to `origin/rewrite/rust` |
| 0003 | Phase 1 — research and behavior specification | DONE — `docs/research/BEHAVIOR_SPEC.md`, ADR-0006 |
| 0002 | Verify AI-tooling instruction/skill discovery in Claude Code and Codex | DONE — ADR-0005; `@AGENTS.md` import confirmed expanding in a fresh session (2026-11-11) |

## Blocked

(none yet)
