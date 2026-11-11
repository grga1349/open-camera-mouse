# 0002 — Verify AI-tooling instruction/skill discovery

Status: READY
Owner/branch: `rewrite/rust`
Depends on: 0001

## Context

Phase 0 created `AGENTS.md` (canonical), a plain-file `CLAUDE.md` pointer (ADR-0002, chosen
specifically to dodge a Windows git-symlink failure mode), and `.agents/skills/` as the
project skill catalog. None of this has been empirically confirmed to actually get discovered
by either CLI from a **fresh session** — this session already had the old `CLAUDE.md` loaded
before the migration happened, so it cannot self-verify the new files via its own system
reminders.

## Objective

In a **new** Claude Code session rooted at this repo, confirm the system reminder shows
`AGENTS.md`'s content (not the old Go-project `CLAUDE.md`) and check whether anything under
`.agents/skills/` appears in the "available skills" list without further configuration. In a
new Codex session rooted at this repo, confirm it loads `AGENTS.md` and check Codex's actual
project-local skill convention (unconfirmed this session — `codex --help` does not expose a
documented skills subcommand; the global `~/.codex/skills/` directory exists, but the
project-level equivalent, if any, needs checking against current Codex docs, not assumed).

## Non-goals

Not about content quality of `AGENTS.md` — just discovery mechanics.

## Acceptance criteria

- Written result (append to this file or a short note in `agent/CURRENT_STATE.md`): does
  Claude Code load `AGENTS.md` via the `CLAUDE.md` pointer pattern as intended, yes/no, with
  what was actually observed in the system reminder.
- Written result: does anything under `.agents/skills/` show up as an available skill in
  either CLI without a symlink/copy — if not, decide (and record as a new ADR if it changes
  ADR-0002/0003's assumptions) whether to duplicate/symlink into `.claude/skills/` and/or
  whatever Codex's actual project convention turns out to be.
- No destructive action taken while testing (read-only discovery check).

## Verification

Start a fresh `claude` session in this repo root; separately, a fresh `codex` session. Report
what each actually shows, not what the docs claim they should show.

## Next step

Feed the result back into ADR-0002/0003 (new ADR if behavior differs from what was assumed)
and into `.agents/skills/` wiring before relying on skill auto-load in later phases.
