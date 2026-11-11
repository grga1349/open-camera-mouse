---
name: rust-architecture
description: Cargo workspace boundaries, dependency direction, public API surface, ADR authorship. Use when adding/splitting a crate, changing what depends on what, or any decision that should become an ADR under docs/architecture/decisions/. Not for day-to-day implementation inside an already-decided boundary (see rust-engineering).
---

## Objective

Keep `tracking-core`/`pointer-core` free of I/O, GUI and OS imports; keep dependency direction
one-way (`apps/desktop` → `app-core` → `{tracking-core, pointer-core, platform}`, never
reversed); decide crate splits on demonstrated need, not speculative future-proofing.

## When to use

Before creating a new crate, before adding a dependency that crosses a layer boundary, before
changing what `tracking-core`/`pointer-core` are allowed to import.

## When not to use

Routine feature work inside one crate that doesn't change its boundary or dependencies.

## Deliverables and gates

An ADR in `docs/architecture/decisions/` (context, options, decision, consequences) for any
non-trivial boundary change — see `docs/architecture/decisions/0000-template.md`. A dependency
graph that stays acyclic. No ADR for trivia (renaming a private function doesn't need one).
