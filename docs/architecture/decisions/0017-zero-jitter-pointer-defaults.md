# ADR-0017: Zero-jitter pointer and steadier tracking as defaults

Status: Accepted (user, 2026-11-15: "do all three, make them default"; "jitter has to be 0")
Date: 2026-11-15

## Context

Test drive part 2 (`docs/qa/TEST_DRIVE.md` #2, #5). The user called cursor jitter and
tracking/loss consistency "horrible". Two short local recordings (still head; head turns) were
replayed offline to diagnose. Recordings are diagnosis-only: none of them, their numbers, or
code that reads them is in the repository. Each failure was reproduced as a fast synthetic test.

Findings:

1. **Loss flapping on a faint target.** The nose texture's `min_eigen` sat around the
   recommended floor of 1.0, so the tracker went Lost ↔ Tracking every frame or two.
   (Lowering the floor alone produced wrong locks on a repeated texture: the aperture problem.)
2. **Cursor jitter at rest.** The tracked point's sub-pixel noise (≈ 0.1 camera px rms) was
   multiplied by the mapping scale. Following 2018, that scale used the *whole bounding
   desktop* width (4000 px for the user's laptop + external display), so 1 camera px = 37.5
   screen px at gain 6, and the cursor moved several px every frame.
3. The steady zone (ADR-0015) was off by default, and too small for the jitter in finding 2.

## Decision

All three changes are on by default (departures from 2018, ADR-0006):

1. **Tracker texture test** (`LkParams::recommended()`, ADR-0011 amendment):
   `min_eigen` 1.0 → 0.25, plus `min_eigen_ratio = 0.05` (λ_min ≥ 0.05·λ_max at level 0).
2. **1€ filter** (Casiez et al. 2012, `pointer_core::one_euro`) on the tracked point in camera
   space, before mapping: `min_cutoff` 1 Hz, `beta` 0.15, `d_cutoff` 4 Hz, using the real
   frame `dt`. It runs in addition to the 2018 smoother (smoothing 0 disables only the latter)
   and is reset whenever the smoother is.
3. **Mapping scale per display** (`pointer_core::scale_reference`, `map_raw_scaled`): the
   camera→screen scale uses the Main monitor (ADR-0014) if set, else the largest display, else
   the bounding desktop. The mapping is still centred on the desktop, and limits and drift
   still apply to the whole desktop, so every display stays reachable.
4. **Steady zone default 5 px** (3 px first; the user raised it to 5 after the device check) (ADR-0015 amendment), after the above, so at rest the cursor
   moves by exactly 0 px. On an axis pinned at a screen limit, the cursor goes to the limit
   itself (the zone's trailing radius would otherwise stop it 5 px short of the menu bar or
   a corner).

## Consequences

- Tests (all synthetic, run in CI):
  - `tracking-core/tests/appearance.rs`: illumination gradient, non-rigid warp,
    scale/rotation, head turns.
  - `fixtures.rs`: `recommended_tracker_never_tracks_flat_occluded_or_wrong_targets`.
  - `pointer-core` `one_euro` tests: tremor reduced ≥ 4×, lag < 1 frame at 60 px/s, no
    overshoot, frame-rate independence.
  - `app-core/tests/controller.rs`: `a_faint_still_target_is_tracked_without_losses`, and
    `the_cursor_does_not_move_at_rest_on_a_wide_desktop` (user's desktop geometry, zero cursor
    change over 270 frames, a 1 px head move still moves it > 15 px).
- The cursor trails a move by up to 5 px and settles a step in ≈ 0.5 s instead of ≈ 0.3 s.
- With the Main monitor set to a small laptop display, reaching a larger second display needs
  more head movement than before; the user may raise the gain.
- Saved settings keep their explicit `steady_px` (e.g. 0); the new default applies only to new
  or reset settings.
- Device check 2026-11-15 (user, steady zone 5): "resolved jittering good part".
- Not fixed: losses during strong head turns (`AppearanceChanged`, the `turn_extreme` scene
  pinned at 31/90 tracked) are the next slice.

## Amendment 2026-11-16 — real gain 1–60, new defaults (test drive #8, #10)

User decision (Phase 6c, ADR-0019). Scaling by one monitor (point 2 above) cut the effective
speed on multi-monitor setups; the user found speed 20 too slow. A first pass (hidden ×3 on a
1–20 slider, steady zone growing with speed) was device-tried and replaced the same day: the
user wants **real numbers** on the controls and more of them.

- **Gain is the real mapping gain, 1–60, step 1, default 24/24** (= the user's preferred 8
  on the ×3 scale). A departure from 2018's 6/6 (ADR-0006).
- **Smoothing default 0.10** (2018 0.05).
- **Steady zone: automatic, 0.5 camera px of head movement** (test drive #11, replacing a
  same-day fixed 11 px option the user found didn't work; ADR-0015 amendment). Radius in
  screen px = 0.5 × gain × Main-monitor size / frame size (faster axis). Measured on the real
  S1 still recording (gain 8/24/60, 14.5 s of sitting still; moves / back-and-forth reversals):
  no zone 202–341 / 6–11; 0.2 cam px (≈ the old 11 px at gain 24) 110–191 / 3–6; **0.5 cam px
  67–116 / 1–2**; 1.0 cam px 41–64 / 1. The same camera-px zone gives nearly the same counts at
  every gain. The moves left are slow head drift (1–2.5 cam px over seconds), not shake; no
  small zone holds that.

Checks: `the_cursor_does_not_move_at_rest_*` (synthetic S1 tremor, the user's two displays,
0 cursor moves at **every gain 1–60**, a 1 camera px move still followed), `settings.rs`
`defaults_are_valid_and_match_2018_except_tuned_ones`; filter settle tests allow 3 px of the
1€ filter's residual lag at gain 24. Saved settings keep their values: a saved gain is now
read as a real gain.

## Amendment 2026-11-16 (later) — defaults gain 22, smoothing 0.20

User decision after the device trial: default gain **22/22**, smoothing **0.20**. Further
tuning of the steady zone and smoothing moves to Phase 6d (ADR-0020), measured on recordings.

## Amendment 2026-11-16 (Phase 6d) — edge approach without a snap (test drive #12)

Decision 4's "a pinned axis goes to the limit itself" released the zone's whole trail in one
frame: a jump of up to the radius onto the edge (S2 58.54 s: 44 px; synthetic slow approach:
33 px = the radius). Replaced by `pointer_core::steady::approach_edge`: the cursor trails the
target by at most `EDGE_TRAIL` (0.25) × the target's distance to the limit it is heading for.
Edges and corners stay reachable (on the limit when the target is), the trail shortens over the
last 4 zone radii, and the cursor moves at most 1.25 × the target there. Measured on S2 (last
steps onto the edge): zone off 26/30/27/20 px, old snap …/44, factor 1.0 …/39, 0.25
31/37/34/25. Trade-off: within 4 radii of a limit the zone holds less toward that limit (the
cursor may creep toward it, never shakes). CI: `a_slow_move_onto_an_edge_does_not_jump`.

## Amendment 2026-11-16 (Phase 6d) — steady zone 0.2 camera px; drift stops at the limit (#13)

User decision after the device check (0.2 / 0.25 / 0.35 tried; "0.2 feels best"), on the
user's direction to accept a little shake and imprecision and get as close to a real mouse
as possible. `docs/research/ZONE_VARIANTS.md`: ≤ 0.2 reversals/s with ≤ 0.15 cam px hold error
is infeasible even non-causally on S1, so the zone is a trade-off. S1 sweep (laptop Main,
gain 22): 0.5 → 0 visible (≥ 2 screen px) reversals/s, hold error 0.43 cam px (21 px), lag
140 ms; **0.2 → 0.1 visible/s, 0.26 cam px (13 px), 110 ms**. The 1€ filter is unchanged (a
lower `min_cutoff` didn't help at the same zone).

- `STEADY_ZONE_CAM_PX` **0.5 → 0.2**.
- **Offset drift capped at the head's overshoot** (`pointer_core::limits::drift_offset`,
  new `past` argument). 2018's whole-tick drift (≈ 0.47 cam px per frame) pulled the mapping
  back inside the limit while the head still pushed past it, so the cursor sawed off the edge.
  A 0.5 zone (33 px at gain 22) hid this; at 0.2 the cursor stepped 5–8 px back off the edge
  and ended 18 px inside it (`a_slow_move_onto_an_edge_does_not_jump` failed). Capped, the
  end state is 2018's (the held head maps onto the limit) without the overshoot. Unit test
  `drift_stops_where_the_head_meets_the_limit`.

Checks: `the_cursor_does_not_move_at_rest_*` at every gain 1–60 (synthetic tremor, still 0
moves), `a_slow_move_onto_an_edge_does_not_jump` (≤ 1.25 × the head's step, stays on the
edge), replay S1/S2 at the app zone: S1 175/299 moves, 12 reversals (0.5: 102 / 1); S2 step
max unchanged. Caveat: visible wobble scales with gain × Main width; re-check at high gains
and on the 2560 px Main in Phase 8.

## Amendment 2026-11-16 (Phase 6d) — default gain 20 (#13)

User decision after the device check (22 / 18 / 16 tried, across two screens): "20 and 22
feel the best"; 20 chosen because most users have a single monitor. Evidence:
`docs/research/ZONE_VARIANTS.md` §6 (`replay_gain_sweep`, zone 0.2). At 20 vs 22, the S1 hold
error goes 13 → 12 px on the laptop Main (the camera-px error is unchanged; on screen it scales with
the gain). Visible shake at rest stays 0.1/s. The S2 turns still cover the whole Main (157 / 109 %
of width / height). Gains ≤ 16 needed bigger head turns than the user made in S2.

- `Settings::default()` gain **22/22 → 20/20**. Saved settings keep their gain; only new
  files, repaired values and "Restore defaults" get 20.

Checks: `defaults_are_valid_and_match_2018_except_tuned_ones`, settings-store load/repair
tests, Options "Restore defaults" kittest. The rest tests already cover every gain from 1 to 60.
