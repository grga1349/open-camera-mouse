# Noise-adaptive Kalman filter (NAKF) vs the steady zone

Phase 6d part 3, task 0011 queue item 0b (user request, 2026-11-16). Question: can a
noise-adaptive Kalman filter replace the steady zone, and possibly the 1€ filter and the EMA
too (one filter instead of 1€ + EMA + zone)?

**Verdict: no. NAKF loses on shake.** No setting reaches the shake target (≤ 0.2
reversals/s at rest). As a drop-in for 1€ + EMA under the zone it trades lower lag and
residual for 3× the shake and larger steps on fast turns. Per the task rule, #13 continues
with the zone variants (`CM2018_JITTER_COMPARISON.md` §5.1). No production code changed.

## 1. Method (MEASURED)

- Corpus: S1 (still, cut at 23.5 s) and S2 (turns), 2026-11-15, local only. Tool: replay
  tool test `replay_nakf` (`~/ocm-recordings/_tools/replay-tool`, outside the repo), with the
  same metrics as `replay_2018` (`rest_screen`, `lag_vs_raw`): moves/s, reversals/s within
  250 ms, jitter rms vs a ±0.5 s mean, step p95/max, span, lag and residual vs the unfiltered
  marker. Desktop as recorded, defaults (gain 22, smoothing 0.20), scale 49.5 × 41.2 screen
  px per camera px.
- Input: the tracker's marker from a controller replay at the defaults (tracking frames
  after selection + recenter + 0.5 s). Each candidate filter runs in camera space, then the
  point is mapped around the desktop centre and clamped. There is no offset drift, so rest
  metrics are valid but S2 positions near edges are not (the S2 controller row's residual of
  7.6 comes from offset drift and doesn't compare).
- Harness check: today's chain emulated in the same harness (1€ → map → EMA → clamp → zone
  → `approach_edge`, using the real `pointer_core` types) matches the controller: S1 zone 0.5
  0.10 vs 0.10 reversals/s, residual 0.43 vs 0.45 cam px; zone 0 3.7 reversals/s, residual
  0.12, lag 70 ms (doc §2: 3.9 / 0.12 / 70). Lag resolution is 10 ms, and the zone row's lag
  is noisy (140 vs 110 ms).
- NAKF prototype: per-axis constant-velocity Kalman (state position + velocity, white
  acceleration process noise `q`), measurement = marker. Adaptation: (a) **measurement noise
  `r`** learnt from second differences of the measurements while at rest (`var = 6r` for
  white noise), clamped to [0.002, 0.1] cam px²; (b) **process noise** raised by the 2D
  normalised innovation squared, averaged over 0.1 s (manoeuvre detection):
  `q = q_rest · max(1, ε̄/thr)^γ`, shared by both axes like the 1€ cutoff. Grid: `q_rest`
  0.001–2, γ 0–4, thr 2–8, `r` adaptive or fixed 0.01–0.5.

## 2. Results (MEASURED)

### S1 — at rest (target: reversals/s ≤ 0.2, residual ≤ 0.15 cam px, lag ≤ 80 ms)

| Pointer path | reversals/s | jitter rms px | step max px | span cam px | lag ms | residual cam px |
|---|---|---|---|---|---|---|
| **today (controller, zone 0.5)** | **0.10** | 4.0 | 5.5 | 1.76 | 110 | 0.45 |
| today, zone 0 (1€ + EMA) | 3.71 | 7.6 | 6.2 | 2.73 | 70 | 0.12 |
| today, zone 0.3 | 0.50 | 4.7 | 4.9 | 2.13 | 140 | 0.31 |
| 1€ only | 4.92 | 7.9 | 6.8 | 2.83 | 40 | 0.12 |
| KF, fixed (q 0.2, r 0.01) | 4.72 | 10.2 | 12.0 | 3.05 | 70 | 0.11 |
| NAKF q 0.02 γ 4 thr 2 (best lag/residual) | 5.92 | 11.3 | 12.1 | 3.15 | 40 | 0.095 |
| NAKF q 0.2 γ 2 thr 4 | 6.12 | 11.4 | 12.1 | 3.18 | 40 | 0.086 |
| NAKF q 0.001 γ 2 thr 4 (heavy) | 3.51 | 8.5 | 12.1 | 3.06 | 70 | 0.21 |
| NAKF q 0.002 γ 3, fixed r 0.2 (heavier) | 2.41 | 5.5 | 12.0 | 2.57 | 70 | 0.50 |
| NAKF q 0.001 γ 4, fixed r 0.5 (heaviest) | 2.61 | 5.2 | 10.8 | 2.38 | 70 | 0.55 |
| NAKF q 0.2 γ 4 thr 2 **+ zone 0.5** | 0.30 | 5.8 | 8.9 | 2.24 | 70 | 0.35 |
| NAKF q 0.02 γ 4 thr 2 + zone 0.5 | 0.50 | 5.4 | 8.9 | 2.16 | 110 | 0.34 |
| NAKF q 0.2 γ 4 thr 2 + zone 0.3 | 1.10 | 7.6 | 12.0 | 2.64 | 70 | 0.22 |

