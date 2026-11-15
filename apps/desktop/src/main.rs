//! Desktop shell: renders `app-core` state and sends `app-core` commands. Nothing else.
//! Skeleton: one window, a status line, a start/stop button, the settings-load notice and the
//! hotkey status (start / pause and recenter keys, global or window-only). The real UI and runtime are task 0008
//! slices D6/D7; saving on change comes with the Options window (D6).

use std::time::Instant;

use app_core::{Command, Controller, DesktopRect, Input, Settings};
use eframe::egui;
use platform::hotkeys::{GlobalHotkeys, Hotkey, KeyRouter, Registration, Source};

fn main() -> eframe::Result {
    let (settings, notice) = load_settings();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Open Camera Mouse",
        options,
        Box::new(|cc| {
            // Here, on the event-loop (main) thread, as macOS and Windows require.
            let ctx = cc.egui_ctx.clone();
            let hotkeys = GlobalHotkeys::register(move || ctx.request_repaint());
            let lines = Hotkey::ALL.map(|k| hotkey_line(k, hotkeys.status(k)));
            Ok(Box::new(DesktopApp {
                shell: Shell::new(settings, notice, lines.to_vec()),
                hotkeys,
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
    /// Kept alive for the app's lifetime; dropping it unregisters the hotkeys.
    hotkeys: GlobalHotkeys,
}

/// One line per hotkey: what it does and whether it works outside this window (P13).
fn hotkey_line(key: Hotkey, status: &Registration) -> String {
    let action = match key {
        Hotkey::ToggleStart => "start / pause",
        Hotkey::Recenter => "recenter",
    };
    let name = key.key_name();
    match status {
        Registration::Active => format!("{name}: {action}, works in any app."),
        Registration::Failed(why) => {
            format!("{name}: {action}, works only in this window: {why}.")
        }
        Registration::Unsupported(why) => {
            format!("{name}: {action}, works only in this window ({why}).")
        }
    }
}

/// The controller plus the one monotonic clock it is driven with.
#[derive(Debug)]
struct Shell {
    controller: Controller,
    epoch: Instant,
    /// Shown once at the top of the window (settings-load fallback).
    notice: Option<String>,
    /// Merges global and in-window hotkey presses so one press is one command.
    keys: KeyRouter,
    /// Hotkey status lines, from registration at startup.
    hotkey_lines: Vec<String>,
}

impl Shell {
    fn new(settings: Settings, notice: Option<String>, hotkey_lines: Vec<String>) -> Self {
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
            keys: KeyRouter::new(),
            hotkey_lines,
        }
    }

    fn now_us(&self) -> u64 {
        self.epoch.elapsed().as_micros() as u64
    }

    fn hotkey(&mut self, key: Hotkey, source: Source) {
        if let Some(c) = self.keys.press(key, source, self.now_us()) {
            self.command(c);
        }
    }

    fn command(&mut self, c: Command) {
        let now_us = self.now_us();
        self.controller.handle(Input::Command(c), now_us);
    }
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        for key in self.hotkeys.drain() {
            self.shell.hotkey(key, Source::Global);
        }
        egui::CentralPanel::default().show(ui, |ui| shell_ui(ui, &mut self.shell));
    }
}

/// The whole UI, separated from eframe so tests can drive it headlessly.
fn shell_ui(ui: &mut egui::Ui, shell: &mut Shell) {
    for key in window_hotkeys(ui) {
        shell.hotkey(key, Source::Window);
    }
    ui.heading("Open Camera Mouse");
    if let Some(n) = &shell.notice {
        ui.label(n);
    }
    ui.label(format!("State: {:?}", shell.controller.state()));
    ui.label(format!("Platform: {}", platform::backend_name()));
    let start_label = format!("Start / Stop ({})", Hotkey::ToggleStart.key_name());
    if ui.button(start_label).clicked() {
        shell.command(Command::ToggleStart);
    }
    for line in &shell.hotkey_lines {
        ui.label(line);
    }
}

/// Hotkey presses this window received (focused). Repeats are ignored: one press, one command.
fn window_hotkeys(ui: &egui::Ui) -> Vec<Hotkey> {
    ui.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } => Hotkey::from_key_name(key.name()),
                _ => None,
            })
            .collect()
    })
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
        harness
            .get_by_label(&format!(
                "Start / Stop ({})",
                Hotkey::ToggleStart.key_name()
            ))
            .click();
        harness.run();
        harness.get_by_label("State: Starting");
    }

    /// The start / pause key on this OS (F10 on macOS, F11 elsewhere).
    fn start_key() -> egui::Key {
        egui::Key::from_name(Hotkey::ToggleStart.key_name()).expect("an egui key")
    }

    fn test_shell(notice: Option<String>) -> Shell {
        Shell::new(Settings::default(), notice, Vec::new())
    }

    #[test]
    fn in_window_start_key_toggles_start_once() {
        let mut harness = Harness::new_ui_state(
            |ui, shell: &mut Shell| shell_ui(ui, shell),
            test_shell(None),
        );
        harness.key_press(start_key());
        harness.run();
        harness.get_by_label("State: Starting");
    }

    #[test]
    fn in_window_start_key_after_a_global_press_is_the_same_press() {
        let mut shell = test_shell(None);
        shell.hotkey(Hotkey::ToggleStart, Source::Global);
        let mut harness = Harness::new_ui_state(|ui, shell: &mut Shell| shell_ui(ui, shell), shell);
        harness.get_by_label("State: Starting");
        harness.key_press(start_key());
        harness.run();
        // Not toggled back to Stopped / a second command.
        harness.get_by_label("State: Starting");
    }

    #[test]
    fn a_refused_key_keeps_the_reason_as_written() {
        let line = hotkey_line(
            Hotkey::Recenter,
            &Registration::Failed("another app or a system shortcut already uses F12".into()),
        );
        assert_eq!(
            line,
            "F12: recenter, works only in this window: another app or a system shortcut already uses F12."
        );
    }

    #[test]
    fn a_reason_starting_with_macos_is_not_recapitalised() {
        let line = hotkey_line(
            Hotkey::ToggleStart,
            &Registration::Failed("macOS uses this key for something else".into()),
        );
        assert!(line.contains(": macOS uses"), "{line}");
    }

    #[test]
    fn hotkey_status_lines_are_shown() {
        let lines = vec![
            hotkey_line(Hotkey::ToggleStart, &Registration::Active),
            hotkey_line(
                Hotkey::Recenter,
                &Registration::Unsupported("global hotkeys aren't available on Wayland".into()),
            ),
        ];
        let harness = Harness::new_ui_state(
            |ui, shell: &mut Shell| shell_ui(ui, shell),
            Shell::new(Settings::default(), None, lines),
        );
        harness.get_by_label(&format!(
            "{}: start / pause, works in any app.",
            Hotkey::ToggleStart.key_name()
        ));
        harness.get_by_label(
            "F12: recenter, works only in this window (global hotkeys aren't available on Wayland).",
        );
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
