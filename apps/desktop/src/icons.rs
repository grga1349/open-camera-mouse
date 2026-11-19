//! Icons painted with `Painter` primitives (no SVG crate, no licence
//! notice). Simplified shapes after the lab's sprite, drawn in the text colour.

use std::f32::consts::TAU;

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

use crate::status::Icon;

/// Points on a circle arc from `a0` to `a1` (radians, 0 = right, clockwise on screen).
fn arc(c: Pos2, r: f32, a0: f32, a1: f32, n: usize) -> Vec<Pos2> {
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            c + r * Vec2::angled(a)
        })
        .collect()
}

/// A dashed circle (egui has no dashed-circle primitive): `dashes` dashes, half the
/// circumference drawn.
pub fn dashed_circle(painter: &Painter, c: Pos2, r: f32, stroke: Stroke, dashes: usize) {
    let step = TAU / dashes as f32;
    for i in 0..dashes {
        let a0 = i as f32 * step;
        painter.add(Shape::line(arc(c, r, a0, a0 + step * 0.55, 6), stroke));
    }
}

/// Paints `icon` filling `rect` (square), lines `w` wide.
pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32, w: f32) {
    let s = Stroke::new(w, color);
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.5 - w;
    let p = |x: f32, y: f32| c + Vec2::new(x * r, y * r);
    let line = |a: Pos2, b: Pos2| {
        painter.line_segment([a, b], s);
    };
    match icon {
        Icon::Play => {
            painter.add(Shape::convex_polygon(
                vec![p(-0.55, -0.8), p(0.8, 0.0), p(-0.55, 0.8)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Paused => {
            let bar = Vec2::new(r * 0.32, r * 1.5);
            for x in [-0.45, 0.45] {
                painter.rect_filled(Rect::from_center_size(p(x, 0.0), bar), 1.0, color);
            }
        }
        Icon::Tracking => {
            painter.circle_stroke(c, r * 0.85, s);
            painter.circle_filled(c, r * 0.25, color);
        }
        Icon::Held => {
            dashed_circle(painter, c, r * 0.85, s, 6);
            painter.circle_filled(c, r * 0.25, color);
        }
        Icon::NoTarget => {
            // Corners of a viewfinder plus a centre cross.
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let corner = p(0.9 * sx, 0.9 * sy);
                line(corner, p(0.4 * sx, 0.9 * sy));
                line(corner, p(0.9 * sx, 0.4 * sy));
            }
            line(p(-0.3, 0.0), p(0.3, 0.0));
            line(p(0.0, -0.3), p(0.0, 0.3));
        }
        Icon::Searching => {
            painter.circle_stroke(p(-0.2, -0.2), r * 0.6, s);
            line(p(0.25, 0.25), p(0.9, 0.9));
        }
        Icon::Lost => {
            painter.circle_stroke(c, r * 0.9, s);
            line(p(-0.4, -0.4), p(0.4, 0.4));
            line(p(0.4, -0.4), p(-0.4, 0.4));
        }
        Icon::Recenter => {
            dashed_circle(painter, c, r * 0.6, s, 6);
            for (a, b) in [
                (p(0.0, -1.0), p(0.0, -0.75)),
                (p(0.0, 1.0), p(0.0, 0.75)),
                (p(-1.0, 0.0), p(-0.75, 0.0)),
                (p(1.0, 0.0), p(0.75, 0.0)),
            ] {
                line(a, b);
            }
        }
        Icon::Mouse => {
            let body = Rect::from_center_size(c, Vec2::new(r * 1.1, r * 1.8));
            painter.rect_stroke(body, r * 0.5, s, egui::StrokeKind::Middle);
            line(p(0.0, -0.9), p(0.0, -0.4));
        }
        Icon::Lock | Icon::PointerLock => {
            let body = Rect::from_min_max(p(-0.75, -0.1), p(0.75, 0.9));
            painter.rect_stroke(body, 2.0, s, egui::StrokeKind::Middle);
            painter.add(Shape::line(
                arc(p(0.0, -0.1), r * 0.45, TAU / 2.0, TAU, 10),
                s,
            ));
            if icon == Icon::PointerLock {
                painter.circle_filled(p(0.0, 0.4), r * 0.15, color);
            }
        }
        Icon::CameraOff | Icon::Unplug => {
            if icon == Icon::CameraOff {
                let body = Rect::from_min_max(p(-0.9, -0.5), p(0.4, 0.5));
                painter.rect_stroke(body, 2.0, s, egui::StrokeKind::Middle);
                painter.add(Shape::convex_polygon(
                    vec![p(0.45, 0.0), p(0.9, -0.4), p(0.9, 0.4)],
                    Color32::TRANSPARENT,
                    s,
                ));
            } else {
                let plug = Rect::from_min_max(p(-0.5, -0.3), p(0.5, 0.5));
                painter.rect_stroke(plug, 2.0, s, egui::StrokeKind::Middle);
                line(p(-0.25, -0.3), p(-0.25, -0.75));
                line(p(0.25, -0.3), p(0.25, -0.75));
                line(p(0.0, 0.5), p(0.0, 0.95));
            }
            line(p(-0.95, -0.95), p(0.95, 0.95));
        }
        Icon::Error => {
            painter.add(Shape::convex_polygon(
                vec![p(0.0, -0.95), p(0.95, 0.8), p(-0.95, 0.8)],
                Color32::TRANSPARENT,
                s,
            ));
            line(p(0.0, -0.35), p(0.0, 0.2));
            painter.circle_filled(p(0.0, 0.5), w * 0.8, color);
        }
        Icon::Loading => {
            painter.add(Shape::line(arc(c, r * 0.85, 0.0, TAU * 0.75, 18), s));
        }
    }
}

/// An icon as a non-interactive widget of `size` points.
pub fn show(ui: &mut Ui, icon: Icon, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        let color = ui.visuals().text_color();
        paint(ui.painter(), rect, icon, color, 1.6);
    }
    resp
}
