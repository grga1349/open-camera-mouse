# Plan review

For substantial or risky plans and significant plan changes (`.agents/PLANNING.md`). The
reviewer is independent (strong tier, read-only, `.agents/MODELS.md`) and gets the plan,
the requirements and the ADRs it touches, not the planner's reasoning. Without subagents,
the main agent does it as a separate pass and reports it as self-review.

## Brief (adapt the paths)

```text
You are a senior engineer about to execute this plan without access to its author.
Read <roadmap or plan>, the requirements it cites and <ADRs>. Read-only.
Report only:
1. Questions you would have to guess at, where a wrong guess changes what gets built
   (most important first; say why each matters).
2. Findings from the checklist below, each with the item id and a concrete consequence.
Skip wording nits and anything the plan explicitly leaves to the implementer.
If nothing important is left, say so.
```

## Checklist

- **Coverage:** every requirement, symptom and user decision maps to an item; no item
  without a requirement (scope creep).
- **Order:** the riskiest unknowns and the user decisions/device runs on the critical path
  come first; `Needs` are real and acyclic.
- **Items:** outcomes, not layers or file lists; one session each; criteria observable;
  nothing that decides nothing ("handle edge cases", "add tests").
- **Verification:** a failing-first test per fix at the right layer; mandatory levels named
  (CI per OS, device per OS, physical vs synthetic input, camera); no "verified" a device run
  hasn't earned.
- **Decisions:** product questions are `D`-rows with recommendations; architecture changes
  have an ADR before code; no default, hotkey or visual change hidden in an item.
- **Failure modes:** inputs or states the requirements imply but no test exercises.
- **Proportion:** the plan is shorter than the code it describes.

## Handling the result

Sort the questions into: already answered (put the answer in the plan), minor (decide and
mark it as the agent's call), the user's (one message, most important first). Findings
change the plan before approval; the main agent decides which to accept and says why.

Adapted from the builder check in obra/superpowers `brainstorming` (commit 22f0b46, MIT,
Copyright (c) 2025 Jesse Vincent).
