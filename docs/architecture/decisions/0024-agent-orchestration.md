# ADR-0024: Agent orchestration — one canonical workflow under `.agents/`

Status: Accepted (user request 2026-11-25)
Date: 2026-11-25
Amends: ADR-0005 (instruction sources), ADR-0022 (prompt-driven workflow)

## Context

Work so far ran in one agent context per session. The v0.4.1 click-path review (session 12)
showed the cost: a single review consumed a large share of the main context. The user asked
for a hierarchical workflow: an accountable main agent that plans and integrates, and
subagents with cheaper or independent models for implementation, tests, investigation and
review, usable from Claude Code, Codex and later runtimes, with a main-context mode (`mc`)
for direct control and a spike mode.

Verified 2026-11-25:
- **Claude Code:** the Agent tool takes a per-call `model` (`opus`/`sonnet`/`haiku`) and a
  subagent type; `Explore`/`Plan` lack file-edit tools. An `Explore` + `haiku` subagent ran
  a read-only task in this repo and reported `claude-haiku-5-5`.
- **Codex** (`codex-cli 0.166.0`): `multi_agent` stable and enabled; docs: subagents may be
  started when `AGENTS.md` asks, model and reasoning effort settable per spawn, built-in
  `explorer`/`worker` types, subagents inherit the parent sandbox. Not exercised in this repo.

## Options considered

1. **Provider-specific agent files** (`.claude/agents/*.md`, `.codex/agents/*.toml`), one
   per role: static registration, tool restrictions on Claude. Two parallel trees to keep in
   sync (ADR-0005 rejected duplication for skills on the same grounds); Codex named-agent
   selection is reported broken on some versions/platforms.
2. **Orchestration scripts or a framework:** new tooling and dependencies for a process
   both runtimes already support natively.
3. **One provider-neutral policy in `.agents/`, dynamic spawning with explicit role
   instructions and a per-spawn model.** Chosen.

## Decision

- `AGENTS.md` stays the short entry point and points to `.agents/WORKFLOW.md` (roles,
  standard/main-context/spike modes, delegation contract, checkpoints, fallback) and
  `.agents/MODELS.md` (tiers → Claude and Codex models). Roles are a table in `WORKFLOW.md`,
  not a `roles/` directory: each fits in one line, and skills carry the know-how.
- No provider adapters: both runtimes spawn dynamically with a model choice. Add the
  smallest adapter only if a runtime later requires static registration.
- `docs/COMMANDS.md` gains `mc` and `resume`; `rhmo`, `r`, `h` integrate the modes.
- The main agent keeps ownership of decisions, integration, roadmap/handoff and git; the
  `AGENTS.md` "Never" list applies to every agent.

## Consequences

- Cheaper models do bounded work; the main context keeps decisions and evidence, not raw
  output. Independent review becomes the default instead of self-review.
- Read-only is enforced only partly (Claude `Explore` has no edit tools but has a shell;
  Codex subagents inherit the parent sandbox): it is mostly an instruction.
- Model names in `MODELS.md` will age; the tiers don't.
- Untested here: Codex delegation, Claude `opus`/`sonnet` overrides and writer subagents.
  Revisit after the first sessions that use them.
