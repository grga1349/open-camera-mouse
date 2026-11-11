//! Port traits implemented by `platform` adapters and by test fakes (ADR-0007).

use thiserror::Error;
use tracking_core::{GrayFrame, Point2};

/// What the app asks the camera for (ADR-0008, Q-R6): 640×480 at 30 fps, nearest match.
/// Adapters report what they actually got in [`CaptureFormat`]; the app never assumes the
/// request was honoured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRequest {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl Default for CaptureRequest {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            fps: 30,
        }
    }
}

/// The format the device negotiated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaptureFormat {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
}

/// One captured frame: luma in the camera's native orientation, plus a monotonic
/// capture timestamp. Mirroring happens later, at ingest (ADR-0008).
#[derive(Debug, Clone, PartialEq)]
pub struct CameraFrame {
    pub luma: GrayFrame,
    pub timestamp_us: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CameraError {
    #[error("no camera found")]
    NoDevice,
    #[error("camera permission denied")]
    PermissionDenied,
    #[error("camera disconnected")]
    Disconnected,
    #[error("camera backend error: {0}")]
    Backend(String),
}

/// A camera capture source.
pub trait CameraSource {
    fn open(&mut self, request: CaptureRequest) -> Result<CaptureFormat, CameraError>;
    /// Blocks until the next frame or an error.
    fn next_frame(&mut self) -> Result<CameraFrame, CameraError>;
    fn close(&mut self);
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PointerError {
    #[error("pointer control permission denied")]
    PermissionDenied,
    #[error("pointer injection is unsupported on this session: {0}")]
    Unsupported(String),
    #[error("pointer backend error: {0}")]
    Backend(String),
}

/// System pointer control, in virtual-desktop screen coordinates.
pub trait PointerSink {
    fn move_to(&mut self, p: Point2) -> Result<(), PointerError>;
    /// Current OS cursor position, used to detect physical mouse movement (handover,
    /// ADR-0006). `None` if the platform cannot report it (e.g. Wayland).
    fn position(&self) -> Option<Point2>;
}
