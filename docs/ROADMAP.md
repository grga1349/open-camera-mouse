# Roadmap — nine phases (live)

The live phase plan for the Rust rewrite. Condensed from the frozen planning seed
(`docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md`), which stays the historical
source. Change phases here, via an ADR, never by silently renumbering. Per-task status lives in
`tasks/BACKLOG.md`. This file only tracks phase status.

Working rule: when a phase starts, its coordinator writes that phase's task file(s) in
`tasks/active/` from the "Deliverables" and "Gate" below. Task files are not written ahead of
the phase.

| # | Phase | Status | Evidence |
|---|---|---|---|
| 0 | Foundation / safe migration | DONE | tasks 0001, 0002; ADR-0001, -0002, -0005 |
| 1 | Research and specifications | DONE | task 0003; `docs/research/BEHAVIOR_SPEC.md`; ADR-0006 |
| 2 | Rust architecture | DONE | task 0004; ADR-0007, -0008; `docs/architecture/DEPENDENCIES.md`, `PLATFORM_RISKS.md`; CI green |
| 3 | Tracking baseline | **ACTIVE** | task 0005 |
| 4 | Tracking improvements | — | |
| 5 | Design system | — | |
| 6 | Desktop app | — | |
| 7 | Multiplatform / release | — | |
| 8 | Hardening / product readiness | — | |

## Standing inputs for every phase from 2 on

- `docs/research/BEHAVIOR_SPEC.md` §7 (expected behavior R-*) and §9 (open questions Q-*).
- ADR-0006: favor Camera Mouse 2018 behavior where practical. Deviations cite defect /
  unknown / platform / prior user decision. Hotkeys are fixed F11/F12.
- ADR-0007 (crate boundaries, Accepted), ADR-0008 (mirror at ingest; 640×480@30) and
  ADR-0004 (tracking candidates, Proposed).

## Phase details

**0 — Foundation.** Rewrite branch, `legacy/` archive, docs skeleton, AI workflow, backlog,
handoff. *Gate:* no data loss; AI instructions/skills tested in both CLIs; seed preserved;
outstanding risks documented.

**1 — Research and specifications.** Read legacy source and all RE reports, chart behavior
and defaults, record confirmed/inferred/unknown, specify expected behavior and benchmark
baselines. *Gate:* traceable behavior spec with discrepancies and explicit unanswered
questions.

**2 — Rust architecture.** Research (with current sources, not memory) Cargo workspace
patterns, egui/eframe versions, camera backends, OS permission/cursor APIs, Rust image
representation, dependency licenses. Create the minimal workspace and ADRs. Ratify or revise
ADR-0003. Decide BEHAVIOR_SPEC Q-R5 (mirroring) and Q-R6 (camera resolution/FPS). Separate the
pure tracking core from I/O. *Gate:* CI compiles the skeleton and runs unit tests with no
webcam on macOS, Windows and Linux; a dependency decision rationale and a platform risk
matrix exist.

**3 — Tracking baseline.** Pure-Rust pyramidal LK with the 2018 preset (ADR-0006: presumptive
baseline) plus a pure-Rust NCC comparator (legacy preset). No OpenCV. Synthetic fixtures
(translation, rotation, brightness, occlusion, edges …) and a stable timing harness per
BEHAVIOR_SPEC §8. Decide Q-R1 (units of motion bounds). *Gate:* reproducible baseline
metrics, correctness tests, no unjustified superiority claims.

**4 — Tracking improvements.** Separate experiments: forward-backward validation, confidence /
photometric residual, adaptive search, loss recovery, multi-point consensus (only if it keeps
the semantics of the user-selected point), re-detection, mapping/smoothing refinements.
Ablation tests; guard against drift and latency. *Gate:* measured benefit, or explicitly
rejected experiments.

**5 — Design system.** Standalone `docs/design/design-lab.html` with multiple directions and
interactive states (tracking / lost / paused / no camera / permission), typography, colour,
spacing, high contrast, large targets, keyboard use. *Gate:* approved UI reference and an
egui feasibility check.

**6 — Desktop app.** egui/eframe UI wired to camera adapters, tracker and pointer mapping:
target selection, settings, calibration/recenter, lifecycle and errors, cursor-permission
paths, persistence, handover and dwell per ADR-0006. *Gate:* end-to-end workflows on the
primary dev OS; integration tests and a manual QA checklist pass.

**7 — Multiplatform / release.** Validate macOS (Apple Silicon, permissions), Windows (cursor
injection, capture), Linux (X11/Wayland limits). Packaging, signing/notarization, licenses,
CI, reproducible builds, release docs. *Gate:* release criteria per platform, with known
exceptions explicit.

**8 — Hardening / product readiness.** Stability, CPU/RAM and latency budgets, failure
recovery, accessibility testing, real-user workflows, regression, error reporting, support
docs. *Gate:* a written readiness decision based on evidence.

Research and design may overlap implementation where dependencies allow. Each phase has an
owner, status, artifacts, verification and a handoff. A checklist is not proof of completion.
Logs and tests are.
