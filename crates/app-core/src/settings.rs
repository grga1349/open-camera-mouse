//! User settings (BEHAVIOR_SPEC R-A5): one struct, validated on every update and on load, so
//! an invalid value can't be set (R-P2, fixes L-1). Persistence (file format, schema
//! migration on disk) is task 0008 slice D2 and lives outside this pure crate.
//!
//! Defaults follow ADR-0006 (2018) and the approved design lab (task 0007). Improvements on
//! 2018 are opt-in and off by default (`loss_recovery`, `dwell.enabled`).

use thiserror::Error;

/// Colour theme (design lab: Carbon = dark, default; Paper = light).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Carbon,
    Paper,
}

/// Distance the pointer keeps from each edge of the virtual desktop, in screen px (design
/// lab "Screen limits", default 0; 2018 W/E/N/S limits, ADR-0006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScreenLimits {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

/// Dwell clicking (R-K1/R-K2, ADR-0006 provisional defaults). Behaviour is slice D1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DwellSettings {
    pub enabled: bool,
    /// Hold still this long to click.
    pub dwell_s: f32,
    /// No second click until the pointer leaves the radius or this time passes.
    pub refractory_s: f32,
    /// "Hold-still area" as a fraction of the screen size.
    pub radius_frac: f32,
}

impl Default for DwellSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            dwell_s: 1.0,
            refractory_s: 2.0,
            radius_frac: 0.05,
        }
    }
}

/// Every user-changeable setting.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// Per-axis gain (R-P3), 2018 default 6/6.
    pub gain_x: f32,
    pub gain_y: f32,
    /// "Keep both speeds the same": when on, `gain_y` follows `gain_x`.
    pub link_gain: bool,
    /// Pointer smoothing, **higher = steadier** (R-P2: the name matches the polarity). This
    /// is `1 − d` for the 2018 damping `d` (weight of the new position) per 70 ms, so the
    /// 2018 `Damping = 0.95` is `smoothing = 0.05`. Must stay below 1 (1 would freeze the
    /// pointer, L-1).
    pub smoothing: f32,
    pub limits: ScreenLimits,
    pub dwell: DwellSettings,
    /// Camera device id from the platform adapter; `None` = system default.
    pub camera: Option<String>,
    /// Mirror at ingest (ADR-0008), default on: head right moves the pointer right.
    pub mirror: bool,
    /// "Find the target again" (ADR-0010, ADR-0011 amendment): re-detection after a loss.
    /// Off by default (2018: a loss waits for re-selection or recenter).
    pub loss_recovery: bool,
    pub theme: Theme,
    pub show_diagnostics: bool,
    /// UI text scale (design lab: 100–130 %).
    pub text_scale: f32,
    pub larger_controls: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gain_x: 6.0,
            gain_y: 6.0,
            link_gain: true,
            smoothing: 0.05,
            limits: ScreenLimits::default(),
            dwell: DwellSettings::default(),
            camera: None,
            mirror: true,
            loss_recovery: false,
            theme: Theme::Carbon,
            show_diagnostics: false,
            text_scale: 1.0,
            larger_controls: false,
        }
    }
}

/// A setting outside its allowed range. NaN and infinities are always out of range.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("{field} = {value} is outside {min}..={max}")]
pub struct SettingsError {
    pub field: &'static str,
    pub value: f64,
    pub min: f64,
    pub max: f64,
}

/// Inclusive ranges, shared with the UI so sliders can't offer an invalid value.
pub mod ranges {
    use std::ops::RangeInclusive;
    pub const GAIN: RangeInclusive<f32> = 1.0..=20.0;
    pub const SMOOTHING: RangeInclusive<f32> = 0.0..=0.95;
    pub const LIMIT_PX: RangeInclusive<u32> = 0..=1000;
    pub const DWELL_S: RangeInclusive<f32> = 0.3..=3.0;
    pub const REFRACTORY_S: RangeInclusive<f32> = 0.5..=5.0;
    pub const DWELL_RADIUS_FRAC: RangeInclusive<f32> = 0.01..=0.15;
    pub const TEXT_SCALE: RangeInclusive<f32> = 1.0..=1.3;
}

fn check_f32(
    field: &'static str,
    v: f32,
    r: std::ops::RangeInclusive<f32>,
) -> Result<(), SettingsError> {
    if r.contains(&v) {
        Ok(())
    } else {
        Err(SettingsError {
            field,
            value: v.into(),
            min: (*r.start()).into(),
            max: (*r.end()).into(),
        })
    }
}

fn check_u32(
    field: &'static str,
    v: u32,
    r: std::ops::RangeInclusive<u32>,
) -> Result<(), SettingsError> {
    if r.contains(&v) {
        Ok(())
    } else {
        Err(SettingsError {
            field,
            value: v.into(),
            min: (*r.start()).into(),
            max: (*r.end()).into(),
        })
    }
}

