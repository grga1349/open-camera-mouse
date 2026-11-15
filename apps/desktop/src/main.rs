//! Desktop shell: renders `app-core` state and sends `app-core` commands. Nothing else.
//!
//! Two fixed-size windows per the approved design lab (task 0007): Tracking (root viewport)
//! and Options (`show_viewport_immediate`). The camera, the controller and the pointer run on
//! the engine thread ([`engine::EngineThread`], ADR-0016), independent of repaints; the UI
//! sends commands and paints snapshots. Hotkeys are drained in `App::logic` (main thread).

mod app;
mod engine;
mod icons;
mod options;
mod perf;
mod preview;
mod recorder;
mod status;
#[cfg(test)]
mod test_frames;
mod theme;
mod widgets;

use std::time::{Duration, Instant};

use app::{Shell, options_ui, options_viewport, tracking_ui, tracking_viewport};
use app_core::Settings;
use app_core::ports::{CameraSource, PointerSink};
use eframe::egui;
use engine::{Clock, Engine, EngineThread, ThreadedCamera};
use perf::SharedPerf;
use platform::hotkeys::{GlobalHotkeys, Hotkey, Source};
use status::Action;

/// Idle repaint, for hotkeys and the settings save; the engine repaints on every change.
const IDLE_REPAINT: Duration = Duration::from_secs(1);
/// Repaint while a settings save waits for its debounce.
const SAVE_REPAINT: Duration = Duration::from_millis(100);

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
            let epoch = Instant::now();
            let perf = perf::Perf::from_env();
            let engine = start_engine(settings.clone(), epoch, perf.clone(), &cc.egui_ctx);
            let mut shell = Shell::new(
                settings,
                notice,
                lines.to_vec(),
                all_ok,
                store,
                Box::new(engine),
                Clock::System(epoch),
            );
            dev_open_options(&mut shell);
            Ok(Box::new(DesktopApp {
                shell,
                hotkeys,
                perf,
            }))
        }),
    )
}

/// The engine thread with the camera worker (timestamps on the engine's clock) and the
/// system pointer, both made on that thread.
fn start_engine(
    settings: Settings,
    epoch: Instant,
    perf: Option<SharedPerf>,
    ctx: &egui::Context,
) -> EngineThread {
    let make: engine::EngineFactory = Box::new(move |wake| {
        let factory: engine::CameraFactory = Box::new(move |device| {
            Box::new(platform::NokhwaCamera::new(device, epoch)) as Box<dyn CameraSource>
        });
        let camera = ThreadedCamera::spawn(factory, wake);
        let pointer = platform::SystemPointer::new().map(|p| Box::new(p) as Box<dyn PointerSink>);
        Engine::new(settings, Clock::System(epoch), camera, pointer)
            .with_recorder(recorder::Recorder::from_env())
            .with_perf(perf)
    });
    let ctx = ctx.clone();
    EngineThread::spawn(make, move || ctx.request_repaint())
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
    perf: Option<SharedPerf>,
}

impl DesktopApp {
    /// Requests only the OS can serve.
    fn platform_actions(&mut self) {
        for a in std::mem::take(&mut self.shell.platform_actions) {
            match a {
                Action::OpenPointerSettings => {
                    // macOS lists the app under Accessibility only after it asked once.
                    #[cfg(target_os = "macos")]
                    if let Ok(p) = platform::SystemPointer::new() {
                        let _ = p.request_access();
                    }
                    if let Err(e) = platform::SystemPointer::open_access_settings() {
                        self.shell.notice = Some(format!("Couldn't open the settings: {e}."));
                    }
                }
                Action::CheckPointerAccess => self.shell.check_access(),
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
    /// Runs before `ui`, and also while the windows are minimised or hidden: global hotkeys
    /// and the settings save. The engine thread does the rest.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for key in self.hotkeys.drain() {
            self.shell.hotkey(key, Source::Global);
        }
        self.platform_actions();
        self.shell.save_if_due(false);
        // The engine repaints on every change it publishes (frames, countdowns).
        ctx.request_repaint_after(if self.shell.save_pending() {
            SAVE_REPAINT
        } else {
            IDLE_REPAINT
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let started = Instant::now();
        self.draw(ui);
        if let Some(p) = &self.perf {
            let mut p = p.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(d) = self.shell.last_upload.take() {
                p.upload(d);
            }
            p.ui(started.elapsed());
        }
    }
}

impl DesktopApp {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
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
        // Clicks in this pass (Check again, …): serve them now, not a frame later.
        self.platform_actions();
    }
}

impl Drop for DesktopApp {
    /// A change made just before quitting is still written.
    fn drop(&mut self) {
        self.shell.save_if_due(true);
    }
}