Rounding the output to whole screen px (what the OS shows) lowers every continuous filter's
reversals by about 25 % (e.g. NAKF q 0.2 γ 2: 6.1 → 4.5; today zone 0: 3.7 → 3.1) and
changes no ranking.

### S2 — head turns

| Pointer path | reversals/s | step p95 px | step max px | span cam px |
|---|---|---|---|---|
| today, zone 0.5 (emulated) | 1.16 | 61.8 | **184** | 49.5 |
| today, zone 0 | 4.29 | 62.0 | 184 | 50.5 |
| NAKF q 0.02 γ 4 thr 2 | 7.78 | 60.9 | 211 | 51.2 |
| NAKF q 0.2 γ 2 thr 4 | 7.52 | 63.4 | 221 | 51.3 |
| NAKF q 0.002 γ 3, fixed r 0.2 | 3.28 | 69.0 | 221 | **60.4** |
| NAKF q 0.2 γ 4 thr 2 + zone 0.5 | 1.81 | 60.1 | 217 | 50.1 |

## 3. Why (INFERRED from the numbers)

1. **The noise at rest isn't white.** The adaptive `r` falls to its floor (0.002 cam px²) on
   both recordings. Frame-to-frame (second-difference) noise is tiny. The motion that makes
   the shake is low-frequency head tremor and drift (1–2.5 cam px over seconds, task 0011
   baseline). A Kalman filter treats that as real motion, which it is. No linear estimator
   separates it from slow intended moves without lag. The heaviest settings still shake at
   2.4–2.6 reversals/s with a residual (0.50–0.55) worse than the zone's.
2. **A continuous estimator always moves.** "Shake 0 at rest" (task 0011 gate) needs a hold,
   that is, something non-linear. The zone is that hold; the filter in front of it only
   decides how much the zone has to absorb.
3. **Constant-velocity overshoot.** The velocity state extrapolates at the end of turns. On
   S2 the step max grows from 184 px to 211–260 px, and heavy settings overshoot the span
   (50 → 60 cam px). That works against #12 (jumps).
4. **The manoeuvre detector fights the drift.** Raising `q` on large innovations makes the
   filter follow drift episodes quickly (lag 40 ms). That is good for lag and residual, but
   it hands the zone more motion to absorb, so shake under the zone triples (0.10 → 0.30).

## 4. What NAKF did show

- Lower lag and residual are available in front of the zone: NAKF + zone 0.5 has lag 70 ms
  and residual 0.35 vs 110–140 ms / 0.43–0.45 today, at 0.30 reversals/s. This is the same
  trade-off as shrinking the zone (today zone 0.3: 0.50 / 0.31 / 140 ms). NAKF does not move
  the trade-off curve. It picks a different point on it, with worse fast-turn steps.
- Measurement noise proper (second difference) is ≤ 0.045 cam px per axis. The tracker is
  not the jitter source; the head is. This matches `OPENCV_LK_PARITY.md`.

## 5. Recommendation

- Keep 1€ + EMA + steady zone. Don't add NAKF, not even as an option. It loses on the target
  that matters (shake) and on fast-turn steps, and adds tuning parameters.
- Continue #13 with the zone variants in `CM2018_JITTER_COMPARISON.md` §5.1, i.e. reduce the
  zone's **hold error** (residual 0.45) without giving up its hold. The data here suggest the
  lever is the zone's release behaviour (how it trails and catches up), not the filter in
  front of it.
- The prototype (`Nakf`, `emulate_cam`, `emulate_ours`, `replay_nakf`) stays in the local
  replay tool. `emulate_ours` is a cheap harness for the zone variants: same input, same
  metrics, validated against the controller.
