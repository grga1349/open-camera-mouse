//! Time-based (dt-aware) position smoothing (BEHAVIOR_SPEC R-P2, ADR-0006).
//!
//! 2018 damps once per ~70 ms cursor tick: `c = d·m + (1 − d)·c_prev` with `d = Damping`
//! (weight of the new position, default 0.95). Here the same filter runs once per camera
//! frame with the retained fraction scaled to the elapsed time:
//! `retain = smoothing ^ (dt / 70 ms)`, where `smoothing = 1 − d`. At exactly one 70 ms step
//! this is the 2018 formula, and the response to a given motion is the same at any frame
//! rate (fixes L-9).

use tracking_core::Point2;

/// The 2018 cursor tick the smoothing strength is defined against.
pub const REFERENCE_TICK_US: u64 = 70_000;

/// Fraction of the previous position kept after `dt_us` for a smoothing strength (higher =
/// steadier). `smoothing` outside `0..1` is clamped, so the filter can never freeze (L-1).
pub fn retain_fraction(smoothing: f32, dt_us: u64) -> f32 {
    let s = smoothing.clamp(0.0, 0.999);
    if s == 0.0 {
        return 0.0; // No smoothing: always the new position, even with no time passed.
    }
    if dt_us == 0 {
        return 1.0;
    }
    let ticks = dt_us as f64 / REFERENCE_TICK_US as f64;
    (s as f64).powf(ticks) as f32
}

/// Exponential moving average of the cursor position over time.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Smoother {
    state: Option<(Point2, u64)>,
}

impl Smoother {
    /// Forgets the history; the next [`update`](Self::update) snaps to its input.
    pub fn reset(&mut self) {
        self.state = None;
    }

    /// Restarts the filter at `p` (e.g. after the cursor was moved to the desktop centre).
    pub fn reset_to(&mut self, p: Point2, now_us: u64) {
        self.state = Some((p, now_us));
    }

    /// Time of the last update or reset, if any.
    pub fn last_us(&self) -> Option<u64> {
        self.state.map(|(_, t)| t)
    }

    /// The last output, if any.
    pub fn current(&self) -> Option<Point2> {
        self.state.map(|(p, _)| p)
    }

    /// Blends `target` into the state for the time since the previous update and returns
    /// the smoothed position. A clock that goes backwards counts as no time passed.
    pub fn update(&mut self, target: Point2, now_us: u64, smoothing: f32) -> Point2 {
        let last_us = self.state.map_or(now_us, |(_, t)| t);
        let out = match self.state {
            None => target,
            Some((prev, t)) => {
                let k = retain_fraction(smoothing, now_us.saturating_sub(t));
                Point2::new(
                    k * prev.x + (1.0 - k) * target.x,
                    k * prev.y + (1.0 - k) * target.y,
                )
            }
        };
        self.state = Some((out, now_us.max(last_us)));
        out
    }

    /// Overwrites the stored position (keeping its time), e.g. after a limit clamped it, so
    /// the next blend starts from where the cursor really is (2018 stores the clamped `c`).
    pub fn set_current(&mut self, p: Point2) {
        if let Some((_, t)) = self.state {
            self.state = Some((p, t));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn one_reference_tick_is_the_2018_formula() {
        // 2018: c = 0.95·m + 0.05·c_prev.
        let mut s = Smoother::default();
        s.update(Point2::new(0.0, 0.0), 0, 0.05);
        let out = s.update(Point2::new(100.0, -40.0), REFERENCE_TICK_US, 0.05);
        assert!(close(out.x, 95.0, 1e-3), "{out:?}");
        assert!(close(out.y, -38.0, 1e-3), "{out:?}");
    }

    #[test]
    fn zero_smoothing_follows_exactly() {
        let mut s = Smoother::default();
        s.update(Point2::new(0.0, 0.0), 0, 0.0);
        assert_eq!(
            s.update(Point2::new(7.0, 9.0), 1, 0.0),
            Point2::new(7.0, 9.0)
        );
        // Even with no time passed.
        assert_eq!(
            s.update(Point2::new(1.0, 2.0), 1, 0.0),
            Point2::new(1.0, 2.0)
        );
    }

    #[test]
    fn first_update_snaps() {
        let mut s = Smoother::default();
        assert_eq!(
            s.update(Point2::new(5.0, 6.0), 123, 0.9),
            Point2::new(5.0, 6.0)
        );
    }

    /// R-P2: the response to the same motion is the same at 15, 30, 60 and 144 fps.
    #[test]
    fn fps_independent_step_response() {
        let run = |fps: u64, smoothing: f32| -> Vec<(u64, f32)> {
            let dt = 1_000_000 / fps;
            let mut s = Smoother::default();
            s.update(Point2::new(0.0, 0.0), 0, smoothing);
            let mut samples = Vec::new();
            let mut t = 0;
            while t < 1_000_000 {
                t += dt;
                let out = s.update(Point2::new(1000.0, 0.0), t, smoothing);
                samples.push((t, out.x));
            }
            samples
        };
        for smoothing in [0.05, 0.5, 0.95] {
            let reference = |t: u64| 1000.0 * (1.0 - retain_fraction(smoothing, t));
            for fps in [15, 30, 60, 144] {
                for (t, x) in run(fps, smoothing) {
                    assert!(
                        close(x, reference(t), 0.05),
                        "fps {fps} smoothing {smoothing} t {t}: {x} vs {}",
                        reference(t)
                    );
                }
            }
        }
    }

    #[test]
    fn higher_smoothing_is_steadier() {
        let after = |smoothing| {
            let mut s = Smoother::default();
            s.update(Point2::new(0.0, 0.0), 0, smoothing);
            s.update(Point2::new(100.0, 0.0), 33_333, smoothing).x
        };
        assert!(after(0.95) < after(0.5));
        assert!(after(0.5) < after(0.05));
    }

    #[test]
    fn backwards_clock_counts_as_no_time() {
        let mut s = Smoother::default();
        s.update(Point2::new(0.0, 0.0), 1_000_000, 0.5);
        let out = s.update(Point2::new(100.0, 0.0), 500_000, 0.5);
        assert_eq!(out, Point2::new(0.0, 0.0));
        // And the stored time didn't move backwards.
        let out = s.update(Point2::new(100.0, 0.0), 1_070_000, 0.5);
        assert!(close(out.x, 50.0, 1e-3), "{out:?}");
    }
}
