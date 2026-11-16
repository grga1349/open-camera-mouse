# 0010 — Phase 6c: tuning round

Status: DONE (2026-11-16) — closed into Phase 6d by user decision (ADR-0020)
Owner/branch: `rewrite/rust`
Depends on: 0009 (done)

## Context

Phase 6c per `docs/ROADMAP.md` and ADR-0019: one more round of test → tune → fix on the dev
Mac after the 6b test drive, before Phase 7. Build under test starts at `944cbf1` (code
identical to `df4c670`, CI green on `71ac27a`; the `944cbf1` run failed in macOS
`test (no webcam)` on a docs-only commit — flaky test, logged as #7).

Inputs: `docs/qa/TEST_DRIVE.md` (#1–#6 and their notes), ADR-0006 (2018 defaults), -0011
(tracker), -0015 (steady zone), -0016 (engine thread), -0017 (pointer filter/scale), -0018
(anchored gate), `agent/HANDOFF.md` §"Known, not bugs". Skills: `rust-testing`,
`tracking-validation`, `egui-development`, `platform-integration`, `code-review`.

Carried in from 6b: #4 main-monitor recenter on-device check; minimised pointer rate and
dwell not re-checked on device since the engine thread (pointer now runs at full rate when
minimised); `ui` ≈ 15/17 ms with Options open; ADR-0018 slow-creep limit.

## Objective

- Log every issue or tuning wish in `docs/qa/TEST_DRIVE.md` under "Phase 6c" (#7 on).
- Per row: measure first (replay metric, `OCM_PERF`, headless test), change, add an automated
  check, then the user checks on the device.
- A changed default needs the user's OK and an amendment to the ADR that owns the value.
- One row or a small related group per session.

## Non-goals

- New features unless asked first; then an option, off by default unless the user approves.
- "Known, not bugs" in `agent/HANDOFF.md`, unless the user asks.
- Windows/Linux on-device validation and packaging (Phase 7).

## Acceptance criteria (gate)

- Every 6c row fixed (commit + automated check) or deferred to a named phase with a reason.
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

- 2026-11-16: task file written; #7 (CI flake) logged; waiting on the user's first reports.
- 2026-11-16: #8 speed (first ×3 pass replaced by real gain 1–60, default 24), #9 Screen tab, #10 Text size removed + defaults smoothing 0.10, steady 11 px (ADR-0017 amendment). Tests green (`app-core`, `platform`, `open-camera-mouse`), fmt + clippy clean; uncommitted, device check pending.
- 2026-11-16: #11 steady zone automatic (0.5 camera px, grows with gain and Main monitor), option removed; S1 replay sweep. Tests green, fmt + clippy clean; uncommitted, device check pending.
- 2026-11-16 close-out: defaults gain 22 / smoothing 0.20 (ADR-0017 amendment). The user wants the smallest steady zone that fixes jitter, a fix for jumps (#12), and a settings sweep on recordings with synthetic tests calibrated to them → Phase 6d (ADR-0020). Open rows carried: #7, #11 (zone size), #12.
