//! The Options window (design lab, 400 × 620, fixed): Pointer / Clicking / Camera / General
//! tabs, Restore defaults, Close. It edits a copy of the settings; the caller validates the
//! result through `app-core` and saves it (R-A5). Changes apply immediately.

use app_core::ports::CaptureFormat;
use app_core::settings::ranges;
use app_core::{DesktopRect, MainMonitor, Settings, Theme};
use eframe::egui::{self, Align, Layout, RichText, ScrollArea, Ui};
use platform::CameraDevice;

use crate::theme;
use crate::widgets::{Button, field_label, help, readout, segmented, slider_row, switch, tabs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Pointer,
    Clicking,
    Camera,
    Screen,
    General,
}

const TABS: [(Tab, &str); 5] = [
    (Tab::Pointer, "Pointer"),
    (Tab::Clicking, "Clicking"),
    (Tab::Camera, "Camera"),
    (Tab::Screen, "Screen"),
    (Tab::General, "General"),
];

/// Read-only facts the Options window shows.
#[derive(Debug, Clone, Copy)]
pub struct Facts<'a> {
    pub cameras: &'a [CameraDevice],
    pub capture: Option<CaptureFormat>,
    /// One line per hotkey: key, action and whether it works in other apps (task 0008 D5).
    pub hotkey_lines: &'a [String],
    /// The displays in "Main monitor" order (left to right), from the controller.
    pub displays: &'a [DesktopRect],
    /// Index of the OS primary display in `displays`, if known (#14).
    pub primary: Option<usize>,
}

/// "Restore defaults": behaviour back to the defaults; the chosen camera and the
/// appearance (theme, larger controls) stay, they aren't tuning.
pub fn restored(current: &Settings) -> Settings {
    Settings {
        camera: current.camera.clone(),
        theme: current.theme,
        larger_controls: current.larger_controls,
        ..Settings::default()
    }
}

/// Draws the window contents edge to edge (the caller's panel has no margin): the tab row,
/// the scrolling body with the lab's 16 px padding (so the scroll bar sits in the margin, not
/// over the values) and the footer bar. Returns `true` when Close was pressed.
pub fn show(ui: &mut Ui, s: &mut Settings, tab: &mut Tab, facts: &Facts<'_>) -> bool {
    let t = theme::current(ui);
    let m = theme::metrics(theme::is_larger(ui));
    // Lab `.tabs`: padding 6 10 0, the row's border runs the full width.
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 10,
            right: 10,
            top: 6,
            bottom: 0,
        })
        .show(ui, |ui| tabs(ui, tab, &TABS));
    ui.add_space(-ui.spacing().item_spacing.y);

    let footer_h = m.hit + 20.0 + m.border_w;
    let body_h = (ui.available_height() - footer_h).max(80.0);
    ScrollArea::vertical()
        .max_height(body_h)
        .min_scrolled_height(body_h)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new().inner_margin(16).show(ui, |ui| {
                ui.set_width(ui.available_width());
                // Lab `.panel { gap: 16px }` between fields.
                ui.spacing_mut().item_spacing.y = 8.0;
                match tab {
                    Tab::Pointer => pointer(ui, s),
                    Tab::Clicking => clicking(ui, s),
                    Tab::Camera => camera(ui, s, facts),
                    Tab::Screen => screen(ui, s, facts),
                    Tab::General => general(ui, s, facts),
                }
            });
        });

    // Lab `.ow-foot`: `--surface`, a top border, padding 10 16, buttons at both ends.
    ui.add_space(-ui.spacing().item_spacing.y);
    let mut close = false;
    let foot = egui::Frame::new()
        .fill(t.surface)
        .inner_margin(egui::Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if Button::secondary("Restore defaults").show(ui).clicked() {
                    *s = restored(s);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    close = Button::primary("Close").show(ui).clicked();
                });
            });
        });
    let r = foot.response.rect;
    ui.painter().hline(
        r.x_range(),
        r.top() + m.border_w / 2.0,
        egui::Stroke::new(m.border_w, t.line),
    );
    close
}

/// The rest of the lab's 16 px gap between fields (item spacing gives the other 8).
fn space(ui: &mut Ui) {
    ui.add_space(8.0);
}

