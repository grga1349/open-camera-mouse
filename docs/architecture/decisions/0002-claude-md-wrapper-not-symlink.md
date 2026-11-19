# ADR-0002: `CLAUDE.md` is a plain pointer file, not a symlink to `AGENTS.md`

Status: Accepted — amended by ADR-0005 (pointer is now an `@AGENTS.md` import)
Date: 2026-11-11

## Context

The planning seed proposed `AGENTS.md` as canonical with `CLAUDE.md -> AGENTS.md` as a
relative symlink, explicitly flagged as "verify client behavior before relying on link" and
"do not assume symlinks work with every client." The project's own stated requirement is that
this repository work across macOS, Windows and Linux, not just the current macOS dev machine.

## Options considered

- **Real symlink `CLAUDE.md -> AGENTS.md`.** Rejected: Git stores symlinks as blobs; on a
  Windows checkout without Developer Mode / admin rights and `core.symlinks=true`, Git writes
  a plain text file *containing the literal string* `AGENTS.md` instead of a working symlink.
  Claude Code would then load that one-line string as the entire instruction set — silently
  broken, not failing loudly. This is exactly the class of risk the seed flagged and the kind
  of cross-platform assumption this project explicitly wants to avoid baking in.
- **Duplicate full content in both files.** Rejected: guarantees drift; defeats "one canonical
  source" from the start.
- **Tiny `CLAUDE.md` wrapper that instructs the agent to read `AGENTS.md`.** Chosen.

## Decision

`CLAUDE.md` is an ordinary, short text file (not a symlink) whose entire content is an
instruction to read `AGENTS.md` first, plus a one-paragraph fallback summary in case that read
is somehow skipped. This works identically on every platform and every git configuration,
because it's just a regular file. Codex needs no equivalent wrapper — it reads `AGENTS.md`
directly at the repo root by its own convention.

## Consequences

- There is a small duplication risk: the one-paragraph fallback in `CLAUDE.md` could drift
  from `AGENTS.md` over time. Mitigation: keep that paragraph to a single orientation sentence
  with no specifics worth drifting (see the actual file) — it should never contain anything
  that isn't equally true as a one-line fact.
- This does not yet verify that Claude Code actually reads project-local
  `.agents/skills/<name>/SKILL.md` the same way it reads root `CLAUDE.md`/`AGENTS.md` — that's
  a separate question — resolved in task 0002 (`tasks/done/`), see ADR-0005.
- If a future Claude Code/Codex release changes root-instruction-file discovery conventions,
  revisit this ADR rather than assuming the wrapper pattern is permanent.
