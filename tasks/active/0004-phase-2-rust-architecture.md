# 0004 — Phase 2: Rust architecture

Status: ACTIVE (started 2026-11-11)
Owner/branch: `rewrite/rust`
Depends on: 0003 (done)

## Context

Phase 2 per `docs/ROADMAP.md`. Inputs: `docs/research/BEHAVIOR_SPEC.md` §7 (R-*) and §9
(Q-*), ADR-0003 (crate boundaries, Proposed), ADR-0004 (tracking candidates, Proposed),
ADR-0006 (favor 2018; absolute pointer; F11/F12 fixed).

## Objective

1. Research from current, live sources (not memory): Cargo workspace patterns, egui/eframe
   version, camera capture backends (macOS/Windows/Linux), cursor-injection and permission
   APIs per OS, global-hotkey crates, Rust image/frame representation, dependency licenses.
2. Record decisions as ADRs: ratify or supersede ADR-0003; dependency choices with
   rationale; Q-R5 (mirroring) and Q-R6 (camera resolution/FPS).
3. Write a platform risk matrix (`docs/architecture/PLATFORM_RISKS.md`).
4. Create the minimal compiling workspace: crates per the ratified boundary, pure
   `tracking-core`/`pointer-core` with no I/O deps, placeholder public APIs that do not
   assume LK or NCC (ADR-0004), unit tests that need no webcam.
5. CI (GitHub Actions) that builds and tests the skeleton on macOS, Windows and Linux and runs
   the `AGENTS.md` quality gates.

## Non-goals

No tracking algorithm (Phase 3). No real camera/cursor adapter implementation beyond trait
definitions and fakes (Phase 6/7). No egui UI beyond an empty window that compiles. No
design lab (Phase 5).

## Files/scope

New: root `Cargo.toml`, `rust-toolchain.toml`, `crates/*`, `apps/desktop`, `.github/workflows/`,
`docs/architecture/` (ADRs 0007+, `PLATFORM_RISKS.md`, `DEPENDENCIES.md`). Updates:
ADR-0003 status, BEHAVIOR_SPEC §9 (Q-R5/Q-R6 resolved), `docs/ROADMAP.md`. No changes to
`legacy/` or the seed archive.

## Acceptance criteria (ROADMAP Phase 2 gate)

- CI compiles the skeleton and runs unit tests with no webcam on macOS, Windows and Linux —
  evidenced by a green CI run, not a local build alone.
- A dependency decision rationale exists (`docs/architecture/DEPENDENCIES.md` + ADRs).
- A platform risk matrix exists.
- `tracking-core` and `pointer-core` have no camera/OS/GUI dependencies (checked via
  `cargo tree`).
- Q-R5 and Q-R6 are decided and recorded.

## Verification

`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` locally; `cargo tree -p tracking-core` / `-p pointer-core` show no
I/O crates; the CI run URL with all three OS jobs green.

## Progress (2026-11-11)

Done locally, not yet committed:
- Research → `docs/architecture/DEPENDENCIES.md` (crates.io/GitHub queried 2026-11-11),
  `docs/architecture/PLATFORM_RISKS.md` (P1–P20).
- ADR-0007 (supersedes ADR-0003: same crates, ports in app-core), ADR-0008 (Q-R5 mirror
  at ingest, default on; Q-R6 request 640×480@30, downsample to ≤ 640 wide).
- Workspace: `Cargo.toml`, `rust-toolchain.toml` (1.99.1), 4 crates + `apps/desktop`
  (eframe 0.37), `deny.toml`, `scripts/check-core-deps.sh`, `.github/workflows/ci.yml`.
- Local evidence (macOS arm64): `cargo fmt --check` ok; clippy `-D warnings` ok;
  `cargo test --workspace --locked` 14 passed / 0 failed (incl. one egui_kittest test);
  `check-core-deps.sh` ok; `cargo check --all-targets` ok for `x86_64-pc-windows-msvc`
  and `x86_64-unknown-linux-gnu`.

Remaining for the gate:
- Push and get a green CI run on all three OS jobs + `cargo deny` (not run locally:
  installing cargo-deny was blocked by the permission classifier).
