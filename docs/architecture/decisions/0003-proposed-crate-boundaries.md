# ADR-0003: Proposed Rust workspace crate boundaries

Status: Superseded by ADR-0007 (crate set kept; dependency direction changed: ports live in app-core)
Date: 2026-11-11

## Context

The planning seed's architecture doc (`docs/archive/rust-rewrite-seed/05-architecture-and-testing.md`)
proposed three crates; its later addendum
(`docs/archive/rust-rewrite-seed/10-rust-skills-architecture-testing-research.md`) revised
this to four crates plus the desktop app and explicitly superseded the earlier count, and
rejected a standalone `tracking-cli`. Phase 0 records the proposal so Phase 2 starts from a
documented rationale instead of re-deriving it, but does **not** create any `Cargo.toml` or
crate scaffolding — that would be jumping ahead of Phase 1's behavior spec.

## Options considered

- **Three crates** (`tracking-core`, `pointer-core`, `platform`) with app logic folded into
  `apps/desktop`. Simpler at first, but mixes UI composition with command/state/lifecycle
  ownership, which the seed's own "Graphite-style dispatcher" lesson argues against.
- **Four crates** (`tracking-core`, `pointer-core`, `app-core`, `platform`) + `apps/desktop`.
  Separates a pure, UI-less app controller (commands in, state/events out) from the egui
  composition layer, so the controller can be exercised by `cargo test` without any GUI.
- **A standalone `tracking-cli` binary** for manual algorithm testing. Explicitly rejected by
  the seed's addendum: deterministic `cargo test`/benchmark targets over synthetic and
  recorded frames cover this need without a second maintained entry point.

## Decision (proposed)

Adopt the four-crate layout plus one desktop app, no CLI:

- `tracking-core` — pure algorithm: frame view in, position/status/diagnostics out. No
  camera/OS/GUI/OpenCV imports.
- `pointer-core` — image→screen mapping, smoothing, limits, dwell-click state machine. Pure.
- `app-core` — commands (`StartTracking`, `SelectPoint`, `SetSensitivity`, ...), events, state,
  controller. No GUI framework dependency; this is what both the desktop app and integration
  tests drive.
- `platform` — camera capture, permissions, pointer injection, OS-specific adapters
  (`macos.rs`/`windows.rs`/`linux.rs` behind build tags/cfg).
- `apps/desktop` — egui/eframe composition only: renders `app-core` state, sends `app-core`
  commands. Thin by construction.

This is not yet validated against a real dependency graph or a compiling workspace. Phase 2's
gate (`docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md`) is: CI compiles the skeleton
and runs unit tests with no webcam required, with a documented platform risk matrix. Until
that gate passes, treat this crate split as a hypothesis, not a commitment — Phase 2 may merge
`app-core` back into `apps/desktop` if the split proves to add ceremony without payoff.

## Consequences

- No code changes from this ADR by itself.
- Phase 1 (behavior spec) should be written without assuming this crate split exists yet, so
  the spec stays reusable even if Phase 2 picks a different boundary.
- Phase 2 should either accept this ADR as-is (flip Status to Accepted) or supersede it with a
  new ADR recording what changed and why — not silently drift from it.
