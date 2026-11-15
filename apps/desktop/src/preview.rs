//! The camera preview: colour texture, target markers, the recenter ring and badge, the
//! hover (pick) ring, overlays and the click → frame mapping.
//!
//! R-C2: a click is mapped to frame coordinates through the **same** rectangle the image is
//! drawn in ([`fit_rect`]), so letterboxing can't shift the selection.

use std::f32::consts::TAU;

use app_core::ports::Point2;
use app_core::{MarkerKind, View};
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Painter, Pos2, Rect, Role, Sense, Shape, Stroke,
    TextureHandle, Ui, UiBuilder, Vec2, WidgetInfo,
};

use crate::icons;
use crate::status::{Action, Camera, Card, Status, can_pick, seconds};
use crate::theme;
use crate::widgets::{self, Kind};

/// The preview area is 4:3 (ADR-0008 canonical 640×480); other frame shapes letterbox.
pub const AREA_ASPECT: f32 = 4.0 / 3.0;
/// Frame size assumed for drawing before the first frame arrives.
const DEFAULT_FRAME: (u32, u32) = (640, 480);
/// Marker ring radius in frame pixels (design lab: r ≈ 15).
const MARKER_R: f32 = 15.0;
/// Accessible name of the preview (lab `aria-label`).
pub const PREVIEW_LABEL: &str =
    "Camera preview. Click a point on your face to select the tracking target.";

const CAMERA_BG: Color32 = Color32::from_gray(10);
const SCRIM: Color32 = Color32::from_black_alpha(140);
const LABEL_BG: Color32 = Color32::from_black_alpha(190);

/// The largest rectangle of the frame's aspect ratio centred in `area`.
pub fn fit_rect(area: Rect, frame: (u32, u32)) -> Rect {
    let (fw, fh) = (frame.0.max(1) as f32, frame.1.max(1) as f32);
    let scale = (area.width() / fw).min(area.height() / fh);
    Rect::from_center_size(area.center(), Vec2::new(fw * scale, fh * scale))
}

/// Screen position → frame coordinates through the drawing rect. `None` outside the
/// picture (e.g. on a letterbox bar).
pub fn to_frame(pos: Pos2, drawn: Rect, frame: (u32, u32)) -> Option<Point2> {
    if !drawn.contains(pos) || drawn.width() <= 0.0 || drawn.height() <= 0.0 {
        return None;
    }
    let u = (pos.x - drawn.min.x) / drawn.width();
    let v = (pos.y - drawn.min.y) / drawn.height();
    Some(Point2::new(u * frame.0 as f32, v * frame.1 as f32))
}

/// Frame coordinates → screen position (inverse of [`to_frame`]).
pub fn to_screen(p: Point2, drawn: Rect, frame: (u32, u32)) -> Pos2 {
    Pos2::new(
        drawn.min.x + p.x / frame.0.max(1) as f32 * drawn.width(),
        drawn.min.y + p.y / frame.1.max(1) as f32 * drawn.height(),
    )
}

/// What happened in the preview this pass.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Output {
    /// A click on the picture while picking is allowed, in frame coordinates.
    pub pick: Option<Point2>,
    /// A card button.
    pub action: Option<Action>,
}

/// Inputs that aren't in [`View`].
pub struct Extras<'a> {
    pub texture: Option<&'a TextureHandle>,
    pub mirrored: bool,
    /// Diagnostics lines, when the option is on.
    pub diagnostics: Option<Vec<String>>,
}

