---
name: rust-engineering
description: Idiomatic Rust implementation — ownership/borrowing, error types, API surface, lint/test gates. Use when writing or reviewing any Rust source in crates/ or apps/desktop. Not for algorithm design (see computer-vision) or architecture-level boundary decisions (see rust-architecture).
---

## Objective

Write Rust that follows standard idioms: accept interfaces (traits) at boundaries, return
concrete structs; prefer composition; typed errors (`thiserror`/custom enums) over stringly
errors; minimal `unsafe`, and never without a written rationale comment at the unsafe block.

## When to use

Any time you're writing or reviewing Rust implementation code, once Phase 2 creates the
workspace. Not useful before that — there's no Rust to apply this to yet.

## When not to use

Don't reach for this for algorithm correctness (tracking math) or workspace-level dependency
direction — those are `computer-vision` and `rust-architecture` respectively.

## Deliverables and gates

`cargo fmt --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean,
relevant `cargo test` passing, no unexplained `unsafe`, public API has rustdoc comments on
anything exported from a crate's `lib.rs`.
