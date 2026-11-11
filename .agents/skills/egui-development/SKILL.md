---
name: egui-development
description: eframe/egui lifecycle, immediate-mode state patterns, AccessKit-based accessibility, egui_kittest interaction tests. Use for apps/desktop UI work (Phase 5-6 onward). Not for the app-core command/state controller it renders (see rust-architecture) or OS-level camera/cursor adapters (see platform-integration).
---

## Objective

Keep `apps/desktop` thin: it renders `app-core` state and sends `app-core` commands, it does
not own tracking/pointer/camera logic. Use `egui_kittest` (https://docs.rs/egui_kittest/latest/egui_kittest/)
to drive controls by accessible label and assert on model state — this is more token-efficient
and more deterministic than AI-driven screenshot analysis; prefer it over ad hoc manual
screenshot review for anything that can be expressed as a state assertion.

## When to use

Building settings/preview/status UI, or any interaction test for it. Not useful before Phase
5/6 — there's no egui app yet.

## When not to use

Don't use this skill for the underlying state machine itself; that's `app-core`
(`rust-architecture`), testable with plain `cargo test` and no GUI framework.

## Deliverables and gates

State-driven UI (no hidden widget-local state that `app-core` doesn't know about);
`egui_kittest` interaction tests for each control; accessible labels on every interactive
element; optional visual snapshot tests with a pinned renderer version and human review on
change, not auto-accept.
