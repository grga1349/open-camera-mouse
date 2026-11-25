---
name: rust-refactoring
description: Behaviour-preserving restructuring of existing Rust code — module depth and cohesion, oversized functions, state-machine invalid states, duplication, API surface, testability — plus read-only code health audits. Use when a concrete problem (bug class, hard change, duplicated fix, untestable logic) points at internal structure, or when asked to audit code health. Not for writing new code idiomatically (see rust-engineering) or moving crate boundaries / dependency direction / ADRs (see rust-architecture).
---

## Objective

Make existing code easier to change and test without changing what it does. A refactor is
justified by evidence of a problem, never by taste. Observable behaviour (R-* in
`docs/BEHAVIOR.md`, defaults, hotkeys, UI) stays exactly as it is; anything that would change it
is a feature or a fix and goes through the normal `AGENTS.md` workflow.

## When to use

- A bug class keeps coming back in one area, or one fix had to be repeated in several places.
- A change was hard: it touched many files, or needed knowledge scattered across modules.
- Logic cannot be tested through its interface without heavy setup.
- The user asks for a code health audit or a refactoring plan.

Live example: one gain change caused a cursor jump, a slider feedback loop and a dwell click
inside a physical drag; the cause was spread over the `Control` / `PointerOwner` / `AppState`
interplay in `crates/app-core/src/controller.rs` (details in `references/audit-checklist.md`
signal 4). That is the kind of evidence that can justify a refactor; this skill does not say
that one is needed.

## When not to use

- No evidence, only "this looks messy" or "this could be more generic". Say so and stop.
- Writing new code or fixing a bug in place: `rust-engineering`, with the measure-first rule.
- Splitting or merging crates, changing dependency direction, public crate API: `rust-architecture`
  (an ADR is required before code, ADR-0007).
- Algorithm changes: `computer-vision`; changing a default: ask the user (ADR-0006).

## Two modes

**Audit (read-only).** Produces findings and a proposed sequence. It never triggers a refactor by
itself: end by asking which finding, if any, the user wants done. No code, tests or file edits.
Use `references/audit-checklist.md` for what to look at and the report format.

**Implementation (approved).** Only for a finding the user approved. Per `AGENTS.md`: one slice
per session, small steps, gates after each step, ask before committing.

## Evidence required

State, before proposing anything: the problem, where it shows (file and function, commit or
review finding, failing or impossible test), and how the refactor removes it. Acceptable
evidence: a bug class with two or more instances, a duplicated fix in git history, a test that
cannot be written today, a change that needed edits in N places. Not evidence: line counts,
function length alone, lint style, similarity of two code blocks that change for different reasons.

## What to look for (apply the evidence rule to each)

Depth, locality, invalid states, public surface, large functions, control flow, ownership,
duplication vs abstraction, seams: criteria and traps in `references/look-for.md`. Key rules:
deep modules pass the deletion test (merge pass-throughs, add no layers); make invalid state
combinations unrepresentable or pin them with a characterisation test; split only along a real
seam, no size targets; a trait needs two real implementations; leave duplicates that change
independently.

## Implementation protocol

1. **Characterise.** Write or find tests that pin current behaviour through the public interface
   (`crates/app-core/tests/controller.rs` rig, kittest `apps/desktop/src/app_tests.rs`, pointer
   and tracker fixtures per `tracking-validation`). They must pass before any edit. Missing
   coverage is added first and committed as its own step.
2. **One move per step.** Rename, move, extract, inline, narrow visibility, merge enum: each
   separate and independently revertible. No behaviour change and no style churn in the same step.
3. **Gates after each step** (`AGENTS.md`; scoped, one heavy command at a time):
   `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
   `cargo test -p <crate> --locked`, `scripts/check-core-deps.sh` if dependencies moved.
4. **Tests follow the interface.** Tests that only pinned a removed internal go; tests that pin
   behaviour stay unchanged. If a behaviour test had to be edited, the step was not behaviour-
   preserving: stop and reassess.
5. **Hot paths.** If a step touches the frame path (`tracking-core`, `pointer-core`,
   `Controller::handle` per frame, `engine.rs`), take `OCM_PERF` numbers (or a `benchmarking`
   run) before and after, on the same machine, and report them. Watch for new allocations,
   cloning of frames, and extra locking. Recordings are diagnosis only and never committed.
6. **Docs.** Update docs the change made stale (`docs/ARCHITECTURE.md` module notes, comments that
   cite moved items). If a boundary or a decision changed, it needed an ADR first.

## Do not

Add dependencies or crates; introduce async; change defaults, hotkeys or UI; rewrite for style
(clippy and `rustfmt` already decide style); rename widely without a reason; split a file only
because it is long; leave a refactor half done across sessions without recording the state in
the conversation or PR.

## Deliverables and gates

Audit: the findings report (checklist file), no repo changes. Implementation: the steps done,
the gate and test output for each, `OCM_PERF` before/after when a hot path was touched, anything
untested on an OS or only checkable by hand, then ask whether to commit.

## References

- `references/audit-checklist.md`: load when running an audit or when choosing which candidate to propose.
- `references/look-for.md`: full criteria for depth, locality, invalid states, surface, seams.

Attribution: the deep-module vocabulary (depth, seam, locality, leverage), the deletion test and
the one-adapter-is-hypothetical rule are adapted from Matt Pocock's `codebase-design` and
`improve-codebase-architecture` skills, https://github.com/mattpocock/skills (paths
`skills/engineering/codebase-design/SKILL.md`, `skills/engineering/improve-codebase-architecture/SKILL.md`),
commit 6f7f7831a625f040a8e06a0e7954826b87ce775a, MIT License, Copyright (c) 2026 Matt Pocock;
adapted, not copied verbatim. The MIT permission notice applies to that adapted material.
