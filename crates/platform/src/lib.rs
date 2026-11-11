//! OS adapters implementing `app-core` ports (ADR-0007).
//!
//! Phase 2 skeleton: one module per OS behind `cfg`, so every target compiles in CI from
//! day one. Real adapters land in Phase 6/7 (see docs/architecture/PLATFORM_RISKS.md).

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
