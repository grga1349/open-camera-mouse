# Steady-zone variants for #13 (imprecise)

Phase 6d part 3, task 0011 queue item 1 (#13), user decision 2026-11-16. Question: is there a
variant of the steady zone (`CM2018_JITTER_COMPARISON.md` §5.1) that cuts its hold error
(residual ≈ 0.45 cam px) and still holds still at rest? Target from that doc: reversals/s
≤ 0.2 at rest, residual ≤ 0.15 cam px, lag ≤ 80 ms, S2 step max ≤ 184 px.

**Verdict: the target can't be met by any rule, not even one that knows the future.** On S1
the best possible path with 0.2 reversals/s still has a residual of 0.25 cam px. Reaching
0.15 needs about 0.5 reversals/s. Today's zone 0.5 (0.10 reversals/s, residual 0.43) is
within about 0.1 cam px of the best possible path at that shake. None of the 25 real-time
variants beats it on both axes at once. No production code changed. Next step is the
user's call (§4).

## 1. Method (MEASURED)

- Corpus, harness, metrics: the same as `NAKF_VS_STEADY_ZONE.md` §1. S1 (still, cut at
  23.5 s, 10.0 s of rest, 300 samples) and S2 (turns), local only. Replay tool tests
  `replay_zone_variants` and `replay_hold_bound` (`~/ocm-recordings/_tools/replay-tool`).
  Defaults: gain 22, smoothing 0.20, scale 49.5 × 41.2 screen px per camera px.
- Every variant runs today's chain (1€ → map → EMA → clamp), with the zone stage swapped
  for the variant, then `approach_edge`. All zones are in camera px of head movement.
- Lag is on a 10 ms grid. The zone rows' lag is noisy: today's row reads 110 ms in the NAKF
  doc and 140 ms here, from the same code.

## 2. Real-time variants (MEASURED, S1 at rest)

| variant | reversals/s | residual cam px | lag ms |
|---|---|---|---|
| today: zone 0.5 | **0.10** | 0.43 | 140 |
| zone 0 (no hold) | 3.71 | **0.12** | 70 |
| A zone 0.3, 1€ `min_cutoff` 1.0 / 0.5 / 0.3 | 0.50 / 0.70 / 0.30 | 0.31 / 0.36 / 0.38 | 140–170 |
| A zone 0.25, 1€ `min_cutoff` 1.0 / 0.5 / 0.3 | 0.80 / 0.70 / 0.70 | 0.29 / 0.33 / 0.37 | 110–140 |
| B shake-only zone 0.5, k 0.25 | 0.30 | 0.34 | 170 |
| B shake-only zone 0.5, k ≥ 0.5 | 3.0–3.2 | 0.19–0.21 | 70 |
| B shake-only zone 0.5, k 0.5, floor 0.2 | 0.70 | 0.31 | 140 |
| B shake-only zone 0.75, k 0.25 / k 0.5 floor 0.2 | 0.30 | 0.48–0.51 | 110–140 |
| C settle release zone 0.5, 5 settings | 0.30–1.61 | 0.30–0.44 | 140–170 |

- **A, a smaller zone with a lower 1€ cutoff.** A lower cutoff steadies the input but adds
  lag and residual; at a given shake it doesn't beat the plain zone.
- **B, shake-only** ("trail closes while the head keeps moving one way, full hold on a
  reversal"). Once the trail is allowed to close (k ≥ 0.5), the hold is gone. At rest the
  head *drifts* slowly in one direction, so the trail closes and the shake passes through.
  A refined version (radius regrows after 200–300 ms without a pull; reversal = back by
  0.1–0.2 radius) gave the same numbers: drift pulls on almost every frame, so the regrow
  never fires.
- **C, release on settle** (glide to a slow mean after 150–500 ms inside the zone). It
  barely lowers the residual (0.41–0.44 unless it shakes). The hold error comes from holding
  against slow drift, not from stopping short after moves.
- **S2:** every variant keeps step max at 184–185 px (the 34.2 s fast turn). None of them
  adds a jump.

## 3. Lower bound (MEASURED, non-causal)

`replay_hold_bound`: for each number of direction changes allowed (both axes together), the
path with the smallest error vs the tracker marker over S1's 10 s of rest. It is a least-squares
fit with K monotone pieces per axis (pool-adjacent-violators + dynamic programming). It
knows the future, so no real-time rule can do better at the same shake.

| direction changes in 10 s | reversals/s | best residual cam px |
|---|---|---|
| 0 | 0.0 | 0.44 |
| 1 | 0.1 | 0.33 |
| 2 | 0.2 | **0.25** |
| 3 | 0.3 | 0.22 |
| 5 | 0.5 | **0.15** |
| 7 | 0.7 | 0.12 |
| 10 | 1.0 | 0.09 |

Reading:

- The rest "noise" is slow head wander with ≈ 0.44 cam px rms around a fixed point. Holding
  still means not following it, and that *is* the hold error. ≤ 0.2 reversals/s and ≤ 0.15
  residual contradict each other on this recording (one user, 10 s, a stated limit).
- Today's zone 0.5 sits ≈ 0.1 cam px above the bound at its shake (0.43 vs ≈ 0.30–0.33).
  At most ≈ 5 screen px at gain 22 on the laptop Main can be won without adding shake.
- Zone 0.25 (0.8 reversals/s, 0.29) is ≈ 0.19 above its bound (≈ 0.10). Small zones waste
  more than the default does.

## 4. What this leaves for #13 (the user's call)

1. **Accept some shake.** For example, ≈ 0.5 reversals/s buys a residual of ≈ 0.3 with
   today's real-time rules (zone 0.3), down to 0.15 in the limit. Whether 0.5 slow
   reversals/s looks like "shake" needs a device check. Default change = user OK + an
   ADR-0015/-0017 amendment.
2. **Lower the default gain** (D1 in the 2018 comparison). The hold error in camera px
   doesn't change, but on screen it scales with the gain: 0.43 cam px is 21 px at gain 22 on the
   laptop, ≈ 6 px at gain 6 (2018's default) on the same Main. This is the only lever that cuts the screen-px error
   without adding shake.
3. **Measure "imprecise" differently.** The residual measures holding against wander. If
   the complaint is about deliberate small corrections (needing to push 0.5 cam px before
   anything moves), that is a different metric. `metrics::reached` on a synthetic precision
   probe would show it, and a variant that releases the zone on a deliberate push could help
   there even though it doesn't help the rest residual. Ask the user which one they feel.
4. Close #13 as "zone at the limit" and move on to #12 / #14.

Not recommended: B (shake-only) and C (settle release) as they stand. Either they lose the
hold or they don't lower the residual.

## 5. Sweet spot (user decision 2026-11-16: accept a little shake and imprecision)

The user chose option 1: find the point closest to a real mouse. Replay tool
`replay_zone_sweep` (S1 at rest, defaults, laptop Main scale 49.5 × 41.2). It adds a
**visible-shake** metric: direction changes whose swing on both sides is ≥ 2 (or 5) screen
px, within 500 ms of each other. The raw reversal count also counts sub-pixel wobble that
nobody can see. Residual in screen px = cam px × 49.5.

| zone cam px | reversals/s | visible ≥ 2 px /s | lag ms | residual cam px (screen px) |
|---|---|---|---|---|
| 0 | 3.71 | 0.80 | 70 | 0.12 (6) |
| 0.15 | 1.61 | 0.30 | 110 | 0.22 (11) |
| 0.2 | 1.00 | 0.10 | 110 | 0.26 (13) |
| **0.25** | 0.80 | **0** | **110** | **0.29 (15)** |
| 0.3 | 0.50 | 0 | 140 | 0.31 (15) |
| 0.35 | 0.60 | 0 | 140 | 0.35 (17) |
| 0.4 | 0.40 | 0 | 140 | 0.37 (18) |
| 0.5 (today) | 0.10 | 0 | 140 | 0.43 (21) |

- Every zone from 0.25 up has **no visible shake** on S1. The remaining reversals are
  wobbles under 2 screen px. 0.25 is the smallest such zone. Compared with today it cuts the
  hold error by 30 % (21 → 15 px) and the lag by about 30 ms (lag grid 10 ms, noisy).
- A lower 1€ `min_cutoff` (0.7) only helps at zone 0.15, where it gives 0.1 visible/s with
  residual 0.25. That is no better than zone 0.25 at the default cutoff, so the 1€ filter stays
  as it is.
- S2 step max stays at 184 px for every row. S2 residual and reversals include deliberate
  turns and offset drift, so they aren't used to choose.
- Caveat: the 2 px "visible" threshold is at gain 22 on this Main. The wobble scales with
  gain × Main width, so at gain 60 or on a 2560 px Main the same wobble is ≈ 2.7× / 1.8×
  larger. The zone is in camera px, so its hold scales the same way, but sub-threshold
  wobble may become visible. Settle it on the device check.

**Proposal:** zone **0.25 cam px** (from 0.5), 1€ unchanged. Candidates for the device check:
0.2 / 0.25 / 0.35. A default change needs the user's OK on device, an ADR-0015/-0017
amendment, and the rest CI tests (`the_cursor_does_not_move_at_rest_*`, gains 1–60)
re-checked at the new zone.

**Outcome (2026-11-16):** on the device the user tried 0.2 / 0.25 / 0.35 and chose **0.2**.
It ships as the default (ADR-0015/-0017 amendments), together with the drift cap at screen
limits, which the smaller zone exposed.
