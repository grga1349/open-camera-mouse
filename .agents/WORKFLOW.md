# Agent workflow: orchestration

How the main agent runs work, alone or with subagents (ADR-0024). Provider-neutral; model
names are in [`MODELS.md`](MODELS.md). Planning and tracking across sessions:
[`PLANNING.md`](PLANNING.md). `AGENTS.md` stays the entry point and wins on any conflict;
skills stay in `.agents/skills/` and are referenced, never copied into a delegation.

**Principle: the fewest agents, the least context and the cheapest suitable model that
deliver a correct, verified result.** Single-agent execution is the default; delegation is an
escalation. This file is the user's standing permission to delegate when the conditions below
hold (except in main context mode), not a request to delegate.

## Roles

Roles are capabilities, not pipeline stages. The main agent fills any of them itself unless
a separate context pays off (below).

| Role | Tier (`MODELS.md`) | Writes? | Does |
|---|---|---|---|
| Main agent | strongest | yes | Owns the conversation, plan, decisions, integration, roadmap/handoff, git (on the user's yes); by default also investigates, writes tests, implements, verifies, updates docs and self-reviews |
| Planner / architect | strong | no | A second design opinion or an independent plan review (`PLANNING.md`) |
| Implementer | coding | yes, assigned files | A bounded implementation against an approved plan and acceptance criteria |
| Test engineer | coding | yes, test files | Independent regression / correct-behaviour / exploratory tests; names the ones that need real hardware |
| Reviewer | strong | no | Independent review per the `code-review` skill; never the implementer |
| Investigator | light, coding or strong | no | A bounded lookup or a root-cause trace too large for the main context |
| Doc writer | coding or light | yes, docs only | Large doc updates from the main agent's facts |
| Spike lead | strongest | yes, isolated | The main agent in spike mode (below) |

The main agent is the accountable engineering lead: it reads the code it changes, keeps
enough understanding to judge every result, and checks subagent claims (re-runs a test,
reads the diff) before relying on them.

## When to delegate

Delegate only for a concrete benefit: independent reasoning or verification, real context
isolation (a large investigation whose raw output the main context doesn't need), specialised
investigation, meaningful parallelism, or a net saving. A new context pays to start: it
re-reads instructions, skills and code and transfers results back. A cheaper model in a new
context is not automatically cheaper than continuing in the main one. Never spawn an agent
because a role exists, and never for work a few direct searches or reads can do.

Scale to complexity and risk (guidelines, not modes):

- **Trivial / routine** (docs fix, small setting, routine bug fix): all in the main context,
  with its tests and verification. No subagents.
- **Moderate:** prefer the main context; at most one bounded delegation (an implementation
  or an investigation) when it clearly pays. Tests-first, implementation and docs stay in one
  context where practical.
- **Complex / high-risk:** use the roles selectively (independent tests, implementation,
  investigation, review) where each materially improves the outcome; prefer bounded briefs
  over broad repository sweeps. Before a large agent chain, tell the user why it is worth it.

## Standard mode (default)

1. **Understand** per `AGENTS.md` step 1, reading only what the task needs.
2. **Plan** in the main context: approach, files, acceptance criteria, verification. Resolve
   product/architecture decisions with the user first (`AGENTS.md` step 2, ADR before code).
3. **Tests first:** write the failing regression test(s) and run them. An independent test
   engineer only where test independence adds real value (complex behavioural regression,
   ambiguous failure scenario, substantial algorithm change).
4. **Implement** against the plan and the failing tests.
5. **Verify** with the gates per `AGENTS.md`: scoped during development, broader at
   integration and release boundaries, one heavy command at a time. Reuse earlier evidence
   only when the code, config and dependencies it covered are unchanged; never claim a check
   that wasn't run.
6. **Review** (below), then integrate findings and re-verify.
7. **Docs:** update what the change made stale.
8. **Checkpoint** (below) and stop at the work-item boundary.

A request for a plan, investigation, review or handoff is not permission to implement.

## Review

- **Self-review** (main agent, separate pass over the diff with the `code-review` checklist)
  is enough for routine, low-risk changes.
- **Immediate independent review** (reviewer subagent) when risk warrants it: tracking or
  pointer algorithms, native input event semantics, unsafe Rust / FFI, concurrency and
  resource lifecycle, significant architecture changes, high-risk regressions, substantial
  accessibility impact.
- Otherwise an independent review may be **consolidated** at a phase or milestone boundary
  instead of per item; record in the active roadmap which items await it.
- The reviewer gets the requirements, relevant ADRs/skills, the diff and the verification
  output, not the implementer's reasoning. A correction that materially changes reviewed
  production logic gets a focused re-review of that change (or is reported as not
  re-reviewed). Unchanged code is not re-reviewed.
- Self-review is always reported as self-review, never as independent review. Release
  gates and validation (`release` skill) are unchanged.

## Delegation contract

Each delegated task states: objective and deliverable; the requirements, decisions, ADRs and
skills (`.agents/skills/<name>/SKILL.md` paths) that apply; the files it may change (or
read-only); constraints; acceptance criteria and the verification to run; the report format.
Give precise paths and only the context the task needs: never whole roadmaps, handoffs, ADR
collections or the conversation. Don't have two agents investigate the same thing; reuse a
finished subagent's context (e.g. `SendMessage`) for a related follow-up when supported.

The report back: result, changed files, verification commands with their outcome,
uncertainties, blockers, open decisions. Short; no transcript.

Subagents never: make product decisions, change approved architecture, edit the roadmap or
handoff, commit/push/merge/tag/release, widen their scope, weaken tests or gates, or spawn
further agents unless told to. Reviewers and investigators are read-only. No two writers on
overlapping files at once; dependent work is serialised. The main agent is accountable for
everything integrated.

## Main context mode (`mc`)

The user wants direct control, usually because something went wrong. On `mc`: start no new
delegations; stop or let finish any running ones where the runtime allows, and report what
they changed; check `git status`; don't assume the previous plan still holds; ask the user
what we are investigating or correcting. While active, the main agent delegates nothing
without the user's explicit OK. A prompt that asks for a task "in the main context" is done
the same way.

Active until `resume` or the end of the task. If the task continues in another session,
record the mode in the active handoff.

## Spike mode

When a session starts with a spike request, the main agent is the spike lead: it owns the
question, hypotheses, method, experiment design, evaluation and conclusion, and runs the
spike itself by default. It delegates only independent or substantial research, prototypes,
fixtures or benchmarks that gain from a separate context; not one agent per question.
Experiments stay isolated (scratch directory, a git worktree or a clearly marked branch) and
never become production code silently: adoption is a separate decision through standard
mode (and an ADR where `AGENTS.md` needs one). A spike ends with evidence, limitations,
trade-offs and a recommendation, then a checkpoint.

## Context, usage and checkpoints

Context and subscription usage are limited. Load only the docs, skills and references the
task needs; reuse what is already in context instead of re-reading unchanged files; prefer
targeted searches to broad scans; keep decisions, criteria, short findings and verification
evidence in the main context, not raw output. Use a real utilisation figure when one is
visible (status line, `/context`, or what the user reports about remaining 5-hour or weekly
usage); never invent token counts, costs, percentages or savings. When usage is tight:
fewer agents, consolidated work, less redundant investigation and review, and no work that
can't reach a meaningful checkpoint. Gates and mandatory verification still apply.

Stop at a checkpoint when: a work item, roadmap phase or spike is done; the context or usage
is getting tight; a product/architecture decision blocks; repeated attempts fail; or
continuing would widen scope. Never start the next major work item by yourself.

Checkpoint report: what was done; decisions and findings; agents and models actually used
(none is a valid answer); tests and verification results; review type (self or independent)
and how findings were resolved; risks and what is unverified; the recommended next step.
Then ask about `h`, `cp`/`hcp`; a checkpoint is never a yes to commit or push.

## Failure and fallback

- A delegation that fails twice is not retried blindly: the main agent investigates itself or
  asks the user.
- No subagents or no per-agent model choice in the runtime: work in the main context, with
  review reported as self-review. Missing delegation never blocks ordinary work.
- Never claim a model or an independent agent was used unless the runtime confirmed it.