impl Settings {
    /// Version of the persisted settings schema (R-A5). Bump on any incompatible change and
    /// teach the loader (slice D2) to migrate or fall back to defaults.
    pub const SCHEMA_VERSION: u32 = 1;

    /// Validates every field and returns the normalised settings: linked gains are made
    /// equal (`gain_y := gain_x`), an empty camera id becomes `None`. Used on every update
    /// and on load.
    pub fn validated(mut self) -> Result<Self, SettingsError> {
        check_f32("gain_x", self.gain_x, ranges::GAIN)?;
        if self.link_gain {
            self.gain_y = self.gain_x;
        }
        check_f32("gain_y", self.gain_y, ranges::GAIN)?;
        check_f32("smoothing", self.smoothing, ranges::SMOOTHING)?;
        let l = self.limits;
        check_u32("limits.left", l.left, ranges::LIMIT_PX)?;
        check_u32("limits.right", l.right, ranges::LIMIT_PX)?;
        check_u32("limits.top", l.top, ranges::LIMIT_PX)?;
        check_u32("limits.bottom", l.bottom, ranges::LIMIT_PX)?;
        let d = self.dwell;
        check_f32("dwell.dwell_s", d.dwell_s, ranges::DWELL_S)?;
        check_f32("dwell.refractory_s", d.refractory_s, ranges::REFRACTORY_S)?;
        check_f32(
            "dwell.radius_frac",
            d.radius_frac,
            ranges::DWELL_RADIUS_FRAC,
        )?;
        check_f32("text_scale", self.text_scale, ranges::TEXT_SCALE)?;
        if self.camera.as_deref().is_some_and(|c| c.trim().is_empty()) {
            self.camera = None;
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_match_2018() {
        let s = Settings::default().validated().unwrap();
        assert_eq!((s.gain_x, s.gain_y), (6.0, 6.0));
        // 2018 Damping 0.95 is the weight of the new position: smoothing = 1 − 0.95.
        assert!((s.smoothing - 0.05).abs() < 1e-6);
        assert!(!s.loss_recovery, "Find the target again is off by default");
        assert!(!s.dwell.enabled);
        assert!(s.mirror);
        assert_eq!(s.theme, Theme::Carbon);
    }

    #[test]
    fn out_of_range_and_non_finite_values_are_rejected() {
        let cases: Vec<(&str, Settings)> = vec![
            (
                "gain_x",
                Settings {
                    gain_x: 0.0,
                    ..Settings::default()
                },
            ),
            (
                "gain_x",
                Settings {
                    gain_x: f32::NAN,
                    ..Settings::default()
                },
            ),
            (
                "gain_y",
                Settings {
                    link_gain: false,
                    gain_y: 21.0,
                    ..Settings::default()
                },
            ),
            (
                "smoothing",
                Settings {
                    smoothing: 1.0,
                    ..Settings::default()
                },
            ),
            (
                "smoothing",
                Settings {
                    smoothing: -0.01,
                    ..Settings::default()
                },
            ),
            (
                "limits.bottom",
                Settings {
                    limits: ScreenLimits {
                        bottom: 1001,
                        ..ScreenLimits::default()
                    },
                    ..Settings::default()
                },
            ),
            (
                "dwell.dwell_s",
                Settings {
                    dwell: DwellSettings {
                        dwell_s: f32::INFINITY,
                        ..DwellSettings::default()
                    },
                    ..Settings::default()
                },
            ),
            (
                "dwell.radius_frac",
                Settings {
                    dwell: DwellSettings {
                        radius_frac: 0.0,
                        ..DwellSettings::default()
                    },
                    ..Settings::default()
                },
            ),
            (
                "text_scale",
                Settings {
                    text_scale: 1.5,
                    ..Settings::default()
                },
            ),
        ];
        for (field, s) in cases {
            let err = s.validated().expect_err(field);
            assert_eq!(err.field, field);
        }
    }

    #[test]
    fn linked_gain_is_normalised_and_unlinked_is_kept() {
        let s = Settings {
            gain_x: 8.0,
            gain_y: 3.0,
            ..Settings::default()
        };
        assert_eq!(s.validated().unwrap().gain_y, 8.0);
        let s = Settings {
            gain_x: 8.0,
            gain_y: 3.0,
            link_gain: false,
            ..Settings::default()
        };
        assert_eq!(s.validated().unwrap().gain_y, 3.0);
    }

    #[test]
    fn blank_camera_id_means_default_camera() {
        let s = Settings {
            camera: Some("  ".into()),
            ..Settings::default()
        };
        assert_eq!(s.validated().unwrap().camera, None);
    }
}
