# Shortcut commands

A prompt that consists of only one of these words is a command; resolve it here before
treating it as an ordinary prompt. The same word inside a longer prompt is just text.

Every command follows `AGENTS.md` in full: the workflow, principles, quality gates,
skills and "Never" list. A command never skips verification, never decides an open product
question on its own (ask instead), and never changes a default, hotkey or the visual direction.

**Active handoff / roadmap:** the temporary handoff and roadmap that `AGENTS.md`'s "Read"
table lists for the current branch (e.g. `docs/V0.4.1_HANDOFF.md` + `docs/V0.4.1_ROADMAP.md`).
If there are none, the command says so and stops; it never creates them.

| Command | Does |
|---|---|
| `rhmo` | Read the active handoff, then its roadmap, and do the step under "Resume here": one slice or item, by the normal workflow, in the mode the handoff records (standard orchestration unless it says main context or spike). If that step waits on the user (a decision, a device check), say so and take the next unblocked item it names; if none is unblocked, ask. |
| `h` | Rewrite the active handoff for the next session: update "Resume here" and add this session's entry (what changed, tests, gates run, what wasn't verified, open decisions, roles and models used). Record the mode if it isn't standard (main context, spike). Tick finished roadmap items. No commit. |
| `hcp` | `h`, then `cp`. |
| `cp` | Commit and push the current task's changes. The command is the user's yes for this one commit and push (nothing else: no merge, tag, release or force-push). |
| `v` | Run the gates for what changed (Rust and/or website, scoped, one heavy command at a time) and report each result with its output summary, plus what can only be checked by hand. |
| `r` | Review the uncommitted changes (or the branch against `main` if the tree is clean) with the `code-review` skill: correctness, regressions, missing tests, ADR-0007 boundaries and other ADRs. Report findings by severity; change nothing. Prefer an independent reviewer subagent (strong tier, `.agents/MODELS.md`); the main agent checks its findings before reporting, and says so if the review was done in its own context instead. |
| `mc` | Main context mode (`.agents/WORKFLOW.md`): start no delegations, stop or wind down running ones and report what they changed, check `git status`, drop assumptions about the previous plan, and ask what we are investigating or correcting. The main agent then works directly until `resume` or the end of the task. |
| `resume` | Leave main context mode: back to standard orchestration for the current task. Not a resume of the handoff (that is `rhmo`). |

`h`, `hcp`, `cp` and `v` are always run by the main agent; a subagent may run gate commands
for `v`, but the main agent reads their output and reports it.

## `cp` details

1. `git status` / `git diff`: if the tree holds changes unrelated to the current task, or
   files that must never be committed (recordings, local paths, secrets), stop and ask.
2. If the gates haven't passed since the last edit, run `v` first; on a failure, stop.
3. Stage the task's files by name, commit with a subject in the repo's style (`git log`;
   e.g. `v0.4.1 slice 4: …`) and the attribution trailer, push the current branch. On the
   default branch, ask first.
4. Report the commit hash and the push result.
