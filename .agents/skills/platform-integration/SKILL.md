---
name: platform-integration
description: Camera capture, OS cursor injection, permissions adapters per platform (macOS/Windows/Linux), with fake-adapter unit tests and a manual smoke-test checklist. Use for platform crate work. Not for the pure tracking/pointer math that platform adapters feed (see computer-vision, rust-architecture).
---

## Objective

Each platform's camera/cursor/permission behavior lives behind a trait, implemented per-OS
behind build tags/cfg (`macos.rs`/`windows.rs`/`linux.rs`), with a fake implementation for
tests. No physical-device test by default in CI — those are a separate, explicit, infrequent
manual smoke pass per OS, not a hard merge gate.

## When to use

Implementing or reviewing `platform` crate adapters; deciding camera backend
(nokhwa vs. direct AVFoundation/Media Foundation/V4L2) or cursor injection approach per OS;
Wayland-specific limitations; multi-monitor/high-DPI coordinate handling.

## When not to use

Don't use this for the tracking algorithm or UI — only for the OS-facing adapter layer.

## Deliverables and gates

An adapter trait + fake implementation with unit tests covering the contract; a documented
manual OS smoke-test checklist (permissions prompt, capture starts, cursor moves, pause
works); licensing/maintenance check on any chosen camera backend crate before adopting it.
