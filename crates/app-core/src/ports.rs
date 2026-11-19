//! Port traits implemented by `platform` adapters and by test fakes (ADR-0007).

use thiserror::Error;
pub use tracking_core::{GrayFrame, Point2};

pub use crate::color::RgbFrame;
pub use crate::controller::PointerAccess;
pub use pointer_core::DesktopRect;

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

/// One captured frame in the camera's native orientation: luma for tracking, an optional
/// same-size colour image for the preview (ADR-0008 amendment), and a monotonic capture
/// timestamp. Mirroring and downsampling happen later, at ingest, to both (ADR-0008).
#[derive(Debug, Clone, PartialEq)]
pub struct CameraFrame {
    pub luma: GrayFrame,
    /// Same width and height as `luma`. `None` from fakes that only need tracking.
    pub rgb: Option<RgbFrame>,
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

/// The display layout: the virtual desktop's bounding rectangle (R-P5) and each display's
/// own bounds, in the same global coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct Desktop {
    pub bounds: DesktopRect,
    /// Every active display, in the adapter's order (the controller sorts them).
    pub displays: Vec<DesktopRect>,
    /// Index into `displays` of the OS primary display (macOS main display, Windows primary
    /// monitor, X11 RandR primary output). `None` = the adapter can't tell; the controller then
    /// takes the display at the global origin (ADR-0014 amendment, #14).
    pub primary: Option<usize>,
}

/// System pointer control, in virtual-desktop screen coordinates (the OS's global pointer
/// space: origin at the primary display's top-left, y down, may be negative on other displays).
pub trait PointerSink {
    /// Whether this process may move the pointer and click. Can be slow (≈10 ms on macOS,
    /// task 0008 D4): call it at start and when polling for a grant, not per frame.
    fn access(&self) -> PointerAccess;
    /// Moves the OS cursor. A point outside every display is moved onto the nearest one.
    fn move_to(&mut self, p: Point2) -> Result<(), PointerError>;
    /// Left click at `p` (dwell, R-K1).
    fn click(&mut self, p: Point2) -> Result<(), PointerError>;
    /// Current OS cursor position, used to detect physical mouse movement (handover,
    /// ADR-0006). `None` if the platform cannot report it (e.g. Wayland). The cursor resting
    /// where a recent `move_to` put it is reported as the requested point, so snapping and
    /// late-landing moves never look like the user taking over.
    fn position(&self) -> Option<Point2>;
    /// Re-reads the display layout: the virtual desktop's bounding rectangle (R-P5) and the
    /// displays. Poll it (about once a second) to feed `Input::DesktopChanged`.
    fn desktop(&mut self) -> Result<Desktop, PointerError>;
}
