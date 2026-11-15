//! Linux pointer: X11 XTest through `x11rb`'s pure-Rust connection (task 0008 D4, ADR-0012
//! §"Pointer adapter"). Compiled in CI; on-device validation is Phase 7.
//!
//! A Wayland session is `Unsupported` (PLATFORM_RISKS P10/P11): there is no general global
//! pointer injection or position, and XTest under XWayland only reaches X11 windows.

use std::time::Instant;

use app_core::ports::{Desktop, DesktopRect, Point2, PointerAccess, PointerError, PointerSink};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::xproto::{
    BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT, ConnectionExt as _, MOTION_NOTIFY_EVENT, Window,
};
use x11rb::protocol::xtest::{self, ConnectionExt as _};
use x11rb::rust_connection::RustConnection;

use super::geometry::{Echo, bounding, finite, snap};
use super::is_wayland;

/// X11 button 1 = left.
const LEFT_BUTTON: u8 = 1;

/// The Linux X11 [`PointerSink`].
#[derive(Debug)]
pub struct SystemPointer {
    conn: RustConnection,
    root: Window,
    displays: Vec<DesktopRect>,
    echo: Echo,
    epoch: Instant,
}

fn backend(e: impl std::fmt::Display) -> PointerError {
    PointerError::Backend(e.to_string())
}

impl SystemPointer {
    /// `Unsupported` on Wayland, without an X11 display or without the XTEST extension.
    pub fn new() -> Result<Self, PointerError> {
        let session = std::env::var("XDG_SESSION_TYPE").ok();
        let wayland = std::env::var("WAYLAND_DISPLAY").ok();
        if is_wayland(session.as_deref(), wayland.as_deref()) {
            return Err(PointerError::Unsupported(
                "Wayland session: the cursor can't be moved by other apps. Log in with an X11 session (\"Ubuntu on Xorg\" or similar)".into(),
            ));
        }
        let (conn, screen) = x11rb::connect(None)
            .map_err(|e| PointerError::Unsupported(format!("no X11 display: {e}")))?;
        if conn
            .extension_information(xtest::X11_EXTENSION_NAME)
            .map_err(backend)?
            .is_none()
        {
            return Err(PointerError::Unsupported(
                "the X server has no XTEST extension".into(),
            ));
        }
        let root = conn.setup().roots[screen].root;
        let mut me = Self {
            conn,
            root,
            displays: Vec::new(),
            echo: Echo::default(),
            epoch: Instant::now(),
        };
        me.displays = me.monitors()?;
        Ok(me)
    }

    /// X11 needs no permission: always `Granted`.
    pub fn request_access(&self) -> PointerAccess {
        PointerAccess::Granted
    }

    /// No permission settings to open on X11.
    pub fn open_access_settings() -> Result<(), PointerError> {
        Err(PointerError::Unsupported(
            "no pointer permission on X11".into(),
        ))
    }

    fn now_us(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_micros()).unwrap_or(u64::MAX)
    }

    fn raw_position(&self) -> Option<Point2> {
        let reply = self.conn.query_pointer(self.root).ok()?.reply().ok()?;
        Some(Point2::new(
            f32::from(reply.root_x),
            f32::from(reply.root_y),
        ))
    }

    fn fake(&self, kind: u8, detail: u8, at: Point2) -> Result<(), PointerError> {
        self.conn
            .xtest_fake_input(
                kind,
                detail,
                x11rb::CURRENT_TIME,
                self.root,
                at.x.round() as i16,
                at.y.round() as i16,
                0,
            )
            .map_err(backend)?;
        Ok(())
    }

    fn move_inner(&mut self, p: Point2) -> Result<Point2, PointerError> {
        if !finite(p) {
            return Err(PointerError::Backend(format!(
                "non-finite pointer target {p:?}"
            )));
        }
        let applied = snap(p, &self.displays);
        let before = self.raw_position();
        self.fake(MOTION_NOTIFY_EVENT, 0, applied)?;
        self.conn.flush().map_err(backend)?;
        let now = self.now_us();
        self.echo.record(p, applied, before, now);
        Ok(applied)
    }

    /// RandR 1.5 monitors, or the root window as one display if RandR can't say.
    fn monitors(&self) -> Result<Vec<DesktopRect>, PointerError> {
        let from_randr = self
            .conn
            .randr_get_monitors(self.root, true)
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| {
                r.monitors
                    .iter()
                    .map(|m| DesktopRect {
                        x: f32::from(m.x),
                        y: f32::from(m.y),
                        width: f32::from(m.width),
                        height: f32::from(m.height),
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|v| !v.is_empty());
        if let Some(v) = from_randr {
            return Ok(v);
        }
        let g = self
            .conn
            .get_geometry(self.root)
            .map_err(backend)?
            .reply()
            .map_err(backend)?;
        Ok(vec![DesktopRect {
            x: 0.0,
            y: 0.0,
            width: f32::from(g.width),
            height: f32::from(g.height),
        }])
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
        let at = self.move_inner(p)?;
        self.fake(BUTTON_PRESS_EVENT, LEFT_BUTTON, at)?;
        self.fake(BUTTON_RELEASE_EVENT, LEFT_BUTTON, at)?;
        self.conn.flush().map_err(backend)
    }

    fn position(&self) -> Option<Point2> {
        self.raw_position()
            .map(|p| self.echo.report(p, self.now_us()))
    }

    fn desktop(&mut self) -> Result<Desktop, PointerError> {
        self.displays = self.monitors()?;
        let bounds =
            bounding(&self.displays).ok_or_else(|| PointerError::Backend("no monitor".into()))?;
        Ok(Desktop {
            bounds,
            displays: self.displays.clone(),
        })
    }
}
