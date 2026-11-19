//! Pointer adapters: move, click, read the cursor, describe the desktop (task 0008 D4,
//! ADR-0012 §"Pointer adapter"). One [`SystemPointer`] per OS behind `cfg`, all implementing
//! [`app_core::ports::PointerSink`]; [`geometry`] holds the shared, pure parts.
//!
//! - macOS: posted CoreGraphics mouse events from a source with local-event suppression 0.
//!   Needs Accessibility; without it events are dropped silently, so [`SystemPointer::access`]
//!   must be checked (it costs ≈10 ms, poll it rarely). Warping was measured and rejected: it
//!   needs no permission but blocks the physical mouse, so handover can't work.
//! - Windows: `SetCursorPos` + `SendInput` clicks. Compiled in CI, validated in Phase 7.
//! - Linux: X11 XTest through `x11rb` (pure-Rust connection). A Wayland session is
//!   `Unsupported` (ARCHITECTURE.md P10/P11). Compiled in CI, validated in Phase 7.

pub mod geometry;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::SystemPointer;
#[cfg(target_os = "macos")]
pub use macos::SystemPointer;
#[cfg(target_os = "windows")]
pub use windows::SystemPointer;

/// Whether the session is Wayland, where global pointer injection and reading are unavailable
/// (P10/P11). Inputs are `XDG_SESSION_TYPE` and `WAYLAND_DISPLAY`. XWayland's `DISPLAY` doesn't
/// help: XTest there moves the pointer only over X11 windows.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn is_wayland(session_type: Option<&str>, wayland_display: Option<&str>) -> bool {
    match session_type.map(str::trim) {
        Some(t) if t.eq_ignore_ascii_case("wayland") => true,
        Some(t) if t.eq_ignore_ascii_case("x11") => false,
        _ => wayland_display.is_some_and(|d| !d.trim().is_empty()),
    }
}

#[cfg(test)]
mod tests {
    use super::is_wayland;

    #[test]
    fn wayland_detection() {
        assert!(is_wayland(Some("wayland"), None));
        assert!(is_wayland(Some("Wayland"), Some("wayland-0")));
        assert!(!is_wayland(Some("x11"), Some("wayland-0")));
        assert!(is_wayland(None, Some("wayland-0")));
        assert!(is_wayland(Some("tty"), Some("wayland-0")));
        assert!(!is_wayland(None, None));
        assert!(!is_wayland(None, Some("")));
    }
}
