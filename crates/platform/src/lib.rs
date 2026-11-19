//! OS adapters implementing `app-core` ports (ADR-0007).
//!
//! One module per OS behind `cfg`, so every target compiles in CI from day one. [`camera`]
//! (one `nokhwa` adapter for all three OS, D3) and [`settings_store`] are the same on every
//! OS; [`pointer`] (D4) has one backend per OS; [`hotkeys`] (D5) wraps `global-hotkey`, which
//! has its own per-OS backends (ARCHITECTURE.md P12/P13).

pub mod camera;
pub mod hotkeys;
pub mod pointer;
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
pub use hotkeys::{GlobalHotkeys, Hotkey, KeyRouter, Registration};
pub use pointer::SystemPointer;
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
