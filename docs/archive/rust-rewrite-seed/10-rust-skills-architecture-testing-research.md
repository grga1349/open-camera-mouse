# 10 — Rust skills, architecture, agent delegation, and testing research

Status: RESEARCH SEED / NOT IMPLEMENTATION. November 2026.

This document belongs in `dump/` as context for the future repository setup. It is not a final repo config, not an instruction to install dependencies blindly, and not a substitute for confirming installed CLI versions. Read together with `dump/ENTRY.md`, `dump/initial-promt.md`, the legacy repo, and `../camera-mouse-reverse/`.

## Executive decisions

**Correction to earlier brainstorming:** A separate `tracking-cli` executable was proposed and explicitly rejected as unnecessary. The product is a camera-driven desktop app; algorithm testing will use Rust unit/integration tests and benchmarks fed with recorded or synthetic frames. Do not revive the CLI by mechanically copying architectural examples.

1. Build a deterministic, offline-testable tracking pipeline as a first-class feature, not an afterthought. Camera frames, tracking results, state transitions, and cursor commands must be testable without a physical webcam or mouse movement. Use Rust tests, not an additional CLI executable.
2. Use a small Cargo workspace with `tracking-core`, `pointer-core`, `app-core`, `platform` and one desktop app. No separate `tracking-cli` in the initial architecture. Begin with fewer crates if cohesive code fits; do not prematurely split every module into a crate.
3. Use project-local skills with narrowly scoped, testable procedures and reference files. Agent roles are separate from skills; inject relevant skills at subagent creation and avoid indiscriminate preload.
4. First milestone: recover the 2018 Camera Mouse optical-flow behavior as a baseline using archived reverse engineering evidence. Do not treat decompiler findings as measured proof of implementation quality.
5. Highest priority is quality testing for math/algorithms: synthetic sequence tests, properties, regression datasets, error metrics, quality thresholds, and benchmarks. GUI click automation can follow, and OS-wide end-to-end automation can wait.
6. Do not require every CLI to install or enable every skill; verify skill discovery in local Claude/Codex versions. Repository instructions cannot necessarily disable globally installed skills.
7. Keep `dump/` as seed material. The implementation agent may derive actual repo files under `docs/`, `.agents/`, `.claude/`, `.codex/`, etc. only after reviewing the seed and repository state.

## Lessons from public Rust projects

### Graphite — editor backend and message dispatch

- Official architecture: https://graphite.art/volunteer/guide/codebase-overview/editor-structure/
- Editor backend owns product state, with a dispatcher processing messages sequentially. This supports clear ownership and fewer competing mutable borrows.
- Adaptation: an app controller processes commands (`StartTracking`, `StopTracking`, `SelectPoint`, `SetSensitivity`, etc.) and emits state/results; desktop UI and tests can share the same controller. Do **not** assume a complex async actor framework is needed.

### PhotoCraft — shared command interface, thin UI

- Source: https://github.com/storytold/photocraft
- Claims in public project documentation: engine-first design, egui desktop, headless CLI (reference project only), command registry, tests, modular crates. Treat claimed counts and benchmarks as project claims rather than independent validation.
- Adaptation: do NOT copy a large multi-crate architecture. Copy the idea of a clear app/core boundary and deterministic command handling. Exercise commands through Rust integration tests, not a CLI.
- If user meant Graphite rather than PhotoCraft by “Photoshop clone”, examine both; their architectural lessons complement each other.

## Suggested source layout (subject to architectural review)

