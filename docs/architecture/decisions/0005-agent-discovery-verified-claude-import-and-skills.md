# ADR-0005: `CLAUDE.md` imports `AGENTS.md`; skills stay in `.agents/skills/` only

Status: Accepted
Date: 2026-11-11
Amends: ADR-0002 (keeps its no-symlink decision, changes how the pointer works)

## Context

Task 0002 verified instruction/skill discovery empirically from fresh sessions, not from
docs alone.

**Claude Code** (fresh session, 2026-11-11): the startup system reminder contained the
*pointer* `CLAUDE.md` text only — **not** `AGENTS.md`'s content. `AGENTS.md` was read only
because the agent obeyed the "read AGENTS.md first" sentence. Current docs
(code.claude.com/docs/en/memory, "AGENTS.md" section) confirm: when a `CLAUDE.md` exists,
Claude reads "your `CLAUDE.md` files only", unless the `CLAUDE.md` imports `AGENTS.md` via
`@AGENTS.md`. None of the ten `.agents/skills/*` appeared in the session's available-skills
list. The skills docs list only `.claude/skills/<name>/SKILL.md` (plus `--add-dir`); no
`.agents/skills` support and no settings key to add skill directories.

**Codex** (`codex-cli 0.166.0`, fresh `codex exec -s read-only --ephemeral`, asked to answer
only from preloaded context): quoted `# Open Camera Mouse — Rust rewrite` as its project
instructions, and listed all ten `.agents/skills/*` skills as repository skills. Matches
current Codex docs, which scan `.agents/skills` from CWD up to the repo root.

## Options considered

Instructions:
- **Keep the prose pointer.** Works only if the agent complies; observed to work once, but
  it's a soft guarantee and costs a tool call per session.
- **`@AGENTS.md` import in a plain `CLAUDE.md`.** Documented, relative, in-repo (no external
  import approval dialog), no symlink, so ADR-0002's Windows concern doesn't apply. Chosen.

Skills for Claude Code:
- **Symlink `.claude/skills` → `.agents/skills`.** Rejected for the same Windows reason as
  ADR-0002.
- **Duplicate into `.claude/skills/`.** Rejected: guaranteed drift, ten files × two copies.
- **Generated copy + a drift check script.** Viable later, but adds tooling before there is
  any Rust code to justify it.
- **No auto-load; on-demand read.** `AGENTS.md` (now always in context via the import) lists
  each skill and tells Claude Code to `Read` `.agents/skills/<name>/SKILL.md` when the task
  matches. Chosen for now.

## Decision

`CLAUDE.md` is a plain file whose body is `@AGENTS.md` plus a one-line fallback. Skills live
only in `.agents/skills/`: Codex auto-discovers them; Claude Code reads them on demand per the
`AGENTS.md` skills table.

## Consequences

- One canonical source for both instructions and skills; nothing to keep in sync.
- Claude Code loses skill auto-triggering by description; it depends on the agent following
  the table. If that proves unreliable in practice, revisit with the generated-copy option.
- Revisit if Claude Code adds `.agents/skills` discovery (then this becomes zero-cost).
- Not yet observed: a Windows checkout. The import is a plain text line, so no
  platform-specific behavior is expected, but it's unverified until a Windows session runs.