fn pointer(ui: &mut Ui, s: &mut Settings) {
    let before = (s.gain_x, s.gain_y);
    slider_row(
        ui,
        "Horizontal speed",
        &mut s.gain_x,
        ranges::GAIN,
        1.0,
        |v| format!("{v:.0}"),
        Some("How far the pointer moves when you turn your head. Default 20"),
    );
    space(ui);
    slider_row(
        ui,
        "Vertical speed",
        &mut s.gain_y,
        ranges::GAIN,
        1.0,
        |v| format!("{v:.0}"),
        None,
    );
    let was_linked = s.link_gain;
    switch(ui, &mut s.link_gain, "Keep both speeds the same", None);
    if s.link_gain {
        if s.gain_y != before.1 {
            s.gain_x = s.gain_y;
        } else if s.gain_x != before.0 || !was_linked {
            s.gain_y = s.gain_x;
        }
    }
    space(ui);
    slider_row(
        ui,
        "Smoothing",
        &mut s.smoothing,
        ranges::SMOOTHING,
        0.01,
        |v| format!("{v:.2}"),
        Some("Higher is steadier but slower to follow. Default 0.20"),
    );
}

/// Where the pointer may go and where it starts (test drive #9): screen limits and, with
/// several monitors, the Main monitor.
fn screen(ui: &mut Ui, s: &mut Settings, facts: &Facts<'_>) {
    ui.horizontal(|ui| {
        field_label(ui, "Screen limits");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new("px").monospace());
        });
    });
    // Lab `.limits`: four equal columns, a small muted label over each number field.
    let gap = 8.0;
    let col_w = (ui.available_width() - 3.0 * gap) / 4.0;
    let h = ui.spacing().interact_size.y;
    let muted = theme::current(ui).muted;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        let lim = &mut s.limits;
        for (name, v) in [
            ("Left", &mut lim.left),
            ("Right", &mut lim.right),
            ("Top", &mut lim.top),
            ("Bottom", &mut lim.bottom),
        ] {
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.spacing_mut().item_spacing.y = 3.0;
                let l = ui.label(RichText::new(name).small().color(muted));
                ui.scope(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    ui.add_sized([col_w, h], egui::DragValue::new(v).range(ranges::LIMIT_PX))
                        .labelled_by(l.id);
                });
            });
        }
    });
    help(
        ui,
        "Keep the pointer this far from each screen edge. Default 0",
    );
    // Only with several monitors: with one there is nothing to choose.
    if facts.displays.len() > 1 {
        space(ui);
        main_monitor(ui, s, facts.displays, facts.primary);
    }
}

/// One monitor's name in "Main monitor": its number left to right, size, and "primary" for
/// the OS primary display (macOS main display, Windows primary monitor, X11 RandR primary).
pub fn display_name(i: usize, d: &DesktopRect, primary: Option<usize>) -> String {
    let primary = if primary == Some(i) {
        " · primary"
    } else {
        ""
    };
    format!(
        "Monitor {} · {} × {}{primary}",
        i + 1,
        d.width.round(),
        d.height.round()
    )
}

/// "Main monitor" (test drive #4, ADR-0014): the monitor recenter and pause put the cursor
/// on. Shown only with more than one monitor.
fn main_monitor(ui: &mut Ui, s: &mut Settings, displays: &[DesktopRect], primary: Option<usize>) {
    const PRIMARY: &str = "Primary monitor";
    const ALL: &str = "Centre of all monitors";
    let l = field_label(ui, "Main monitor");
    let current = match s.recenter_display {
        MainMonitor::Primary => PRIMARY.to_owned(),
        MainMonitor::All => ALL.to_owned(),
        MainMonitor::Display(i) => displays
            .get(i as usize)
            .map(|d| display_name(i as usize, d, primary))
            .unwrap_or_else(|| format!("Monitor {} (not connected)", i + 1)),
    };
    let h = ui.spacing().interact_size.y;
    egui::ComboBox::from_id_salt("main_monitor")
        .selected_text(current)
        .width(ui.available_width())
        .height(h * 7.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut s.recenter_display, MainMonitor::Primary, PRIMARY);
            ui.selectable_value(&mut s.recenter_display, MainMonitor::All, ALL);
            for (i, d) in displays.iter().enumerate() {
                ui.selectable_value(
                    &mut s.recenter_display,
                    MainMonitor::Display(i as u32),
                    display_name(i, d, primary),
                );
            }
        })
        .response
        .labelled_by(l.id);
    help(
        ui,
        "Recenter and pause put the pointer in the middle of this monitor. Monitors are \
         numbered left to right. The pointer can still reach every monitor.",
    );
}

