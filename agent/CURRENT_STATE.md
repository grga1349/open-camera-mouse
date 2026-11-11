# Current state

Last updated: 2026-11-11 (task 0002 session)

## Branch / commit

Branch: `rewrite/rust` (created from `main`, which is untouched and still builds the shipping
v0.3.0 Go app). Phase 0 is committed as `a1e4f4e` and pushed to `origin/rewrite/rust`.

## Milestone

Phase 0 (foundation) is DONE — `tasks/done/0001-phase-0-foundation.md`. Task 0002 (AI-tooling
discovery) is DONE — `tasks/done/0002-...md`, ADR-0005. Next up: task 0003 (Phase 1).

Task 0002 changes (committed, not pushed): `CLAUDE.md` now imports `@AGENTS.md`; AGENTS.md skills and
compatibility sections updated; ADR-0005 added; ADR-0002 marked amended.

## What exists now that didn't before this session

- `legacy/` — the entire old Go/Wails/GoCV app, moved via `git mv` (history preserved).
- `AGENTS.md`, `CLAUDE.md` (plain pointer file, see ADR-0002) at repo root.
- `docs/product/PRODUCT.md`, `docs/research/RE_EVIDENCE.md`,
  `docs/architecture/decisions/0000`–`0004`, `docs/design/DESIGN_PLAN.md`.
- `tasks/BACKLOG.md` + `tasks/active/0001`-`0003` (+ empty `done/`, `blocked/`).
- `agent/CURRENT_STATE.md` (this file), `agent/HANDOFF.md`, `agent/WORKFLOW.md`.
- `.agents/skills/` — 10 skill stubs (see task 0002 for discovery-verification status).
- New root `README.md`, extended `.gitignore`.
- `docs/archive/rust-rewrite-seed/` (ex-`dump/`) + `SEED_CONSUMPTION_REPORT.md`.

## What does NOT exist yet (by design)

No `Cargo.toml`, no crates, no tracking algorithm code, no egui app, no `design-lab.html`.
Phase 0 explicitly excludes all of this.

## Immediate next action

Start task 0003 (Phase 1 research spec) in a fresh Claude Code session — which also confirms
the `@AGENTS.md` import expands (AGENTS.md content should appear in the startup reminder).

## Open risks not yet resolved

- `@AGENTS.md` import in `CLAUDE.md` not yet observed expanding in a fresh session.
- Claude Code does not auto-load `.agents/skills/` (Codex does); relies on the AGENTS.md
  table telling Claude to read the SKILL.md on demand (ADR-0005).
- No Windows checkout has been tested.
- Crate boundaries (ADR-0003) and tracking-algorithm choice (ADR-0004) are proposals only.
