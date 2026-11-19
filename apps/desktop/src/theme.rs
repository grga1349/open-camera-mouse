//! Carbon (dark) and Paper (light) from the approved design lab's token table
//! (`docs/PRODUCT.md` §"UI design").
//!
//! The UI is monochrome. Exactly two hues exist and each has one helper here, so the rule is
//! enforced in one place (Phase 6 acceptance criterion 9): [`marker_green`] for the target
//! marker and [`primary_colors`] for yellow primary buttons (painted by `widgets::Button`).

use app_core::Theme;
use eframe::egui::{
    Color32, CornerRadius, FontId, Margin, Stroke, Style, TextStyle, Ui, Vec2, Visuals,
};

/// Colour tokens of one theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_2: Color32,
    pub line: Color32,
    pub line_strong: Color32,
    pub text: Color32,
    pub muted: Color32,
    /// Primary button fill (yellow). Only [`primary_colors`] reads it.
    primary: Color32,
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const CARBON: Tokens = Tokens {
    bg: hex(0x000000),
    surface: hex(0x0a0a0a),
    surface_2: hex(0x151515),
    line: hex(0x262626),
    line_strong: hex(0x3f3f3f),
    text: hex(0xfafafa),
    muted: hex(0x8f8f8f),
    primary: hex(0xffd60a),
};

pub const PAPER: Tokens = Tokens {
    bg: hex(0xececec),
    surface: hex(0xf8f8f8),
    surface_2: hex(0xffffff),
    line: hex(0xd6d6d6),
    line_strong: hex(0xa8a8a8),
    text: hex(0x111111),
    muted: hex(0x5a5a5a),
    primary: hex(0xf5c518),
};

/// Text on a yellow primary button, both themes.
const PRIMARY_TEXT: Color32 = hex(0x111111);

/// The marker colour, the same in both themes because it sits on the camera image.
const MARKER_GREEN: Color32 = hex(0x00ff00);

/// Grey of the lost marker (lab `#c8c8c8`), and the white of the pick / recenter ring. Both
/// are neutrals, not hues.
pub const LOST_GREY: Color32 = hex(0xc8c8c8);
pub const PICK_WHITE: Color32 = Color32::WHITE;

pub fn tokens(theme: Theme) -> &'static Tokens {
    match theme {
        Theme::Carbon => &CARBON,
        Theme::Paper => &PAPER,
    }
}

/// Tokens of the theme the UI is currently painted with.
pub fn current(ui: &Ui) -> &'static Tokens {
    if ui.visuals().dark_mode {
        &CARBON
    } else {
        &PAPER
    }
}

/// The marker colour. The one place green is allowed.
pub fn marker_green() -> Color32 {
    MARKER_GREEN
}

/// Fill and text colour of a primary button (yellow, black text). The one place yellow is
/// allowed.
pub fn primary_colors(ui: &Ui) -> (Color32, Color32) {
    (current(ui).primary, PRIMARY_TEXT)
}

/// Sizes that change with "Larger controls".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub border_w: f32,
    pub focus_w: f32,
    pub font: f32,
    pub hit: f32,
    pub marker_w: f32,
}

pub fn metrics(larger: bool) -> Metrics {
    if larger {
        Metrics {
            border_w: 2.0,
            focus_w: 4.0,
            font: 16.0,
            hit: 48.0,
            marker_w: 3.0,
        }
    } else {
        Metrics {
            border_w: 1.0,
            focus_w: 2.0,
            font: 14.0,
            hit: 36.0,
            marker_w: 2.0,
        }
    }
}

/// Whether "Larger controls" is in effect, read back from the applied style.
pub fn is_larger(ui: &Ui) -> bool {
    ui.spacing().interact_size.y >= metrics(true).hit
}

