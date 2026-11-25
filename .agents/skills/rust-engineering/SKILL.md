---
name: rust-engineering
description: Idiomatic Rust for this workspace - ownership and borrowing, API and type design, errors and invariants, allocation cost on the per-frame path, threads/channels/locks, resource shutdown, unsafe/FFI contracts. Use when writing or reviewing Rust in crates/ or apps/desktop. Not for crate boundaries or ADRs (rust-architecture), tracking maths (computer-vision), test design (rust-testing), timing methodology (benchmarking), or restructuring existing code (rust-refactoring).
---

## Objective

Write Rust that is idiomatic, makes wrong states hard to express, and keeps the per-frame
camera -> tracker -> pointer loop cheap and predictable. Stay inside the existing
dependencies (no new crates, no async runtime, ADR-0007) and the workspace lints
(`unsafe_code = "deny"`, clippy `-D warnings`).

## When to use

Writing or reviewing Rust implementation code. Load a reference only when the change touches
its topic.

## When not to use

Crate/dependency direction and ADRs: `rust-architecture`. Algorithm correctness:
`computer-vision`. How to test: `rust-testing`. How to time: `benchmarking`. Changing
existing structure without changing behaviour: `rust-refactoring`.

## Ownership and borrowing

Weakest parameter type that works (`&str`, `&[T]`, `&T`; `Copy` types by value); a `.clone()`
needs a stored or sent value as justification, not a borrow-checker error; no intermediate
`collect()`. Details in `references/ownership-hot-path.md`.

## API and type design

- Make invalid states unrepresentable: an enum with data per state beats a struct of
  `Option`s plus a flag; the controller's state machine (`app-core/src/controller.rs`) is the
  model. Parse once at the boundary into a validated type (settings `validated()` in
  `app-core/src/settings.rs`), then pass the validated type inward.
- A newtype pays when two values of the same primitive type can be confused or carry an
  invariant (pixels vs screen points, a session id vs a frame counter, a clamped gain).
  It does not pay for a one-use local; do not wrap everything.
- Constructors that can fail return `Result` (see `SystemPointer::new`); keep fields private
  when an invariant exists. `#[non_exhaustive]` only for enums that are public API and really
  will grow.
- Static dispatch (generics) on the hot path, `dyn` at port boundaries (`PointerSink`,
  `Desktop`, camera link) where the platform picks the implementation at runtime. Do not add a
  trait with one implementation "for testing" unless the port already needs a fake.
- Derive `Debug` on every public type (`missing_debug_implementations` warns). Rustdoc on
  everything exported from a `lib.rs`; comments explain why (measured numbers, platform
  quirks), not what.

## Errors and invariants

- `thiserror` is already a workspace dependency of `tracking-core`, `app-core` and `platform`
  (e.g. `PointerError` in `app-core/src/ports.rs`): use small enums with `#[from]`/messages
  there. No `anyhow`; no new error crates. `apps/desktop` reports to the user, it does not
  invent a second error vocabulary.
- Fallible I/O, device, and parse paths return `Result`; propagate with `?` and add context
  at the adapter boundary (`Backend(format!("SendInput ...: {e}"))`). Do not `unwrap()` or
  `expect()` outside tests, except a documented infallible case with a message saying why.
- A violated internal invariant is a bug: `debug_assert!` or a test, not an error variant the
  user can see. A bad external value (NaN landmark, zero-size frame) is data: reject or clamp
  it at the boundary and say which.
- Errors on the per-frame path must not become a log line per frame; count and rate-limit.

## Hot path and allocation

The per-frame loop reuses buffers (allocate once, `clear()` and refill, `&mut [T]` outputs; no
`format!`, `to_string()`, `collect()` or `Box` per frame). Measure before optimising
(`OCM_PERF`, divan; see `benchmarking`); an allocation claim needs a counting test or bench.
Share frames as `Arc` clones, latest-wins slots over unbounded queues. Details in
`references/ownership-hot-path.md`.

## Concurrency and shutdown

Std threads, `mpsc`, `Mutex`, atomics only. Load `references/concurrency.md` for lock
poisoning, atomic ordering, channel shutdown, non-`Send` OS handles, and the camera/engine
thread ownership pattern.

Summary rules: one owner thread per OS handle; locks held briefly, never across a camera read,
a channel `send`/`recv`, or a callback; every spawned thread has a stop message and a `join`
in `Drop`; shutdown is bounded and closes the camera.

## unsafe and FFI

`unsafe` exists only in `platform`, behind a local `allow(unsafe_code)` (item-level where
possible, file-level only for raw `windows-sys` files) with a reason comment (ADR-0007), each block with a `// SAFETY:` comment. Load `references/unsafe-ffi.md`
for the house style, what a good SAFETY comment must state, and the review checklist. Prefer
a safe wrapper from `objc2-*`/`windows-sys`/`x11rb` over new raw pointer code; keep each
unsafe block to one call.

## Deliverables and gates

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked        # scope with -p / a filter locally
```

Plus: no `unwrap`/`expect` in non-test code without a reason, no unexplained `unsafe`, no
new dependency, rustdoc on new exports, and for hot-path or threading changes the test,
bench or `OCM_PERF` number that shows it. Say which OS an untestable change was not run on.
Prefer `#[expect(lint)]` with a reason over `#[allow]` when a lint must be silenced (not for
`unsafe_code` in `platform`, which uses `allow` by house style).

## References

- `references/ownership-hot-path.md`: borrowing/cloning rules, per-frame allocation rules.
- `references/concurrency.md`: load for threads, channels, locks, atomics, Drop/shutdown.
- `references/unsafe-ffi.md`: load for any `unsafe`, FFI, or non-`Send` handle change.

## Attribution

Guidance on borrowing vs cloning, error handling, performance and type-state is paraphrased
from Apollo GraphQL's `rust-best-practices` skill (https://github.com/apollographql/skills,
`skills/rust-best-practices`, commit 9dda1397730370aeb8df4ff8f2015c34c86b5003, MIT, adapted).