```
Cargo.toml
Cargo.lock
rust-toolchain.toml
crates/
  tracking-core/
    src/
      lib.rs                 # public API (reexports/types)
      frame.rs               # grayscale frame view, dimensions, stride
      config.rs              # LK, pyramid, validation parameters
      result.rs              # position, status, reason, metrics
      tracker.rs             # tracking state machine
      image/
        mod.rs
        pyramid.rs
        gradients.rs
        sampling.rs
      optical_flow/
        mod.rs
        pyramidal_lk.rs
        solver.rs
      validation/
        mod.rs
        bounds.rs
        outliers.rs
        forward_backward.rs
      recovery.rs
    tests/
      synthetic_motion.rs
      tracking_sequences.rs
  pointer-core/
    src/
      lib.rs
      mapping.rs
      smoothing.rs
      dwell.rs
      limits.rs
  app-core/
    src/
      lib.rs
      commands.rs
      events.rs
      state.rs
      controller.rs
  platform/
    src/
      lib.rs
      camera.rs
      cursor.rs
      permissions.rs
      macos.rs               # conditional modules; split per OS as needed
      windows.rs
      linux.rs
apps/
  desktop/src/
    main.rs
    app.rs
    ui/preview.rs
    ui/settings.rs
    ui/status.rs
fixtures/
  synthetic/                # deterministic generators and expected metadata
  recorded/                 # consented, redistributable small clips
benches/
docs/                     # generated later from seed
```

`tracking-core` should accept a frame view / buffer and return a value; it must not import egui, camera bindings or OS cursor APIs. `pointer-core` converts positions using explicit coordinate spaces and adjustable smoothing. Platform adapters own permissions and devices. Application controller owns lifecycle and commands. UI renders state and sends commands.

Set explicit rules for frame ownership/stride, grayscale conversion, fps/time units, integer/floating coordinates, dropped frames, and thread boundaries. Benchmark before deciding on SIMD/GPU. Avoid overusing `Arc<Mutex<...>>`; prefer bounded channels where capture/compute/UI work at different rates.

## Agent skill model

A skill is a reusable workflow with `SKILL.md`, optional `references/`, `scripts/`, `assets/`. An agent is a delegate with a scope, tool permissions, and required output. Skill description should say *when* to use it and *when not* to. Only skills needed for a task should be attached/preloaded.

Suggested project skills (not necessarily downloaded community packages):

| Skill | Precise objective | Deliverables and gates |
|---|---|---|
| rust-engineering | Idiomatic borrowing/ownership, error types, API docs, sound performance | `cargo fmt --check`, clippy, test; no unsafe without written rationale |
| rust-architecture | Workspace boundaries, dependency directions, rustdoc/API visibility, ADR | Dependency graph, public API, minimal traits, ADR |
| computer-vision | Grayscale, interpolation, image pyramids, Lucas–Kanade, numerical stability | Equations, image coordinate assumptions, reference checks |
| tracking-validation | Build synthetic video, truth trajectories, error metrics, failure taxonomy | Deterministic fixture, thresholds, failing regression |
| rust-testing | Rust unit/integration/property/golden tests and quality gates | Repro command, meaningful assertions, failure trace |
| benchmarking | Micro/per-frame/sequence perf, allocations, warmup, environment | Baseline report, min/median/p95, inputs/device metadata |
| egui-development | eframe lifecycle, immediate-mode state, AccessKit, kittest | State-driven UI, interaction tests, accessible labels |
| platform-integration | Camera/OS pointer adapters, permissions, no physical-device tests by default | Adapter contract, fake adapter tests, OS smoke checklist |
| agent-handoff | Persist task, decisions, changed files, test status before context loss | Atomic handoff linked to Git state |
| code-review | Independent review focusing on correctness, failure modes and evidence | Severity-ordered findings, no unverified assertions |

Possible later skills: accessibility, profiling, release-packaging, ffi-safety, fuzzing. Avoid 30 generic skills at project creation.

### Skill-to-agent cookbook (example, conceptual—not a native CLI config)

- `architect`: rust-architecture + code-review; read-only research first; owns ADR recommendation.
- `vision-researcher`: computer-vision + tracking-validation; reads reverse-engineering evidence; provides math and fixture specs.
- `tracking-implementer`: rust-engineering + computer-vision + rust-testing; writes only tracking-core and assigned tests.
- `benchmark-analyst`: benchmarking + tracking-validation; measures and reports, does not approve own implementation quality.
- `desktop-engineer`: egui-development + rust-engineering + platform-integration; UI and adapter integration.
- `independent-reviewer`: code-review + rust-testing; isolated context, should not edit code unless explicitly tasked.
- `orchestrator`: agent-handoff + planning; delegates separate files/worktrees and integrates results.

