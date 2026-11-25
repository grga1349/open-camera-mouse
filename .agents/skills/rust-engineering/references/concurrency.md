# Concurrency and shutdown (std threads only)

Reference for `rust-engineering`. Ground truth: `apps/desktop/src/engine.rs` (ADR-0016),
`crates/platform/src/hotkeys.rs`, `crates/platform/src/camera/mod.rs`.

## Model

No async runtime. Threads communicate by `mpsc` messages and a few shared slots.
Decide per piece of data: owned by one thread (default), sent as a message, or shared
read-mostly behind `Arc<Mutex<_>>`. Do not share mutable state "to be fast"; a message is
cheaper to reason about and the loop runs at camera rate (30 Hz), not at lock-contention
scale.

Existing pattern to copy:
- Engine thread owns the controller, tracker and pointer; the UI sends `EngineCmd` over a
  channel and reads `Arc<Snapshot>` from a `Mutex<Arc<Snapshot>>` (latest wins, clone the
  `Arc` out, release the lock).
- Camera thread (`ThreadedCamera`) reads frames into a one-slot `Mutex<Option<(session,
  Frame)>>`; a newer frame replaces an unread one and bumps an `AtomicU64` `replaced`
  counter. A `session` id tags frames so a late frame from a closed session is ignored.
- Commands to the camera worker: `Open` / `Close` / `Quit` via channel.

## Locks

- Poisoning: a panic while holding a lock poisons it. The house policy is
  `lock().unwrap_or_else(PoisonError::into_inner)`: the data is a plain snapshot or slot with
  no cross-field invariant that a half-finished update could break, and a poisoned UI is
  worse than a stale value. Use the same form; do not `unwrap()` a lock. If a protected
  value does have a multi-field invariant, make the update atomic (build the new value
  outside, swap under the lock) rather than relying on poison.
- Hold a guard for the shortest span: copy or clone out, drop, then work. Never hold one across
  a camera read, `send`/`recv`, `join`, a callback, or the egui repaint request (`wake()`).
- One lock at a time. If two are ever needed, document the order next to the fields.
- `Mutex` over `RwLock` unless readers are measurably contended; `try_lock` only with a
  defined fallback.

## Atomics

- A counter or flag read only for display or stats (`replaced`, drop counters): `Relaxed`.
- A flag that publishes other data (a "ready" bool after writing a buffer): `Release` on the
  store, `Acquire` on the load, or just put the data and flag under one mutex.
- `SeqCst` where simple and not hot (the manual test `Clock` uses it); do not mix orderings
  on one variable without a comment.
- No atomic-based hand-rolled lock or queue. Use `Mutex`/`mpsc`.

## Channels

- `mpsc::channel` for commands (small, infrequent). Frames do not go through an unbounded
  channel: use the latest-wins slot, or `sync_channel(1)` with a documented drop policy.
- `send` errors mean the receiver is gone (shutting down): ignore with `let _ =` and a reason,
  as `Drop` does. A worker's `recv`/`recv_timeout` returning `Disconnected` means exit the
  loop. Use `recv_timeout` where the thread also has periodic work or must notice a stop.
- Requests that need an answer (e.g. initial open) use a one-shot `sync_channel(1)`.

## Non-Send OS handles

Some handles must stay on the thread that made them: the macOS `CGEventSource` inside
`SystemPointer` is not `Send`, and the type is documented that way. Rules:
- Create it on the thread that uses it (inside the engine thread's closure), not on the UI
  thread and then moved. Pass a `Send` factory/constructor, not the handle.
- Never add `unsafe impl Send` to make it compile; if the borrow checker says "cannot be
  sent", the design question is "which thread should own this?".
- Camera handles: nokhwa devices are opened, read and closed on the camera worker only.
- Global hotkeys (`global-hotkey`) have platform thread/event-loop requirements; follow
  `hotkeys.rs` and say which OS was not run.

## Ownership, Drop, shutdown

- Every owner of a thread holds `Option<JoinHandle<()>>` and a stop path, and implements
  `Drop`: send `Quit`, `take()` the handle, `join()` (see `ThreadedCamera`, `EngineThread`).
  Drop must be bounded: join within a bound. If a blocking FFI call can hang (nokhwa
  `frame()` has no timeout; an open that exceeds `OPEN_TIMEOUT` leaves its thread), wait a
  capped time (`CLOSE_WAIT`) then detach, as `NokhwaCamera::close` does, rather than join
  forever. Reads time out (`FRAME_TIMEOUT`). An unbounded join is a bug.
- Drop order matters: dropping the engine drops the camera, which turns the camera light off.
  Do not leak the camera to a detached thread.
- Do not `panic!` in a worker without a plan: a panicking worker leaves the UI with no
  frames. Wrap panicking FFI-backed libraries in `catch_unwind` and map the panic to an error
  (`camera/mod.rs`, around nokhwa calls). Convert device errors to messages (`Err` -> report -> close session) as the worker
  already does; `join()` result can be inspected and logged.
- No `thread::sleep` polling for state; use channel timeouts or condvars.

## Checks

- Controller logic is tested without threads (`crates/app-core/tests/controller.rs`); keep
  new logic in the pure layer so it needs no concurrency test. UI tests use `InlineEngine`.
- A shutdown or ordering fix needs a test that fails before: e.g. dropping the handle joins
  within a bound, a stale-session frame is ignored, a poisoned lock still yields a value.
- Compile-time guard for a type that must not cross threads: `static_assertions` is not
  available; use a documented comment, or a compile-fail doc test only if cheap.
- Real-device behaviour (camera release, hotkeys per OS) is a by-hand check: state it.

Ideas on Send/Sync and channel-vs-shared-state selection are compatible with Actionbook's
`m07-concurrency` (https://github.com/actionbook/rust-skills, commit
5c40d3ad785193231b7d0dbfb8e1eb447e5edd94); that repository has no licence file, so ideas only,
no text used.
