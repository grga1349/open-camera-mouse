//! Small custom widgets the lab needs and egui lacks: a toggle switch with ON/OFF text,
//! tabs, a segmented control and a labelled slider row. Each takes keyboard focus, draws the
//! focus ring and reports an AccessKit label and state.

use std::ops::RangeInclusive;

use eframe::egui::{
    self, Align, Key, Layout, Response, RichText, Role, Sense, Stroke, StrokeKind, Ui, Vec2,
    WidgetInfo,
};

use crate::theme;

/// A label row (`label` strong, optional muted `help` under it) with "ON"/"OFF" and a switch
/// on the right, aligned with the label line. Returns the switch's response.
pub fn switch(ui: &mut Ui, on: &mut bool, label: &str, help: Option<&str>) -> Response {
    let mut resp = None;
    ui.horizontal_top(|ui| {
        let right = 96.0;
        let text_w = (ui.available_width() - right).max(80.0);
        let label_id = ui
            .vertical(|ui| {
                ui.set_width(text_w);
                let l = ui.label(RichText::new(label).strong());
                if let Some(h) = help {
                    ui.label(RichText::new(h).weak());
                }
                l.id
            })
            .inner;
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            // Named by the label row, so a screen reader (and a test) finds one node.
            resp = Some(toggle(ui, on, "").labelled_by(label_id));
            ui.label(RichText::new(if *on { "ON" } else { "OFF" }).monospace());
        });
    });
    resp.expect("switch drawn")
}

/// The switch itself (egui demo `toggle_switch` pattern). `label` may be empty when the
/// caller names it with `labelled_by`.
pub fn toggle(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    let h = 22.0;
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(h * 1.9, h), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let state = *on;
    let label = label.to_owned();
    response.widget_info(|| WidgetInfo::selected(Role::Switch, true, state, &label));
    if ui.is_rect_visible(rect) {
        let t = theme::current(ui);
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let radius = rect.height() / 2.0;
        let (fill, knob) = if *on {
            (t.text, t.bg)
        } else {
            (t.surface_2, t.text)
        };
        let painter = ui.painter();
        painter.rect(
            rect,
            radius,
            fill,
            Stroke::new(theme::metrics(theme::is_larger(ui)).border_w, t.line_strong),
            StrokeKind::Inside,
        );
        let x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        painter.circle_filled(egui::pos2(x, rect.center().y), radius * 0.72, knob);
        focus_ring(ui, &response, rect.expand(3.0), radius + 3.0);
    }
    response
}

/// The focus ring around a custom widget: text colour, `--focus-w` wide.
fn focus_ring(ui: &Ui, response: &Response, rect: egui::Rect, radius: f32) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            radius,
            ui.visuals().selection.stroke,
            StrokeKind::Outside,
        );
    }
}

/// A row of tabs with the selected one underlined. Left/Right/Home/End move between tabs while
/// one has focus (lab `role="tablist"`).
pub fn tabs<T: Copy + PartialEq>(ui: &mut Ui, selected: &mut T, items: &[(T, &str)]) {
    let mut responses = Vec::with_capacity(items.len());
    ui.horizontal(|ui| {
        // Equal widths that shrink with the window, so all tabs fit at 130 % text and with
        // larger controls.
        let gap = ui.spacing().item_spacing.x;
        let n = items.len().max(1) as f32;
        let w = ((ui.available_width() - gap * (n - 1.0)) / n).max(40.0);
        let h = ui.spacing().interact_size.y;
        ui.spacing_mut().button_padding.x = 4.0;
        for (value, name) in items {
            let on = *selected == *value;
            let text = if on {
                RichText::new(*name).strong()
            } else {
                RichText::new(*name).weak()
            };
            let mut r = ui.add_sized(
                [w, h],
                egui::Button::selectable(on, text).frame_when_inactive(false),
            );
            let name = (*name).to_owned();
            r.widget_info(|| WidgetInfo::selected(Role::Tab, true, on, &name));
            if r.clicked() && !on {
                *selected = *value;
                r.mark_changed();
            }
            if on {
                let rect = r.rect;
                ui.painter().line_segment(
                    [rect.left_bottom(), rect.right_bottom()],
                    Stroke::new(2.0, theme::current(ui).text),
                );
            }
            responses.push(r);
        }
    });
    let focused = responses.iter().position(|r| r.has_focus());
    if let Some(i) = focused {
        let n = items.len();
        let target = ui.input(|inp| {
            if inp.key_pressed(Key::ArrowRight) {
                Some((i + 1) % n)
            } else if inp.key_pressed(Key::ArrowLeft) {
                Some((i + n - 1) % n)
            } else if inp.key_pressed(Key::Home) {
                Some(0)
            } else if inp.key_pressed(Key::End) {
                Some(n - 1)
            } else {
                None
            }
        });
        if let Some(j) = target {
            *selected = items[j].0;
            responses[j].request_focus();
        }
    }
}

/// Two or more mutually exclusive options in a framed row (radio semantics).
pub fn segmented<T: Copy + PartialEq>(ui: &mut Ui, value: &mut T, items: &[(T, &str)]) {
    let t = theme::current(ui);
    egui::Frame::new()
        .stroke(Stroke::new(1.0, t.line_strong))
        .corner_radius(6.0)
        .inner_margin(2.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (v, name) in items {
                    let on = *value == *v;
                    let mut r = ui.add(egui::Button::selectable(on, *name));
                    let name = (*name).to_owned();
                    r.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, on, &name));
                    if r.clicked() && !on {
                        *value = *v;
                        r.mark_changed();
                    }
                }
            });
        });
}

/// Label (left) and value (right, monospace) on one line, a full-width slider under them and
/// an optional muted help line. The slider is labelled by the label for AccessKit.
pub fn slider_row(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    step: f64,
    shown: impl Fn(f32) -> String,
    help: Option<&str>,
) -> Response {
    let label_id = ui
        .horizontal(|ui| {
            let l = ui.label(label);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(shown(*value)).monospace());
            });
            l.id
        })
        .inner;
    let width = ui.available_width();
    let r = ui
        .scope(|ui| {
            ui.spacing_mut().slider_width = width;
            ui.add(
                egui::Slider::new(value, range)
                    .step_by(step)
                    .show_value(false)
                    .clamping(egui::SliderClamping::Always),
            )
            .labelled_by(label_id)
        })
        .inner;
    if let Some(h) = help {
        ui.label(RichText::new(h).weak());
    }
    r
}
