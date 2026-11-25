# Model routing

Which model each role in [`WORKFLOW.md`](WORKFLOW.md) uses, per runtime, once delegation is
justified (`WORKFLOW.md` "When to delegate"). The first choice is the existing main context;
a new context, even on a cheaper model, re-reads instructions and code and is not
automatically cheaper. When delegating, use the cheapest tier that will do the task
reliably; go up a tier when the task proves harder or a result can't be trusted. Reserve
extra strong-tier agents for work that needs independent deep reasoning (risk-triggered
review, hard root-cause traces). Don't spawn a light-tier agent for what a few direct
searches or reads answer. Model names change: when one here is no longer offered, use that
runtime's current model of the same tier and update this file.

## Tiers

| Tier | Roles |
|---|---|
| strongest | main agent, spike lead |
| strong | planner/architect, independent reviewer, complex investigation |
| coding | implementer, test engineer, complex investigation (bounded), docs |
| light | minor investigation, small docs edits |

## Claude Code

Subagents are started with the Agent tool; the model is a per-call `model` value, so no
static agent files are needed. Read-only roles use a subagent type without file-edit tools.

| Tier | `model` | Subagent type | Notes |
|---|---|---|---|
| strongest | Opus (the session model) | — | The main agent; the user picks it at session start |
| strong | `opus` | `Explore` (read-only) | Reviewer, complex investigation |
| coding | `sonnet` | `general-purpose` | Implementer, test engineer, doc writer |
| light | `haiku` | `Explore` (read-only) | Minor investigation |

- `Explore`/`Plan` have no Edit/Write tools but do have a shell: read-only is still an
  instruction for shell commands.
- Reasoning effort: leave the default unless this file says otherwise; reviewers and spikes
  may pass `effort: "high"`.
- Verified 2026-11-25 (Claude Code, this repo): an `Explore` subagent with `model: haiku`
  ran a read-only task, reported `claude-haiku-5-5`, changed nothing. `opus`/`sonnet`
  overrides and `general-purpose` writers: documented, not yet exercised here.

## Codex

Subagents are on by default in current Codex (CLI, app, IDE; `multi_agent` feature stable
and enabled in `codex-cli 0.166.0`). The parent may request delegation because `AGENTS.md`
asks for it; model and reasoning effort can be set per spawn (spawn value → `[agents]`
defaults → parent). Built-in agent types: `explorer` (read-heavy), `worker` (implementation),
`default`.

| Tier | Model (2026-11) | Effort | Agent type |
|---|---|---|---|
| strongest | `gpt-6.2-astra` | high or xhigh | — (main agent; the user's session model) |
| strong | `gpt-6.2-astra` | high | `explorer` |
| coding | `gpt-6.2-sol` | medium | `worker` (`explorer` for read-only) |
| light | `gpt-6-luna` | low | `explorer` |

- Subagents inherit the parent's sandbox and approvals; a custom agent can't make itself
  read-only. Read-only roles are read-only by instruction.
- Custom agents (`.codex/agents/*.toml`) are not used: per-spawn model choice covers the
  table, and reports of named-agent selection failing on some versions/platforms
  (openai/codex issues #26363, #26828) make them a liability.
- Source: the Codex subagents docs (learn.chatgpt.com/docs/agent-configuration/subagents),
  model slugs from the local model cache (2026-11-25). Not yet exercised in this repo.

## Other runtimes

No subagents or no per-agent model: follow `WORKFLOW.md` "Failure and fallback".
