# Roadmap — nine phases (live)

The live phase plan for the Rust rewrite. Condensed from the frozen planning seed
(`docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md`), which stays the historical
source. Phase 6b (test drive) was inserted by ADR-0013 and Phase 6c (tuning round) by ADR-0019 and Phase 6d (calibrated pointer behaviour) by ADR-0020, without renumbering. Change phases here, via an ADR, never by silently renumbering. Per-task status lives in
`tasks/BACKLOG.md`. This file only tracks phase status.

Working rule: when a phase starts, its coordinator writes that phase's task file(s) in
`tasks/active/` from the "Deliverables" and "Gate" below. Task files are not written ahead of
the phase.

| # | Phase | Status | Evidence |
|---|---|---|---|
| 0 | Foundation / safe migration | DONE | tasks 0001, 0002; ADR-0001, -0002, -0005 |
| 1 | Research and specifications | DONE | task 0003; `docs/research/BEHAVIOR_SPEC.md`; ADR-0006 |
| 2 | Rust architecture | DONE | task 0004; ADR-0007, -0008; `docs/architecture/DEPENDENCIES.md`, `PLATFORM_RISKS.md`; CI green |
| 3 | Tracking baseline | DONE | task 0005; ADR-0009; `docs/research/TRACKING_BASELINE.md`; CI green |
| 4 | Tracking improvements | DONE | task 0006; ADR-0010, -0011; `docs/research/TRACKING_IMPROVEMENTS.md` |
| 5 | Design system | DONE | task 0007; `docs/design/design-lab.html` (approved v3), `EGUI_FEASIBILITY.md` |
| 6 | Desktop app | DONE | task 0008; ADR-0012; `docs/qa/MANUAL_QA.md` (rows 1–18 pass); CI green |
| 6b | Test drive (ADR-0013) | DONE | task 0009; `docs/qa/TEST_DRIVE.md` #1–#6; ADR-0014…0018 |
| 6c | Tuning round (ADR-0019) | DONE | task 0010; `docs/qa/TEST_DRIVE.md` #7–#11; closed into 6d (ADR-0020) |
| 6d | Calibrated pointer behaviour (ADR-0020) | NEXT | task 0011 (written when the phase starts); `docs/qa/TEST_DRIVE.md` from #12 |
| 7 | Multiplatform / release | — | carry-over: MANUAL_QA rows 19–20 (`.app` permissions) |
| 8 | Hardening / product readiness | — | carry-over: MANUAL_QA row 21 (reduced motion); `OCM_RECORD` threshold re-measurement (re-detection arming, R-P4, dwell radius) |

## Standing inputs for every phase from 2 on

- `docs/research/BEHAVIOR_SPEC.md` §7 (expected behavior R-*) and §9 (open questions Q-*).
- ADR-0006: favor Camera Mouse 2018 behavior where practical. Deviations cite defect /
  unknown / platform / prior user decision. Hotkeys are fixed F11/F12 (start/pause F10 on macOS, ADR-0012).
- ADR-0007 (crate boundaries, Accepted), ADR-0008 (mirror at ingest; 640×480@30) and
  ADR-0004 (tracking candidates, Accepted), ADR-0011 (recommended tracker: `LkParams::recommended()`).

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

**6b — Test drive (ADR-0013).** The user uses the app on the dev Mac for real work; each
issue they report is logged in an issue table (`docs/qa/TEST_DRIVE.md`: date, what happened,
severity, triage, fix commit or deferral), reproduced, and either fixed (regression test where
it can be tested headless) or explicitly deferred to a named phase. Fixes only, no new
features. *Gate:* every logged issue fixed or deferred with a reason; quality gates green; the
user says the app is ready for Phase 7.

**6c — Tuning round (ADR-0019).** One more round on the dev Mac after 6b: the user drives
the dev build; issues and tuning wishes are logged in `docs/qa/TEST_DRIVE.md` (from #7),
measured, changed with an automated check (CI test, local replay metric or `OCM_PERF`) and
re-checked on the device. A changed default needs the user's OK and an amendment to the ADR
that owns it. Carried in from 6b: on-device check of main-monitor recenter (#4), minimised
rate and dwell since the engine thread, `ui` time with Options open. *Gate:* every 6c row
fixed or deferred with a reason; quality gates green; the user says the app is ready for
Phase 7.

**6d — Calibrated pointer behaviour (ADR-0020).** Recordings of S0–S3 (+ precise-move and
jump scenarios), shared metrics (shake, creep, jumps, lag, precision) for replays and synthetic
runs, a settings sweep over the recordings, synthetic signals fitted to the recordings (only the
fitted numbers in the repo), then fixes for jitter with the smallest dead zone and for jumps.
*Gate:* shake 0 at rest and no unexplained jumps on recordings and on calibrated CI tests at
every gain 1–60; the smallest sufficient zone, with its sweep table; every 6d row fixed or
deferred; quality gates green; the user says ready for Phase 7.

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
