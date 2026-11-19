//! The lab's controls, painted to its CSS (`docs/design-lab.html`):
//! switch, slider, tabs, segmented control, buttons with a key chip, field frames and help
//! text. egui's own widgets keep the behaviour where they have it (the slider's dragging,
//! keys and AccessKit value); the look is ours. Each control takes keyboard focus, draws the
//! lab's focus ring (`--focus-w` in the text colour, 2 px outside) and reports an AccessKit
//! label and state.

use std::ops::RangeInclusive;

use eframe::egui::{
    self, Align, Align2, Color32, FontId, Galley, Key, Layout, Pos2, Rect, Response, RichText,
    Role, Sense, Stroke, StrokeKind, TextStyle, Ui, Vec2, WidgetInfo,
};
use std::sync::Arc;

use crate::theme::{self, Tokens};

/// Lab `--radius-sm`: buttons, fields, the segmented control.
pub const RADIUS_SM: f32 = 6.0;
/// Lab `--gap` between a field's parts (label line, control, help).
const FIELD_GAP: f32 = 4.0;

fn font(ui: &Ui, style: TextStyle) -> FontId {
    style.resolve(ui.style())
}

fn body_size(ui: &Ui) -> f32 {
    font(ui, TextStyle::Body).size
}

fn layout(ui: &Ui, text: &str, font: FontId, color: Color32) -> Arc<Galley> {
    ui.painter().layout_no_wrap(text.to_owned(), font, color)
}

/// The lab's focus ring: `outline: var(--focus-w) solid var(--focus); outline-offset: 2px`.
pub fn focus_ring(ui: &Ui, response: &Response, rect: Rect, radius: f32) {
    if response.has_focus() {
        let t = theme::current(ui);
        let w = theme::metrics(theme::is_larger(ui)).focus_w;
        ui.painter().rect_stroke(
            rect.expand(2.0),
            radius + 2.0,
            Stroke::new(w, t.text),
            StrokeKind::Outside,
        );
    }
}

/// Help text under a control: muted, 0.88 em (lab `.help`).
pub fn help(ui: &mut Ui, text: &str) -> Response {
    ui.label(RichText::new(text).small().color(theme::current(ui).muted))
}

/// A field label (lab `.field .lbl label`, weight 600).
pub fn field_label(ui: &mut Ui, text: &str) -> Response {
    ui.label(RichText::new(text).strong())
}

// ---------------------------------------------------------------- switch

/// Lab `.switch button`: 44 × 24 track, knob 18.
const SWITCH: Vec2 = Vec2::new(44.0, 24.0);
const KNOB_D: f32 = 18.0;

/// A label row (`label` strong, optional muted `help` under it) with "ON"/"OFF" and a switch
/// on the right, centred on the label's first line (lab `.switch`). The whole row is a click
/// target, as the lab's `<label>` is; only the switch takes focus. Returns the switch's
/// response.
pub fn switch(ui: &mut Ui, on: &mut bool, label: &str, help_text: Option<&str>) -> Response {
    let t = theme::current(ui);
    let gap = 12.0;
    let st_font = FontId::monospace(body_size(ui) * 0.78);
    let st_w = 2.6 * st_font.size;
    let right = st_w + 8.0 + SWITCH.x;
    let top = ui.cursor().min;
    let width = ui.available_width();
    let text_w = (width - right - gap).max(80.0);

    let row = ui.scope(|ui| {
        let mut label_id = None;
        let mut line_h = 0.0;
        ui.allocate_ui_with_layout(Vec2::new(text_w, 0.0), Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.set_width(text_w);
            let l = ui.add(egui::Label::new(RichText::new(label).strong()).wrap());
            line_h = ui.fonts_mut(|f| f.row_height(&font(ui, TextStyle::Body)));
            label_id = Some(l.id);
            if let Some(h) = help_text {
                help(ui, h);
            }
        });
        (label_id.expect("label drawn"), line_h)
    });
    let (label_id, line_h) = row.inner;
    let text_rect = row.response.rect;

    // The switch, centred on the first text line, at the right edge.
    let line_mid = top.y + line_h / 2.0;
    let sw_rect =
        Rect::from_center_size(Pos2::new(top.x + width - SWITCH.x / 2.0, line_mid), SWITCH);
    let id = ui.scope_id().with(("switch", label));
    let resp = ui.interact(sw_rect, id, Sense::click());
    let state = *on;
    resp.widget_info(|| WidgetInfo::selected(Role::Switch, ui.is_enabled(), state, ""));
    let resp = resp.labelled_by(label_id);

    // The row (text and "ON"/"OFF") toggles too; it isn't a separate focus stop.
    let row_rect = Rect::from_min_max(
        top,
        Pos2::new(top.x + width, text_rect.max.y.max(sw_rect.max.y)),
    );
    let row_resp = ui.interact(row_rect, id.with("row"), Sense::CLICK);

    let mut resp = resp;
    if resp.clicked() || (row_resp.clicked() && !resp.clicked()) {
        *on = !*on;
        resp.mark_changed();
    }

    // Reserve the row's height (at least `--hit`, unless the label alone is shorter, as in
    // the lab's inline "Keep both speeds the same").
    let used_h = (row_rect.height()).max(if help_text.is_some() {
        ui.spacing().interact_size.y
    } else {
        0.0
    });
    let used = Rect::from_min_size(top, Vec2::new(width, used_h));
    ui.advance_cursor_after_rect(used);

    if ui.is_rect_visible(row_rect) {
        let st = if *on { "ON" } else { "OFF" };
        ui.painter().text(
            Pos2::new(sw_rect.left() - 8.0, line_mid),
            Align2::RIGHT_CENTER,
            st,
            st_font,
            t.muted,
        );
        paint_switch(ui, &resp, sw_rect, *on, t);
    }
    resp
}