/// Draws the preview at full available width, 4:3.
pub fn show(ui: &mut Ui, view: &View, status: &Status, extras: &Extras<'_>) -> Output {
    let width = ui.available_width();
    let (area, response) =
        ui.allocate_exact_size(Vec2::new(width, width / AREA_ASPECT), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Image, true, PREVIEW_LABEL));
    let frame = view.frame_size.unwrap_or(DEFAULT_FRAME);
    let drawn = fit_rect(area, frame);
    let large = theme::is_larger(ui);
    let painter = ui.painter_at(area);
    let mut out = Output::default();

    painter.rect_filled(area, 8.0, CAMERA_BG);
    if status.camera != Camera::Off
        && let Some(tex) = extras.texture
    {
        painter.image(
            tex.id(),
            drawn,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    if matches!(status.camera, Camera::Dimmed | Camera::Frozen) {
        painter.rect_filled(drawn, 0.0, SCRIM);
    }

    let scale = drawn.width() / frame.0.max(1) as f32;
    let r = (MARKER_R * scale).max(8.0);
    let w = theme::metrics(large).marker_w;
    let at = |p: Point2| to_screen(p, drawn, frame);

    if let Some(m) = view.marker {
        draw_marker(
            ui,
            &painter,
            at(m.position),
            r,
            w,
            m.kind,
            view.dwell_progress,
        );
    }
    if let Some(rc) = view.recenter {
        icons::dashed_circle(
            &painter,
            at(rc.center),
            r,
            Stroke::new(w, theme::PICK_WHITE),
            10,
        );
        badge(
            &painter,
            Pos2::new(area.center().x, area.max.y - 24.0),
            &format!("Recentering in {}", seconds(rc.remaining_us)),
            Align2::CENTER_CENTER,
        );
    }

    let pick = can_pick(view.state) && status.card.is_none() && !status.loading;
    let hover = response
        .hover_pos()
        .filter(|_| pick)
        .filter(|p| drawn.contains(*p));
    if let Some(h) = hover {
        icons::dashed_circle(&painter, h, r, Stroke::new(w, theme::PICK_WHITE), 10);
        if view.marker.is_none() {
            badge(
                &painter,
                h + Vec2::new(r + 8.0, 0.0),
                "Click to select",
                Align2::LEFT_CENTER,
            );
        }
    } else if pick && view.marker.is_none() && view.recenter.is_none() {
        badge(
            &painter,
            Pos2::new(area.center().x, area.max.y - 24.0),
            "Click a point on your face",
            Align2::CENTER_CENTER,
        );
    }
    if pick
        && response.clicked()
        && let Some(p) = response
            .interact_pointer_pos()
            .and_then(|pos| to_frame(pos, drawn, frame))
    {
        out.pick = Some(p);
    }

    let live = match status.camera {
        Camera::Live if extras.mirrored => Some("Live · mirrored"),
        Camera::Live => Some("Live"),
        Camera::Dimmed => Some("Paused"),
        Camera::Frozen => Some("No signal · last frame"),
        Camera::Off => None,
    };
    if let Some(text) = live {
        badge(
            &painter,
            area.min + Vec2::new(10.0, 16.0),
            text,
            Align2::LEFT_CENTER,
        );
    }
    if status.camera == Camera::Dimmed {
        painter.circle_filled(area.center(), 44.0, Color32::from_black_alpha(150));
        icons::paint(
            &painter,
            Rect::from_center_size(area.center(), Vec2::splat(44.0)),
            crate::status::Icon::Paused,
            Color32::WHITE,
            2.0,
        );
    }
    if let Some(lines) = &extras.diagnostics {
        diagnostics(&painter, area, lines);
    }

    // Lab `.overlay.scrim`: the theme background at 82 % over the picture.
    let scrim = theme::current(ui).bg.gamma_multiply(0.82);
    if status.loading {
        painter.rect_filled(area, 8.0, scrim);
        overlay(ui, area, "loading", |ui| {
            let t = theme::current(ui);
            ui.add(egui::Spinner::new().size(30.0).color(t.text));
            ui.label(egui::RichText::new("Starting camera…").color(t.text));
        });
    }
    if let Some(card) = &status.card {
        painter.rect_filled(area, 8.0, scrim);
        overlay(ui, area, "card", |ui| {
            out.action = show_card(ui, card);
        });
    }
    out
}

/// Lays out `add` centred over `area` (both ways), in a child `Ui` that doesn't move the
/// parent's cursor (the status line stays below the preview). egui lays out top-down in one
/// pass, so the content's height from the previous pass places it; a changed height asks for
/// one more pass.
fn overlay(ui: &mut Ui, area: Rect, key: &str, add: impl FnOnce(&mut Ui)) {
    let inner = area.shrink(20.0);
    let id = ui.scope_id().with(("overlay", key));
    let last_h: Option<f32> = ui.data(|d| d.get_temp(id));
    let h = last_h.unwrap_or(0.0).min(inner.height());
    let top = inner.center().y - h / 2.0;
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(Rect::from_min_max(Pos2::new(inner.left(), top), inner.max))
            .layout(Layout::top_down(Align::Center)),
    );
    add(&mut child);
    let used = child.min_rect().height();
    if last_h.is_none_or(|l| (l - used).abs() > 0.5) {
        ui.data_mut(|d| d.insert_temp(id, used));
        ui.ctx().request_repaint();
    }
}

