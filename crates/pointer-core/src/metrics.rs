//! Cursor-behaviour metrics (Phase 6d, ADR-0020 step 2).
//!
//! One definition per behaviour, computed the same way on a replay of a local recording (the
//! replay tool outside the repo) and on a synthetic run in CI, so the two can be compared and
//! the synthetic signals calibrated against the recordings.
//!
//! Every metric works on a cursor path: one [`PathSample`] per camera frame, `cursor = None`
//! where the camera does not control the cursor (not tracking, mouse owns it, paused). `None`
//! splits the path into segments; no step or window spans a gap. Distances are reported in
//! **camera px** (screen px divided by the screen px per camera px of the mapping, per axis),
//! so a number means the same head movement at every gain and monitor size.
//!
//! - *creep* ([`rest`]): frames on which the cursor moved at all, and its span, at rest.
//! - *shake* ([`rest`]): direction reversals within [`SHAKE_WINDOW_US`], and the
//!   high-frequency rms (the residual after a centred moving average of
//!   2·[`HF_HALF_WINDOW`] + 1 frames, ≈ 0.5 s at 30 fps, so slow drift is excluded).
//! - *jumps* ([`unexplained_jumps`], [`max_unexplained`]): single-frame cursor steps beyond
//!   what the tracker point moved on that frame (a fast turn is not a jump; measured on S2,
//!   plain steps > 2 cam px were all turns). [`jumps`] and [`step_stats`] give raw steps; the
//!   caller attributes causes, since it knows what else happened on that frame.
//! - *lag* ([`lag_ms`]): delay of a run behind an unfiltered reference run over the same input.
//! - *precision*: needs a controlled input, so it is a probe on synthetic runs; [`reached`]
//!   is its pass rule.

use tracking_core::Point2;

/// One frame of a cursor path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathSample {
    pub t_us: u64,
    /// The cursor position the camera set on this frame, `None` when it has no control.
    pub cursor: Option<Point2>,
}

/// Screen px per camera px of the mapping, per axis (`gain · reference size / frame size`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    pub x: f64,
    pub y: f64,
}

/// A reversal counts as shake only if the previous step on that axis was this recent.
pub const SHAKE_WINDOW_US: u64 = 250_000;
/// Half-width (frames) of the moving average that separates shake from drift.
pub const HF_HALF_WINDOW: usize = 7;
/// Steps smaller than this (screen px) are "no move" (float noise from the pipeline).
pub const MOVE_EPS_PX: f64 = 1e-3;
/// [`reached`]: a deliberate move must arrive at least this fraction of its size.
pub const REACHED_FRACTION: f64 = 0.5;

/// Creep and shake of a path at rest.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RestMetrics {
    /// Consecutive controlled frame pairs (the denominator of `moves`).
    pub steps: usize,
    /// Pairs on which the cursor moved (creep, count).
    pub moves: usize,
    /// Direction reversals per axis within [`SHAKE_WINDOW_US`] (shake, count).
    pub reversals: usize,
    /// High-frequency rms, camera px (shake, size).
    pub hf_rms: f64,
    /// Largest per-axis range of the cursor, camera px (creep, size).
    pub span: f64,
}

/// Single-frame step statistics, camera px.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StepStats {
    pub p99: f64,
    pub max: f64,
}

/// A step larger than the threshold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jump {
    /// Time of the frame that arrived after the step.
    pub t_us: u64,
    /// Step size, camera px.
    pub size: f64,
}

fn d(a: Point2, b: Point2, s: Scale) -> (f64, f64) {
    (
        (f64::from(b.x) - f64::from(a.x)) / s.x,
        (f64::from(b.y) - f64::from(a.y)) / s.y,
    )
}

/// Consecutive controlled pairs `(t of the later frame, from, to)`.
fn pairs(path: &[PathSample]) -> impl Iterator<Item = (u64, Point2, Point2)> + '_ {
    path.windows(2)
        .filter_map(|w| Some((w[1].t_us, w[0].cursor?, w[1].cursor?)))
}

