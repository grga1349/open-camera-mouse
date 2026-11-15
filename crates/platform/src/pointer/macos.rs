//! macOS pointer: posted CoreGraphics events (task 0008 D4, ADR-0012 §"Pointer adapter").
//!
//! Measured on the dev Mac: posted moves need Accessibility (dropped silently without it) and
//! land in 0.3–0.9 ms (rarely ~38 ms). With the event source's local-event suppression set to
//! 0, physical mouse movement still shows through while the camera moves the cursor at 30 Hz
//! (89 of 163 samples while the user moved the mouse; 0 of 162 with `CGWarpMouseCursorPosition`,
//! which is why warping is not used).

use std::process::Command;
use std::time::Instant;

use app_core::ports::{Desktop, DesktopRect, Point2, PointerAccess, PointerError, PointerSink};
use objc2_core_foundation::{CFRetained, CGPoint};
use objc2_core_graphics::{
    CGDirectDisplayID, CGDisplayBounds, CGError, CGEvent, CGEventSource, CGEventSourceStateID,
    CGEventTapLocation, CGEventType, CGGetActiveDisplayList, CGMouseButton,
    CGPreflightPostEventAccess, CGRequestPostEventAccess,
};

use super::geometry::{Echo, bounding, finite, snap};

/// System Settings → Privacy & Security → Accessibility.
const ACCESSIBILITY_SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
const MAX_DISPLAYS: usize = 32;

/// The macOS [`PointerSink`]. Not `Send` (the CoreGraphics event source isn't): keep it on the
/// thread that drives the controller.
#[derive(Debug)]
pub struct SystemPointer {
    source: CFRetained<CGEventSource>,
    displays: Vec<DesktopRect>,
    echo: Echo,
    epoch: Instant,
}

impl SystemPointer {
    pub fn new() -> Result<Self, PointerError> {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .ok_or_else(|| PointerError::Backend("CGEventSourceCreate failed".into()))?;
        // The default 0.25 s would hide physical mouse movement after every posted move.
        CGEventSource::set_local_events_suppression_interval(Some(&source), 0.0);
        Ok(Self {
            source,
            displays: active_displays()?,
            echo: Echo::default(),
            epoch: Instant::now(),
        })
    }

    /// Asks macOS to show its Accessibility prompt (only the first time per app; afterwards it
    /// returns the current state without a prompt). The grant takes effect without a restart.
    pub fn request_access(&self) -> PointerAccess {
        to_access(CGRequestPostEventAccess())
    }

    /// Opens System Settings at Privacy & Security → Accessibility.
    pub fn open_access_settings() -> Result<(), PointerError> {
        let status = Command::new("open")
            .arg(ACCESSIBILITY_SETTINGS_URL)
            .status()
            .map_err(|e| PointerError::Backend(format!("open System Settings: {e}")))?;
        if status.success() {
            Ok(())
        } else {
            Err(PointerError::Backend(format!(
                "open System Settings: {status}"
            )))
        }
    }

    fn now_us(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_micros()).unwrap_or(u64::MAX)
    }

    fn raw_position() -> Option<Point2> {
        let event = CGEvent::new(None)?;
        let p = CGEvent::location(Some(&event));
        Some(Point2::new(p.x as f32, p.y as f32))
    }

    fn post(&self, kind: CGEventType, at: Point2) -> Result<(), PointerError> {
        let event = CGEvent::new_mouse_event(
            Some(&self.source),
            kind,
            CGPoint::new(f64::from(at.x), f64::from(at.y)),
            CGMouseButton::Left,
        )
        .ok_or_else(|| PointerError::Backend("CGEventCreateMouseEvent failed".into()))?;
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
        Ok(())
    }

    /// Snaps, posts a move and records it for [`Echo`].
    fn move_inner(&mut self, p: Point2) -> Result<Point2, PointerError> {
        if !finite(p) {
            return Err(PointerError::Backend(format!(
                "non-finite pointer target {p:?}"
            )));
        }
        let applied = snap(p, &self.displays);
        let before = Self::raw_position();
        self.post(CGEventType::MouseMoved, applied)?;
        let now = self.now_us();
        self.echo.record(p, applied, before, now);
        Ok(applied)
    }
}

fn to_access(granted: bool) -> PointerAccess {
    if granted {
        PointerAccess::Granted
    } else {
        PointerAccess::Denied
    }
}

impl PointerSink for SystemPointer {
    fn access(&self) -> PointerAccess {
        to_access(CGPreflightPostEventAccess())
    }

    fn move_to(&mut self, p: Point2) -> Result<(), PointerError> {
        self.move_inner(p).map(|_| ())
    }

    fn click(&mut self, p: Point2) -> Result<(), PointerError> {
        // Posting without Accessibility is a silent no-op; a click is rare enough to afford
        // the ≈10 ms check and report it.
        if !CGPreflightPostEventAccess() {
            return Err(PointerError::PermissionDenied);
        }
        let at = self.move_inner(p)?;
        self.post(CGEventType::LeftMouseDown, at)?;
        self.post(CGEventType::LeftMouseUp, at)
    }

    fn position(&self) -> Option<Point2> {
        Self::raw_position().map(|p| self.echo.report(p, self.now_us()))
    }

    fn desktop(&mut self) -> Result<Desktop, PointerError> {
        self.displays = active_displays()?;
        let bounds = bounding(&self.displays)
            .ok_or_else(|| PointerError::Backend("no active display".into()))?;
        Ok(Desktop {
            bounds,
            displays: self.displays.clone(),
        })
    }
}

/// Bounds of every active display in global display coordinates (points, origin at the main
/// display's top-left, y down; other displays may have negative origins).
fn active_displays() -> Result<Vec<DesktopRect>, PointerError> {
    let mut ids: [CGDirectDisplayID; MAX_DISPLAYS] = [0; MAX_DISPLAYS];
    let mut count: u32 = 0;
    // ADR-0007: `platform` may use `unsafe` for FFI, locally and commented.
    #[allow(unsafe_code)]
    // SAFETY: `ids` is valid for `MAX_DISPLAYS` writes and `count` for one; CoreGraphics
    // writes at most `max_displays` ids and the count, and keeps neither pointer.
    let err = unsafe { CGGetActiveDisplayList(MAX_DISPLAYS as u32, ids.as_mut_ptr(), &mut count) };
    if err != CGError::Success {
        return Err(PointerError::Backend(format!(
            "CGGetActiveDisplayList: {err:?}"
        )));
    }
    let n = (count as usize).min(MAX_DISPLAYS);
    Ok(ids[..n]
        .iter()
        .map(|&id| {
            let b = CGDisplayBounds(id);
            DesktopRect {
                x: b.origin.x as f32,
                y: b.origin.y as f32,
                width: b.size.width as f32,
                height: b.size.height as f32,
            }
        })
        .collect())
}