/// Lab `.card`: `--surface`, `--line-strong` border, radius 8, padding 18 × 20, at most
/// 360 wide; icon, title (1.1 em), muted body, then the buttons, all left-aligned.
fn show_card(ui: &mut Ui, card: &Card) -> Option<Action> {
    let t = theme::current(ui);
    let m = theme::metrics(theme::is_larger(ui));
    let mut action = None;
    egui::Frame::new()
        .fill(t.surface)
        .stroke(Stroke::new(m.border_w, t.line_strong))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(20, 18))
        .show(ui, |ui| {
            ui.set_max_width(360.0 - 40.0);
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                icons::show(ui, card.icon, 28.0);
                ui.add_space(4.0);
                let size = ui.style().text_styles[&egui::TextStyle::Body].size * 1.1;
                ui.label(egui::RichText::new(card.title).size(size).strong());
                ui.label(egui::RichText::new(&card.body).color(t.muted));
                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    for b in &card.buttons {
                        let kind = if b.primary {
                            Kind::Primary
                        } else {
                            Kind::Secondary
                        };
                        if widgets::Button::new(b.label, kind).show(ui).clicked() {
                            action = Some(b.action);
                        }
                    }
                });
            });
        });
    action
}

fn draw_marker(
    ui: &Ui,
    painter: &Painter,
    c: Pos2,
    r: f32,
    w: f32,
    kind: MarkerKind,
    dwell: Option<f32>,
) {
    let green = theme::marker_green();
    match kind {
        MarkerKind::Tracking => {
            painter.circle_stroke(c, r, Stroke::new(w, green));
            if let Some(p) = dwell.filter(|p| *p > 0.0) {
                // Dwell progress: a thin arc just outside the ring, clockwise from the top.
                let a0 = -TAU / 4.0;
                let pts: Vec<Pos2> = (0..=32)
                    .map(|i| c + (r + 4.0 + w) * Vec2::angled(a0 + TAU * p * i as f32 / 32.0))
                    .collect();
                painter.add(Shape::line(pts, Stroke::new(w, green)));
            }
        }
        MarkerKind::Held => {
            icons::dashed_circle(painter, c, r, Stroke::new(w, green), 10);
            badge(
                painter,
                c + Vec2::new(r + 8.0, 0.0),
                "Holding",
                Align2::LEFT_CENTER,
            );
        }
        MarkerKind::Searching => {
            // Growing, fading outline (lab). Needs repaints while visible.
            let k = (ui.input(|i| i.time) % 1.4) as f32 / 1.4;
            let alpha = ((1.0 - 0.7 * k) * 255.0) as u8;
            let grow = Color32::from_rgba_unmultiplied(0, 255, 0, alpha);
            painter.circle_stroke(c, r * (1.3 + 3.0 * k), Stroke::new(w, grow));
            painter.circle_stroke(c, r, Stroke::new(w, green));
            badge(
                painter,
                c + Vec2::new(r + 8.0, 0.0),
                "Searching",
                Align2::LEFT_CENTER,
            );
            ui.ctx().request_repaint();
        }
        MarkerKind::Lost => {
            let s = Stroke::new(w, theme::LOST_GREY);
            painter.circle_stroke(c, r, s);
            let h = r * 0.55;
            painter.line_segment([c + Vec2::new(-h, -h), c + Vec2::new(h, h)], s);
            painter.line_segment([c + Vec2::new(h, -h), c + Vec2::new(-h, h)], s);
            badge(
                painter,
                c + Vec2::new(r + 8.0, 0.0),
                "Lost",
                Align2::LEFT_CENTER,
            );
        }
    }
}

