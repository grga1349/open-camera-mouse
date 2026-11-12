//! Pure pointer mapping (ADR-0003/ADR-0007).
//!
//! The absolute mapping from ADR-0006 (with a test that pins "head right → cursor right",
//! ADR-0008) and the handover/jump-bound rules in [`handover`]. Smoothing, limits, offset
//! drift and dwell are task 0008 slice D1, each with its own tests.

pub mod handover;

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

/// Absolute mapping (ADR-0006):
/// `desk_center + (p + offset − img_center) · gain · desk_size / img_size`, clamped to the
/// virtual desktop.
///
/// `p` is in canonical display space (ADR-0008), so with mirroring on, a head moving to
/// the user's right increases `p.x` and moves the cursor right. No sign flip happens here.
pub fn map_absolute(
    p: Point2,
    offset: Point2,
    frame: FrameSize,
    desktop: DesktopRect,
    gain: Gain,
) -> Point2 {
    let c = desktop.center();
    let sx = gain.x * desktop.width / frame.width;
    let sy = gain.y * desktop.height / frame.height;
    let raw = Point2::new(
        c.x + (p.x + offset.x - frame.width / 2.0) * sx,
        c.y + (p.y + offset.y - frame.height / 2.0) * sy,
    );
    desktop.clamp(raw)
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
