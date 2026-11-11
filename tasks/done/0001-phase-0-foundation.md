# 0001 — Phase 0: repository foundation

Status: DONE
Owner/branch: `rewrite/rust`
Depends on: none
Commit: `a1e4f4e` — "Phase 0: foundation for Rust rewrite on rewrite/rust", pushed to
`origin/rewrite/rust`

## Context

Kickoff of the Rust rewrite (`docs/archive/rust-rewrite-seed/initial-promt.md`). Phase 0 is
foundation only — inventory, safe migration, instruction/skill/task scaffolding — explicitly
**not** tracking-engine implementation.

## Objective

Produce: current-state inventory; a safe `legacy/` migration with no data loss; canonical
`AGENTS.md`/`CLAUDE.md` instructions; a `docs/` skeleton; a `tasks/` system; `agent/` durable
handoff files; a project skill catalog; an updated root `README.md`; an archived, mapped
planning seed; and a verified final git state.

## Non-goals

No `Cargo.toml`, no crates, no tracking algorithm code, no egui app. No deletion of `legacy/`
or the seed archive.

## Files/scope

Root-level restructuring only: `legacy/`, `docs/`, `tasks/`, `agent/`, `.agents/skills/`,
`AGENTS.md`, `CLAUDE.md`, `README.md`, `.gitignore`, `docs/archive/rust-rewrite-seed/`
(ex-`dump/`). Did not touch `.claude/settings.local.json`, `LICENSE`, `.git/`.

## Acceptance criteria

- `git status`/`git diff --stat` show only renames (history preserved) plus new files — no
  deleted content with no destination.
- Every file formerly in `dump/` has either a destination in real docs/tasks/ADRs or a
  documented reason for omission (`docs/archive/rust-rewrite-seed/SEED_CONSUMPTION_REPORT.md`).
- `legacy/` is self-contained enough to explain how to rebuild the old app (points to its own
  preserved docs).
- `AGENTS.md` and `CLAUDE.md` exist at root; `CLAUDE.md` is a plain file, not a symlink
  (ADR-0002).
- No uncommitted secrets or machine-specific paths introduced.

## Verification

```
git status --short
git diff --stat main...rewrite/rust
```

Manual read-through: `agent/CURRENT_STATE.md`, `tasks/BACKLOG.md`,
`docs/archive/rust-rewrite-seed/SEED_CONSUMPTION_REPORT.md`.

## Next step

Hand off to task 0002 (AI-tooling discovery verification, needs a fresh CLI session to test)
and task 0003 (Phase 1 kickoff). Do not start Phase 2/3 work before those.
