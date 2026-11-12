//! Screen limits and offset drift (ADR-0006, BEHAVIOR_SPEC §2.6).
//!
//! 2018 keeps the cursor inside W/E/N/S limits and, while it is pinned at one, drifts the
//! mapping offset by 1 image px per 70 ms tick back toward the centre, so holding the head
//! past an edge slowly re-centres the mapping. Here:
//!
//! - the limits are insets in screen px from the edges of the full virtual desktop, all four
//!   including the desktop origin (2018 omits it on E and S, Q-2018-3);
//! - the drift is a rate ([`DRIFT_IMAGE_PX_PER_S`]) applied over the elapsed time, so it
//!   doesn't depend on the frame rate.

use tracking_core::Point2;

use crate::DesktopRect;
use crate::smoothing::REFERENCE_TICK_US;

/// 2018: 1 image px per 70 ms tick ≈ 14.3 image px/s.
pub const DRIFT_IMAGE_PX_PER_S: f32 = 1_000_000.0 / REFERENCE_TICK_US as f32;

/// Distance the cursor keeps from each desktop edge, in screen px.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Limits {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

/// Which limit, if any, the cursor is pinned at on one axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pin {
    #[default]
    Free,
    /// Past the low edge (W or N).
    Low,
    /// Past the high edge (E or S).
    High,
}

/// The result of [`apply_limits`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limited {
    pub position: Point2,
    pub x: Pin,
    pub y: Pin,
}

/// The allowed `[lo, hi]` range on one axis. Insets that leave no room collapse to the
/// middle of what's left instead of inverting the range.
fn axis_range(origin: f32, size: f32, lo_inset: f32, hi_inset: f32) -> (f32, f32) {
    let lo = origin + lo_inset.max(0.0);
    // Last addressable pixel is origin + size − 1.
    let hi = origin + size - 1.0 - hi_inset.max(0.0);
    if lo <= hi {
        (lo, hi)
    } else {
        let mid = (lo + hi) / 2.0;
        (mid, mid)
    }
}

fn clamp_axis(v: f32, (lo, hi): (f32, f32)) -> (f32, Pin) {
    if v > hi {
        (hi, Pin::High)
    } else if v < lo {
        (lo, Pin::Low)
    } else {
        (v, Pin::Free)
    }
}

/// Clamps `p` into the limited desktop and reports which edges it was pinned at. A point
/// exactly on a limit is not pinned (2018 uses strict comparisons).
pub fn apply_limits(p: Point2, desktop: DesktopRect, limits: Limits) -> Limited {
    let (x, px) = clamp_axis(
        p.x,
        axis_range(desktop.x, desktop.width, limits.left, limits.right),
    );
    let (y, py) = clamp_axis(
        p.y,
        axis_range(desktop.y, desktop.height, limits.top, limits.bottom),
    );
    Limited {
        position: Point2::new(x, y),
        x: px,
        y: py,
    }
}

/// Drifts the image-space `offset` back toward the centre for every pinned axis over
/// `dt_us`. Pinned high (E/S) means the head is too far that way, so the offset decreases.
pub fn drift_offset(offset: Point2, limited: &Limited, dt_us: u64) -> Point2 {
    let step = DRIFT_IMAGE_PX_PER_S * (dt_us as f32 / 1_000_000.0);
    let drift = |pin: Pin| match pin {
        Pin::Free => 0.0,
        Pin::Low => step,
        Pin::High => -step,
    };
    Point2::new(offset.x + drift(limited.x), offset.y + drift(limited.y))
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

    #[test]
    fn zero_limits_clamp_to_the_desktop() {
        let l = apply_limits(Point2::new(2500.0, -3.0), DESK, Limits::default());
        assert_eq!(l.position, Point2::new(1919.0, 0.0));
        assert_eq!((l.x, l.y), (Pin::High, Pin::Low));
        let inside = apply_limits(Point2::new(1919.0, 0.0), DESK, Limits::default());
        assert_eq!((inside.x, inside.y), (Pin::Free, Pin::Free));
    }

    /// The E/S fix: with a negative origin, every edge includes it (2018 E/S didn't).
    #[test]
    fn limits_include_the_desktop_origin_on_every_edge() {
        let desk = DesktopRect {
            x: -1280.0,
            y: -200.0,
            width: 3200.0,
            height: 1280.0,
        };
        let lim = Limits {
            left: 10.0,
            right: 20.0,
            top: 30.0,
            bottom: 40.0,
        };
        let hi = apply_limits(Point2::new(1e6, 1e6), desk, lim);
        assert_eq!(
            hi.position,
            Point2::new(-1280.0 + 3199.0 - 20.0, -200.0 + 1279.0 - 40.0)
        );
        let lo = apply_limits(Point2::new(-1e6, -1e6), desk, lim);
        assert_eq!(lo.position, Point2::new(-1270.0, -170.0));
    }

    #[test]
    fn insets_larger_than_the_desktop_collapse_to_the_middle() {
        let lim = Limits {
            left: 1500.0,
            right: 1500.0,
            top: 0.0,
            bottom: 0.0,
        };
        let a = apply_limits(Point2::new(0.0, 500.0), DESK, lim);
        let b = apply_limits(Point2::new(1919.0, 500.0), DESK, lim);
        assert_eq!(a.position.x, b.position.x);
        assert!((a.position.x - 959.5).abs() < 1e-3);
    }

    #[test]
    fn pinned_offset_drifts_toward_centre_at_the_2018_rate() {
        let pinned = Limited {
            position: Point2::default(),
            x: Pin::High,
            y: Pin::Low,
        };
        // One 2018 tick = 1 image px.
        let o = drift_offset(Point2::new(5.0, 5.0), &pinned, REFERENCE_TICK_US);
        assert!(
            (o.x - 4.0).abs() < 1e-4 && (o.y - 6.0).abs() < 1e-4,
            "{o:?}"
        );
        let free = Limited {
            x: Pin::Free,
            y: Pin::Free,
            ..pinned
        };
        assert_eq!(
            drift_offset(Point2::new(5.0, 5.0), &free, 1_000_000),
            Point2::new(5.0, 5.0)
        );
    }

    #[test]
    fn drift_is_frame_rate_independent() {
        let pinned = Limited {
            position: Point2::default(),
            x: Pin::High,
            y: Pin::Free,
        };
        let run = |fps: u64| {
            let mut o = Point2::default();
            for _ in 0..fps {
                o = drift_offset(o, &pinned, 1_000_000 / fps);
            }
            o.x
        };
        for fps in [15, 30, 60] {
            assert!((run(fps) + DRIFT_IMAGE_PX_PER_S).abs() < 0.01, "fps {fps}");
        }
    }
}
