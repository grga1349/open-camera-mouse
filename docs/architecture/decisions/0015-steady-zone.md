# ADR-0015: Steady zone (cursor dead zone)

Status: Accepted (user request, test drive #5); **effect not yet good enough** (see below)
Date: 2026-11-15

## Context

The user can't hold the cursor still: tracker noise and head micro-movements move it every
frame (`docs/qa/TEST_DRIVE.md` #5).

## Decision

`pointer_core::steady::SteadyZone`: a rubber-band dead zone on the final cursor position,
after smoothing and limits, with no feedback into the smoother. The cursor holds while the
target stays within `steady_px` (setting, 0..=40 screen px, default 0 = off per the opt-in
rule); beyond it the cursor is pulled along, trailing by at most the radius. Dwell sees the
held position. Options → Pointer → "Steady zone".

## Consequences

- Tests: `steady.rs` (jitter ±4 px held exactly, no jumps, nudge moves by the excess),
  `controller.rs` `steady_zone_*` / `without_steady_zone_*`, `app_tests.rs` `pointer_tab_controls`.
- **User feedback 2026-11-15: "nothing helped, cursor moves 10–20 px".** Likely cause: the
  mapping scales by the whole multi-monitor desktop (2 × 2560 wide → ≈ 48 screen px per camera
  px), so jitter is larger than the zone. To revisit in test-drive part 2 (gain per display,
  adaptive filtering such as the 1€ filter, larger default).
