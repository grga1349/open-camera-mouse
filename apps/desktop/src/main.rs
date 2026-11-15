//! Desktop shell: renders `app-core` state and sends `app-core` commands. Nothing else.
//!
//! Two fixed-size windows per the approved design lab (task 0007): Tracking (root viewport)
//! and Options (`show_viewport_immediate`). The runtime loop that runs the camera and moves
//! the pointer is task 0008 D7; until then `Effect`s from the controller aren't executed.

mod app;
mod icons;
mod options;
mod preview;
mod status;
mod theme;
mod widgets;

use app::{Shell, options_ui, options_viewport, tracking_ui, tracking_viewport};
use app_core::Settings;
use eframe::egui;
use platform::hotkeys::{GlobalHotkeys, Hotkey, Source};
use status::Action;

fn main() -> eframe::Result {
    let (settings, notice, store) = load_settings();
    let options = eframe::NativeOptions {
        viewport: tracking_viewport(),
        ..Default::default()
    };
    eframe::run_native(
        "Open Camera Mouse",
        options,
        Box::new(|cc| {
            // Here, on the event-loop (main) thread, as macOS and Windows require.
            let ctx = cc.egui_ctx.clone();
            let hotkeys = GlobalHotkeys::register(move || ctx.request_repaint());
            let lines = Hotkey::ALL.map(|k| app::hotkey_line(k, hotkeys.status(k)));
            let all_ok = Hotkey::ALL.iter().all(|k| hotkeys.status(*k).is_active());
            let mut shell = Shell::new(settings, notice, lines.to_vec(), all_ok, store);
            dev_open_options(&mut shell);
            Ok(Box::new(DesktopApp { shell, hotkeys }))
        }),
    )
}

/// Debug builds only, for side-by-side design checks against the lab without clicking
/// (task 0008 D6b): `OCM_DEV_OPTIONS=pointer|clicking|camera|general` opens Options at that
/// tab on start.
fn dev_open_options(shell: &mut Shell) {
    if !cfg!(debug_assertions) {
        return;
    }
    let tab = match std::env::var("OCM_DEV_OPTIONS").as_deref() {
        Ok("pointer") => options::Tab::Pointer,
        Ok("clicking") => options::Tab::Clicking,
        Ok("camera") => options::Tab::Camera,
        Ok("general") => options::Tab::General,
        _ => return,
    };
    shell.open_options(Some(tab));
}

/// Settings from the user config dir. Never fails: problems become defaults plus a notice.
/// Debug builds: `OCM_DEV_SETTINGS=<file>` uses that file instead, so a design check doesn't
/// touch the real settings.
fn load_settings() -> (Settings, Option<String>, Option<platform::SettingsStore>) {
    let dev = std::env::var_os("OCM_DEV_SETTINGS")
        .filter(|_| cfg!(debug_assertions))
        .map(platform::SettingsStore::at);
    match dev.or_else(platform::SettingsStore::in_user_config_dir) {
        Some(mut store) => {
            let loaded = store.load();
            (
                loaded.settings,
                loaded.notice.map(|n| n.to_string()),
                Some(store),
            )
        }
        None => (
            Settings::default(),
            Some("No home folder was found, so settings can't be saved.".to_owned()),
            None,
        ),
    }
}

#[derive(Debug)]
struct DesktopApp {
    shell: Shell,
    /// Kept alive for the app's lifetime; dropping it unregisters the hotkeys.
    hotkeys: GlobalHotkeys,
}

impl DesktopApp {
    /// Requests only the OS can serve.
    fn platform_actions(&mut self) {
        for a in std::mem::take(&mut self.shell.platform_actions) {
            match a {
                Action::OpenPointerSettings => {
                    if let Err(e) = platform::SystemPointer::open_access_settings() {
                        self.shell.notice = Some(format!("Couldn't open the settings: {e}."));
                    }
                }
                Action::CheckPointerAccess => {
                    // A one-off check; the D7 runtime also polls every few seconds.
                    let access = match platform::SystemPointer::new() {
                        Ok(p) => app_core::ports::PointerSink::access(&p),
                        Err(_) => app_core::PointerAccess::Unsupported,
                    };
                    self.shell.pointer_access(access);
                }
                Action::Start | Action::ChooseCamera | Action::CopyDetails => {}
            }
        }
        if self.shell.wants_camera_list {
            self.shell.wants_camera_list = false;
            self.shell.cameras = platform::list_devices();
        }
    }
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        for key in self.hotkeys.drain() {
            self.shell.hotkey(key, Source::Global);
        }
        self.shell.apply_style(&ctx);
        let margin = egui::Frame::central_panel(&ctx.global_style()).inner_margin(16);
        egui::CentralPanel::default()
            .frame(margin)
            .show(ui, |ui| tracking_ui(ui, &mut self.shell));

        if self.shell.options_open {
            let next_to = ctx.input(|i| i.viewport().outer_rect);
            let shell = &mut self.shell;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("options"),
                options_viewport(next_to),
                |ui, _class| {
                    if ui.input(|i| i.viewport().close_requested()) {
                        shell.options_open = false;
                        shell.save_if_due(true);
                    }
                    // No margin: the Options layout runs its tab rule and footer edge to edge.
                    let frame =
                        egui::Frame::central_panel(&ui.ctx().global_style()).inner_margin(0);
                    egui::CentralPanel::default()
                        .frame(frame)
                        .show(ui, |ui| options_ui(ui, shell));
                },
            );
        }
        self.platform_actions();

        self.shell.save_if_due(false);
        if self.shell.save_pending() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        // Countdowns (recenter, handover) are controller deadlines; keep the text moving.
        let state = self.shell.controller.state();
        if state == app_core::AppState::Recentering
            || self.shell.controller.pointer_owner() == app_core::PointerOwner::Handover
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

impl Drop for DesktopApp {
    /// A change made just before quitting is still written.
    fn drop(&mut self) {
        self.shell.save_if_due(true);
    }
}
