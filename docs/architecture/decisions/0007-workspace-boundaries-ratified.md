# ADR-0007: Workspace boundaries ratified, with ports owned by app-core

Status: Accepted (amended by ADR-0012: `app-core` may also depend on `serde` derive)
Date: 2026-11-11
Supersedes: ADR-0003 (Proposed) in part. The crate set stays the same. The dependency
direction changes.

## Context

Phase 2 (task 0004) had to ratify or revise ADR-0003. ADR-0003 and the `rust-architecture`
skill described the direction as `apps/desktop → app-core → {tracking-core, pointer-core,
platform}`. With that direction, app-core's unit tests compile every OS binding that
`platform` brings in (camera, cursor, hotkeys). Those tests are the webcam-free controller
tests the Phase 2 gate exists to protect, and adding real adapters in Phase 6 would make
them slower and fragile per OS.

## Options considered

- **ADR-0003 as written** (`app-core → platform`). Simple, but the controller's test build
  inherits every OS dependency. Fakes would live in `platform`, next to the real adapters.
- **Ports in app-core** (`platform → app-core`). app-core defines the `CameraSource` /
  `PointerSink` traits and their types. platform implements them. apps/desktop wires the two
  together. app-core tests use in-crate fakes and never build an OS crate.
- **A fifth `ports` crate** shared by app-core and platform. This gives the same isolation
  but adds a crate with no logic, which is the kind of ceremony ADR-0003 warned about.
- **Merge app-core into apps/desktop.** Rejected: that ties controller tests to egui.

## Decision

Keep the four crates and one app from ADR-0003, and own the ports in app-core:

```
apps/desktop ──► app-core ──► pointer-core ──► tracking-core
      │             ▲
      └──► platform ┘          (platform implements app-core::ports)
```

- `tracking-core`, `pointer-core`: pure. Their allowlist of normal dependencies is enforced
  by `scripts/check-core-deps.sh` in CI. Today the only external crate is `thiserror`.
- `app-core`: no GUI and no OS crates. Owns the state machine, commands, events and ports.
- `platform`: OS adapters, with per-target deps under `[target.'cfg(...)'.dependencies]`.
- `apps/desktop` (package `open-camera-mouse`): egui/eframe only. UI functions take
  `&mut egui::Ui` so `egui_kittest` drives them headlessly.

Toolchain: edition 2024, resolver 3, workspace MSRV 1.95 (egui 0.37's MSRV). The repo pins
the compiler to 1.99.1 in `rust-toolchain.toml` so local and CI builds match. `unsafe_code`
is denied workspace-wide. A crate that needs FFI (platform) opts out locally, with a
comment.

## Consequences

- The `rust-architecture` skill's direction line is updated to match this ADR.
- Fakes for controller tests live in app-core (`#[cfg(test)]` today, a `testing` module or
  feature if apps/desktop needs them later).
- Verified by: `cargo tree` per crate (task 0004 log), CI `core crate boundary` step.