Claude Code custom subagent frontmatter may support skill preloading, but confirm current release. Codex agent setup uses distinct files/formats; generate from shared role definitions or maintain small platform-specific wrappers. Skills and tools are not the same. Do not claim that symlinks always work without a discovery smoke test.

## Core test design — do not defer

### A. Pure unit tests

- Image pyramid dimensions, stride, boundaries, gradients and bilinear sampling.
- LK Hessian/linear solver: constant/singular region, well-textured region, finite/NaN handling.
- Coordinates: image center -> screen center, resizing, offsets, negative bounds, different axes.
- Temporal smoothing: initial state, steady input, step response, pause/reset and dropped frames.
- Tracking state transitions: selected, tracking, rejected outlier, lost, recovery, stopped.
- Dwell logic: timer and dead-zone with fake clock. No wall-clock sleeps.

### B. Synthetic frame sequences + ground truth

Create frame generator with deterministic seed and metadata JSON. Render noise-textured patches, corners, translated shapes, rotations, blur, luminance shifts, occlusion, distractors, partial out-of-frame, and zero-texture.

Input: `FrameSequence` with `Frame` and timestamps, initial target location, expected trajectory/visibility. Run tracker offline, collect outputs. Assert on:

- End point error (EPE) per frame; mean/median/p95 and max where useful.
- Percent frames tracked successfully and false-positive tracks.
- Recovery time after disappearance.
- Drift (error after long sequence).
- Cursor jitter while stationary.
- Response latency/step response and pointer overshoot.

Start with simple known integer translations (+5/-3), then subpixel and multi-frame sequences. Avoid using the implementation under test to generate the expected trajectories; use transforms as independent ground truth. Test both success and expected failure; flat region must not produce spurious reliable tracking.

Careful: 2018 limits (10x10 window, pyramid level 3, max 35 px jump, 20 iterations, epsilon .03) are *reverse-engineered reported defaults*, not universal optimal settings. Use them as baseline parameter preset and verify exact semantics (e.g., pyramid level 3 can represent levels 0..3). They do not guarantee correct handling of occlusion/recovery.

### C. Property tests

`proptest`: coordinate mapping remains within configured bounds, output finite for finite inputs, stationary repeated input does not drift beyond tolerance, consistent scaling/translation where algorithm is expected to be equivariant, no panics on arbitrary frame dimensions/strides/cropped edges. Keep random seeds logged so failures reproduce.

### D. Golden regression tests

Persist small, non-sensitive reference sequences and expected trajectories/errors. Use `insta` to snapshot structured summaries (not giant image buffers). Changes in result require review and explicit snapshot update.

### E. Integration & offline functional tests

Use `cargo test -p tracking-core --test tracking_sequences` and `cargo test -p app-core --test controller_integration` (once those test targets exist). Tests should instantiate the same tracking core and application controller as the desktop app, feed a deterministic sequence, and assert machine-readable values such as position, state transitions, error metrics and cursor commands. Integration tests must not move the real cursor; use in-memory/mock pointer sinks and fake capture sources. Deterministic tests are more token-efficient than AI screenshot analysis.

A standalone `tracking-cli` is deliberately **out of scope**. Do not introduce one without a new decision and demonstrated testing gap.

### F. GUI interaction tests (when desktop UI exists)

Use `egui_kittest` via AccessKit to locate accessible controls by label, click, change settings and assert model/state. Optionally use screenshot snapshot testing with stable renderer/version, minimal tolerance and human review on changes. This can run without full OS pointer automation.

Documentation: https://docs.rs/egui_kittest/latest/egui_kittest/

### G. Full OS / webcam E2E (later)

A few smoke tests per OS to verify camera permissions, capture path, preview, select target, cursor injection, pause and recovery. Hardware-specific tests are non-deterministic and not first-line CI. Avoid building a large screenshot-driven AI functional test system.

## Suggested Rust tools and policy

- Built-in: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- `cargo-nextest`: faster isolated test runner for growing suites. https://nexte.st/
- `proptest`: property-based tests (good for invalid dimensions, state machines and numeric invariants). https://docs.rs/proptest/
- `insta`: compact golden snapshots for structured output. https://docs.rs/insta/
- `divan` (or Criterion; pick one): benchmarking with multiple inputs. https://docs.rs/divan/
- `egui_kittest`: egui interaction, accessibility and optional image snapshots. https://docs.rs/egui_kittest/
- `cargo-fuzz` / `cargo-mutants`: later, after a stable engine and credible test suite.

