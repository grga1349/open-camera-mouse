//! Dwell clicking (BEHAVIOR_SPEC R-K1/R-K2, ADR-0006 provisional defaults).
//!
//! Hold the pointer inside a radius around where it came to rest for `dwell_us` and one
//! click fires. It then fires again only after the pointer leaves the radius or the
//! refractory period passes (and it has been still for another `dwell_us`). Nothing fires,
//! and nothing accumulates, while the caller says the pointer isn't eligible (tracking not
//! healthy, or the camera not in control).

use tracking_core::Point2;

use crate::DesktopRect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DwellParams {
    pub dwell_us: u64,
    pub refractory_us: u64,
    /// Radius as a fraction of the desktop's shorter side (2018 `MouseClickRadius` 0.05;
    /// unit provisional, Q-2018-4).
    pub radius_frac: f32,
}

impl Default for DwellParams {
    fn default() -> Self {
        Self {
            dwell_us: 1_000_000,
            refractory_us: 2_000_000,
            radius_frac: 0.05,
        }
    }
}

impl DwellParams {
    pub fn radius_px(&self, desktop: DesktopRect) -> f32 {
        self.radius_frac * desktop.width.min(desktop.height)
    }
}

/// Dwell state machine. Feed it every pointer update.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dwell {
    /// Where the pointer came to rest and since when.
    anchor: Option<(Point2, u64)>,
    /// After a click: no further click before this time unless the pointer leaves.
    refractory_until: Option<u64>,
}

impl Dwell {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Progress of the current dwell in `0..=1`, if one is running and armed.
    pub fn progress(&self, now_us: u64, params: &DwellParams) -> Option<f32> {
        let (_, since) = self.anchor?;
        if self.refractory_until.is_some_and(|r| now_us < r) {
            return None;
        }
        let start = since.max(self.refractory_until.unwrap_or(0));
        let p = now_us.saturating_sub(start) as f32 / params.dwell_us.max(1) as f32;
        Some(p.min(1.0))
    }

    /// One pointer update. Returns the click position when a dwell completes.
    ///
    /// `eligible = false` (lost, held, mouse in control, paused…) resets the dwell: R-K2.
    pub fn update(
        &mut self,
        position: Point2,
        now_us: u64,
        eligible: bool,
        params: &DwellParams,
        desktop: DesktopRect,
    ) -> Option<Point2> {
        if !eligible || !(position.x.is_finite() && position.y.is_finite()) {
            self.reset();
            return None;
        }
        let radius = params.radius_px(desktop);
        let (anchor, since) = match self.anchor {
            Some((a, t)) if (a.x - position.x).hypot(a.y - position.y) <= radius => (a, t),
            _ => {
                // Moved away (or first sample): a new dwell starts here, re-armed (R-K1).
                self.anchor = Some((position, now_us));
                self.refractory_until = None;
                return None;
            }
        };
        if self.refractory_until.is_some_and(|r| now_us < r) {
            return None;
        }
        // After a refractory period the next dwell is timed from its end.
        let start = since.max(self.refractory_until.unwrap_or(0));
        if now_us.saturating_sub(start) >= params.dwell_us {
            self.refractory_until = Some(now_us.saturating_add(params.refractory_us));
            return Some(anchor);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DESK: DesktopRect = DesktopRect {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };
    const P: DwellParams = DwellParams {
        dwell_us: 1_000_000,
        refractory_us: 2_000_000,
        radius_frac: 0.05, // 54 px
    };

    /// Runs `secs` seconds at 30 fps with `pos(t)` and `eligible(t)`, returns click times.
    fn run(
        d: &mut Dwell,
        from_us: u64,
        to_us: u64,
        pos: impl Fn(u64) -> Point2,
        eligible: impl Fn(u64) -> bool,
    ) -> Vec<u64> {
        let mut clicks = Vec::new();
        let mut t = from_us;
        while t <= to_us {
            if d.update(pos(t), t, eligible(t), &P, DESK).is_some() {
                clicks.push(t);
            }
            t += 33_333;
        }
        clicks
    }

    #[test]
    fn fires_once_per_dwell_then_waits_for_refractory() {
        let mut d = Dwell::default();
        let still = |_| Point2::new(500.0, 500.0);
        // 2.9 s still: one click at ~1 s, none during the 2 s refractory.
        let clicks = run(&mut d, 0, 2_900_000, still, |_| true);
        assert_eq!(clicks.len(), 1, "{clicks:?}");
        assert!((1_000_000..1_040_000).contains(&clicks[0]));
    }

    #[test]
    fn small_jitter_inside_the_radius_still_clicks() {
        let mut d = Dwell::default();
        let jitter = |t: u64| Point2::new(500.0 + ((t / 33_333) % 7) as f32 * 5.0, 500.0);
        assert_eq!(run(&mut d, 0, 1_500_000, jitter, |_| true).len(), 1);
    }

    #[test]
    fn leaving_the_radius_re_arms_before_the_refractory_ends() {
        let mut d = Dwell::default();
        let pos = |t: u64| {
            if t < 1_500_000 {
                Point2::new(500.0, 500.0)
            } else {
                Point2::new(800.0, 500.0)
            }
        };
        let clicks = run(&mut d, 0, 2_700_000, pos, |_| true);
        assert_eq!(clicks.len(), 2, "{clicks:?}");
        assert!(
            clicks[1] >= 2_500_000,
            "second dwell timed from the move: {clicks:?}"
        );
    }

    #[test]
    fn steady_pointer_clicks_again_after_refractory_and_another_dwell() {
        let mut d = Dwell::default();
        let clicks = run(&mut d, 0, 4_100_000, |_| Point2::new(10.0, 10.0), |_| true);
        assert_eq!(clicks.len(), 2, "{clicks:?}");
        assert!(clicks[1] >= 4_000_000);
    }

    /// R-K2: never while lost or not in control, and an ineligible spell resets the timer.
    #[test]
    fn never_fires_while_ineligible() {
        let mut d = Dwell::default();
        let still = |_| Point2::new(500.0, 500.0);
        assert!(run(&mut d, 0, 5_000_000, still, |_| false).is_empty());
        // Eligible for 0.9 s, lost for a frame, eligible again: the dwell starts over.
        let elig = |t: u64| !(900_000..933_334).contains(&t);
        let clicks = run(&mut d, 0, 1_800_000, still, elig);
        assert!(clicks.is_empty(), "{clicks:?}");
    }

    #[test]
    fn progress_reports_the_running_dwell() {
        let mut d = Dwell::default();
        let p = Point2::new(100.0, 100.0);
        d.update(p, 0, true, &P, DESK);
        assert_eq!(d.progress(500_000, &P), Some(0.5));
        d.update(p, 1_000_000, true, &P, DESK);
        assert_eq!(d.progress(1_500_000, &P), None, "refractory");
        d.reset();
        assert_eq!(d.progress(0, &P), None);
    }
}
