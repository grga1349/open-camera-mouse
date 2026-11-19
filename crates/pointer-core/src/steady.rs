//! Steady zone (test drive #5, ADR-0015): holds the cursor perfectly still against tracker
//! noise and unintended micro-movements of the head.
//!
//! A "rubber band" dead zone on the final cursor position: while the target stays within
//! `radius` of the cursor, the cursor doesn't move; when the target leaves that disc, the
//! cursor is pulled along the line towards it until the target is back on the edge. So the
//! cursor never jumps, follows any motion beyond the radius one to one, and trails a moving
//! target by at most `radius`. Radius 0 is off (the cursor is the target).
//!
//! Runs after smoothing and screen limits and feeds nothing back into them: the smoother keeps
//! following the head; only what is sent to the OS (and to dwell) is held.
//!
//! Near a screen limit [`approach_edge`] shortens the trail so the edge is reached without a
//! jump: snapping the cursor onto the edge released up to the radius in one frame (test drive
//! #12, ADR-0020).

use tracking_core::Point2;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SteadyZone {
    cursor: Option<Point2>,
}

impl SteadyZone {
    /// Forget the cursor: the next update returns its target unchanged.
    pub fn reset(&mut self) {
        self.cursor = None;
    }

    /// The cursor is at `p` (takeover, re-acquisition).
    pub fn reset_to(&mut self, p: Point2) {
        self.cursor = Some(p);
    }

    /// The cursor position for `target`, with a dead zone of `radius` screen px.
    pub fn update(&mut self, target: Point2, radius: f32) -> Point2 {
        let out = match self.cursor {
            Some(c) if radius > 0.0 => {
                let (dx, dy) = (target.x - c.x, target.y - c.y);
                let d = dx.hypot(dy);
                if d <= radius || !d.is_finite() {
                    c
                } else {
                    let k = (d - radius) / d;
                    Point2::new(c.x + dx * k, c.y + dy * k)
                }
            }
            _ => target,
        };
        self.cursor = Some(out);
        out
    }
}

/// [`approach_edge`]: how much trail the cursor may keep per screen px of the target's
/// distance to a limit. The trail starts shortening 1 / `EDGE_TRAIL` zone radii from the
/// limit and the cursor then moves at most `1 + EDGE_TRAIL` times as fast as the target.
/// Chosen on S2 (ADR-0020): at 0.25 the last steps onto an edge stay within ≈ 7 px of the
/// zone-off cursor's (1.0: 19 px; the old snap: 24 px).
pub const EDGE_TRAIL: f32 = 0.25;

