//! Control handover between the physical mouse and the camera (BEHAVIOR_SPEC R-A2, §2.7,
//! ADR-0006) and the re-acquisition jump bound (R-P4). Pure: positions and times come in as
//! arguments, the caller owns the clock.

use tracking_core::Point2;

use crate::DesktopRect;

/// Handover thresholds. Defaults are the 2018 values (ADR-0006).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandoverParams {
    /// The physical mouse takes control when the OS cursor is farther than this from the last
    /// position the camera commanded (2018 move-stop: 5 px).
    pub move_threshold_px: f32,
    /// The camera takes control back after the mouse has been still this long (2018
    /// `StartOnMouseInactivityDelay`: 4 s).
    pub inactivity_us: u64,
}

impl Default for HandoverParams {
    fn default() -> Self {
        Self {
            move_threshold_px: 5.0,
            inactivity_us: 4_000_000,
        }
    }
}

fn distance(a: Point2, b: Point2) -> f32 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// 2018 move-stop: did the user move the physical mouse away from where the camera put it?
/// A NaN position never counts as a move.
pub fn mouse_moved_away(commanded: Point2, observed: Point2, params: &HandoverParams) -> bool {
    distance(commanded, observed) > params.move_threshold_px
}

/// Inactivity timer for giving control back to the camera: any cursor movement restarts it
/// (2018 `m.a 0x060001fd`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InactivityTimer {
    last: Option<Point2>,
    deadline_us: u64,
}

impl InactivityTimer {
    /// Starts the timer at `now_us`.
    pub fn start(now_us: u64, params: &HandoverParams) -> Self {
        Self {
            last: None,
            deadline_us: now_us.saturating_add(params.inactivity_us),
        }
    }

    /// Records an OS cursor observation. Movement since the previous one restarts the timer.
    pub fn observe(&mut self, position: Point2, now_us: u64, params: &HandoverParams) {
        if self.last.is_some_and(|last| last != position) {
            self.deadline_us = now_us.saturating_add(params.inactivity_us);
        }
        self.last = Some(position);
    }

    pub fn deadline_us(&self) -> u64 {
        self.deadline_us
    }

    pub fn expired(&self, now_us: u64) -> bool {
        now_us >= self.deadline_us
    }

    pub fn remaining_us(&self, now_us: u64) -> u64 {
        self.deadline_us.saturating_sub(now_us)
    }
}

/// Default R-P4 bound: a re-acquired target may move the cursor at most this fraction of the
/// desktop's shorter side (54 px on a 1080 px tall desktop). Provisional, revisit on Phase 6
/// recordings.
pub const REACQUIRE_MAX_JUMP_FRAC: f32 = 0.05;

/// R-P4: may the camera take control back after a re-acquisition that would move the cursor
/// from `from` to `to`? The bound is `max_jump_frac` of the desktop's shorter side.
pub fn within_jump_bound(
    from: Point2,
    to: Point2,
    desktop: DesktopRect,
    max_jump_frac: f32,
) -> bool {
    distance(from, to) <= max_jump_frac * desktop.width.min(desktop.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: HandoverParams = HandoverParams {
        move_threshold_px: 5.0,
        inactivity_us: 4_000_000,
    };

    #[test]
    fn move_stop_threshold_is_strictly_greater_than_5px() {
        let c = Point2::new(100.0, 100.0);
        assert!(!mouse_moved_away(c, Point2::new(105.0, 100.0), &P));
        assert!(!mouse_moved_away(c, Point2::new(103.0, 104.0), &P)); // exactly 5
        assert!(mouse_moved_away(c, Point2::new(105.1, 100.0), &P));
        assert!(mouse_moved_away(c, Point2::new(96.0, 96.0), &P)); // 5.66
        assert!(!mouse_moved_away(c, Point2::new(f32::NAN, 0.0), &P));
    }

    #[test]
    fn inactivity_expires_after_4s_without_movement() {
        let mut t = InactivityTimer::start(1_000, &P);
        let p = Point2::new(10.0, 10.0);
        t.observe(p, 2_000_000, &P);
        t.observe(p, 3_000_000, &P);
        assert!(!t.expired(4_000_999));
        assert_eq!(t.remaining_us(4_000_000), 1_000);
        assert!(t.expired(4_001_000));
    }

    #[test]
    fn any_movement_restarts_inactivity() {
        let mut t = InactivityTimer::start(0, &P);
        t.observe(Point2::new(10.0, 10.0), 1_000_000, &P);
        t.observe(Point2::new(11.0, 10.0), 3_000_000, &P);
        assert!(!t.expired(4_000_000));
        assert_eq!(t.deadline_us(), 7_000_000);
        assert!(t.expired(7_000_000));
    }

    #[test]
    fn jump_bound_uses_shorter_desktop_side() {
        let desk = DesktopRect {
            x: -1920.0,
            y: 0.0,
            width: 3840.0,
            height: 1080.0,
        };
        let a = Point2::new(0.0, 0.0);
        assert!(within_jump_bound(a, Point2::new(54.0, 0.0), desk, 0.05));
        assert!(!within_jump_bound(a, Point2::new(54.1, 0.0), desk, 0.05));
        assert!(!within_jump_bound(
            a,
            Point2::new(f32::NAN, 0.0),
            desk,
            0.05
        ));
    }
}
