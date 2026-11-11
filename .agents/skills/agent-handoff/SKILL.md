---
name: agent-handoff
description: Write agent/CURRENT_STATE.md and agent/HANDOFF.md before context loss, compaction, or switching tools. Use at the end of any work session or before a risky/long operation. Not a substitute for tasks/ (task status lives there, not duplicated here).
---

## Objective

Persist exactly what's needed to resume cold: branch/commit, current task ID, changed files,
key decisions and why, what was actually tested (commands + outcome, not "looks correct"),
blockers, and the first concrete next action. Overwrite `agent/HANDOFF.md` each time — it's a
snapshot, not a log; git history is the log if the past is needed.

## When to use

End of a session, before a risky operation, before manual/automatic context compaction, or
when handing off between Claude Code and Codex.

## When not to use

Don't use this to record task status (that belongs in `tasks/BACKLOG.md`) or architectural
decisions (those belong in `docs/architecture/decisions/`) — link to them, don't restate them.

## Deliverables and gates

`agent/CURRENT_STATE.md` and `agent/HANDOFF.md` both match the actual working tree
(`git status`) at write time — a handoff that doesn't match `git status` is worse than none.
