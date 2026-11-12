//! Desktop shell: renders `app-core` state and sends `app-core` commands. Nothing else.
//! Phase 2 skeleton: one window, a status line and a start/stop button. The real UI and
//! runtime are task 0008 slices D6/D7.

use std::time::Instant;

use app_core::{Command, Controller, DesktopRect, Input, Settings};
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Open Camera Mouse",
        options,
        Box::new(|_cc| Ok(Box::new(DesktopApp::default()))),
    )
}

#[derive(Debug)]
struct DesktopApp {
    shell: Shell,
}

impl Default for DesktopApp {
    fn default() -> Self {
        Self {
            shell: Shell::new(),
        }
    }
}

/// The controller plus the one monotonic clock it is driven with.
#[derive(Debug)]
struct Shell {
    controller: Controller,
    epoch: Instant,
}

impl Shell {
    fn new() -> Self {
        // Placeholder desktop until the pointer adapter reports the real one (D4).
        let desktop = DesktopRect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        Self {
            controller: Controller::new(Settings::default(), desktop)
                .expect("default settings are valid"),
            epoch: Instant::now(),
        }
    }

    fn command(&mut self, c: Command) {
        let now_us = self.epoch.elapsed().as_micros() as u64;
        self.controller.handle(Input::Command(c), now_us);
    }
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| shell_ui(ui, &mut self.shell));
    }
}

/// The whole UI, separated from eframe so tests can drive it headlessly.
fn shell_ui(ui: &mut egui::Ui, shell: &mut Shell) {
    ui.heading("Open Camera Mouse");
    ui.label(format!("State: {:?}", shell.controller.state()));
    ui.label(format!("Platform: {}", platform::backend_name()));
    if ui.button("Start / Stop (F11)").clicked() {
        shell.command(Command::ToggleStart);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn start_button_changes_rendered_state() {
        let mut harness =
            Harness::new_ui_state(|ui, shell: &mut Shell| shell_ui(ui, shell), Shell::new());
        harness.get_by_label("State: Stopped");
        harness.get_by_label("Start / Stop (F11)").click();
        harness.run();
        harness.get_by_label("State: Starting");
    }
}
