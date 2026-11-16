# ADR-0015: Steady zone (cursor dead zone)

Status: Accepted (user request, test drive #5); default amended by ADR-0017 (5 px, on)
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

## Amendment 2026-11-15 (ADR-0017)

Default `steady_px` is now **5** (on), behind the 1€ filter and the per-display scale, so a
still head moves the cursor by exactly 0 px. An axis pinned at a screen limit bypasses the
zone so screen edges stay reachable. Users with a saved `steady_px = 0` keep it.

## Amendment 2026-11-16 (test drive #11, ADR-0017 amendment)

The steady zone is **no longer a user option**. User decision: the fixed screen-px zone "does
not work properly" at higher speeds; remove the option and make the zone grow with every
speed. It is now **0.5 camera px of head movement** (`app_core::STEADY_ZONE_CAM_PX`),
converted to screen px through the current gain and the Main monitor scale, so it covers the
same head movement at gain 1–60. `steady_px` is gone from the settings (old files load; the
field is ignored). Debug builds can try another value on the device with
`OCM_DEV_STEADY=<camera px>` (0 = off).
