# Open Camera Mouse — Rust Rewrite: SEED ENTRY

> **Status: planning source material only.** All files in this `dump/` directory are seed documents for an AI agent to read and transform. They are **not** the eventual repository documentation, architecture, skills, agent configuration or task tracking system. Do not treat this directory as an implementation convention. Do not automatically copy files into the repository root unchanged.

## Purpose and recovery

This package captures the decisions, research, objectives, and staged plan from the project-planning conversation. If chat context is lost, start here, read the linked seed files, inspect the actual repository and `../camera-mouse-reverse/`, and reconcile assumptions against files and current official tool documentation. Preserve this package until the actual project documents are generated and independently validated.

**Execution:** `dump/initial-promt.md` is the **first kickoff prompt**. The actual repository instructions, docs, skills, and tasks will be created by the agent as deliverables; they are not pre-created here. The initial prompt asks for reconnaissance and foundation planning first; the remaining phases are defined below and should be converted into tasks only after assessing the repository.

## Reading map

| File | Purpose |
|---|---|
| `initial-promt.md` | Initial executable prompt for Codex/Claude: begin with safe inventory and foundation design |
| `01-product-and-context.md` | Why we are building this, goals, boundaries and acceptance goals |
| `02-roadmap-nine-phases.md` | All nine planned phases, dependencies, deliverables and gates |
| `03-research-and-evidence.md` | Reverse-engineering findings and what remains unverified |
| `04-repository-migration.md` | Git branch and safe legacy migration strategy |
| `05-architecture-and-testing.md` | Proposed Rust boundaries, tracking, benchmarks and technical research questions |
| `06-ai-workflow.md` | Canonical instructions, project skills, subagents, tasks, handoffs, compact/resume |
| `07-design-direction.md` | Design discovery and HTML design lab, accessibility and egui constraints |
| `08-decisions-and-open-questions.md` | Decisions, hypotheses, risks and pending research |
| `09-operator-playbook.md` | How to launch, validate and proceed phase by phase |
| `10-rust-skills-architecture-testing-research.md` | Detailed Rust crate/module architecture, project-local skill-to-agent cookbook, deterministic tracking tests, egui_kittest, benchmarks, and token-efficient verification; supersedes earlier suggestions of a standalone tracking CLI |

## Latest technical clarifications (read before architecture work)

- The follow-up research addendum `10-rust-skills-architecture-testing-research.md` is part of this seed. It refines, but does not create a new roadmap phase.
- **No standalone headless/tracking CLI initially.** Use `cargo test`, Rust integration tests and benchmark targets over synthetic/recorded frames instead. If an older seed file or an external reference advocates a CLI, this newer explicit user decision prevails.
- Prioritize algorithm correctness and regression quality before UI automation. Later add `egui_kittest` for interaction-level tests. Full physical-camera/OS pointer tests belong in manual or hardware-specific smoke testing.
- Agent skills and subagent roles are separate. Research and test skill discovery in both installed clients; never assume that shared symlinks or repo instructions can disable inherited global skills.
- This addendum is research for the future agent to distill into actual repo docs and configurations, **not** a final `AGENTS.md` or skill file.

## Nine-phase roadmap (source of truth for staging)

0. **Repo foundation** — inspect and preserve, branch, legacy migration, workflow design, repository instructions and tasks.
1. **Research & specifications** — analyze old implementation and `../camera-mouse-reverse` evidence; produce behavior specification.
2. **Rust architecture** — investigate options, record ADRs, choose crates/adapters, create minimal workspace skeleton.
3. **Tracking engine baseline** — pure-Rust PyrLK baseline and legacy matcher, deterministic fixtures and measurements.
4. **Tracking improvements** — validation, recovery, adaptive smoothing, measured tradeoffs.
5. **Design system** — standalone HTML design lab, palettes, states, accessibility, iteration.
6. **Desktop application** — egui/eframe app, webcam, pointer control, settings, calibration, permissions.
7. **Multiplatform & release** — macOS/Windows/Linux validation, packaging, CI and end-user docs.
8. **Hardening & product quality** — reliability, regression suite, performance budgets, user trials and release readiness.

The older conversation also described phases numbered 00–07 (8 items). We preserve its content and explicitly add final hardening as ninth phase. Phase IDs here are canonical for this seed; the agent may propose revised milestones through an ADR, not silently renumber them.

## Instructions for agent consuming this seed

1. Read this entry, `initial-promt.md`, and each relevant seed file, explicitly including the newer `10-rust-skills-architecture-testing-research.md` before choosing architecture, skills, or testing.
2. Inspect the real repository and `../camera-mouse-reverse`; **never claim findings purely because this seed says so**.
3. Check current official Claude Code/Codex/Rust/egui docs before selecting configuration or tools. Settings and support can change.
4. Keep `dump/` as an archive, not as a source of dynamically loaded agent instructions.
5. Work incrementally, create explicit tasks/acceptance criteria and verification evidence.
6. Do not implement the Rust engine as part of initial foundation setup.
7. Produce `dump/SEED_CONSUMPTION_REPORT.md` only once seed-to-repo extraction has been verified, tracking which seed sections became which actual files.
