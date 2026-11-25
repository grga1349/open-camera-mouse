# Code health audit checklist

Read-only. Pick a scope first: a module the user named, or the hot spots from
`git log --oneline` and `git log --stat` (files that keep changing, files touched by several
fixes). Read the ADRs and docs for the area (`docs/ARCHITECTURE.md`, `docs/architecture/decisions/`)
so you do not re-litigate a recorded decision; if a finding contradicts an ADR, say so and say
why the friction is real enough to reopen it.

## Signals to collect (each needs a pointer, not an impression)

1. **Repeat fixes.** `git log -p` / `git log --grep` on the area: the same kind of fix in two or
   more commits, or one fix copied to several call sites. Review findings count.
2. **Hard changes.** A recent change that touched many files or modules for one behaviour.
3. **Untestable logic.** A rule that can only be exercised through the whole app, a thread, or a
   real device. Check whether the controller rig or kittest could reach it.
4. **Hand-synchronised state.** Two or more fields, enums or flags that must agree (for example
   `AppState` + private `Control` + `PointerAccess` + `recenter_deadline_us` + `last_commanded`
   in `controller.rs`). List: which transitions reset which fields; which combinations are
   reachable; which are meaningless; whether a test pins them.
5. **Shallow modules.** Pass-throughs, wrappers whose interface is as large as the body. Apply
   the deletion test.
6. **Surface.** `pub` items nothing outside the crate uses (try narrowing, let the compiler
   answer). Public fields that let callers break an invariant.
7. **Duplication.** Same logic in two places, and whether the two change for the same reason.
8. **Control flow.** Deep nesting, flag variables restating state, side effects whose order
   matters but is implicit.
9. **Ownership and threads.** State shared across the engine thread and the UI via locks where a
   message would do (ADR-0016); fields owned by the wrong module.

## Rejected as findings

Long function with a clear linear flow; "should be a trait" with one implementation; naming or
formatting preferences; anything only justified by "cleaner" or "more extensible"; code that
nobody needs to change.

## Report format

Per finding:

- **Title** and **files** (paths, functions).
- **Evidence:** commits, review finding, test that cannot be written, with references.
- **Problem** in one or two sentences, using depth / locality / invalid-state terms.
- **Proposed change** at the level of moves (not an interface design yet), and what it will not touch.
- **Behaviour risk** and **characterisation tests needed first** (which rig, which scenarios).
- **Hot path?** yes/no, and the `OCM_PERF` or benchmark plan if yes.
- **ADR needed?** yes/no (crate boundary, dependency direction, public crate API: yes).
- **Strength:** strong (repeated cost, clear fix), worth exploring, speculative (do not do now).

End with a proposed order (cheapest evidence-backed step first; tests-only steps before moves),
the findings that were considered and dropped, and the question: which of these, if any, should
be done. Do not start any of them.
