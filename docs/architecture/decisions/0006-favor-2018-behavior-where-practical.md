# ADR-0006: Favor Camera Mouse 2018 behavior where practical

Status: Accepted (product direction from the user, 2026-11-11)
Date: 2026-11-11

## Context

Task 0003 (`docs/research/BEHAVIOR_SPEC.md`) found that Camera Mouse 2018 and `legacy/` differ
on almost every user-facing behavior (§4 discrepancy matrix) and left several items marked
**DECIDE** (§7) and **Q-R*** (§9). The user's direction: *"it should favor 2018 decisions when
practical."*

## Options considered

- Favor `legacy/` (relative pointer, NCC, F11/F12): the most recently shipped behavior.
- Favor 2018 (absolute pointer, LK, mouse/camera handover): the established product in this
  category, and the one users of Camera Mouse already know.
- Decide each item independently, with no default. This leaves every item open until Phase 6.

## Decision

2018 behavior is the **default answer** to any behavior question. A deviation needs one of
these reasons, recorded where it's made:
**(a) defect**: the 2018 behavior is a bug per static evidence (fix it, keep the intent);
**(b) unknown**: the 2018 behavior is UNRESOLVED (use the best-evidenced alternative until
resolved); **(c) platform**: it depends on Windows-only mechanics (keep the behavior, re-implement
per OS); **(d) prior user decision**: it conflicts with an explicit earlier user decision (ask).

Applied to the current open items (BEHAVIOR_SPEC section refs):

| Item | Resolution | Reason if deviating |
|---|---|---|
| Pointer model (R-P1, Q-R2) | **Absolute**: `desk_center + (p + offset − img_center)·gain·desk/img` | — |
| Per-axis gain (R-P3) | HorizG/VertG, default 6/6 | — |
| Smoothing (R-P2) | Position EMA with damping 0.95. Implemented dt-aware with the time constant equivalent to α=0.95 per 70 ms tick, so the feel matches 2018 at any frame rate | (a)/(c): a fixed 70 ms sleep loop is an implementation artifact, not a behavior |
| Screen limits + offset drift (§2.6) | Adopt W/E/N/S limits (default 0) and the ±1 image px/tick offset drift (converted to a rate) | (a): fix the missing desktop origin on E/S; use the virtual desktop |
| Control handover (R-A2, Q-R3) | Adopt: physical move > 5 px yields control, 4 s inactivity resumes, cursor to desktop centre on camera takeover (AutoCenterScheme 0), On-Screen-Keyboard focus guard as a per-OS adapter concern | (c) for the OSK/focus part |
| Loss policy (§2.4) | 2018 default (`Run_54321…` = false): near the edge and moving out → hold; otherwise → invalidate, release control to the mouse, user re-selects | (a): symmetric edge predicate (R-T4); explicit status, no silent hold (R-T2) |
| 35 px outlier guard | Adopt as the LK preset | Units/resolution still open (Q-R1) |
| First frame after selection | **Not** copied: initialise from the selection frame (R-T3) | (a) |
| Tracking algorithm (ADR-0004) | Pyramidal LK with the 2018 preset is the presumptive baseline; NCC stays a measured comparator; Phase 3 measurements can still overturn this | — |
| Dwell clicking (R-K1, Q-R4) | Off by default; dwell 1 s; 2 s refractory after a click; radius 0.05 | (b): radius unit and the exact `MouseClicker` state machine are UNRESOLVED. Provisionally treat radius as a fraction of screen size and re-arm after refractory *or* leaving the radius, until resolved |
| 5-4-3-2-1 / CenterDetect recenter | Keep legacy's guided countdown → pick frame centre | (b): CenterDetect body UNRESOLVED |
| Periodic auto-recenter (Auto54321) | Offer it, default off, interval 3 min | — (adopt once recenter exists) |
| Pause | Adopt: release control, invalidate tracking, cursor to desktop centre | — |
| Camera disconnect | Adopt detection + message (500 ms poll or OS event) | (c) mechanism per OS |
| Mirroring (R-C1, Q-R5) | **Not decided by this ADR.** 2018's `RevHoriz` default is off, but whether the 2018 tracking space is mirrored (and whether DirectShow drivers pre-mirror) is unresolved (Q-2018-7). Head-right must move the cursor right; Phase 2 decides with that as the constraint | (b) |
| Hotkeys | **F11 start/stop, F12 recenter, fixed** (user decision 2026-11-11). Not 2018's F9/F5/Ctrl | (d) |
| Pointer update rate | Per camera frame with the dt-aware filter, not a 70 ms loop | (a)/(c), see Smoothing |

## Consequences

- Phase 2 can design `pointer-core` around absolute mapping with offset, limits and handover.
  A relative mode is not planned. Add one only on explicit request.
- ADR-0004's evaluation still runs, but its default is tilted toward LK. The comparison has to
  show NCC is clearly better to change the baseline.
- Every deviation from 2018 must cite one of reasons (a)–(d). Undocumented divergence is a
  review finding.
- Open items that feed back here: Q-2018-4 (dwell), Q-2018-5 (CenterDetect), Q-2018-7
  (mirroring). Resolving any of them may replace a provisional choice above with the 2018
  behavior.