fn clicking(ui: &mut Ui, s: &mut Settings) {
    let d = &mut s.dwell;
    switch(
        ui,
        &mut d.enabled,
        "Dwell to click",
        Some("Hold the pointer still to click"),
    );
    space(ui);
    ui.add_enabled_ui(d.enabled, |ui| {
        slider_row(
            ui,
            "Dwell time",
            &mut d.dwell_s,
            ranges::DWELL_S,
            0.05,
            |v| format!("{v:.2} s"),
            None,
        );
        space(ui);
        slider_row(
            ui,
            "Pause after a click",
            &mut d.refractory_s,
            ranges::REFRACTORY_S,
            0.1,
            |v| format!("{v:.1} s"),
            Some("No second click until you move away or this time passes."),
        );
        space(ui);
        let mut pct = (d.radius_frac * 100.0).round();
        let r = slider_row(
            ui,
            "Hold-still area",
            &mut pct,
            (ranges::DWELL_RADIUS_FRAC.start() * 100.0)..=(ranges::DWELL_RADIUS_FRAC.end() * 100.0),
            1.0,
            |v| format!("{v:.0} %"),
            Some("How far the pointer may wander and still count as still (share of screen)."),
        );
        if r.changed() {
            d.radius_frac = pct / 100.0;
        }
    });
}

fn camera(ui: &mut Ui, s: &mut Settings, facts: &Facts<'_>) {
    let l = field_label(ui, "Camera");
    let current = match &s.camera {
        None => "System default".to_owned(),
        Some(id) => facts
            .cameras
            .iter()
            .find(|c| &c.id == id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("{id} (not connected)")),
    };
    let width = ui.available_width();
    let h = ui.spacing().interact_size.y;
    egui::ComboBox::from_id_salt("camera")
        .selected_text(current)
        .width(width)
        .height(h * 6.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut s.camera, None, "System default");
            for c in facts.cameras {
                ui.selectable_value(&mut s.camera, Some(c.id.clone()), &c.name);
            }
        })
        .response
        .labelled_by(l.id);
    space(ui);
    switch(
        ui,
        &mut s.mirror,
        "Mirror preview",
        Some("Head right moves the pointer right"),
    );
    space(ui);
    field_label(ui, "Capture format");
    let format = match facts.capture {
        Some(f) => format!("{} × {} · {:.0} fps", f.width, f.height, f.fps),
        None => "Camera not started".to_owned(),
    };
    readout(ui, &format);
    space(ui);
    switch(
        ui,
        &mut s.loss_recovery,
        "Find the target again",
        Some(
            "If tracking is lost, look for the selected point and continue by itself. Off: \
             click your face again or recenter.",
        ),
    );
}

fn general(ui: &mut Ui, s: &mut Settings, facts: &Facts<'_>) {
    field_label(ui, "Appearance");
    segmented(
        ui,
        &mut s.theme,
        &[(Theme::Carbon, "Dark"), (Theme::Paper, "Light")],
    );
    space(ui);
    switch(
        ui,
        &mut s.larger_controls,
        "Larger controls",
        Some("Bigger buttons, switches and focus rings"),
    );
    space(ui);
    field_label(ui, "Keyboard");
    for line in facts.hotkey_lines {
        help(ui, line);
    }
    space(ui);
    field_label(ui, "Mouse takeover");
    help(
        ui,
        "Moving the physical mouse gives it control. The camera takes over again after the \
         mouse is still for 4 s.",
    );
    space(ui);
    switch(
        ui,
        &mut s.show_diagnostics,
        "Show diagnostics",
        Some("Frame rate, tracker score and status on the preview"),
    );
    space(ui);
    // From the crate, so the release version bump updates it.
    help(ui, concat!("Version ", env!("CARGO_PKG_VERSION")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_defaults_keeps_camera_and_appearance() {
        let s = Settings {
            gain_x: 12.0,
            camera: Some("cam-2".into()),
            theme: Theme::Paper,
            larger_controls: true,
            loss_recovery: false,
            ..Settings::default()
        };
        let r = restored(&s);
        assert_eq!(r.gain_x, 20.0);
        // How a v0.4.0 file saved with it off gets the v0.4.1 default (D3).
        assert!(r.loss_recovery);
        assert_eq!(r.camera.as_deref(), Some("cam-2"));
        assert_eq!(r.theme, Theme::Paper);
        assert!(r.larger_controls);
    }
}