fn paint_switch(ui: &Ui, resp: &Response, rect: Rect, on: bool, t: &Tokens) {
    let how_on = ui.ctx().animate_bool_responsive(resp.id, on);
    let border = theme::metrics(theme::is_larger(ui)).border_w;
    let radius = rect.height() / 2.0;
    let (fill, line, knob) = if on {
        (t.text, t.text, t.bg)
    } else {
        (t.surface_2, t.line_strong, t.muted)
    };
    let p = ui.painter();
    p.rect(
        rect,
        radius,
        fill,
        Stroke::new(border, line),
        StrokeKind::Inside,
    );
    // Knob 3 px from the outer edge in both positions (1 px border + 2 px inset).
    let inset = (rect.height() - KNOB_D) / 2.0 + KNOB_D / 2.0;
    let x = egui::lerp((rect.left() + inset)..=(rect.right() - inset), how_on);
    p.circle_filled(Pos2::new(x, rect.center().y), KNOB_D / 2.0, knob);
    focus_ring(ui, resp, rect, radius);
}

// ---------------------------------------------------------------- slider

/// Label (left, strong) and value (right, monospace) on one line, a full-width slider under
/// them and an optional muted help line (lab `.field` + `.rng`). The slider is labelled by
/// the label for AccessKit.
///
/// egui's `Slider` does the work (pointer, arrow keys, AccessKit value and actions) and is
/// painted invisible; the lab's rail and thumb are painted over it at the same geometry.
pub fn slider_row(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    step: f64,
    shown: impl Fn(f32) -> String,
    help_text: Option<&str>,
) -> Response {
    let label_id = ui
        .horizontal(|ui| {
            let l = field_label(ui, label);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(shown(*value)).monospace());
            });
            l.id
        })
        .inner;
    ui.add_space(FIELD_GAP - ui.spacing().item_spacing.y);
    let width = ui.available_width();
    let (lo, hi) = (*range.start(), *range.end());
    let r = ui
        .scope(|ui| {
            ui.spacing_mut().slider_width = width;
            invisible_slider_visuals(ui);
            ui.add(
                egui::Slider::new(value, range)
                    .step_by(step)
                    .show_value(false)
                    .trailing_fill(false)
                    .clamping(egui::SliderClamping::Always),
            )
            .labelled_by(label_id)
        })
        .inner;
    paint_slider(ui, &r, (*value - lo) / (hi - lo).max(f32::EPSILON));
    if let Some(h) = help_text {
        ui.add_space(FIELD_GAP - ui.spacing().item_spacing.y);
        help(ui, h);
    }
    r
}

