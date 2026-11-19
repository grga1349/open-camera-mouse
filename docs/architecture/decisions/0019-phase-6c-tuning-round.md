# ADR-0019: Phase 6c — tuning round before multiplatform release

Status: Accepted (user decision, 2026-11-16)
Date: 2026-11-16
Amends: `docs/ROADMAP.md` (adds a phase between 6b and 7; no renumbering)

## Context

Phase 6b (ADR-0013, task 0009) fixed the six issues from the first test drive: preview
corners, tracking losses (ADR-0017 texture floor; ADR-0018 anchored appearance gate),
re-detection arming, main-monitor recenter (ADR-0014), cursor jitter (ADR-0017), and preview
lag (ADR-0016). The user checked the fixes on the device ("turns hold much better now") and
wants one more round of testing, tuning and fixes before Phase 7, not a jump to packaging.

## Options considered

1. Keep 6b open: its gate ("the user says ready for Phase 7") isn't met. But 6b's issue log
   and plan (part 2, slices 2a–2d) are done; reopening it blurs what was delivered.
2. **Close 6b and add Phase 6c, a tuning round: chosen.** Same rules as 6b, fresh issue
   rows and a fresh task file, so 6b's record stays closed.
3. Fold tuning into Phase 8 hardening: behaviour would still move after Phase 7 packaging
   and cross-OS validation, which ADR-0013 avoided on purpose.

## Decision

**Phase 6c — Tuning round**, after 6b and before 7. Phases 7 and 8 keep their numbers. The user
drives the dev build on the dev Mac. Each issue or tuning wish is logged in
`docs/qa/TEST_DRIVE.md` (new rows from #7, under a "Phase 6c" heading), then measured,
changed, checked by an automated test, and checked again on the device. Tuning of existing
defaults (gains, smoothing, steady zone, dwell, tracker thresholds) is in scope. A changed
default needs the user's OK and is recorded in the ADR that owns the value (ADR-0006,
-0011, -0015, -0017, -0018) as an amendment. New behaviour follows the normal rules (asked
first; option, off by default unless the user approves a default; ADR when non-trivial).
The rules carried over from 6b: recordings are diagnosis-only; measure first; every fix is
checkable by a CI test, a local replay metric or `OCM_PERF`; one slice per session.

*Gate:* every 6c row fixed or deferred to a named phase with a reason; quality gates green
locally and in CI on 3 OS; the user says the app is ready for Phase 7.

## Consequences

- The roadmap has eleven rows; "nine phases" still means 0–8, with 6b and 6c as interludes.
- The 6c task file is written by the session that starts the phase (task 0010), from this
  ADR and the roadmap, not in advance.
- Open 6b items carried into 6c: on-device check of main-monitor recenter (TEST_DRIVE #4);
  minimised pointer rate and dwell not re-checked on device since the engine thread
  (ADR-0016); `ui` ≈ 15/17 ms while Options is open; ADR-0018's slow-creep limit. Phase 7/8
  carry-overs (MANUAL_QA rows 19–21, threshold re-measurement) stay where they are.