Do not install tools/dependencies before checking Rust toolchain compatibility and maintenance. Keep CI deterministic and small. Use `cargo test` as the baseline; nextest optional. Do not make a benchmark test a flaky hard CI gate based on absolute time across heterogeneous machines. Record performance trends instead.

## Token-efficient gates

A local script (future generated `scripts/check.sh`) should run cheap diagnostics and print summarized failures. A task is not completed merely because a model says it looks correct.

Recommended ordering:

1. Before every implementation: define test cases and expected failure behavior.
2. After small code edits: targeted `cargo test -p tracking-core ...` or `cargo test -p pointer-core ...`.
3. Before handing off task: `cargo fmt --check`, targeted clippy, all relevant unit/integration tests.
4. Before merge: workspace tests, `egui_kittest` where applicable, benchmark comparison and independent review.
5. Nightly/manual: broader fixture corpus and OS-specific smoke tests.

Machine-readable test outputs and short summaries should be used instead of streaming hundreds of log lines into agent context. Retain complete logs as artifacts when a failure needs investigation.

## Priority roadmap for validating architecture and testability

Phase 0: Repository/AI seed to canonical files, no implementation. Prove Claude and Codex discover identical instructions/skills and can hand off.

Phase 1: Design the public core APIs and generate synthetic fixture format. Implement minimal testing framework before optical-flow implementation.

Phase 2: Build original-parameter PyrLK baseline with unit + trajectory tests. Prove no-camera test execution using Cargo test targets.

Phase 3: Add tracking improvements one at a time, A/B benchmarks against baseline, with quality metrics and measured regression thresholds.

Phase 4: Pointer core tests, controller fake adapters, real camera integration.

Phase 5: egui UI and kittest functional tests, controlled visual snapshots for design iterations.

Phase 6: Cross-platform hardware smoke tests, packaging and release.

## Open questions requiring explicit decisions

- Is the original PhotoCraft or Graphite the intended reference? Compare both; do not copy by default.
- Does the user prioritize native Windows first, macOS first, or equal platforms? Develop on macOS but avoid claiming OS parity without tests.
- How much pure Rust does the no-OpenCV requirement imply for camera decoding, OS APIs, and graphics rendering? Rust-owned tracker does not require eliminating all native OS APIs.
- A headless CLI is not planned; reconsider only if the Rust test harness demonstrably cannot cover a future workflow.
- Which UI states and accessibility requirements are essential for people using Camera Mouse as assistive input.
- Identify exactly which algorithmic discoveries in `../camera-mouse-reverse/` are verified vs inferred.

## Sources (public, current research as of Nov 2026)

- Graphite editor architecture: https://graphite.art/volunteer/guide/codebase-overview/editor-structure/
- Graphite full codebase: https://graphite.art/volunteer/guide/codebase-overview/
- PhotoCraft repo: https://github.com/storytold/photocraft
- Rust cargo tests: https://doc.rust-lang.org/stable/cargo/guide/tests.html
- Codex customization: https://learn.chatgpt.com/docs/customization/overview
- Codex skill creation: https://learn.chatgpt.com/docs/build-skills?product=breeze
- Claude skills: https://claude.com/resources/articles/steering-claude-code-skills-hooks-rules-subagents-and-more
- Claude skill vs agent: https://claude.com/resources/articles/skills-explained
- egui kittest: https://docs.rs/egui_kittest/latest/egui_kittest/
- proptest: https://docs.rs/crate/proptest/latest
- insta: https://docs.rs/insta/latest/insta/
- divan: https://docs.rs/divan/latest/divan/
- cargo nextest: https://nexte.st/

## Suggested seed ingestion instruction

Read this file as architectural/testing research. Do not copy it verbatim into `AGENTS.md` or use it as a skill. Derive concise canonical docs, project skills, task templates, and test plan after inspecting current repo and installed CLI versions. Explicitly document which recommendations are adopted, deferred, or rejected and why. Preserve this file as an immutable planning snapshot in `dump/`.
