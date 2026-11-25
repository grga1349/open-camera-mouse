# unsafe and FFI (platform crate only)

Reference for `rust-engineering`. Policy (ADR-0007): workspace `unsafe_code = "deny"`;
`platform` adapters use a local `allow(unsafe_code)` (item-level where possible, file-level
only for raw `windows-sys` files) with a reason comment; every
`unsafe` block has a `// SAFETY:` comment. No other crate gets unsafe; ask first.

## First, avoid it

1. Use a safe wrapper that exists: `objc2-core-graphics` / `objc2-core-foundation` expose safe
   functions for most calls in `pointer/macos.rs` (the few unsafe items there carry
   item-level `#[allow(unsafe_code)]`, prefer that form to a file-wide allow); `x11rb` is
   pure Rust. `windows-sys` is raw, hence the file-wide allow in `pointer/windows.rs`.
2. Otherwise isolate: one small private function per FFI call, returning `Result` or
   `Option`, so the unsafe block sits next to the one call it justifies and the rest of the
   adapter is safe code. Do not widen `unsafe` blocks to include logic.
3. Check the real signature in the pinned crate source (`~/.cargo/registry/src/*/windows-sys-0.61.2`),
   not from memory; types, nullability and `BOOL` conventions differ.

## House style (see `windows.rs`)

```rust
// SAFETY: `p` is a valid, writable POINT for the duration of the call.
let ok = unsafe { GetCursorPos(&mut p) };
```
```rust
// SAFETY: `inputs` is a valid array of `inputs.len()` INPUT structs, size passed
// correctly; SendInput only reads it during the call.
```
```rust
// SAFETY: `data` is the `&mut Vec<DesktopRect>` passed by `monitors`, alive and not
// aliased while the enumeration runs.
```

A SAFETY comment is a short proof. It names, for this exact call:
1. what is being asserted (the operation and the contract of the callee),
2. why each premise holds here (a local fact: the value is a live local, length equals what
   is passed, pointer not null because it came from `&mut`, struct size field set),
3. for pointers: validity, alignment, initialisation, aliasing, and the lifetime (only for the
   duration of the call, or until a known drop),
4. for callbacks: what the callee may call back, that the context pointer is the type the
   callback casts to, and that the callback does not unwind across the FFI boundary.

Reject comments such as "pointer is valid", "caller guarantees it", "this is fine", "Miri
passes". State the reason, not the conclusion. For `unsafe fn` (e.g. an `extern "system"`
callback), document a `# Safety` section that names who must uphold what.

## Hazards in this repo

- Return codes: Win32 `BOOL`/count results (`SendInput` returns how many were sent) must be
  checked and turned into `PointerError::Backend`; partial success is failure. macOS calls
  that silently drop without Accessibility must be probed (`CGPreflightPostEventAccess`) by
  design, not assumed.
- Unwinding across FFI is undefined behaviour: callbacks passed to C (`collect_monitor`) must
  not panic; no `unwrap`, indexing that can fail, or `Vec` growth that can abort-by-panic
  without thought. Keep the callback tiny.
- Casts: `x as i32` on a float truncates and saturates; the code rounds and clamps first
  (`geometry::snap`, `finite`). Check ranges before narrowing integer casts passed to FFI.
- `size_of::<T>() as i32` struct-size arguments: use the right type, and set `cbSize` fields.
- Lifetimes of buffers handed to the OS: only for the call unless the API documents
  retention; if it does, own the buffer in a struct that outlives the registration.
- Thread affinity: CoreGraphics event source is not `Send` (see `references/concurrency.md`);
  X11 connections are not shared across threads without care. Do not "fix" with
  `unsafe impl Send`.
- Resources: pair every acquire with a release in `Drop` (`CFRetained` does it for you; raw
  handles need a wrapper with `Drop`). Release on error paths too.
- Untestable on the dev Mac: Windows and Linux unsafe code compiles in CI but is not run
  locally. Say so, and list what to check on-device.

## Review checklist

- Is unsafe needed at all (safe API in the pinned crate)? Is it in `platform` only?
- One call per block, minimal scope, comment proves each premise for this call site?
- Pointer/length/alignment/init/aliasing/lifetime stated where relevant?
- Return value checked, errors mapped, no panic across FFI?
- The local `allow(unsafe_code)` has its reason comment (house style is `allow`, not `expect`); no wider allow added.
- A test for the safe parts (geometry, echo suppression, conversion) in the pure/geometry
  module; `cargo clippy -D warnings` clean on the host OS; other-OS compile via CI.
- Optional, only if tools exist locally: Miri on pure helpers. Miri passing is evidence, not
  a proof; the SAFETY comment still has to hold.

The proof-obligation framing (operation, contract, premises, program-point validity) is
drawn from Google's `unsafe-rust-review` skill (https://github.com/google/rust-skills,
`unsafe_rust_review/SKILL.md`, commit 34b87c594bf7386290a1e3698d5b8e06b6b9edd7,
Apache-2.0, adapted/condensed; the upstream is ~90 KB and not copied).