/// White text on a dark plate, readable on any picture.
fn badge(painter: &Painter, pos: Pos2, text: &str, align: Align2) {
    let galley =
        painter.layout_no_wrap(text.to_owned(), FontId::proportional(13.0), Color32::WHITE);
    let rect = align.anchor_size(pos, galley.size() + Vec2::new(12.0, 6.0));
    painter.rect_filled(rect, 4.0, LABEL_BG);
    painter.galley(rect.min + Vec2::new(6.0, 3.0), galley, Color32::WHITE);
}

fn diagnostics(painter: &Painter, area: Rect, lines: &[String]) {
    let galley = painter.layout_no_wrap(lines.join("\n"), FontId::monospace(12.0), Color32::WHITE);
    let rect = Align2::RIGHT_TOP.anchor_size(
        area.right_top() + Vec2::new(-10.0, 10.0),
        galley.size() + Vec2::new(12.0, 8.0),
    );
    painter.rect_filled(rect, 4.0, LABEL_BG);
    painter.galley(rect.min + Vec2::new(6.0, 4.0), galley, Color32::WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Point2, b: Point2) -> bool {
        (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
    }

    #[test]
    fn a_4_3_frame_fills_the_4_3_area() {
        let area = Rect::from_min_size(Pos2::new(16.0, 40.0), Vec2::new(528.0, 396.0));
        assert_eq!(fit_rect(area, (640, 480)), area);
    }

    #[test]
    fn a_wide_frame_letterboxes_top_and_bottom() {
        let area = Rect::from_min_size(Pos2::new(16.0, 40.0), Vec2::new(528.0, 396.0));
        let drawn = fit_rect(area, (640, 360));
        assert_eq!(drawn.width(), 528.0);
        assert!((drawn.height() - 297.0).abs() < 1e-3);
        assert!((drawn.min.y - (40.0 + 49.5)).abs() < 1e-3);
    }

    #[test]
    fn clicks_map_through_the_drawing_rect_with_letterboxing() {
        let area = Rect::from_min_size(Pos2::new(16.0, 40.0), Vec2::new(528.0, 396.0));
        let frame = (640, 360);
        let drawn = fit_rect(area, frame);
        // Centre of the picture is the frame centre.
        assert!(close(
            to_frame(drawn.center(), drawn, frame).unwrap(),
            Point2::new(320.0, 180.0)
        ));
        // Top-left of the picture (not of the area) is (0, 0).
        assert!(close(
            to_frame(drawn.min, drawn, frame).unwrap(),
            Point2::new(0.0, 0.0)
        ));
        // A click on the letterbox bar selects nothing.
        assert_eq!(
            to_frame(area.min + Vec2::new(10.0, 10.0), drawn, frame),
            None
        );
        // Round trip.
        let p = Point2::new(100.0, 300.0);
        assert!(close(
            to_frame(to_screen(p, drawn, frame), drawn, frame).unwrap(),
            p
        ));
    }

    #[test]
    fn a_tall_frame_pillarboxes() {
        let area = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let drawn = fit_rect(area, (300, 300));
        assert_eq!(
            drawn,
            Rect::from_min_max(Pos2::new(50.0, 0.0), Pos2::new(350.0, 300.0))
        );
        assert_eq!(to_frame(Pos2::new(10.0, 150.0), drawn, (300, 300)), None);
    }
}
