# What to look for (full criteria)

- **Depth.** A module is deep when much behaviour sits behind a small interface (few methods,
  simple parameters, invariants the caller need not know). Deletion test: if removing a module
  makes its complexity vanish, it was a pass-through; if the complexity reappears at every
  caller, it earns its keep. Merge pass-throughs; do not add layers.
- **Locality.** A rule or invariant should live in one place. If fixing one behaviour means editing
  several modules, or a pure helper was extracted for tests while the bugs hide in how it is
  called, the seam is in the wrong place.
- **Invalid states.** When two enums or flags must be kept consistent by hand, ask whether an
  invalid combination can be constructed or reached. Fix by making it unrepresentable (one enum
  carrying its data, a single transition function that resets everything a state owns) or by a
  characterisation test that pins the allowed combinations. Prefer one transition path over
  many call sites that each remember the right resets.
- **Public surface.** Narrow `pub` to `pub(crate)` or private when nothing outside uses it; check
  with the compiler, not by guessing. A crate-level API change that other crates depend on is
  `rust-architecture` territory.
- **Large functions or modules.** Split only along a real seam: separable responsibilities,
  separable tests, a different reason to change. No size targets. A long straight-line `match`
  on a state machine can be correct as is; extraction that needs many parameters or a shared
  `&mut self` borrow usually only moves the complexity.
- **Control flow.** Early returns, `let ... else`, flattening nested `match`/`if let`, removing
  flag variables that restate a state. Keep the order of side effects identical.
- **Ownership.** Who owns the state, who may mutate it, which thread. Move a field to the
  module that enforces its invariant; do not add `Arc<Mutex<_>>` to avoid a design choice.
- **Duplication vs abstraction.** Merge duplicates that must change together. Leave duplicates that
  change independently; two similar lines are cheaper than a wrong abstraction.
- **Seams.** One implementation is a hypothetical seam, so no trait for it. A trait or port needs
  two real implementations (the production adapter plus a fake that tests actually use, as with
  the app-core ports in ADR-0007). Do not add generics, builders or traits "for later".

