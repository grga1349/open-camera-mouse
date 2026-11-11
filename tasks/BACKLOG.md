# Task backlog

Single source of truth for task status — don't duplicate status elsewhere. Lifecycle:
`TRIAGE → READY → ACTIVE → REVIEW → DONE` (or `BLOCKED`). One coordinator updates this file
and moves task files between `active/`, `done/`, `blocked/` as status changes; a checklist is
not completion evidence, passing verification commands are.

## Active

| ID | Title | Status |
|---|---|---|
| 0002 | Verify AI-tooling instruction/skill discovery in Claude Code and Codex | READY |
| 0003 | Phase 1 — research and behavior specification | TRIAGE (unblocks after 0002) |

## Done

| ID | Title | Status |
|---|---|---|
| 0001 | Phase 0 — repository foundation | DONE — commit `a1e4f4e`, pushed to `origin/rewrite/rust` |

## Blocked

(none yet)
