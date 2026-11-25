---
name: engineering-planning
description: Producing and reviewing engineering plans - discovery, feasibility, vertical slices, dependencies and critical path, risk-first order, acceptance criteria and verification (tests first, device checks per OS), user and ADR decisions, spikes, replanning, scope control. Use when planning multi-step or multi-session work, changing an active roadmap, or reviewing a plan. Not for where plans live or statuses (.agents/PLANNING.md), execution or delegation (.agents/WORKFLOW.md), or root-causing a bug (systematic-debugging).
---

# Engineering planning

## Objective

A plan is the set of decisions an implementer could not make alone, in an order that removes
the biggest risk first, with a check for each. Lines that decide nothing ("handle edge
cases", "add tests", "TBD") are gaps; a plan longer than the code it describes is a
transcript. Effort scales with size, uncertainty and risk: a one-line fix needs no plan, a
cross-crate or device-dependent change needs this whole skill.

## Discovery: look before you ask

- Read the code, `docs/BEHAVIOR.md` (R-*), `docs/ARCHITECTURE.md` (P1-P20), the ADRs and the
  active roadmap before asking anything. Never ask what they answer or reopen a decided point.
- Ask only when the answer changes what gets built: product behaviour, defaults, anything
  user-visible, release scope. One question at a time, with your recommendation. Say every
  default you filled in out loud; a default the user never heard is one they could not
  reject.
- Separate facts (measured, read in code, seen by the user) from hypotheses (traced, not
  proven).
- Feasibility: name the riskiest assumption and the cheapest check that would break it (a
  test, a probe in the scratchpad, an `OCM_PERF`/`OCM_INPUT` run, an upstream source read).

## Decomposition

- **Vertical slices:** each item ends with observable behaviour and a test, not a layer
  ("pointer-core change" is not an item; "a gain change keeps the cursor still" is).
- **Right size:** the smallest unit with its own test cycle that a reviewer could reject
  while approving its neighbour; fold scaffolding and docs into the item that needs them.
  One item fits one session (one item, or a slice the user names, per session).
- **Dependencies:** write `Needs` only for real ones (an interface, a decision, a
  measurement). The critical path is the longest chain of `Needs`; user decisions and device
  runs on it are asked for first, because they take wall-clock time.
- **Risk first:** order by what would invalidate the plan (unknown root cause, unproven
  feasibility, a user decision) before what is merely large. Reproduce before fixing;
  measure before tuning.
- **Spikes for unknowns:** when a choice hinges on something nobody knows, plan a timeboxed
  spike with a question and a decision rule ("if the same-pass slider test fails, gap the
  click; else …"). Its outcome is an item, a revised plan or a rejected approach.

## Acceptance criteria and verification

- Criteria are observable and checkable: "pressing, dragging with the head and releasing
  draws a line in our window and another app", not "drags work".
- Test-first: name the failing test and its layer for each fix (layer table in
  `systematic-debugging`). Characterisation tests where behaviour must not change.
- Name the failure modes the requirements imply but no test exercises (e.g. a click right
  after a re-selection, a setting changed while lost, 125 % DPI) and give each a test or a
  check in the owning item.
- Per item, write which verification levels are mandatory: automated, CI on all three OSes,
  device per OS, physical vs synthetic input, external camera. Device checks are user steps
  with exact instructions; on the critical path, batch them into one user run.

## Decisions

- Product (default, hotkey, visual direction, behaviour change, release scope): a `D`-row in
  the roadmap with options and your recommendation; the dependent items are `[?]`.
- Architecture (new crate or dependency, crate boundary, cross-layer change): an ADR before
  the code (`rust-architecture`); a changed default amends the ADR that owns it.
- Detail an implementer can settle: decide it, or say it is theirs.

## Multi-session organisation

Slices in priority order, each closable on its own; front-load user questions so answers
arrive before the dependent slice; a checkpoint at every item boundary (`WORKFLOW.md`).

## Replanning and scope control

- Replan when evidence breaks an assumption: a hypothesis disproved, a measurement out of
  range, a review finding, a fix attempt that failed twice. Update the items, then the resume
  point; don't keep executing a plan you no longer believe.
- If an item grows mid-flight, stop and split or re-size it with the user; never widen it
  silently.
- Scope creep check per item: does it serve the initiative's goal and a requirement? If not,
  it goes to the active roadmap's Leftovers as a proposal (the user decides any transfer to
  `docs/ROADMAP.md`). No speculative abstraction, option
  or setting without a user need (ADR-0006: new behaviour is an opt-in option).

## Plan review

Self-review every plan: each requirement maps to an item; each item has criteria and a
check; ids, names and dependencies are consistent; nothing decides nothing; the length is in
proportion. Substantial or risky plans also get an independent review: brief and checklist
in `references/plan-review.md`.

## Deliverables

A plan in the conversation (one session) or an active roadmap per `.agents/PLANNING.md`
(multi-session, after approval); open decisions with recommendations; the first item ready.
A planning request never includes implementation.

References (load when needed):

- `references/examples.md`: roadmap, item, leftovers entry, reconciliation summary,
  compact handoff. Load when creating or restructuring one.
- `references/plan-review.md`: plan-review brief and checklist. Load to review a plan.

## Attribution

Ideas adapted from `writing-plans` and `brainstorming` in obra/superpowers
(https://github.com/obra/superpowers, commit 22f0b463e7a5e579904dd6211a520321035526f7, MIT,
Copyright (c) 2025 Jesse Vincent); rewritten here, no upstream text beyond short phrases.
