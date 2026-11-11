# Current state

Last updated: 2026-11-11 (task 0004 in progress)

## Branch / commit

Branch: `rewrite/rust`. Last commit `d5dc02d` (task 0003 close). All Phase 2 work below is
**uncommitted** in the working tree.

## Milestone

Phases 0–1 DONE. Phase 2 (Rust architecture) ACTIVE — `tasks/active/0004-phase-2-rust-architecture.md`.
Everything except the CI gate is done locally; see the task's Progress section for evidence.

## What exists now

- Cargo workspace (ADR-0007): `crates/{tracking-core,pointer-core,app-core,platform}`,
  `apps/desktop` (package `open-camera-mouse`, eframe 0.37). Toolchain pinned 1.99.1.
- `tracking-core`: `GrayFrame`, `ingest` (mirror at ingest, ADR-0008), algorithm-neutral
  `Tracker` trait + status/diagnostics types. No algorithm.
- `pointer-core`: ADR-0006 absolute mapping with virtual-desktop clamp.
- `app-core`: R-A1 `AppState`, `Command`, `Event`, `Controller` skeleton; `ports`
  (`CameraSource`, `PointerSink`, `CaptureRequest` 640×480@30).
- `platform`: per-OS cfg modules, placeholders only, no OS crates.
- CI `.github/workflows/ci.yml`, `deny.toml`, `scripts/check-core-deps.sh`.
- Docs: ADR-0007, ADR-0008, `docs/architecture/DEPENDENCIES.md`, `PLATFORM_RISKS.md`.

## Machine setup done this session

rustup installed to `~/.cargo` / `~/.rustup` with `--no-modify-path` (shell PATH not changed;
use `export PATH="$HOME/.cargo/bin:$PATH"` or `. "$HOME/.cargo/env"`). Targets
`x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu` added for cross `cargo check`.

## Immediate next action

Ask the user to commit + push, then watch the CI run. Fix any Windows/Linux/cargo-deny
failure. When all jobs are green: close task 0004, mark Phase 2 DONE in `docs/ROADMAP.md`.

## Open risks

- CI has never run; native Windows/Linux builds and `cargo deny` are unverified.
- Camera crate (nokhwa) has open macOS panics (PLATFORM_RISKS P1) — Phase 6 spike.
- Wayland pointer injection/position (P10/P11) may force X11-only on Linux for v1.
- Carried: 2018 dwell / CenterDetect / native LK internals UNRESOLVED (BEHAVIOR_SPEC §9).
