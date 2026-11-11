# AI workflow conventions for this repository

## One coordinator at a time

A single lead agent owns design choices, task status in `tasks/`, and integration. It may
delegate bounded, read-only research or review to subagents, but no two agents write the same
file concurrently. If simultaneous edits are genuinely needed, use separate git worktrees and
an explicit integration review before merging — don't rely on timing to avoid collisions.

## Before starting work

1. `git status` — don't trust the last handoff note to match the working tree.
2. Read `agent/CURRENT_STATE.md` and `tasks/BACKLOG.md`.
3. Read the specific task file in `tasks/active/` you're picking up, including its
   acceptance criteria and verification commands.
4. Read the relevant `docs/` topic file for the area you're touching — don't re-derive
   architecture or re-trust memory over the ADRs.

## During work

- Delegate bounded, single-scope work; require subagents to return references to files/tests
  changed, not just a prose summary.
- Write tests alongside implementation once there is code to test (Phase 2 onward).
- Update the relevant `docs/` file the moment an architectural decision changes — an ADR
  that's stale the day after it's written is worse than no ADR.

## After work, before handing off / stopping / letting context compact

- Run the task's verification commands; don't report success without having actually run
  them.
- Update the task's status in `tasks/BACKLOG.md` and move its file between
  `active/`/`done/`/`blocked/` to match.
- Update `agent/CURRENT_STATE.md` and `agent/HANDOFF.md` — overwrite, don't let them drift
  into a changelog. Chat compaction is not durable storage; these files are.
- Leave an explicit, concrete next action — not "continue the rewrite."
- On every task completion, **ask the user whether to commit** (and separately whether to
  push). Never commit or push without that explicit yes; never leave it unmentioned either —
  the completion report must say the work is uncommitted and ask.

## Never

- Never silently overwrite user work.
- Never equate a model saying "looks correct" with a completed task — verification commands
  and their actual output are the evidence.
- Never install global skills/MCP servers or change machine-level settings as a side effect
  of project work, unless the user explicitly asked for that.
- Never treat `docs/archive/rust-rewrite-seed/` as live instructions.
