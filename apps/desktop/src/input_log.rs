//! Pointer input as each window receives it, for diagnosing click and drag problems on a
//! device (v0.4.1: a physical-button click on an Options slider while the camera drives the
//! cursor set the gain to 60, macOS, 2026-11-25). Off unless `OCM_INPUT=1`; one stderr line per
//! pointer, button, focus or leave event, with the window it reached and the gain after the
//! pass.

use std::sync::OnceLock;
use std::time::Instant;

use eframe::egui;

/// `Some(start)` when `OCM_INPUT=1`.
fn start() -> Option<Instant> {
    static START: OnceLock<Option<Instant>> = OnceLock::new();
    *START.get_or_init(|| {
        let on = std::env::var("OCM_INPUT").is_ok_and(|v| v == "1");
        if on {
            eprintln!("OCM_INPUT: t ms, window, event, gain x/y after the pass");
        }
        on.then(Instant::now)
    })
}

/// Logs this pass's pointer events for `window` ("tracking", "options").
pub fn log(ctx: &egui::Context, window: &str, gain: (f32, f32)) {
    let Some(start) = start() else { return };
    let t = start.elapsed().as_secs_f64() * 1000.0;
    ctx.input(|i| {
        for e in &i.raw.events {
            if let Some(d) = describe(e) {
                eprintln!(
                    "OCM_INPUT {t:9.1} {window:8} {d}  gain {:.1}/{:.1}",
                    gain.0, gain.1
                );
            }
        }
    });
}

/// One line for the events that matter here; `None` for the rest (keys, text, …).
fn describe(e: &egui::Event) -> Option<String> {
    match e {
        egui::Event::PointerMoved(p) => Some(format!("moved {:.1},{:.1}", p.x, p.y)),
        egui::Event::PointerButton {
            pos,
            button,
            pressed,
            ..
        } => Some(format!(
            "{button:?} {} at {:.1},{:.1}",
            if *pressed { "down" } else { "up" },
            pos.x,
            pos.y
        )),
        egui::Event::PointerGone => Some("gone".into()),
        egui::Event::WindowFocused(f) => Some(format!("focused {f}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::describe;
    use eframe::egui::{Event, Modifiers, PointerButton, pos2};

    #[test]
    fn pointer_events_are_described_and_others_skipped() {
        assert_eq!(
            describe(&Event::PointerMoved(pos2(1.0, 2.5))).as_deref(),
            Some("moved 1.0,2.5")
        );
        let press = Event::PointerButton {
            pos: pos2(10.0, 20.0),
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        };
        assert_eq!(
            describe(&press).as_deref(),
            Some("Primary down at 10.0,20.0")
        );
        assert_eq!(describe(&Event::PointerGone).as_deref(), Some("gone"));
        assert_eq!(describe(&Event::Text("a".into())), None);
    }
}
