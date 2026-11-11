# Open Camera Mouse — Rust rewrite

Canonical instructions for any AI agent (Claude Code, Codex, or other) working in this
repository. Keep this file short; link to topic docs instead of duplicating detail here.

## Status

This repository is mid-migration from a working Go/Wails/GoCV v0.3.0 application to a
from-scratch Rust desktop app. The old app is fully preserved under `legacy/` with build
instructions, history and its own former `CLAUDE.md`. **It is archived, not active policy.**

We are in **Phase 0 (foundation)** of the plan in
`docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md`. Phase 0 deliverables are tracked in
`tasks/`. Do not start implementing the Rust tracking engine until Phase 0 and Phase 1
(`docs/research/`) are done — see `agent/CURRENT_STATE.md` for exactly where things stand.

## Read before acting

1. `agent/CURRENT_STATE.md` — current branch, phase, in-flight task, last verified state.
2. `tasks/BACKLOG.md` and `tasks/active/` — what's next and its acceptance criteria.
3. The relevant topic doc under `docs/` (see map below) — don't guess architecture or RE facts.
4. `git status` — never assume the working tree matches the last handoff note.

## Repository map

| Path | Purpose |
|---|---|
| `legacy/` | Old Go/Wails/GoCV app, frozen as reference and regression baseline. Not built by default; see `legacy/README.md` to restore it. |
| `docs/product/` | Why this product exists, goals, non-goals, v1 scope. |
| `docs/research/` | Camera Mouse 2018 reverse-engineering evidence, labeled VERIFIED/INFERRED/HYPOTHESIS, and comparison against `legacy/`. |
| `docs/architecture/decisions/` | ADRs. Record before, not after, a non-trivial architectural choice. |
| `docs/design/` | UI design plan; the interactive HTML design lab lands here in Phase 5. |
| `docs/archive/rust-rewrite-seed/` | The original planning seed (ex-`dump/`). Historical input, not live instructions — see `SEED_CONSUMPTION_REPORT.md` for what was extracted where. |
| `tasks/` | `BACKLOG.md` + `active/` `done/` `blocked/` task files. Source of truth for what's being worked on. |
| `agent/` | `CURRENT_STATE.md`, `HANDOFF.md`, `WORKFLOW.md` — durable session state. Update before you stop, don't rely on chat compaction. |
| `.agents/skills/` | Project-local skill catalog (see table below). |

## Principles (carried from product decisions, see ADRs for detail)

- No OpenCV in the new tracking engine; write and test our own pure-Rust implementation.
- Rust workspace boundary (proposed, Phase 2): `tracking-core` (pure algorithm, no I/O),
  `pointer-core` (mapping/smoothing/dwell, pure), `app-core` (commands/state/controller),
  `platform` (camera/cursor/permissions, OS-specific), one `apps/desktop` (egui/eframe).
- Deterministic, offline-testable pipeline is a first-class requirement — no standalone
  `tracking-cli`; use `cargo test` / integration tests / benchmarks against synthetic or
  recorded frames instead.
- macOS is the primary dev target; Windows and Linux are required for release, not
  afterthoughts — don't make a decision (symlinks, paths, cursor APIs) that quietly assumes
  POSIX-only or macOS-only behavior. Validate cross-platform claims, don't assume them.
- Treat `../camera-mouse-reverse/` findings as behavioral research, not ground truth to copy
  verbatim. Never paste decompiled/proprietary code into this repository.
- One coordinator owns task state and integration at a time. Parallel agents get separate,
  bounded, non-overlapping file scopes (use worktrees for simultaneous edits).

## Skills

Project-local skills live in `.agents/skills/<name>/SKILL.md`. Discovery compatibility with
the CLI you're running in is **not yet verified for either client** — see
`tasks/active/0002-verify-ai-tooling-discovery.md` before assuming a skill auto-loads.

| Skill | Use for |
|---|---|
| `rust-engineering` | Idiomatic Rust, error types, API design, lint/test gates |
| `rust-architecture` | Workspace boundaries, dependency direction, ADRs |
| `computer-vision` | Grayscale/pyramids/interpolation/optical-flow math |
| `tracking-validation` | Synthetic fixtures, ground truth, error metrics, failure taxonomy |
| `rust-testing` | Unit/integration/property/golden test design |
| `benchmarking` | Perf measurement methodology, regression tracking |
| `egui-development` | eframe/egui patterns, `egui_kittest` interaction tests |
| `platform-integration` | Camera/cursor/permission adapters per OS, fake-adapter testing |
| `agent-handoff` | Writing `agent/` state before context loss |
| `code-review` | Independent review: correctness, failure modes, evidence over assertion |

## Instruction file compatibility

`CLAUDE.md` at repo root is a **plain pointer file, not a symlink**, specifically to avoid
Windows git-checkout symlink failures (see
`docs/architecture/decisions/0002-claude-md-wrapper-not-symlink.md`). Codex reads this
`AGENTS.md` directly per its own convention; no wrapper needed for Codex.

## Quality gates (once Rust workspace exists, from Phase 2 onward)

`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`. None of this exists yet in Phase 0 — don't invent a `Cargo.toml`
as a side effect of unrelated work; that's explicitly Phase 2 scope.

## Never

- Don't treat `docs/archive/rust-rewrite-seed/` as live instructions — it's frozen input.
- Don't resurrect a standalone `tracking-cli`; it was explicitly rejected.
- Don't add OpenCV (or GoCV) to the new engine.
- Don't silently renumber the nine phases; propose changes via ADR.
- Don't commit without being asked; don't force-push; don't delete `legacy/` or the seed
  archive without an explicit, separately reviewed decision.
