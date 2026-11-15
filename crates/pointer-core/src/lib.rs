//! Pure pointer mapping (ADR-0003/ADR-0007).
//!
//! The absolute mapping from ADR-0006 (with a test that pins "head right → cursor right",
//! ADR-0008), dt-aware [`smoothing`] (R-P2), screen [`limits`] with offset drift, [`dwell`]
//! clicking (R-K1/R-K2) and the [`handover`]/jump-bound rules (R-A2, R-P4). Each piece is
//! pure and takes time as an argument; `app-core` composes them per frame as
//! map → smooth → limit → drift → [`steady`] zone → dwell.

pub mod dwell;
pub mod handover;
pub mod limits;
pub mod one_euro;
pub mod smoothing;
pub mod steady;

use tracking_core::Point2;

/// The full virtual desktop in OS screen coordinates. The origin can be negative when a
/// monitor sits left of or above the primary one (BEHAVIOR_SPEC R-P5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesktopRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DesktopRect {
    pub fn center(&self) -> Point2 {
        Point2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    fn clamp(&self, p: Point2) -> Point2 {
        // Last addressable pixel is origin + size − 1.
        Point2::new(
            p.x.clamp(self.x, self.x + self.width - 1.0),
            p.y.clamp(self.y, self.y + self.height - 1.0),
        )
    }
}

/// Per-axis gain (R-P3). The 2018 default is 6/6 (ADR-0006).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gain {
    pub x: f32,
    pub y: f32,
}

impl Default for Gain {
    fn default() -> Self {
        Self { x: 6.0, y: 6.0 }
    }
}

/// Size of the canonical display-space frame the tracker runs on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameSize {
    pub width: f32,
    pub height: f32,
}

/// Absolute mapping (ADR-0006), **not** clamped:
/// `desk_center + (p + offset − img_center) · gain · desk_size / img_size`.
///
/// `p` is in canonical display space (ADR-0008), so with mirroring on, a head moving to
/// the user's right increases `p.x` and moves the cursor right. No sign flip happens here.
/// The result can lie outside the desktop; [`limits::apply_limits`] clamps it and reports
/// the pinned edges that drive the offset drift.
pub fn map_raw(
    p: Point2,
    offset: Point2,
    frame: FrameSize,
    desktop: DesktopRect,
    gain: Gain,
) -> Point2 {
    map_raw_scaled(p, offset, frame, desktop, desktop, gain)
}

/// [`map_raw`] with the scale taken from `reference`'s size instead of the whole desktop's
/// (ADR-0017): `desk_center + (p + offset − img_center) · gain · ref_size / img_size`. With one
/// display and `reference = desktop` it is exactly the 2018 mapping.
pub fn map_raw_scaled(
    p: Point2,
    offset: Point2,
    frame: FrameSize,
    desktop: DesktopRect,
    reference: DesktopRect,
    gain: Gain,
) -> Point2 {
    let c = desktop.center();
    let sx = gain.x * reference.width / frame.width;
    let sy = gain.y * reference.height / frame.height;
    Point2::new(
        c.x + (p.x + offset.x - frame.width / 2.0) * sx,
        c.y + (p.y + offset.y - frame.height / 2.0) * sy,
    )
}

/// The display whose size sets the mapping scale (ADR-0017, test drive #5): the "Main
/// monitor" (`main`, an index into `displays`) when set and present, otherwise the largest
/// display by area (the first of equals), otherwise the whole desktop. Scaling by the whole
/// desktop (2018) made one camera pixel ≈ 37 screen px on a 1440 + 2560 px setup at gain 6,
/// so tracker noise at rest became visible cursor shake.
pub fn scale_reference(
    displays: &[DesktopRect],
    main: Option<usize>,
    desktop: DesktopRect,
) -> DesktopRect {
    main.and_then(|i| displays.get(i).copied())
        .or_else(|| {
            displays.iter().copied().reduce(|a, b| {
                if b.width * b.height > a.width * a.height {
                    b
                } else {
                    a
                }
            })
        })
        .unwrap_or(desktop)
}

