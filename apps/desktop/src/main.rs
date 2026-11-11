//! Desktop shell: renders `app-core` state and sends `app-core` commands. Nothing else.
//! Phase 2 skeleton: one window, a status line and a start/stop button.

use app_core::{Command, Controller};
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Open Camera Mouse",
        options,
        Box::new(|_cc| Ok(Box::new(DesktopApp::default()))),
    )
}

#[derive(Debug, Default)]
struct DesktopApp {
    controller: Controller,
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| shell_ui(ui, &mut self.controller));
    }
}

/// The whole UI, separated from eframe so tests can drive it headlessly.
fn shell_ui(ui: &mut egui::Ui, controller: &mut Controller) {
    ui.heading("Open Camera Mouse");
    ui.label(format!("State: {:?}", controller.state()));
    ui.label(format!("Platform: {}", platform::backend_name()));
    if ui.button("Start / Stop (F11)").clicked() {
        controller.handle(Command::ToggleStart);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn start_button_changes_rendered_state() {
        let mut harness = Harness::new_ui_state(
            |ui, controller: &mut Controller| shell_ui(ui, controller),
            Controller::new(),
        );
        harness.get_by_label("State: Stopped");
        harness.get_by_label("Start / Stop (F11)").click();
        harness.run();
        harness.get_by_label("State: Starting");
    }
}
