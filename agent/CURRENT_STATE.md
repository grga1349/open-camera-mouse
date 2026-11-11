# Current state

Last updated: 2026-11-11 (Phase 0 session)

## Branch / commit

Branch: `rewrite/rust` (created from `main`, which is untouched and still builds the shipping
v0.3.0 Go app). **Nothing in this session has been committed yet** — all moves/new files are
in the working tree / git index, pending explicit user review and commit approval.

## Milestone

Phase 0 (foundation) — see `docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md` for all
nine phases. Phase 0 deliverables are essentially complete as of this note; not yet verified
by a fresh CLI session (task 0002) or committed.

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

Run `git status --short` and `git diff --stat main...rewrite/rust`, confirm nothing
unexpected, then it's the user's call whether to commit this Phase 0 work. After that: task
0002 (verify AI-tooling discovery in a fresh session) and task 0003 (Phase 1 research spec).

## Open risks not yet resolved

- `.agents/skills/` discovery in either CLI is unverified (task 0002).
- Codex's actual project-local skill convention is unconfirmed from official docs this
  session — don't assume a path for it without checking current docs first.
- Crate boundaries (ADR-0003) and tracking-algorithm choice (ADR-0004) are proposals only.
