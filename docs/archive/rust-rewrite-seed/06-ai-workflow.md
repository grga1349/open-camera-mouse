# AI-first repository operations (proposed conventions)

## Goal

Both Claude Code and Codex work against the same project model, without global machine-level hacks, conflicting instructions or pretending chat history is permanent. One main coordinator at a time. Independent subagents for research, implementation, review and testing only when useful.

## Canonical instructions, verified discovery

Proposed eventual root `AGENTS.md` is the canonical lean policy. Proposed `CLAUDE.md -> AGENTS.md` relative symlink should be tested with *actual installed CLI versions*, including discovery from nested directories. If incompatible, use a tiny `CLAUDE.md` wrapper that references a shared doc instead. Keep root instructions around a short page; link to topic docs/skills only when task requires them. Old instructions moved to archived legacy folder and must not be active root policy.

## Repo skills

Canonical candidate `.agents/skills/<name>/SKILL.md` and optional `.claude/skills` symlink into this tree if local skill discovery supports it. Never assume project skills override global/built-in skills; if strict isolation is required, test supported per-user settings/sandbox/environment. Review third-party skills before installing. Candidate skill set:

- `research`: current sources, alternative assessment, evidence table.
- `plan-task`: scope, acceptance criteria, dependencies, measurable verification.
- `implement-rust`: coding standards, minimal diffs, test and lint gates.
- `tracking-experiment`: fixtures, variants, metrics, reproducibility.
- `benchmark`: hardware/env recording, p50/p95, regression checks.
- `design-ui`: iterative design lab, state coverage, egui feasibility.
- `review-change`: adversarial review, scope/security/perf and evidence.
- `handoff`: durable state, open decisions, next command and pointers.

## Subagent roles (not all need persistent files)

Lead orchestrator; research; architecture; computer vision; Rust implementation; UI/UX; QA/performance; reviewer. In Claude, agent definitions and in Codex agent config are distinct formats: share intent, not syntax. Subagents should receive bounded deliverables and return references to files/tests. No two write agents modify the same file at once. Prefer read-only research/review parallelism. When parallel edits are necessary, use separate Git worktrees and integration gate.

## Task files and lifecycle

Proposed final repo `tasks/BACKLOG.md`, `tasks/active/`, `tasks/done/`, `tasks/blocked/`; individual task has ID/title, context, objective, non-goals, files/scope, dependencies, owner/branch, acceptance criteria, verification commands/evidence, status, links to docs/ADRs and next step. Avoid duplicating task status in several conflicting sources.

Lifecycle: TRIAGE → READY → ACTIVE → REVIEW → DONE (or BLOCKED). Coordinator owns state updates and can split tasks when evidence warrants it. Reviews report findings, not automatically mark success. Repeated TODO descriptions do not replace actual tests.

## Durable sessions and compaction

Proposed final repo `agent/CURRENT_STATE.md`, `agent/HANDOFF.md`, `agent/WORKFLOW.md`, optional small log. The handoff records current branch/commit, task ID, changed files, key decisions and reasons, commands/results, untested areas, blockers and first next action. The project state has active milestone, priorities and links. These files must be kept short and updated before handing off, exiting or manually compacting; built-in automatic compaction is not trusted as canonical memory. A helper `handoff` skill can implement this.

Do not build a perpetual autonomous service just to preserve context. The repo + Git + task system are durable; terminal chat sessions are replaceable. Use `resume` functionality when available, but a fresh session must still recover from files.

## Quality rules

- Before task: read relevant state and docs, inspect Git status.
- During: delegate bounded work, write tests, update source-backed docs when architecture changes.
- After: run targeted tests, review diff, update task and handoff, leave reproducible next steps.
- Never silently overwrite user work; no globally installed skills/MCP changes unless requested.
- Treat reverse engineering as behavioral research; keep binary/licensing boundaries clear.
- Test rather than assume symlink/loading/agent capabilities.
