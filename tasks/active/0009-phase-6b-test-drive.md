# 0009 — Phase 6b: test drive

Status: ACTIVE (2026-11-15)
Owner/branch: `rewrite/rust`
Depends on: 0008 (done)

## Context

Phase 6b per `docs/ROADMAP.md` and ADR-0013: before Phase 7, the user uses the app on the dev
Mac for real work and we sort out what comes up. Build under test starts at `fc884e5`
(code identical to `7e9a2bb`, CI 43746097717 green).

Inputs: `docs/qa/MANUAL_QA.md` (rows 1–18 pass), ADR-0006 (2018 defaults), ADR-0012
(platform adapters), `agent/HANDOFF.md` §"Known, not bugs". Skills: `rust-testing`,
`egui-development`, `platform-integration`, `tracking-validation`, `code-review`.

## Objective

- Log every issue the user reports in `docs/qa/TEST_DRIVE.md` (date, build, what happened,
  severity, triage, fix commit or deferral).
- Per issue: get exact steps; reproduce (headless test with fakes in
  `apps/desktop/src/runtime_tests.rs` / `app-core` tests where possible, `OCM_RECORD` for
  tracking problems, screenshots for UI); fix with a regression test, or defer to a named phase
  with a reason.
- One issue or a small related group per session.

## Non-goals

- New features. A wish for new behaviour is asked about first; if the user says yes, it ships as
  an option, off by default, with its own decision record if non-trivial.
- The items listed as "Known, not bugs" in `agent/HANDOFF.md`, unless the user asks.
- Windows/Linux on-device validation and `.app` packaging (Phase 7).

## Acceptance criteria (gate)

- Every row in `docs/qa/TEST_DRIVE.md` is fixed (commit + test, or a stated reason it can't be
  tested headless) or deferred to a named phase with a reason.
- Quality gates green locally and in CI on 3 OS.
- The user says the app is ready for Phase 7.

## Verification

```
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/check-core-deps.sh
```

## Progress

- 2026-11-15: task file and `docs/qa/TEST_DRIVE.md` created; waiting on the user's first reports.
- 2026-11-15 part 1: #1–#5 logged. Done: #1 rounded preview, #4 "Main monitor" (ADR-0014),
  #3 re-detection fix (ADR-0011 amendment). Partial: #5 steady zone (ADR-0015) — user says it
  doesn't help. Open: #2 losses, #5 jitter (top priority, part 2; see `agent/HANDOFF.md`).
  Replay harness `crates/app-core/tests/replay.rs` added.
