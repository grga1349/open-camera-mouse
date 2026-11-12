# Dependency decisions

Phase 2 rationale (task 0004). Versions and licenses come from the crates.io API and the
GitHub repos, queried 2026-11-11, not from memory. Re-query before adopting anything marked
*candidate*. Policy is enforced by `deny.toml` (license allowlist, OpenCV ban, registry
sources) in CI.

Status key: **adopted** = in `Cargo.toml` now · **candidate** = chosen direction, added
in the phase that needs it, after a spike · **rejected**.

## Adopted

| Crate | Version | License | Where | Why |
|---|---|---|---|---|
| `eframe` / `egui` | 0.37.0 (2026-10-15) | MIT OR Apache-2.0 | apps/desktop | Product decision (egui). Default features: wgpu renderer, winit with x11 + wayland, AccessKit. MSRV 1.95 sets the workspace MSRV. |
| `egui_kittest` | 0.37.0 | MIT OR Apache-2.0 | apps/desktop (dev) | Headless UI interaction tests by label. Needs no GPU unless the `wgpu` snapshot feature is enabled, which it isn't. |
| `divan` | 0.1.21 (2025-04-10) | MIT OR Apache-2.0 | tracking-core (dev) | Benchmarks with a built-in allocation profiler, which the §8 zero-allocation target needs; Criterion has none. Chosen over criterion 0.8.2 in Phase 3 despite no release since 2025-04. Swap if it stops building. |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | tracking-core, app-core | Typed errors. Proc-macro only, adds nothing at runtime. The single external crate allowed in the pure cores. |

The full transitive graph (~240 packages, nearly all from eframe) is permissive. It also
includes OFL-1.1 / Ubuntu-font-1.0 for egui's bundled fonts, which `deny.toml` allows
explicitly.

## Image representation

No image crate in the cores. `tracking_core::GrayFrame` is an owned, tightly packed,
row-major `Vec<u8>` luma buffer with checked construction. Reasons:
- The tracker needs only 8-bit luma (both 2018 LK and legacy NCC run on gray).
- The cores must have no dependencies (ADR-0007), and `image` 0.25.10 brings
  codecs we don't need there.
- Most capture formats (NV12, YUYV) have a luma plane, so adapters can fill the buffer
  without colour conversion.

Pyramids and float buffers are added in Phase 3 inside tracking-core. The preview is
**colour** (user decision 2026-11-12): the adapter also delivers an RGB(A) frame for the UI
(Phase 6 D3), outside the cores.

## Candidates (by phase)

| Need | Candidate | Version / license | Notes and risk | Fallback |
|---|---|---|---|---|
| Camera, all OS | `nokhwa` with `input-native` only | 0.10.11 (2026-05-15), Apache-2.0 | Only maintained cross-platform crate (AVFoundation / MSMF / V4L2). **Risk:** open macOS bugs with panics and SIGABRT on format/frame-rate selection (#247, #180, #202, fix PRs #248/#249 unmerged), wrong FPS on Windows (#110, #144), camera not released on macOS (#224). Its optional `input-opencv` feature must stay off, and `deny.toml` bans `opencv`. | Thin per-OS adapters: `objc2-av-foundation` 0.3.2 (macOS), `windows` 0.62.2 Media Foundation, `v4l` 0.14 (Linux, last release 2023). Phase 6 spike decides. |
| Cursor, macOS | `objc2-core-graphics` 0.3.2 (`CGWarpMouseCursorPosition`, `CGEvent`) | Zlib OR Apache-2.0 OR MIT | Direct calls. Posting events needs Accessibility permission. Whether warping does is **unverified**, see PLATFORM_RISKS. | `enigo` |
| Cursor, Windows | `windows-sys` 0.61.2 (`SendInput` absolute on the virtual desk, `GetCursorPos`) | MIT OR Apache-2.0 | Well understood. | `enigo` |
| Cursor, Linux | `x11rb` 0.14.0 (XTest) on X11. `ashpd` 0.14.0 RemoteDesktop portal + libei on Wayland | MIT OR Apache-2.0 / MIT | Wayland absolute pointer depends on the compositor and needs user consent per session. | Document X11-only for v1 if the portal path fails |
| Cursor, cross-platform | `enigo` | 0.6.1 (2025-08-28), MIT | Active repo, but the last release is over a year old, and we'd still need per-OS code for reading the position and for permissions. Kept as the fallback, not the default. | — |
| Global hotkeys F11/F12 | `global-hotkey` (Tauri) | 0.10.0 (2026-11-09), Apache-2.0 OR MIT | macOS uses Carbon hotkeys (no Accessibility needed, INFERRED). Windows works. Linux supports X11 only. | Wayland: portal GlobalShortcuts via `ashpd`, or in-app keys only |
| Settings persistence | `serde` 1.0.229 + `toml` 1.2.1 + `directories` 6.0.0 | MIT OR Apache-2.0 | Schema-versioned file (R-A5). | — |
| Property tests | `proptest` 1.12.0 | MIT OR Apache-2.0 | Phase 3 tracker invariants. | — |
| Snapshot tests | `insta` 1.49.0 | Apache-2.0 | Optional, for diagnostics/golden outputs. | — |

## Rejected

- Anything using OpenCV (AGENTS.md), including `nokhwa`'s `input-opencv` feature.
- `camera_capture` (last release 2017), `kamera` 0.0.2, `eye` 0.5 (dormant), `rdev`
  0.5.3 (2023), `mouse-rs` (2022): unmaintained or too immature.