/// egui paints nothing of the slider: transparent rail, fill and handle.
fn invisible_slider_visuals(ui: &mut Ui) {
    let v = ui.visuals_mut();
    v.selection.bg_fill = Color32::TRANSPARENT;
    for w in [
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
    ] {
        w.bg_fill = Color32::TRANSPARENT;
        w.fg_stroke = Stroke::NONE;
        w.expansion = 0.0;
    }
}

/// Lab `.rng`: a 4 px rail (`--text` up to the thumb, `--line-strong` after), a round thumb
/// (18 px, 26 px with larger controls) in `--text` with a 3 px `--bg` border and a 1 px
/// `--text` ring.
fn paint_slider(ui: &Ui, r: &Response, frac: f32) {
    if !ui.is_rect_visible(r.rect) {
        return;
    }
    let t = theme::current(ui);
    let rect = r.rect;
    // egui's handle centre runs over the rect shrunk by its handle radius (height / 2.5);
    // the thumb is drawn at exactly that point so clicks and paint agree.
    let egui_r = rect.height() / 2.5;
    let thumb_d = if theme::is_larger(ui) { 26.0 } else { 18.0 };
    let x0 = rect.left() + egui_r;
    let x1 = rect.right() - egui_r;
    let x = egui::lerp(x0..=x1, frac.clamp(0.0, 1.0));
    let y = rect.center().y;
    let rail_l = x0 - thumb_d / 2.0;
    let rail_r = x1 + thumb_d / 2.0;
    let p = ui.painter();
    let rail = |a: f32, b: f32, c: Color32| {
        p.rect_filled(
            Rect::from_min_max(Pos2::new(a, y - 2.0), Pos2::new(b, y + 2.0)),
            2.0,
            c,
        );
    };
    // The rail stops at the thumb's edge: a disabled control is painted translucent, and
    // the rail must not show through the thumb.
    let rad = thumb_d / 2.0;
    rail(rail_l, (x - rad).max(rail_l), t.text);
    rail((x + rad).min(rail_r), rail_r, t.line_strong);
    let c = Pos2::new(x, y);
    p.circle_filled(c, rad + 1.0, t.text); // 1 px ring
    p.circle_filled(c, rad, t.bg); // 3 px border
    p.circle_filled(c, rad - 3.0, t.text);
    focus_ring(
        ui,
        r,
        Rect::from_center_size(c, Vec2::splat(thumb_d + 2.0)),
        rad + 1.0,
    );
}

// ---------------------------------------------------------------- tabs

/// A row of tabs (lab `.tabs`): no frame or fill; muted text, `--text` when hovered or
/// selected; the selected tab has a 3 px `--text` underline over the row's `--line` border.
/// Left/Right/Home/End move between tabs while one has focus (lab `role="tablist"`).
///
/// Tabs are left-aligned with 11 px side padding; when they don't fit (130 % text with
/// larger controls) the padding shrinks, down to none, so they still do (five tabs since #9).
pub fn tabs<T: Copy + PartialEq>(ui: &mut Ui, selected: &mut T, items: &[(T, &str)]) {
    let t = theme::current(ui);
    let border = theme::metrics(theme::is_larger(ui)).border_w;
    let h = ui.spacing().interact_size.y;
    let gap = 2.0;
    let f = FontId::new(body_size(ui), egui::FontFamily::Proportional);
    let galleys: Vec<_> = items
        .iter()
        .map(|(_, name)| layout(ui, name, f.clone(), t.text))
        .collect();
    let avail = ui.available_width();
    let text_w: f32 = galleys.iter().map(|g| g.size().x).sum();
    let n = items.len().max(1) as f32;
    let pad = ((avail - text_w - gap * (n - 1.0)) / (2.0 * n)).clamp(0.0, 11.0);

    let row_top = ui.cursor().min;
    let row = Rect::from_min_size(row_top, Vec2::new(avail, h));
    let mut responses = Vec::with_capacity(items.len());
    let mut x = row.left();
    for ((value, name), g) in items.iter().zip(&galleys) {
        let rect = Rect::from_min_size(
            Pos2::new(x, row.top()),
            Vec2::new(g.size().x + 2.0 * pad, h),
        );
        x = rect.right() + gap;
        let on = *selected == *value;
        let mut r = ui.interact(rect, ui.scope_id().with(("tab", *name)), Sense::click());
        let label = (*name).to_owned();
        r.widget_info(|| WidgetInfo::selected(Role::Tab, true, on, &label));
        if r.clicked() && !on {
            *selected = *value;
            r.mark_changed();
        }
        responses.push((r, rect, on));
    }
    ui.advance_cursor_after_rect(row);

    let p = ui.painter();
    // The row's border runs under the whole window, as the lab's `.tabs` does.
    p.hline(
        ui.clip_rect().x_range(),
        row.bottom() - border / 2.0,
        Stroke::new(border, t.line),
    );
    for ((r, rect, on), g) in responses.iter().zip(&galleys) {
        let on = *on;
        let color = if on || r.hovered() || r.has_focus() {
            t.text
        } else {
            t.muted
        };
        let pos = rect.center() - g.size() / 2.0 - Vec2::new(0.0, 1.5);
        p.galley_with_override_text_color(pos, g.clone(), color);
        if on {
            p.rect_filled(
                Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - 3.0), rect.max),
                0.0,
                t.text,
            );
        }
        focus_ring(ui, r, rect.shrink(2.0), 4.0);
    }

    let focused = responses.iter().position(|(r, _, _)| r.has_focus());
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
            responses[j].0.request_focus();
        }
    }
}

