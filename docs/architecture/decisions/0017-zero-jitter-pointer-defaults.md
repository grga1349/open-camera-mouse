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