/// Visuals for a theme. Starts from egui's dark/light and overrides every colour (egui's
/// defaults are bluish).
pub fn visuals(theme: Theme, larger: bool) -> Visuals {
    let t = tokens(theme);
    let m = metrics(larger);
    let mut v = match theme {
        Theme::Carbon => Visuals::dark(),
        Theme::Paper => Visuals::light(),
    };
    v.override_text_color = Some(t.text);
    v.weak_text_color = Some(t.muted);
    v.panel_fill = t.bg;
    v.window_fill = t.bg;
    v.window_stroke = Stroke::new(1.0, t.line);
    v.extreme_bg_color = t.surface;
    v.faint_bg_color = t.surface;
    v.text_edit_bg_color = Some(t.surface_2);
    v.code_bg_color = t.surface_2;
    v.hyperlink_color = t.text;
    v.warn_fg_color = t.text;
    v.error_fg_color = t.text;
    v.window_corner_radius = CornerRadius::same(8);
    v.menu_corner_radius = CornerRadius::same(8);
    v.slider_trailing_fill = true;
    v.selection.bg_fill = t.line_strong;
    v.selection.stroke = Stroke::new(m.focus_w, t.text);

    let r = CornerRadius::same(6);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = t.bg;
    w.noninteractive.weak_bg_fill = t.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.line);
    w.noninteractive.fg_stroke = Stroke::new(1.0, t.text);
    w.noninteractive.corner_radius = r;
    w.inactive.bg_fill = t.surface_2;
    w.inactive.weak_bg_fill = t.surface_2;
    w.inactive.bg_stroke = Stroke::new(m.border_w, t.line_strong);
    w.inactive.fg_stroke = Stroke::new(1.0, t.text);
    w.inactive.corner_radius = r;
    w.hovered.bg_fill = t.line;
    w.hovered.weak_bg_fill = t.line;
    w.hovered.bg_stroke = Stroke::new(m.border_w, t.text);
    w.hovered.fg_stroke = Stroke::new(1.5, t.text);
    w.hovered.corner_radius = r;
    // egui draws a focused widget with the `active` visuals: this stroke is the focus ring.
    w.active.bg_fill = t.line;
    w.active.weak_bg_fill = t.line;
    w.active.bg_stroke = Stroke::new(m.focus_w, t.text);
    w.active.fg_stroke = Stroke::new(2.0, t.text);
    w.active.corner_radius = r;
    w.open = w.active;
    v
}

/// The full style for a theme and the "Larger controls" option.
pub fn style(theme: Theme, larger: bool) -> Style {
    let m = metrics(larger);
    let mut s = Style {
        visuals: visuals(theme, larger),
        ..Style::default()
    };
    s.spacing.item_spacing = Vec2::new(8.0, 8.0);
    s.spacing.window_margin = Margin::same(16);
    s.spacing.button_padding = if larger {
        Vec2::new(16.0, 10.0)
    } else {
        Vec2::new(12.0, 6.0)
    };
    s.spacing.interact_size = Vec2::new(40.0, m.hit);
    s.spacing.slider_width = 360.0;
    s.spacing.combo_width = 220.0;
    s.text_styles = [
        (TextStyle::Small, FontId::proportional(m.font - 2.0)),
        (TextStyle::Body, FontId::proportional(m.font)),
        (TextStyle::Button, FontId::proportional(m.font)),
        (TextStyle::Heading, FontId::proportional(m.font + 4.0)),
        (TextStyle::Monospace, FontId::monospace(m.font - 1.0)),
    ]
    .into();
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every UI colour is a grey (R = G = B); only the two helpers carry a hue.
    #[test]
    fn theme_tokens_are_neutral_except_primary() {
        for t in [CARBON, PAPER] {
            for c in [
                t.bg,
                t.surface,
                t.surface_2,
                t.line,
                t.line_strong,
                t.text,
                t.muted,
            ] {
                assert!(c.r() == c.g() && c.g() == c.b(), "{c:?} has a hue");
            }
            assert_ne!(t.primary.r(), t.primary.b(), "primary is yellow");
        }
        assert_eq!(marker_green(), Color32::from_rgb(0, 255, 0));
    }

    #[test]
    fn visuals_use_only_neutral_colours() {
        for theme in [Theme::Carbon, Theme::Paper] {
            for larger in [false, true] {
                let v = visuals(theme, larger);
                let w = &v.widgets;
                for c in [
                    v.panel_fill,
                    v.window_fill,
                    v.extreme_bg_color,
                    v.faint_bg_color,
                    v.hyperlink_color,
                    v.selection.bg_fill,
                    v.selection.stroke.color,
                    w.inactive.bg_fill,
                    w.inactive.bg_stroke.color,
                    w.hovered.bg_fill,
                    w.active.bg_fill,
                    w.active.bg_stroke.color,
                ] {
                    assert!(c.r() == c.g() && c.g() == c.b(), "{theme:?}: {c:?}");
                }
            }
        }
    }

    #[test]
    fn larger_controls_grow_targets_and_focus_ring() {
        let small = style(Theme::Carbon, false);
        let large = style(Theme::Carbon, true);
        assert_eq!(small.spacing.interact_size.y, 36.0);
        assert_eq!(large.spacing.interact_size.y, 48.0);
        assert_eq!(small.visuals.widgets.active.bg_stroke.width, 2.0);
        assert_eq!(large.visuals.widgets.active.bg_stroke.width, 4.0);
    }
}
