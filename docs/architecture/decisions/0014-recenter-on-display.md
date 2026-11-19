# ADR-0014: "Main monitor" for recenter and pause

Status: Accepted (user decision, test drive #4)
Date: 2026-11-15
Amends: ADR-0006 (2018 recenter/pause target = desktop centre, kept as the default),
ADR-0012 (`PointerSink::desktop` now also returns the displays)

## Context

With several monitors, the cursor is mapped onto the bounding box of all displays (R-P5).
Recenter, pause and the takeover after a recenter put it at the centre of that box, which on
two side-by-side monitors is the seam between them, or a dead zone with uneven monitors.
The user asked to choose the monitor the cursor starts on (`docs/qa/TEST_DRIVE.md` #4).

## Options considered

1. Limit the cursor to the chosen monitor (the whole head range maps onto it). Finer
   control, but the other displays become unreachable.
2. Only move the **home point**: recenter, pause and takeover put the cursor at the chosen
   display's centre; the mapping still spans every display with the same scale.
3. Both, as two modes.

## Decision

Option 2 (user's choice). A setting `recenter_display: Option<u32>` (`None` = centre of all
displays = 2018 behaviour = default, so it is off by default per the "improvements are
opt-in" rule). The index is into the displays sorted left to right, then top to bottom, the
order a user reads their monitors in; Options → Pointer → "Main monitor" (shown only with more than one monitor) names them
"Monitor N · W × H" (plus "primary" for the display at the global origin).

Takeover keeps `map_raw` unchanged and shifts only the image-space offset by
`(home − desktop centre) / scale` per axis, so the selected point maps to the home point and
every display stays reachable. Screen limits and offset drift work as before.

`PointerSink::desktop` returns `Desktop { bounds, displays }` (the adapters already enumerated
displays for snapping); `Input::DesktopChanged` carries it.

## Consequences

- An index with no display behind it (monitor unplugged) falls back to the centre of all,
  without changing the setting, so it works again when the monitor comes back.
- The index is positional: rearranging monitors in the OS can change which one is
  "Display 2". Accepted for v1; a stable per-display id differs per OS (macOS
  `CGDirectDisplayID`, Windows has none that survives reboots reliably).
- Tests: `crates/app-core/tests/controller.rs` (`recenter_on_*`, `displays_are_numbered_*`,
  `pause_puts_the_cursor_on_the_chosen_display`), `apps/desktop/src/app_tests.rs` (`main_monitor_*`).


## Amendment 2026-11-16 (Phase 6d, #14) — default = the OS primary display

User request (test drive #14, approved): "Main monitor" defaults to the primary monitor, not
the centre of all displays.

- The setting is now `MainMonitor`: `Primary` (default), `All` (centre of all = 2018, scale by
  the largest display), `Display(n)` (index left to right, as before). In TOML it is `"primary"`,
  `"all"` or the index. Old indexes load unchanged. Files from before this change have no
  value for "centre of all", so they now read as `Primary`, which is the intended new default. An
  unknown name also reads as `Primary`.
- `Desktop::primary` (index into the adapter's display list): macOS `CGMainDisplayID` (the menu
  bar display); Windows the monitor at (0, 0), which is the primary monitor by definition of
  virtual-screen coordinates; X11 the RandR primary monitor (root window → the one display).
  The controller maps it into the sorted order. If the adapter can't say, it takes the display
  at the global origin, and if there is none, the centre of all.
- The primary display also sets the mapping scale (ADR-0017) when "Primary" is chosen.
- Options → Screen → "Main monitor": "Primary monitor", "Centre of all monitors", then each
  monitor. The "· primary" tag now comes from the adapter's report, not from the origin.

Tests: `controller.rs` `the_default_main_monitor_is_the_primary_display` (the adapter's report
wins over position), `an_unknown_primary_is_the_display_at_the_origin`,
`with_no_primary_known_or_at_the_origin_the_centre_of_all_is_used`; `settings_store`
`main_monitor_saves_and_loads_every_choice`; kittest `main_monitor_*`. The rest tests pin "Centre
of all" (largest-display scale, the strictest case).