/// Maximal runs of controlled samples.
fn segments(path: &[PathSample]) -> Vec<Vec<Point2>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    for s in path {
        match s.cursor {
            Some(p) => cur.push(p),
            None if !cur.is_empty() => out.push(std::mem::take(&mut cur)),
            None => {}
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Creep and shake of a path the user meant to hold still.
pub fn rest(path: &[PathSample], s: Scale) -> RestMetrics {
    let mut m = RestMetrics::default();
    let eps = [MOVE_EPS_PX / s.x, MOVE_EPS_PX / s.y];
    // Per axis: (sign, time) of the previous step that moved on that axis.
    let mut last: [Option<(f64, u64)>; 2] = [None, None];
    for w in path.windows(2) {
        let (t, Some(a), Some(b)) = (w[1].t_us, w[0].cursor, w[1].cursor) else {
            // A gap resets the reversal memory.
            last = [None, None];
            continue;
        };
        m.steps += 1;
        let (dx, dy) = d(a, b, s);
        if dx.abs() * s.x > MOVE_EPS_PX || dy.abs() * s.y > MOVE_EPS_PX {
            m.moves += 1;
        }
        for (axis, v) in [dx, dy].into_iter().enumerate() {
            if v.abs() <= eps[axis] {
                continue;
            }
            let sign = v.signum();
            if let Some((ps, pt)) = last[axis]
                && ps != sign
                && t - pt <= SHAKE_WINDOW_US
            {
                m.reversals += 1;
            }
            last[axis] = Some((sign, t));
        }
    }

    let (mut sq, mut n) = (0.0f64, 0usize);
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    let k = HF_HALF_WINDOW;
    for seg in segments(path) {
        for p in &seg {
            let (x, y) = (f64::from(p.x) / s.x, f64::from(p.y) / s.y);
            (x0, x1, y0, y1) = (x0.min(x), x1.max(x), y0.min(y), y1.max(y));
        }
        if seg.len() < 2 * k + 1 {
            continue;
        }
        for i in k..seg.len() - k {
            let w = &seg[i - k..=i + k];
            let len = w.len() as f64;
            let mx = w.iter().map(|p| f64::from(p.x)).sum::<f64>() / len;
            let my = w.iter().map(|p| f64::from(p.y)).sum::<f64>() / len;
            let rx = (f64::from(seg[i].x) - mx) / s.x;
            let ry = (f64::from(seg[i].y) - my) / s.y;
            sq += rx * rx + ry * ry;
            n += 1;
        }
    }
    if n > 0 {
        m.hf_rms = (sq / n as f64).sqrt();
    }
    if x1 >= x0 {
        m.span = (x1 - x0).max(y1 - y0);
    }
    m
}

/// Steps larger than `threshold` camera px.
pub fn jumps(path: &[PathSample], s: Scale, threshold: f64) -> Vec<Jump> {
    pairs(path)
        .filter_map(|(t, a, b)| {
            let (dx, dy) = d(a, b, s);
            let size = dx.hypot(dy);
            (size > threshold).then_some(Jump { t_us: t, size })
        })
        .collect()
}

/// Cursor steps that the tracker's own movement doesn't explain: on frame `i` the cursor
/// stepped more than `margin` camera px beyond what the tracker point (`marker`, camera px, the
/// same frames as `path`) moved. Fast head turns move both and are not jumps; a step from
/// recentering, a limit, a handover or a tracker re-acquisition shows up here. `size` is the
/// unexplained part.
pub fn unexplained_jumps(
    path: &[PathSample],
    marker: &[Option<Point2>],
    s: Scale,
    margin: f64,
) -> Vec<Jump> {
    excess_steps(path, marker, s)
        .filter(|&(_, e)| e > margin)
        .map(|(t_us, size)| Jump { t_us, size })
        .collect()
}

/// Largest cursor step beyond the tracker's step (camera px), see [`unexplained_jumps`].
pub fn max_unexplained(path: &[PathSample], marker: &[Option<Point2>], s: Scale) -> Option<f64> {
    excess_steps(path, marker, s)
        .map(|(_, e)| e)
        .reduce(f64::max)
}

fn excess_steps<'a>(
    path: &'a [PathSample],
    marker: &'a [Option<Point2>],
    s: Scale,
) -> impl Iterator<Item = (u64, f64)> + 'a {
    path.windows(2)
        .zip(marker.windows(2))
        .filter_map(move |(w, m)| {
            let (a, b) = (w[0].cursor?, w[1].cursor?);
            let (dx, dy) = d(a, b, s);
            let (ma, mb) = (m[0]?, m[1]?);
            let tracker = f64::from(mb.x - ma.x).hypot(f64::from(mb.y - ma.y));
            Some((w[1].t_us, dx.hypot(dy) - tracker))
        })
}

/// p99 and max single-frame step, camera px. `None` without any controlled pair.
pub fn step_stats(path: &[PathSample], s: Scale) -> Option<StepStats> {
    let mut v: Vec<f64> = pairs(path)
        .map(|(_, a, b)| {
            let (dx, dy) = d(a, b, s);
            dx.hypot(dy)
        })
        .collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    Some(StepStats {
        p99: percentile(&v, 0.99),
        max: *v.last().unwrap_or(&0.0),
    })
}