// ---------------------------------------------------------------- segmented

/// Mutually exclusive options in one full-width frame of equal parts (lab `.seg`): a
/// `--line-strong` border and dividers, `--surface-2` parts, the selected one filled with
/// `--text` and its label in `--bg`. Radio semantics.
pub fn segmented<T: Copy + PartialEq>(ui: &mut Ui, value: &mut T, items: &[(T, &str)]) {
    let t = theme::current(ui);
    let border = theme::metrics(theme::is_larger(ui)).border_w;
    let h = ui.spacing().interact_size.y;
    let (outer, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::hover());
    let n = items.len().max(1);
    let w = outer.width() / n as f32;
    let f = FontId::new(body_size(ui), egui::FontFamily::Proportional);
    let p = ui.painter().clone();
    p.rect_filled(outer, RADIUS_SM, t.surface_2);
    for (i, (v, name)) in items.iter().enumerate() {
        let rect = Rect::from_min_size(
            Pos2::new(outer.left() + w * i as f32, outer.top()),
            Vec2::new(w, h),
        );
        let on = *value == *v;
        let mut r = ui.interact(rect, ui.scope_id().with(("seg", *name)), Sense::click());
        let label = (*name).to_owned();
        r.widget_info(|| WidgetInfo::selected(Role::RadioButton, ui.is_enabled(), on, &label));
        if r.clicked() && !on {
            *value = *v;
            r.mark_changed();
        }
        if on {
            let cr = egui::CornerRadius {
                nw: if i == 0 { RADIUS_SM as u8 } else { 0 },
                sw: if i == 0 { RADIUS_SM as u8 } else { 0 },
                ne: if i == n - 1 { RADIUS_SM as u8 } else { 0 },
                se: if i == n - 1 { RADIUS_SM as u8 } else { 0 },
            };
            p.rect_filled(rect, cr, t.text);
        }
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            *name,
            f.clone(),
            if on { t.bg } else { t.text },
        );
        if i > 0 {
            p.vline(
                rect.left(),
                outer.y_range(),
                Stroke::new(border, t.line_strong),
            );
        }
        focus_ring(
            ui,
            &r,
            rect,
            if i == 0 || i == n - 1 { RADIUS_SM } else { 0.0 },
        );
    }
    p.rect_stroke(
        outer,
        RADIUS_SM,
        Stroke::new(border, t.line_strong),
        StrokeKind::Inside,
    );
}

// ---------------------------------------------------------------- buttons

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Yellow fill, black text (`theme::primary_colors`).
    Primary,
    /// `--surface-2` fill, `--line-strong` border, `--text` border on hover.
    Secondary,
}

