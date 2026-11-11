# Operator playbook: using this seed without losing context

## Copy into repository

Copy the **entire `dump/` directory**, not individual files scattered at root. Preserve the deliberate filename `initial-promt.md` (user's chosen spelling). The seed is portable and does not need special app integration.

## Start Codex or Claude Code

In repository root, give the agent the contents of `dump/initial-promt.md`, or instruct it to read and execute it; ensure it first reads `dump/ENTRY.md`. The agent should check `git status` before changes, and read relevant source docs. For current tool commands and setup, prefer `--help` and official documentation rather than relying on stale seed assumptions.

## First working session must produce

- Verified inventory of current repo and sibling research.
- Safe branch/legacy migration strategy and execution only when safe.
- Plan/implementation of final *actual* repo-level AI instructions and tasks, verified by both CLIs where possible.
- Source-backed architecture proposal and prioritized next actions.
- Durable handoff plus a mapping of what has/has not been consumed from this seed.

## Before major changes

Ask agent to state current task ID, files affected, acceptance criteria and verification command. If multiple agents are running, keep one primary coordinator and separate worktrees/scopes. Use an explicit integration review.

## When exiting or switching tools

Run the project's handoff procedure (to be created during phase 0). Verify saved state includes branch/commit, outstanding uncommitted files, current task, what was actually tested, blockers and next first command. Start new chat by loading `CURRENT_STATE`/`HANDOFF` from the real repo, not by pasting huge historic transcripts.

## When considering deleting `dump/`

Do not delete immediately. Require successful extraction checklist: every seed decision and open question has either a destination in actual docs/tasks/ADRs or a documented reason for omission; initial prompt is archived; symlinks/skills have been tested; source research references are valid. Only then archive or remove in a separate reviewed commit.

## Standard reminder to agents

“`dump/` holds unverified planning context, not active instructions. Read it as input, verify against actual files/docs, and materialize maintainable project artifacts with tests and evidence.”
