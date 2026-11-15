# Prompt for an online research agent (test drive part 2)

Copy everything below the line into a web-research agent. Bring its report back into
`docs/research/` (e.g. `HEAD_POINTER_STABILITY_RESEARCH.md`) before implementing anything.

---

I'm building an open-source, pure-Rust, webcam head-pointer for people who can't use a mouse
(a rewrite of Camera Mouse 2018). No OpenCV, no ML runtime: the tracking is ours. I need a
research report with current, cited sources (papers, open-source code, docs; give URLs and
dates) on the two problems below. For each technique, give: how it works (math), typical
parameters, CPU cost at 640×480 @ 30 fps, failure modes, and an open-source implementation
I can read (licence named). Rank by expected benefit for this exact setup. Mark claims as
verified (source says so) or inferred.

## Current pipeline

- Camera 640×480 @ 30 fps, grayscale, mirrored.
- The user clicks one point on their face (usually the nose tip). We track that single point
  with pyramidal Lucas–Kanade (21×21 window, 4 levels, 20 iterations, eps 0.03), gain/bias
  compensation, and an appearance gate: reject the step if ZNCC between the current 21×21
  window and the window from the *selection frame* drops below 0.7 (then "lost").
- Recovery after loss: ZNCC template search (coarse-to-fine) within 0.25 × width of the last
  position, margin 0.03 to the second peak, 3 consecutive confirming finds, plus a 41×41
  context ZNCC ≥ 0.7.
- Cursor mapping is absolute: `cursor = desktop_centre + (point − frame_centre) × gain ×
  desktop_width / 640` (gain 6). Then an EMA (weight 0.95 per 70 ms), screen limits, and a
  rubber-band dead zone (radius N px).
- The user's desktop is two 2560-wide monitors, so one camera pixel ≈ 48 screen px.

## Problem 1: the tracked point is lost too easily

Losses happen as the head turns or tilts: the ZNCC against the selection template slides from
0.97 to below 0.7 over about a second (LK drifts along smooth skin, and the appearance
changes with pose). Research:
- Template update strategies that avoid drift (e.g. Matthews–Ishikawa–Baker "template update
  problem", drift-corrected updates, keeping the first template as an anchor).
- Multi-point / patch-cloud tracking around the selected point with robust consensus (median
  flow / forward–backward error, RANSAC similarity), still reporting the user's chosen point.
- Better features on low-texture skin: larger or multi-scale windows, gradient-orientation or
  census/rank transforms, illumination normalisation.
- How other open-source head pointers do it: Enable Viacam (eViacam), Camera Mouse
  (all versions), HeadMouse, Project GameFace (Google), Precision Gaze Mouse, OptiKey, Nose
  Tracker, Talon's head tracking, AsTeRICS. What do they track, how do they detect and recover
  loss, what defaults do they use?
- Lightweight face/landmark detection that could run in pure Rust or with a small pure-Rust
  dependency (e.g. Pico / PICO cascade, NPD, small CNN via `tract`/`candle`): CPU cost, licence,
  and whether it can re-anchor the nose point after a loss.

## Problem 2: the cursor can't be held still

While the user holds their head as still as they can, the cursor wanders 10–20 screen px.
Research:
- Filters for pointer jitter: 1€ filter (Casiez et al. 2012), its recommended parameters for
  head/hand pointers, double-exponential, Kalman with a constant-velocity model, adaptive
  dead zones / hysteresis, "sticky" or speed-dependent gain (pointer acceleration transfer
  functions for head pointers).
- What eViacam / Camera Mouse / GameFace / commercial head pointers (TrackerPro, SmartNav,
  HeadMouse Nano) do for stability: dwell-friendly filtering, motion thresholds, relative vs
  absolute mapping, per-display gain.
- Measured tracker noise of sub-pixel LK on faces, and how to reduce it at the source
  (temporal averaging of several points, larger windows, higher resolution crop around the
  face).
- Accessibility guidance on target sizes and stabilisation for dwell clicking (WCAG, studies
  with motor-impaired users).

## Output

1. A ranked shortlist (top 5 per problem) with the details above.
2. A recommended combination to try first, with starting parameters.
3. A test protocol: what to record (head poses, durations) and which metrics separate good
   from bad (losses per minute while the face is visible, drift off the selected spot,
   cursor wander while still, lag on deliberate moves).
