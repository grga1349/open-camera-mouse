# Ownership, borrowing and the hot path

Reference for `rust-engineering`.

## Ownership and borrowing

- Parameters take the weakest type that works: `&str`, `&[T]`, `&T`; take ownership only when
  the callee stores or consumes it. Small `Copy` types (points, rects, ids) go by value.
- A `.clone()` must be justified by a stored or sent value, not by a borrow-checker error.
  First try: reorder, narrow the borrow scope, split the struct so fields borrow
  independently, or move the value and return it.
- Return owned values from constructors and snapshots; return borrows from accessors. Use
  `Cow` only where both cases really occur.
- Prefer iterators over index loops where it reads better, but no intermediate `collect()`
  just to iterate again (`clippy::needless_collect`). In numeric pixel code, plain indexed
  loops over slices are fine when the iterator form is slower or unreadable; measure.

## Hot path and allocation

- The per-frame loop (capture -> mirror/convert -> track -> pointer step -> publish) reuses
  buffers: allocate pyramids, scratch and frame buffers once, `clear()` and refill, take
  `&mut [T]` outputs instead of returning fresh `Vec`s. No `format!`, `to_string()`,
  `collect()` into a new `Vec`, or `Box` per frame.
- Measure before optimising: reproduce with `OCM_PERF` or a divan bench (see `benchmarking`),
  change one thing, compare. Keep a simple version if the gain is within noise. A fix gets an
  automated check (AGENTS.md); an allocation claim needs a counting test or bench, not a
  guess.
- Sharing a frame between threads: `Arc` the immutable frame and hand out clones of the
  `Arc`, never the pixels. Latest-wins slots (`engine.rs` frame slot with a `replaced`
  counter) beat unbounded queues for video.
- Clone cost to keep in mind: `Vec`/`String`/`Image` clone = alloc + copy; `Arc` clone =
  atomic increment. Settings structs are small and cloned on change, not per frame.
