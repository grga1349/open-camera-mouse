# ADR-0020: Phase 6d — pointer behaviour calibrated against recordings

Status: Accepted (user decision, 2026-11-16)
Date: 2026-11-16
Amends: `docs/ROADMAP.md` (adds a phase between 6c and 7; no renumbering). Closes Phase 6c
(ADR-0019) into 6d, as ADR-0019 closed 6b.

## Context

Phase 6c (task 0010) tuned by feel on the device: real gain 1–60 (#8), a Screen tab (#9),
Text size removed (#10), the steady zone made automatic in camera px (#11), defaults gain 22 /
smoothing 0.20. Two problems remain, and tuning one value at a time by feel is too slow to
solve them:

- **Jitter vs dead zone.** The steady zone holds the cursor still, but costs precision. The
  user wants the *smallest* zone that removes jitter. The S1 replay showed that a still head
  produces two different signals: frame-to-frame shake (mostly removed by the 1€ filter) and
  slow drift (1–2.5 camera px over seconds). A dead zone sized for drift is far too large.
- **Jumping.** The user sees the cursor jump (TEST_DRIVE #12; exact steps to be collected at
  the start of 6d).

The synthetic tests that guard these behaviours use hand-picked noise (e.g. uniform ±0.14 px
tremor). They passed while the device still jittered: they aren't calibrated to real heads.

## Options considered

1. Keep tuning in 6c by feel: each round costs a device session and gives one data point.
2. Fold this into Phase 8 hardening: behaviour would still change after Phase 7 packaging.
3. **Phase 6d: measure many settings on recordings, calibrate the synthetic tests to the
   recordings, then fix jitter and jumps against both. Chosen.**

## Decision

**Phase 6d — calibrated pointer behaviour**, after 6c and before 7. Same rules as 6b/6c
(ADR-0013, -0019): the user drives the dev build; issues in `docs/qa/TEST_DRIVE.md` (from #12);
recordings are diagnosis-only and never enter the repo; a changed default needs the user's
OK and an amendment to the ADR that owns it; one slice per session; measure first.

Work, in order (slices are planned by the session that starts the phase, task 0011):

1. **Corpus.** Fresh recordings (`scripts/record-scenario.sh`, `docs/qa/RECORDING_PROTOCOL.md`)
   of S0–S3, plus new scenarios if the work needs them: precise slow moves (small
   targets), and a reproduction of the jumps. Keep each ask about 2 minutes.
2. **Metrics.** One definition per behaviour, computed the same way on a recording replay and
   on a synthetic run: *shake* (direction reversals and high-frequency rms of the cursor at
   rest), *creep* (cursor moves / span at rest), *jumps* (cursor steps beyond a threshold,
   p99 / max step, with cause), *lag* (vs an unfiltered run), *precision* (smallest
   deliberate move that reaches the cursor). The replay tool (local) and the CI tests share
   these definitions.
3. **Settings sweep.** Replay every recording over a grid of settings (gain, smoothing, steady
   zone, 1€ parameters, and any candidate fix) and tabulate the metrics. This is the evidence
   for defaults and for the smallest steady zone that removes shake.
4. **Calibrated synthetic tests.** Model the recorded head and tracker signals with a few
   numbers (tremor amplitude and spectrum, drift rate, tracker noise, jump events) fitted to
   the recordings, so a synthetic run reproduces the recorded metrics within a stated
   tolerance. Only the fitted numbers go into the repo. CI tests then check the behaviour on
   the calibrated signals at every gain 1–60.
5. **Fixes.** Jitter with the smallest dead zone (possibly a different mechanism than the
   current rubber-band zone, e.g. separating shake from drift), and the jump causes. Each
   with a CI test on calibrated signals, a replay metric, and a device check.

*Gate:* shake 0 at rest and no unexplained jumps on the recordings and on the calibrated CI
tests at every gain 1–60; the chosen steady zone is the smallest that achieves it, with the
sweep table in the task file; every 6d row fixed or deferred with a reason; quality gates
green locally and in CI on 3 OS; the user says the app is ready for Phase 7.

## Consequences

- The roadmap gets a 6d row; 6c is closed (task 0010 → done). Open 6c rows carry into 6d:
  #7 (macOS CI flake: needs the failing test name from the job log).
- The replay tool stays outside the repo (`~/ocm-recordings/_tools/replay-tool`), since it
  reads recordings. It needs updating first: `steady_px` was removed in #11 (use
  `Controller::set_steady_zone_cam_px`).
- Phase 7/8 carry-overs are unchanged.