/// One axis of the cursor near the limits `[lo, hi]` (test drive #12, ADR-0020): the cursor
/// trails `target` by at most [`EDGE_TRAIL`] times the target's distance to the limit it is
/// heading for. Farther than `radius / EDGE_TRAIL` from a limit the zone already guarantees
/// that, so nothing changes; nearer, the trail shortens smoothly and the cursor is exactly on
/// the limit when the target is (menu bar and corners stay reachable, ADR-0017). Moving away
/// from a limit is held by the zone as usual.
pub fn approach_edge(cursor: f32, target: f32, lo: f32, hi: f32) -> f32 {
    let a = target - EDGE_TRAIL * (hi - target);
    let b = target + EDGE_TRAIL * (target - lo);
    if a <= b { cursor.clamp(a, b) } else { cursor }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dist(a: Point2, b: Point2) -> f32 {
        (a.x - b.x).hypot(a.y - b.y)
    }

    /// Deterministic jitter in [−a, a] per axis (no RNG dependency).
    fn jitter(i: u32, a: f32) -> (f32, f32) {
        let h = |k: u32| {
            let z = k.wrapping_mul(0x9E37_79B9).rotate_left(13) ^ 0x5bd1_e995;
            (z.wrapping_mul(0x27d4_eb2d) >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0
        };
        (h(2 * i) * a, h(2 * i + 1) * a)
    }

    #[test]
    fn zero_radius_is_off() {
        let mut s = SteadyZone::default();
        for i in 0..50 {
            let t = Point2::new(100.0 + i as f32 * 0.3, 50.0);
            assert_eq!(s.update(t, 0.0), t);
        }
    }

    #[test]
    fn first_update_starts_at_the_target() {
        let mut s = SteadyZone::default();
        let t = Point2::new(300.0, 200.0);
        assert_eq!(s.update(t, 6.0), t);
    }

    #[test]
    fn jitter_inside_the_radius_holds_the_cursor_perfectly_still() {
        let mut s = SteadyZone::default();
        let home = Point2::new(960.0, 540.0);
        s.reset_to(home);
        // Measured on the test-drive recording: ±4 px of wander while holding still.
        for i in 0..1000 {
            let (dx, dy) = jitter(i, 4.0);
            let out = s.update(Point2::new(home.x + dx, home.y + dy), 6.0);
            assert_eq!(out, home, "frame {i}");
        }
    }

    #[test]
    fn motion_beyond_the_radius_is_followed_without_a_jump() {
        let mut s = SteadyZone::default();
        s.reset_to(Point2::new(0.0, 0.0));
        let r = 6.0;
        let mut last = Point2::new(0.0, 0.0);
        // A slow, steady move right, 2 px per frame.
        for i in 1..=100 {
            let t = Point2::new(2.0 * i as f32, 0.0);
            let out = s.update(t, r);
            assert!(
                dist(out, last) <= 2.0 + 1e-4,
                "never moves more than the target"
            );
            assert!(dist(out, t) <= r + 1e-4, "trails by at most the radius");
            last = out;
        }
        // After the start-up, one to one: the cursor moved 200 − 6 px.
        assert!((last.x - 194.0).abs() < 1e-3, "{last:?}");
    }

    #[test]
    fn a_small_deliberate_nudge_moves_the_cursor_by_the_excess() {
        let mut s = SteadyZone::default();
        s.reset_to(Point2::new(500.0, 500.0));
        // Pushing 8 px down with a 6 px zone moves the cursor 2 px: fine positioning works.
        let out = s.update(Point2::new(500.0, 508.0), 6.0);
        assert!(dist(out, Point2::new(500.0, 502.0)) < 1e-4, "{out:?}");
        // Relaxing back inside the zone leaves it there.
        assert_eq!(s.update(Point2::new(500.0, 503.0), 6.0), out);
    }

    #[test]
    fn reset_drops_the_hold() {
        let mut s = SteadyZone::default();
        s.reset_to(Point2::new(0.0, 0.0));
        s.reset();
        let t = Point2::new(3.0, 0.0);
        assert_eq!(s.update(t, 6.0), t);
    }

    #[test]
    fn far_from_the_limits_the_cursor_is_untouched() {
        // Target 300 px from either limit, cursor 30 px behind it.
        assert_eq!(approach_edge(270.0, 300.0, 0.0, 1919.0), 270.0);
        assert_eq!(approach_edge(1649.0, 1619.0, 0.0, 1919.0), 1649.0);
    }

    #[test]
    fn on_a_limit_the_cursor_is_on_it() {
        assert_eq!(approach_edge(1889.0, 1919.0, 0.0, 1919.0), 1919.0);
        assert_eq!(approach_edge(30.0, 0.0, 0.0, 1919.0), 0.0);
    }

    /// A target creeping onto the edge with a 30 px zone: the cursor closes the trail in
    /// steps of at most `1 + EDGE_TRAIL` times the target's, never a jump, and ends on it.
    #[test]
    fn approaching_a_limit_closes_the_trail_without_a_jump() {
        let (lo, hi, r) = (0.0, 1919.0, 30.0);
        let mut zone = SteadyZone::default();
        zone.reset_to(Point2::new(1800.0, 500.0));
        let mut last = 1800.0f32;
        let mut t = 1800.0f32;
        while t < hi {
            t = (t + 2.0).min(hi);
            let mut c = zone.update(Point2::new(t, 500.0), r);
            c.x = approach_edge(c.x, t, lo, hi);
            zone.reset_to(c);
            let max = 2.0 * (1.0 + EDGE_TRAIL) + 1e-3;
            assert!(c.x - last <= max && c.x >= last, "{last} -> {}", c.x);
            last = c.x;
        }
        assert_eq!(last, hi);
    }

    #[test]
    fn collapsed_limits_leave_the_cursor_alone() {
        // `axis_range` collapses impossible insets to one point: lo == hi, target there.
        assert_eq!(approach_edge(400.0, 500.0, 500.0, 500.0), 500.0);
    }
}
