# Nine-phase plan — source blueprint, not live task database

Each phase has **inputs, deliverables, acceptance gate and important constraints**. Actual task files should be generated later after repo inspection.

## Phase 0 — Foundation / safe migration
**Inputs:** current repo, Git status, old AI config, seed bundle. **Deliverables:** rewrite branch, archived `legacy/` implementation, vetted final project structure, current docs skeleton, AI workflow implementation, initial task backlog and handoff. **Gate:** no data loss; existing sources accessible; AI instructions and skills tested for both installed CLIs; `dump/` preserved; documented outstanding risks. **Do not implement tracking.**

## Phase 1 — Research and specifications
Read real legacy source and all reverse-engineering reports; chart end-to-end behavior and defaults; map UI and camera pipeline; identify exact source/addresses for original algorithm; record confirmed/inferred/unknown; specify expected behavior and benchmark baselines. **Gate:** traceable behavior spec, discrepancies and explicit unanswered questions.

## Phase 2 — Rust architecture
Research Cargo workspace patterns, egui/eframe versions, camera backends, OS permission/cursor APIs, Rust image representation, dependency licenses. Create minimal workspace and ADRs; design separation of pure tracking core from I/O. **Gate:** CI compiles skeleton and runs unit tests with no webcam required; dependency decision rationale and platform risk matrix exist.

## Phase 3 — Tracking baseline
Implement a minimal pure-Rust pyramidal Lucas–Kanade single-point tracker inspired by *behavioral findings*, with own implementation, no OpenCV dependency. Preserve legacy template matcher as comparator if feasible without pulling old dynamic libraries. Synthetic translations, rotation/brightness/occlusion fixtures and stable timing harness. **Gate:** reproducible baseline metrics, correctness tests, no unjustified superiority claims.

## Phase 4 — Tracking improvements
Experiment separately with forward-backward validation, confidence or photometric residual, adaptive search, target-loss recovery, multiple-point consensus (only if it preserves semantics of user-selected point), template re-detection, smoothing/mapping improvements. Use ablation tests; guard against drift and latency costs. **Gate:** measured benefit or explicit rejected experiments.

## Phase 5 — Design system
Build standalone `docs/design/design-lab.html` in the actual repo (not seed), with multiple directions and interactive representative states; document typography, colors, spacing, statuses, responsive behavior and high contrast. Include tracking/lost/paused/no camera/permission states, large targets and keyboard interactions. **Gate:** approved UI reference and egui feasibility check.

## Phase 6 — Desktop app
Create egui/eframe UI wired to camera adapters, tracker and pointer mapping. Support target selection, settings, calibration, lifecycle and errors, cursor control permission paths, settings persistence. **Gate:** end-to-end workflows work on primary dev OS, integration tests and manual QA checklist pass.

## Phase 7 — Multiplatform / release
Validate macOS (Apple Silicon and permissions), Windows (cursor injection and capture), Linux (X11/Wayland limitations). Packaging, signing/notarization feasibility, dependency licenses, CI, reproducible builds, release docs. **Gate:** platform-by-platform release criteria with known exceptions explicit.

## Phase 8 — Hardening / product readiness
Stability, CPU/RAM and frame latency budgets, failure recovery, accessibility testing, real-user workflows, regression, error reporting and support docs. **Gate:** written readiness decision based on evidence.

## Parallelism and phase discipline

Research/design can overlap implementation where dependencies allow, but feature choices should be validated against architecture and fixtures. Each phase has owner, status, artifacts, verification and next handoff. Use worktrees for agents editing separate scopes; single orchestrator integrates. A checklist is not proof of completion: logs and tests are.
