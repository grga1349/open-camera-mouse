//! OS adapters implementing `app-core` ports (ADR-0007).
//!
//! One module per OS behind `cfg`, so every target compiles in CI from day one. Pointer and
//! hotkey adapters land in task 0008 slices D4–D5 (see docs/architecture/PLATFORM_RISKS.md).
//! [`camera`] (one `nokhwa` adapter for all three OS, D3) and [`settings_store`] are the
//! same on every OS.

pub mod camera;
pub mod settings_store;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux as imp;
#[cfg(target_os = "macos")]
use macos as imp;
#[cfg(target_os = "windows")]
use windows as imp;

pub use camera::{CameraDevice, NokhwaCamera, list_devices};
pub use settings_store::{Loaded, SaveError, Saved, SettingsNotice, SettingsStore};

/// Which OS backend this build uses.
pub fn backend_name() -> &'static str {
    imp::BACKEND
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_backend_is_selected_for_this_target() {
        assert!(!super::backend_name().is_empty());
    }
}
