# Task backlog

Single source of truth for task status — don't duplicate status elsewhere. Lifecycle:
`TRIAGE → READY → ACTIVE → REVIEW → DONE` (or `BLOCKED`). One coordinator updates this file
and moves task files between `active/`, `done/`, `blocked/` as status changes; a checklist is
not completion evidence, passing verification commands are.

## Active

| ID | Title | Status |
|---|---|---|
| 0003 | Phase 1 — research and behavior specification | REVIEW — `docs/research/BEHAVIOR_SPEC.md` |
| 0004 | Phase 2 — Rust architecture (workspace skeleton, ADRs, platform risk matrix) | TRIAGE — pointer model settled by ADR-0006 (absolute, favor 2018); hotkeys Q-R8 open with user |

## Done

| ID | Title | Status |
|---|---|---|
| 0001 | Phase 0 — repository foundation | DONE — commit `a1e4f4e`, pushed to `origin/rewrite/rust` |
| 0002 | Verify AI-tooling instruction/skill discovery in Claude Code and Codex | DONE — ADR-0005; `@AGENTS.md` import confirmed expanding in a fresh session (2026-11-11) |

## Blocked

(none yet)
