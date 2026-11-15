//! Windows pointer: `SetCursorPos` moves, `SendInput` clicks (task 0008 D4, ADR-0012
//! §"Pointer adapter"). Compiled in CI; on-device validation is Phase 7.
//!
//! Coordinates are physical pixels of the virtual screen, which matches the controller's
//! desktop as long as the process is per-monitor DPI aware (winit sets that). No permission is
//! needed, but UIPI blocks input into elevated windows and the secure desktop: those calls
//! fail and are reported as `Backend` errors.

// ADR-0007: `platform` may use `unsafe` for FFI, locally and commented. Every call below is a
// plain Win32 function taking values or pointers to locals.
#![allow(unsafe_code)]

use std::time::Instant;

use app_core::ports::{DesktopRect, Point2, PointerAccess, PointerError, PointerSink};
use windows_sys::Win32::Foundation::{LPARAM, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT, SendInput,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};
use windows_sys::core::BOOL;

use super::geometry::{Echo, bounding, finite, snap};

/// The Windows [`PointerSink`].
#[derive(Debug)]
pub struct SystemPointer {
    displays: Vec<DesktopRect>,
    echo: Echo,
    epoch: Instant,
}

impl SystemPointer {
    pub fn new() -> Result<Self, PointerError> {
        Ok(Self {
            displays: monitors()?,
            echo: Echo::default(),
            epoch: Instant::now(),
        })
    }

    /// Windows needs no permission: always `Granted`.
    pub fn request_access(&self) -> PointerAccess {
        PointerAccess::Granted
    }

    /// No permission settings to open on Windows.
    pub fn open_access_settings() -> Result<(), PointerError> {
        Err(PointerError::Unsupported(
            "no pointer permission on Windows".into(),
        ))
    }

    fn now_us(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_micros()).unwrap_or(u64::MAX)
    }

    fn raw_position() -> Option<Point2> {
        let mut p = POINT { x: 0, y: 0 };
        // SAFETY: `p` is a valid, writable POINT for the duration of the call.
        let ok = unsafe { GetCursorPos(&mut p) };
        (ok != 0).then(|| Point2::new(p.x as f32, p.y as f32))
    }

    fn move_inner(&mut self, p: Point2) -> Result<Point2, PointerError> {
        if !finite(p) {
            return Err(PointerError::Backend(format!(
                "non-finite pointer target {p:?}"
            )));
        }
        let applied = snap(p, &self.displays);
        let before = Self::raw_position();
        // SAFETY: value arguments only.
        let ok = unsafe { SetCursorPos(applied.x.round() as i32, applied.y.round() as i32) };
        if ok == 0 {
            return Err(PointerError::Backend(format!(
                "SetCursorPos failed: {}",
                std::io::Error::last_os_error()
            )));
        }
        let now = self.now_us();
        self.echo.record(p, applied, before, now);
        Ok(applied)
    }
}

fn mouse_input(flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

impl PointerSink for SystemPointer {
    fn access(&self) -> PointerAccess {
        PointerAccess::Granted
    }

    fn move_to(&mut self, p: Point2) -> Result<(), PointerError> {
        self.move_inner(p).map(|_| ())
    }

    fn click(&mut self, p: Point2) -> Result<(), PointerError> {
        self.move_inner(p)?;
        let inputs = [
            mouse_input(MOUSEEVENTF_LEFTDOWN),
            mouse_input(MOUSEEVENTF_LEFTUP),
        ];
        // SAFETY: `inputs` is a valid array of `inputs.len()` INPUT structs, size passed
        // correctly; SendInput only reads it during the call.
        let sent = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        };
        if sent as usize == inputs.len() {
            Ok(())
        } else {
            Err(PointerError::Backend(format!(
                "SendInput sent {sent} of {}: {}",
                inputs.len(),
                std::io::Error::last_os_error()
            )))
        }
    }

    fn position(&self) -> Option<Point2> {
        Self::raw_position().map(|p| self.echo.report(p, self.now_us()))
    }

    fn desktop(&mut self) -> Result<DesktopRect, PointerError> {
        self.displays = monitors()?;
        bounding(&self.displays).ok_or_else(|| PointerError::Backend("no monitor".into()))
    }
}

unsafe extern "system" fn collect_monitor(
    monitor: HMONITOR,
    _hdc: HDC,
    _clip: *mut RECT,
    data: LPARAM,
) -> BOOL {
    // SAFETY: `data` is the `&mut Vec<DesktopRect>` passed by `monitors`, alive and not
    // otherwise borrowed while EnumDisplayMonitors runs (it calls back synchronously).
    let out = unsafe { &mut *(data as *mut Vec<DesktopRect>) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        rcWork: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        dwFlags: 0,
    };
    // SAFETY: `monitor` comes from the enumeration; `info` is valid with `cbSize` set.
    if unsafe { GetMonitorInfoW(monitor, &mut info) } != 0 {
        let r = info.rcMonitor;
        out.push(DesktopRect {
            x: r.left as f32,
            y: r.top as f32,
            width: (r.right - r.left) as f32,
            height: (r.bottom - r.top) as f32,
        });
    }
    1 // continue
}

/// Every monitor's rectangle in virtual-screen coordinates (may be negative).
fn monitors() -> Result<Vec<DesktopRect>, PointerError> {
    let mut out: Vec<DesktopRect> = Vec::new();
    // SAFETY: null HDC and clip rect enumerate all monitors; the callback only touches `out`
    // through the pointer passed as `data`, which outlives the synchronous call.
    let ok = unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(collect_monitor),
            &mut out as *mut Vec<DesktopRect> as LPARAM,
        )
    };
    if ok == 0 || out.is_empty() {
        return Err(PointerError::Backend(format!(
            "EnumDisplayMonitors found no monitor: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(out)
}