/// [`map_raw`] clamped to the virtual desktop.
pub fn map_absolute(
    p: Point2,
    offset: Point2,
    frame: FrameSize,
    desktop: DesktopRect,
    gain: Gain,
) -> Point2 {
    desktop.clamp(map_raw(p, offset, frame, desktop, gain))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: FrameSize = FrameSize {
        width: 640.0,
        height: 480.0,
    };
    const DESK: DesktopRect = DesktopRect {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };

    #[test]
    fn scale_reference_is_the_main_display_else_the_largest() {
        let laptop = DesktopRect {
            x: 0.0,
            y: 0.0,
            width: 1440.0,
            height: 900.0,
        };
        let external = DesktopRect {
            x: 1440.0,
            y: -540.0,
            width: 2560.0,
            height: 1440.0,
        };
        let both = DesktopRect {
            x: 0.0,
            y: -540.0,
            width: 4000.0,
            height: 1440.0,
        };
        let d = [laptop, external];
        assert_eq!(scale_reference(&d, Some(0), both), laptop);
        assert_eq!(scale_reference(&d, Some(1), both), external);
        assert_eq!(scale_reference(&d, None, both), external);
        // A Main monitor that is gone falls back to the largest.
        assert_eq!(scale_reference(&d, Some(5), both), external);
        assert_eq!(scale_reference(&[], None, both), both);
    }

    #[test]
    fn scaled_mapping_uses_the_reference_size_around_the_desktop_centre() {
        let reference = DesktopRect {
            x: 0.0,
            y: 0.0,
            width: 960.0,
            height: 540.0,
        };
        let g = Gain::default();
        let p = Point2::new(330.0, 250.0);
        let full = map_raw(p, Point2::default(), FRAME, DESK, g);
        let half = map_raw_scaled(p, Point2::default(), FRAME, DESK, reference, g);
        let c = DESK.center();
        assert!(
            ((half.x - c.x) * 2.0 - (full.x - c.x)).abs() < 1e-3,
            "{half:?} {full:?}"
        );
        assert!(
            ((half.y - c.y) * 2.0 - (full.y - c.y)).abs() < 1e-3,
            "{half:?} {full:?}"
        );
        assert_eq!(
            map_raw_scaled(p, Point2::default(), FRAME, DESK, DESK, g),
            full,
            "reference = desktop is the 2018 mapping"
        );
    }

    #[test]
    fn frame_center_maps_to_desktop_center() {
        let out = map_absolute(
            Point2::new(320.0, 240.0),
            Point2::default(),
            FRAME,
            DESK,
            Gain::default(),
        );
        assert_eq!(out, Point2::new(960.0, 540.0));
    }

    #[test]
    fn display_space_right_moves_cursor_right() {
        let g = Gain { x: 1.0, y: 1.0 };
        let a = map_absolute(Point2::new(320.0, 240.0), Point2::default(), FRAME, DESK, g);
        let b = map_absolute(Point2::new(330.0, 240.0), Point2::default(), FRAME, DESK, g);
        assert!(b.x > a.x, "{b:?} should be right of {a:?}");
        assert_eq!(b.y, a.y);
    }

    #[test]
    fn mirrored_ingest_turns_head_right_into_cursor_right() {
        // A raw (unmirrored) webcam sees the user's head move to *their* right as moving
        // toward smaller x. Mirroring at ingest (ADR-0008) maps raw x to width − 1 − x.
        let mirror = |raw_x: f32| FRAME.width - 1.0 - raw_x;
        let g = Gain { x: 1.0, y: 1.0 };
        let before = map_absolute(
            Point2::new(mirror(320.0), 240.0),
            Point2::default(),
            FRAME,
            DESK,
            g,
        );
        let after = map_absolute(
            Point2::new(mirror(310.0), 240.0),
            Point2::default(),
            FRAME,
            DESK,
            g,
        );
        assert!(after.x > before.x);
    }

    #[test]
    fn clamps_to_virtual_desktop_with_negative_origin() {
        let desk = DesktopRect {
            x: -1280.0,
            y: -200.0,
            width: 3200.0,
            height: 1280.0,
        };
        let far = map_absolute(
            Point2::new(10_000.0, -10_000.0),
            Point2::default(),
            FRAME,
            desk,
            Gain::default(),
        );
        assert_eq!(far, Point2::new(1919.0, -200.0));
    }
}
