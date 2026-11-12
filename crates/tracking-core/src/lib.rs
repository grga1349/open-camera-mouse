//! Pure tracking core (ADR-0003/ADR-0007).
//!
//! Frames in, position + status + diagnostics out. No camera, OS, GUI, clock or I/O
//! (BEHAVIOR_SPEC R-T1). The API is deliberately algorithm-neutral so pyramidal LK and NCC
//! (ADR-0004) both fit behind [`Tracker`]: [`lk::LkTracker`] (B-LK18) and
//! [`ncc::NccTracker`] (B-NCC).

pub mod frame;
pub mod image;
pub mod lk;
pub mod ncc;
pub mod pyramid;
pub mod redetect;

pub use frame::{FrameError, GrayFrame, downsample_to_width, ingest};

/// A sub-pixel position in canonical display-space frame coordinates (ADR-0008).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Why the tracker kept the previous position instead of a new estimate (R-T2: never silent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldReason {
    /// Near a frame edge and moving outward (2018 loss policy, ADR-0006).
    NearEdgeMovingOut,
    /// The estimate exceeded the configured motion bound (outlier guard).
    MotionOutlier,
}

/// Why this frame has no valid estimate. Whether the reference survives is per-algorithm and
/// documented on each tracker (2018 LK drops it; legacy NCC keeps its template and retries).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LossReason {
    OutOfFrame,
    LowConfidence,
    /// Forward and backward flow disagree (forward-backward validation, Phase 4 E1).
    Inconsistent,
    /// The window no longer looks like the selection (appearance gate, Phase 4 E2).
    AppearanceChanged,
    /// The tracker has no reference yet: nothing was selected.
    NoSelection,
    /// The track was lost earlier; the tracker kept its selection reference and is searching
    /// for it, with no confident, unambiguous match this frame (re-detection, Phase 4 E4).
    Searching,
}

/// Per-step tracker status (R-T2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackStatus {
    Tracking,
    Held(HoldReason),
    Lost(LossReason),
}

/// Algorithm-specific quality numbers. Fields an algorithm doesn't produce stay `None`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Diagnostics {
    /// Photometric residual (e.g. LK mean absolute error).
    pub residual: Option<f32>,
    /// Match score (NCC correlation in [−1, 1]).
    pub score: Option<f32>,
    /// Window texture: LK `λ_min(G) / |W|` (intensity² per px²).
    pub min_eigen: Option<f32>,
    /// Solver iterations used, if iterative.
    pub iterations: Option<u32>,
    /// Forward-backward error `|a − a'|` in tracker px, when validation ran.
    pub fb_error: Option<f32>,
}

/// One tracker step's result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackOutput {
    pub position: Point2,
    pub status: TrackStatus,
    pub diagnostics: Diagnostics,
}

/// Errors from tracker calls that are caller mistakes, not tracking outcomes.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TrackError {
    #[error("selection {x},{y} is outside the {width}x{height} frame")]
    SelectionOutOfFrame {
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    },
    #[error("frame size changed from {from:?} to {to:?} while tracking")]
    FrameSizeChanged { from: (u32, u32), to: (u32, u32) },
}

/// A single-point tracker. Implementations must be deterministic: the same sequence of
/// `select`/`step` calls with the same frames yields the same outputs (R-T1).
pub trait Tracker {
    /// Sets the reference from the selection frame itself (R-T3).
    fn select(&mut self, frame: &GrayFrame, point: Point2) -> Result<(), TrackError>;

    /// Advances one frame.
    fn step(&mut self, frame: &GrayFrame) -> Result<TrackOutput, TrackError>;

    /// Drops the reference. The next `step` reports `Lost(NoSelection)`.
    fn reset(&mut self);
}

/// Checks a selection point against a frame, for use by [`Tracker::select`] implementations.
pub fn check_selection(frame: &GrayFrame, point: Point2) -> Result<(), TrackError> {
    let inside = point.x >= 0.0
        && point.y >= 0.0
        && point.x < frame.width() as f32
        && point.y < frame.height() as f32;
    if inside {
        Ok(())
    } else {
        Err(TrackError::SelectionOutOfFrame {
            x: point.x,
            y: point.y,
            width: frame.width(),
            height: frame.height(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_bounds_are_half_open() {
        let f = GrayFrame::filled(4, 3, 0).unwrap();
        assert!(check_selection(&f, Point2::new(0.0, 0.0)).is_ok());
        assert!(check_selection(&f, Point2::new(3.9, 2.9)).is_ok());
        assert!(check_selection(&f, Point2::new(4.0, 1.0)).is_err());
        assert!(check_selection(&f, Point2::new(1.0, -0.1)).is_err());
        assert!(check_selection(&f, Point2::new(f32::NAN, 1.0)).is_err());
    }
}
