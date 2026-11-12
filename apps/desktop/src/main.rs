//! Desktop shell: renders `app-core` state and sends `app-core` commands. Nothing else.
//! Skeleton: one window, a status line, a start/stop button and the settings-load notice.
//! The real UI and runtime are task 0008 slices D6/D7; saving on change comes with the
//! Options window (D6).

use std::time::Instant;

use app_core::{Command, Controller, DesktopRect, Input, Settings};
use eframe::egui;

fn main() -> eframe::Result {
    let (settings, notice) = load_settings();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Open Camera Mouse",
        options,
        Box::new(|_cc| {
            Ok(Box::new(DesktopApp {
                shell: Shell::new(settings, notice),
            }))
        }),
    )
}

/// Settings from the user config dir. Never fails: problems become defaults plus a notice.
fn load_settings() -> (Settings, Option<String>) {
    match platform::SettingsStore::in_user_config_dir() {
        Some(mut store) => {
            let loaded = store.load();
            (loaded.settings, loaded.notice.map(|n| n.to_string()))
        }
        None => (
            Settings::default(),
            Some("No home folder was found, so settings can't be saved.".to_owned()),
        ),
    }
}

#[derive(Debug)]
struct DesktopApp {
    shell: Shell,
}

/// The controller plus the one monotonic clock it is driven with.
#[derive(Debug)]
struct Shell {
    controller: Controller,
    epoch: Instant,
    /// Shown once at the top of the window (settings-load fallback).
    notice: Option<String>,
}

impl Shell {
    fn new(settings: Settings, notice: Option<String>) -> Self {
        // Placeholder desktop until the pointer adapter reports the real one (D4).
        let desktop = DesktopRect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        Self {
            // Loaded settings are already repaired; fall back rather than panic regardless.
            controller: Controller::new(settings, desktop)
                .or_else(|_| Controller::new(Settings::default(), desktop))
                .expect("default settings are valid"),
            epoch: Instant::now(),
            notice,
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
    if let Some(n) = &shell.notice {
        ui.label(n);
    }
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
        let mut harness = Harness::new_ui_state(
            |ui, shell: &mut Shell| shell_ui(ui, shell),
            test_shell(None),
        );
        harness.get_by_label("State: Stopped");
        harness.get_by_label("Start / Stop (F11)").click();
        harness.run();
        harness.get_by_label("State: Starting");
    }

    fn test_shell(notice: Option<String>) -> Shell {
        Shell::new(Settings::default(), notice)
    }

    #[test]
    fn settings_notice_is_shown() {
        let text = "Your settings file was damaged, so defaults are in use.";
        let harness = Harness::new_ui_state(
            |ui, shell: &mut Shell| shell_ui(ui, shell),
            test_shell(Some(text.to_owned())),
        );
        harness.get_by_label(text);
    }
}