/// A lab `.btn`: semibold label, optional key as a `kbd` chip (mono 0.76 em, 1 px border in
/// the text colour, 75 % opacity), at least `--hit` tall, 14 px side padding. `small` is the
/// lab's `.btn.small` (0.85 × `--hit`, 11 px padding, 0.92 em) used in status and cards.
/// `width` fixes the width (`None` = fit the content). The AccessKit name is the label, then
/// the key.
#[derive(Debug, Clone, Copy)]
pub struct Button<'a> {
    pub label: &'a str,
    pub key: Option<&'a str>,
    pub kind: Kind,
    pub small: bool,
    pub width: Option<f32>,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str, kind: Kind) -> Self {
        Self {
            label,
            key: None,
            kind,
            small: false,
            width: None,
        }
    }
    pub fn primary(label: &'a str) -> Self {
        Self::new(label, Kind::Primary)
    }
    pub fn secondary(label: &'a str) -> Self {
        Self::new(label, Kind::Secondary)
    }
    pub fn key(mut self, key: &'a str) -> Self {
        self.key = Some(key);
        self
    }
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    /// The accessible name: "Start  F10" (the same text the D6 buttons showed).
    pub fn accessible_name(&self) -> String {
        match self.key {
            Some(k) => format!("{}  {k}", self.label),
            None => self.label.to_owned(),
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let t = theme::current(ui);
        let m = theme::metrics(theme::is_larger(ui));
        let (fill, fg) = match self.kind {
            Kind::Primary => theme::primary_colors(ui),
            Kind::Secondary => (t.surface_2, t.text),
        };
        let size = body_size(ui) * if self.small { 0.92 } else { 1.0 };
        let pad = if self.small { 11.0 } else { 14.0 };
        let h = m.hit * if self.small { 0.85 } else { 1.0 };
        let label = layout(
            ui,
            self.label,
            FontId::new(size, egui::FontFamily::Proportional),
            fg,
        );
        let chip_font = FontId::monospace(size * 0.76);
        let chip = self.key.map(|k| layout(ui, k, chip_font, fg));
        let chip_pad = Vec2::new(5.0, 1.0);
        let chip_size = chip
            .as_ref()
            .map(|g| g.size() + 2.0 * chip_pad + Vec2::splat(2.0));
        let gap = 8.0;
        let content_w = label.size().x + chip_size.map_or(0.0, |c| gap + c.x);
        let w = self.width.unwrap_or(content_w + 2.0 * pad);
        let (rect, r) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
        let name = self.accessible_name();
        r.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &name));
        if !ui.is_rect_visible(rect) {
            return r;
        }
        let hovered = r.hovered() && ui.is_enabled();
        let (fill, line) = match self.kind {
            Kind::Primary if hovered => (fill.gamma_multiply(0.9), fill.gamma_multiply(0.9)),
            Kind::Primary => (fill, fill),
            Kind::Secondary if hovered => (fill, t.text),
            Kind::Secondary => (fill, t.line_strong),
        };
        let p = ui.painter();
        p.rect(
            rect,
            RADIUS_SM,
            fill,
            Stroke::new(m.border_w, line),
            StrokeKind::Inside,
        );
        let mut x = rect.center().x - content_w / 2.0;
        let mid = rect.center().y;
        p.galley(Pos2::new(x, mid - label.size().y / 2.0), label.clone(), fg);
        // Faux semibold: the default font has no bold face; a second pass ½ px right thickens
        // the strokes (font decision still open, task 0008 D6b).
        p.galley(
            Pos2::new(x + 0.5, mid - label.size().y / 2.0),
            label.clone(),
            fg,
        );
        x += label.size().x;
        if let (Some(g), Some(cs)) = (chip, chip_size) {
            let c = Rect::from_min_size(Pos2::new(x + gap, mid - cs.y / 2.0), cs);
            let col = fg.gamma_multiply(0.75);
            p.rect_stroke(c, 4.0, Stroke::new(1.0, col), StrokeKind::Inside);
            p.galley_with_override_text_color(c.center() - g.size() / 2.0, g, col);
        }
        focus_ring(ui, &r, rect, RADIUS_SM);
        r
    }
}

// ---------------------------------------------------------------- fields

/// A lab `.readout`: `--surface-2`, `--line` border, muted monospace text, 8 × 10 padding.
pub fn readout(ui: &mut Ui, text: &str) {
    let t = theme::current(ui);
    let m = theme::metrics(theme::is_larger(ui));
    egui::Frame::new()
        .fill(t.surface_2)
        .stroke(Stroke::new(m.border_w, t.line))
        .corner_radius(RADIUS_SM)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(text).monospace().color(t.muted));
        });
}
