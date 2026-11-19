//! Display geometry and move bookkeeping shared by every OS backend. Pure, so it is tested on
//! every OS in CI.
//!
//! Two facts measured on the dev Mac (task 0008 D4, ADR-0012 §"Pointer adapter") shape it:
//! - The virtual desktop's bounding box contains dead zones when displays differ in size or
//!   are offset. The OS clamps a move into a dead zone somewhere surprising (macOS put
//!   (5000, 100) at (1440, 100), not on the nearest display), so the adapter snaps targets to
//!   the nearest display itself.
//! - Posted moves land asynchronously: usually 0.3–0.9 ms after the call, rarely up to ~38 ms,
//!   i.e. later than one 33 ms frame.
//!
//! Both would make the OS cursor differ from the controller's commanded point and look like the
//! user grabbing the mouse (handover, 5 px). [`Echo`] recognises positions the adapter itself
//! caused and reports them as the requested point.

use app_core::ports::{DesktopRect, Point2};

/// Bounding rectangle of all displays, the controller's desktop (R-P5). `None` if empty.
pub fn bounding(displays: &[DesktopRect]) -> Option<DesktopRect> {
    let first = displays.first()?;
    let (mut x0, mut y0) = (first.x, first.y);
    let (mut x1, mut y1) = (first.x + first.width, first.y + first.height);
    for d in &displays[1..] {
        x0 = x0.min(d.x);
        y0 = y0.min(d.y);
        x1 = x1.max(d.x + d.width);
        y1 = y1.max(d.y + d.height);
    }
    Some(DesktopRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

/// Clamps to one display's addressable pixels (last pixel = origin + size − 1).
fn clamp_to(p: Point2, d: &DesktopRect) -> Point2 {
    Point2::new(
        p.x.clamp(d.x, (d.x + d.width - 1.0).max(d.x)),
        p.y.clamp(d.y, (d.y + d.height - 1.0).max(d.y)),
    )
}

/// `p` if it lies on a display, otherwise the nearest point on any display. `p` unchanged if
/// there are no displays.
pub fn snap(p: Point2, displays: &[DesktopRect]) -> Point2 {
    let mut best = p;
    let mut best_d2 = f32::INFINITY;
    for d in displays {
        let c = clamp_to(p, d);
        let d2 = (c.x - p.x).powi(2) + (c.y - p.y).powi(2);
        if d2 == 0.0 {
            return p;
        }
        if d2 < best_d2 {
            best_d2 = d2;
            best = c;
        }
    }
    best
}

/// A move is rejected before it reaches the OS if its target isn't a finite point.
pub fn finite(p: Point2) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

/// How long a position the adapter caused still counts as "ours". 2.5× the slowest landing
/// measured (~38 ms) and still short of the next-but-two frame.
pub const ECHO_WINDOW_US: u64 = 100_000;
/// Distance within which an observed cursor position matches one the adapter caused. Covers
/// integer rounding by the OS (warp and X11/Windows coordinates are whole pixels); well under
/// the 5 px handover threshold.
pub const ECHO_TOLERANCE_PX: f32 = 1.5;
const SLOTS: usize = 8;

/// Positions the adapter caused: the latest move's snapped target (always; the cursor resting
/// where it was put is never the user's), plus recent earlier targets and the cursor position
/// read just before each move (still current if a move hasn't landed yet), for
/// [`ECHO_WINDOW_US`].
#[derive(Debug, Clone, Default)]
pub struct Echo {
    recent: [Option<(Point2, u64)>; SLOTS],
    next: usize,
    /// (requested, applied) of the latest move.
    last: Option<(Point2, Point2)>,
}

impl Echo {
    /// Records a move: `requested` is what the controller asked for, `applied` what was sent
    /// to the OS after snapping, `before` the cursor position just before sending.
    pub fn record(
        &mut self,
        requested: Point2,
        applied: Point2,
        before: Option<Point2>,
        now_us: u64,
    ) {
        self.last = Some((requested, applied));
        if let Some(b) = before {
            self.push(b, now_us);
        }
        self.push(applied, now_us);
    }

    fn push(&mut self, p: Point2, now_us: u64) {
        self.recent[self.next] = Some((p, now_us));
        self.next = (self.next + 1) % SLOTS;
    }

    /// Translates an OS cursor reading into the controller's coordinates: a position the
    /// adapter caused reads as the last requested point; anything else (the user moved the
    /// mouse) is passed through unchanged.
    pub fn report(&self, observed: Point2, now_us: u64) -> Point2 {
        let Some((requested, applied)) = self.last else {
            return observed;
        };
        let near = |p: &Point2| (p.x - observed.x).hypot(p.y - observed.y) <= ECHO_TOLERANCE_PX;
        let ours = near(&applied)
            || self
                .recent
                .iter()
                .flatten()
                .any(|(p, at)| now_us.saturating_sub(*at) <= ECHO_WINDOW_US && near(p));
        if ours { requested } else { observed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> DesktopRect {
        DesktopRect {
            x,
            y,
            width,
            height,
        }
    }

    /// The dev Mac's layout (D4): main 1440×900 at the origin, a 2560×1440 display to its
    /// right, raised so its origin is negative.
    fn dev_mac() -> Vec<DesktopRect> {
        vec![
            rect(0.0, 0.0, 1440.0, 900.0),
            rect(1440.0, -540.0, 2560.0, 1440.0),
        ]
    }

    #[test]
    fn bounding_box_handles_negative_origins() {
        assert_eq!(
            bounding(&dev_mac()),
            Some(rect(0.0, -540.0, 4000.0, 1440.0))
        );
        assert_eq!(
            bounding(&[
                rect(-1920.0, 0.0, 1920.0, 1080.0),
                rect(0.0, 0.0, 1920.0, 1080.0)
            ]),
            Some(rect(-1920.0, 0.0, 3840.0, 1080.0))
        );
        assert_eq!(bounding(&[]), None);
    }

    #[test]
    fn points_on_a_display_are_unchanged() {
        let d = dev_mac();
        for p in [
            Point2::new(0.0, 0.0),
            Point2::new(1439.0, 899.0),
            Point2::new(3000.0, -400.0),
            Point2::new(3999.0, 899.0),
        ] {
            assert_eq!(snap(p, &d), p);
        }
    }

    #[test]
    fn dead_zone_and_outside_points_go_to_the_nearest_display() {
        let d = dev_mac();
        // Above the main display, left of the second one: nearest is the main display's top
        // edge (macOS itself put this at (128.8, 1.3) for a posted move).
        assert_eq!(
            snap(Point2::new(100.0, -300.0), &d),
            Point2::new(100.0, 0.0)
        );
        // Close to the second display's left edge.
        assert_eq!(
            snap(Point2::new(1400.0, -500.0), &d),
            Point2::new(1440.0, -500.0)
        );
        // Right of everything: the second display's right edge (macOS put it at x 1440).
        assert_eq!(
            snap(Point2::new(5000.0, 100.0), &d),
            Point2::new(3999.0, 100.0)
        );
        // Below everything.
        assert_eq!(
            snap(Point2::new(700.0, 2000.0), &d),
            Point2::new(700.0, 899.0)
        );
    }

    #[test]
    fn snap_without_displays_is_identity() {
        assert_eq!(snap(Point2::new(5.0, 6.0), &[]), Point2::new(5.0, 6.0));
    }

    #[test]
    fn finite_rejects_nan_and_infinity() {
        assert!(finite(Point2::new(1.0, 2.0)));
        assert!(!finite(Point2::new(f32::NAN, 2.0)));
        assert!(!finite(Point2::new(1.0, f32::INFINITY)));
    }

    #[test]
    fn echo_passes_through_before_any_move() {
        let e = Echo::default();
        assert_eq!(e.report(Point2::new(3.0, 4.0), 0), Point2::new(3.0, 4.0));
    }

    #[test]
    fn a_snapped_move_reads_as_requested() {
        let mut e = Echo::default();
        let req = Point2::new(100.0, -300.0);
        e.record(
            req,
            Point2::new(100.0, 0.0),
            Some(Point2::new(50.0, 50.0)),
            0,
        );
        // OS rounding to whole pixels is within tolerance.
        assert_eq!(e.report(Point2::new(100.0, 0.4), 33_000), req);
    }

    #[test]
    fn a_move_that_has_not_landed_yet_reads_as_requested() {
        let mut e = Echo::default();
        e.record(
            Point2::new(10.0, 10.0),
            Point2::new(10.0, 10.0),
            Some(Point2::new(500.0, 500.0)),
            0,
        );
        e.record(
            Point2::new(20.0, 10.0),
            Point2::new(20.0, 10.0),
            Some(Point2::new(10.0, 10.0)),
            33_000,
        );
        // 38 ms late: still at the first target, or even still before the first move.
        assert_eq!(
            e.report(Point2::new(10.0, 10.0), 66_000),
            Point2::new(20.0, 10.0)
        );
        assert_eq!(
            e.report(Point2::new(500.0, 500.0), 66_000),
            Point2::new(20.0, 10.0)
        );
    }

    #[test]
    fn a_user_move_passes_through() {
        let mut e = Echo::default();
        e.record(
            Point2::new(10.0, 10.0),
            Point2::new(10.0, 10.0),
            Some(Point2::new(9.0, 10.0)),
            0,
        );
        let moved = Point2::new(40.0, 12.0);
        assert_eq!(e.report(moved, 33_000), moved);
        // Just past the tolerance is the user's, too.
        let nudged = Point2::new(10.0, 11.6);
        assert_eq!(e.report(nudged, 33_000), nudged);
    }

    #[test]
    fn old_positions_stop_counting_after_the_window() {
        let mut e = Echo::default();
        e.record(
            Point2::new(10.0, 10.0),
            Point2::new(10.0, 10.0),
            Some(Point2::new(500.0, 500.0)),
            0,
        );
        e.record(
            Point2::new(10.0, 10.0),
            Point2::new(10.0, 10.0),
            None,
            200_000,
        );
        // The user put the mouse back exactly where it was before the first move, long ago.
        let back = Point2::new(500.0, 500.0);
        assert_eq!(e.report(back, 250_000), back);
        // The current target still counts.
        assert_eq!(
            e.report(Point2::new(10.0, 10.0), 250_000),
            Point2::new(10.0, 10.0)
        );
    }

    #[test]
    fn the_latest_target_never_expires() {
        let mut e = Echo::default();
        let req = Point2::new(100.0, -300.0);
        e.record(Point2::new(90.0, -300.0), Point2::new(90.0, 0.0), None, 0);
        e.record(req, Point2::new(100.0, 0.0), None, 1_000);
        // Frames stalled for a second: the cursor still resting on the snapped point is ours…
        assert_eq!(e.report(Point2::new(100.0, 0.0), 1_000_000), req);
        // …but the earlier target has expired.
        assert_eq!(
            e.report(Point2::new(90.0, 0.0), 1_000_000),
            Point2::new(90.0, 0.0)
        );
    }
}
