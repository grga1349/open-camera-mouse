# Agent workflow: orchestration

How the main agent runs work with subagents (ADR-0024). Provider-neutral; model names are in
[`MODELS.md`](MODELS.md). Planning and tracking across sessions: [`PLANNING.md`](PLANNING.md). `AGENTS.md` stays the
entry point and wins on any conflict; skills
stay in `.agents/skills/` and are referenced, never copied into a delegation.

This file is the user's standing request to delegate as described here: a runtime default
of "spawn subagents only when the user asks" is satisfied by it, except in main context mode.

## Roles

| Role | Tier (`MODELS.md`) | Writes? | Does |
|---|---|---|---|
| Main agent | strongest | yes | Owns the conversation, the plan, decisions, integration, roadmap/handoff, git (on the user's yes) |
| Planner / architect | strong | no | Only when a second design opinion or an independent plan review (`PLANNING.md`) is worth it; planning normally stays in the main context |
| Implementer | coding | yes, assigned files | Implements an approved plan against acceptance criteria |
| Test engineer | coding | yes, test files | Regression (fails before the fix), correct-behaviour, exploratory tests; names the ones that need real hardware |
| Reviewer | strong | no | Independent review per the `code-review` skill; never the implementer |
| Investigator, minor | light | no | Bounded lookups: where is X, what calls Y, what does the doc say |
| Investigator, complex | strong or coding | no | Root-cause tracing across crates or OS layers |
| Doc writer | coding or light | yes, docs only | Updates docs the change made stale, per the main agent's facts |
| Spike lead | strongest | yes, isolated | The main agent in spike mode (below) |

The main agent is the accountable engineering lead, not a dispatcher: it reads the code it
changes, keeps enough understanding to judge every result, and checks subagent claims
(re-runs a test, reads the diff) before relying on them.

## Standard mode (default)

Scale to risk: a one-line fix or a docs edit is done directly; the full pipeline is for
non-trivial code changes.

1. **Understand.** Main agent per `AGENTS.md` step 1; may delegate bounded read-only
   investigations (in parallel when independent).
2. **Plan** in the main context: approach, files, acceptance criteria, verification. Resolve
   product/architecture decisions with the user first (`AGENTS.md` step 2, ADR before code).
3. **Tests first.** Test engineer writes the failing regression test(s) and runs them; may run
   alongside investigations when files don't overlap.
4. **Implement.** Implementer, against the plan and the failing tests.
5. **Verify.** Gates per `AGENTS.md` (scoped, one heavy command at a time), run or re-run by
   the main agent or a delegate whose output it reads.
6. **Independent review.** Reviewer gets the requirements, relevant ADRs/skills, the diff and
   the verification output, not the implementer's reasoning or conclusions.
7. **Integrate.** Main agent weighs findings, delegates or makes corrections, re-verifies.
   A correction that changes production logic goes back to the reviewer for a focused
   re-review of that change (or is reported as not re-reviewed).
8. **Checkpoint** (below) and stop at the work-item boundary.

A request for a plan, investigation, review or handoff is not permission to implement.

## Delegation contract

Each delegated task states: objective and deliverable; the requirements, decisions, ADRs and
skills (`.agents/skills/<name>/SKILL.md` paths) that apply; the files it may change (or
read-only); constraints; acceptance criteria and the verification to run; the report format.
Give only the context the task needs, not the conversation.

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
what we are investigating or correcting. While active, the main agent does investigation,
planning, tests, implementation and verification itself and delegates only with the user's
explicit OK. A prompt that asks for a task "in the main context" is done the same way.

Active until `resume` or the end of the task. If the task continues in another session,
record the mode in the active handoff.

## Spike mode

When a session starts with a spike request, the main agent is the spike lead: it owns the
question, hypotheses, method, experiment design, evaluation and conclusion; it may delegate
bounded research, prototypes, fixtures and benchmarks. Experiments stay isolated (scratch
directory, a git worktree or a clearly marked branch) and never become production code
silently: adoption is a separate decision through standard mode (and an ADR where
`AGENTS.md` needs one). A spike ends with evidence, limitations, trade-offs and a
recommendation, then a checkpoint.

## Context and checkpoints

The main context is a budget. Detail work goes to subagent contexts; the main context keeps
decisions, acceptance criteria, short findings and verification evidence, not raw output.
Use the runtime's real utilisation figure when one is visible (e.g. a status line, or the
user's `/context` output); otherwise judge conservatively and never invent a percentage.

Stop at a checkpoint when: a work item, roadmap phase or spike is done; the context is
getting full; a product/architecture decision blocks; repeated attempts fail; or continuing
would widen scope. Never start the next major work item by yourself.

Checkpoint report: what was done; decisions and findings; roles and models actually used
(and any fallback); tests and verification results; review findings and how they were
resolved; risks and what is unverified; the recommended next step. Then ask about `h`,
`cp`/`hcp`; a checkpoint is never a yes to commit or push.

## Failure and fallback

- A delegation that fails twice is not retried blindly: the main agent investigates itself or
  asks the user.
- No subagents or no per-agent model choice in the runtime: do the same workflow in the main
  context, keep the role steps separate, and say so in the checkpoint. Self-review is
  reported as self-review, never as independent review. Missing delegation never blocks
  ordinary work.
- Never claim a model or an independent agent was used unless the runtime confirmed it.