/// Nearest-rank percentile of sorted values.
fn percentile(sorted: &[f64], q: f64) -> f64 {
    let i = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// Delay (ms) of `run` behind `reference`, two paths over the same frames (same length, same
/// times): the shift of up to `max_shift` frames that maximises the Pearson correlation of
/// their per-frame velocities (both axes), refined by a parabola through the peak, times the
/// median frame interval. `None` if the paths are too short or don't correlate.
pub fn lag_ms(reference: &[PathSample], run: &[PathSample], max_shift: usize) -> Option<f64> {
    let vel = |p: &[PathSample]| -> (Vec<f64>, Vec<f64>) {
        p.windows(2)
            .map(|w| match (w[0].cursor, w[1].cursor) {
                (Some(a), Some(b)) => (f64::from(b.x - a.x), f64::from(b.y - a.y)),
                _ => (0.0, 0.0),
            })
            .unzip()
    };
    let ((ax, ay), (bx, by)) = (vel(reference), vel(run));
    let n = ax.len().min(bx.len());
    if max_shift == 0 || n < 4 * max_shift {
        return None;
    }
    let corr: Vec<f64> = (0..=max_shift)
        .map(|k| {
            let x: Vec<f64> = ax[..n - k].iter().chain(&ay[..n - k]).copied().collect();
            let y: Vec<f64> = bx[k..n].iter().chain(&by[k..n]).copied().collect();
            pearson(&x, &y)
        })
        .collect();
    let (k, &peak) = corr.iter().enumerate().max_by(|x, y| x.1.total_cmp(y.1))?;
    if peak <= 0.0 {
        return None;
    }
    let frac = if k > 0 && k < max_shift {
        let (l, r) = (corr[k - 1], corr[k + 1]);
        let den = l - 2.0 * peak + r;
        if den < 0.0 { 0.5 * (l - r) / den } else { 0.0 }
    } else {
        0.0
    };
    let mut dts: Vec<f64> = run
        .windows(2)
        .map(|w| w[1].t_us.saturating_sub(w[0].t_us) as f64 / 1000.0)
        .collect();
    dts.sort_by(f64::total_cmp);
    Some((k as f64 + frac) * percentile(&dts, 0.5))
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let m = x.len() as f64;
    let (mx, my) = (x.iter().sum::<f64>() / m, y.iter().sum::<f64>() / m);
    let (mut xy, mut xx, mut yy) = (0.0, 0.0, 0.0);
    for (p, q) in x.iter().zip(y) {
        xy += (p - mx) * (q - my);
        xx += (p - mx) * (p - mx);
        yy += (q - my) * (q - my);
    }
    xy / (xx * yy).sqrt().max(1e-12)
}

/// Precision probe rule: a deliberate head move of `intended` camera px (x, y) "reaches the
/// cursor" if the cursor moved at least [`REACHED_FRACTION`] of it along its direction.
pub fn reached(before: Point2, after: Point2, intended: (f64, f64), s: Scale) -> bool {
    let (dx, dy) = d(before, after, s);
    let len = intended.0.hypot(intended.1);
    if len <= 0.0 {
        return false;
    }
    let along = (dx * intended.0 + dy * intended.1) / len;
    along >= REACHED_FRACTION * len
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: Scale = Scale { x: 10.0, y: 10.0 };

    fn path(points: &[Option<(f32, f32)>]) -> Vec<PathSample> {
        points
            .iter()
            .enumerate()
            .map(|(i, p)| PathSample {
                t_us: i as u64 * 33_333,
                cursor: p.map(|(x, y)| Point2::new(x, y)),
            })
            .collect()
    }

    #[test]
    fn a_still_cursor_has_no_creep_or_shake() {
        let p = path(&[Some((100.0, 50.0)); 40]);
        let m = rest(&p, S);
        assert_eq!(m.steps, 39);
        assert_eq!((m.moves, m.reversals), (0, 0));
        assert_eq!((m.hf_rms, m.span), (0.0, 0.0));
    }

    #[test]
    fn alternating_steps_are_shake_and_slow_drift_is_creep_only() {
        // ±1 screen px every frame: every step after the first reverses.
        let shake: Vec<_> = (0..40)
            .map(|i| Some((100.0 + (i % 2) as f32, 50.0)))
            .collect();
        let m = rest(&path(&shake), S);
        assert_eq!(m.moves, 39);
        assert_eq!(m.reversals, 38);
        assert!(m.hf_rms > 0.04, "{m:?}");
        assert!((m.span - 0.1).abs() < 1e-9, "{m:?}");

        // A steady 2 screen px per frame ramp: creep, no reversal, almost no hf content.
        let drift: Vec<_> = (0..40)
            .map(|i| Some((100.0 + 2.0 * i as f32, 50.0)))
            .collect();
        let m = rest(&path(&drift), S);
        assert_eq!((m.moves, m.reversals), (39, 0));
        assert!(m.hf_rms < 1e-6, "{m:?}");
        assert!((m.span - 7.8).abs() < 1e-6, "{m:?}");
    }

    #[test]
    fn reversals_further_apart_than_the_window_are_not_shake() {
        // Right, hold 0.5 s, back left: a slow wander, not shake.
        let mut pts = vec![Some((100.0, 0.0)), Some((101.0, 0.0))];
        pts.extend(std::iter::repeat_n(Some((101.0, 0.0)), 15));
        pts.push(Some((100.0, 0.0)));
        assert_eq!(rest(&path(&pts), S).reversals, 0);
    }

    #[test]
    fn gaps_split_steps_and_reset_reversals() {
        let p = path(&[
            Some((0.0, 0.0)),
            Some((1.0, 0.0)),
            None,
            Some((50.0, 0.0)),
            Some((49.0, 0.0)),
        ]);
        let m = rest(&p, S);
        assert_eq!(m.steps, 2, "no step across the gap");
        assert_eq!(m.reversals, 0, "the gap resets the direction memory");
        assert!(jumps(&p, S, 1.0).is_empty(), "the gap is not a 49 px jump");
    }

    #[test]
    fn jumps_and_step_stats_are_in_camera_px() {
        let mut pts: Vec<_> = (0..100).map(|i| Some((i as f32, 0.0))).collect();
        pts[50] = Some((49.0 + 40.0, 30.0));
        let p = path(&pts);
        let j = jumps(&p, S, 3.0);
        assert_eq!(j.len(), 2, "out and back: {j:?}");
        assert_eq!(j[0].t_us, 50 * 33_333);
        assert!(
            (j[0].size - 5.0).abs() < 1e-6,
            "40, 30 screen px = 5 cam px"
        );
        let st = step_stats(&p, S).unwrap();
        assert!((st.max - j[0].size.max(j[1].size)).abs() < 1e-9);
        // 99 steps, 2 of them jumps: the 99th percentile is a jump.
        assert!(st.p99 > 3.0, "{st:?}");
        assert_eq!(step_stats(&path(&[None, Some((0.0, 0.0))]), S), None);
    }

    #[test]
    fn steps_the_tracker_explains_are_not_jumps() {
        // The cursor follows the tracker 1:1 (10 screen px per camera px), including a fast
        // 4 cam px turn; then it alone steps 3 cam px (e.g. a recenter).
        let mk: Vec<Option<Point2>> = (0..20)
            .map(|i| Some(Point2::new(if i >= 5 { 4.0 } else { 0.0 }, 0.0)))
            .collect();
        let mut pts: Vec<_> = mk.iter().map(|m| m.map(|m| (m.x * 10.0, 0.0))).collect();
        for p in pts.iter_mut().skip(12) {
            *p = p.map(|(x, y)| (x + 30.0, y));
        }
        let p = path(&pts);
        assert_eq!(jumps(&p, S, 2.0).len(), 2, "plain steps see the turn too");
        let j = unexplained_jumps(&p, &mk, S, 1.0);
        assert_eq!(j.len(), 1, "{j:?}");
        assert_eq!(j[0].t_us, 12 * 33_333);
        assert!((j[0].size - 3.0).abs() < 1e-6);
        assert!((max_unexplained(&p, &mk, S).unwrap() - 3.0).abs() < 1e-6);
        // No tracker point, no verdict.
        assert!(unexplained_jumps(&p, &vec![None; 20], S, 1.0).is_empty());
    }

    #[test]
    fn lag_recovers_a_known_delay() {
        let wave = |i: usize| {
            (
                40.0 * (i as f32 * 0.21).sin() + 15.0 * (i as f32 * 0.07).cos(),
                0.0,
            )
        };
        let reference = path(&(0..200).map(|i| Some(wave(i))).collect::<Vec<_>>());
        let delayed = path(
            &(0..200usize)
                .map(|i| Some(wave(i.saturating_sub(3))))
                .collect::<Vec<_>>(),
        );
        let lag = lag_ms(&reference, &delayed, 15).unwrap();
        assert!((lag - 100.0).abs() < 10.0, "3 frames ≈ 100 ms, got {lag}");
        assert!(lag_ms(&reference, &reference, 15).unwrap() < 5.0);
        assert_eq!(lag_ms(&reference[..10], &delayed[..10], 15), None);
    }

    #[test]
    fn reached_needs_half_the_move_along_its_direction() {
        let a = Point2::new(0.0, 0.0);
        assert!(reached(a, Point2::new(6.0, 0.0), (1.0, 0.0), S));
        assert!(!reached(a, Point2::new(4.0, 0.0), (1.0, 0.0), S));
        assert!(
            !reached(a, Point2::new(0.0, 10.0), (1.0, 0.0), S),
            "sideways"
        );
        assert!(!reached(a, a, (0.0, 0.0), S));
    }
}
