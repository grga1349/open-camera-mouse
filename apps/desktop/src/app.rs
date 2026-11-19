//! The shell between eframe and the engine (ADR-0016): renders the Tracking and Options
//! windows from the engine's [`Snapshot`], and turns clicks and keys into engine commands.
//! No tracking, pointer or camera logic lives here; the controller runs on the engine thread.

use std::sync::Arc;

#[cfg(test)]
use app_core::{AppState, PointerOwner, View};
use app_core::{Command, Input, Settings, Theme};
use eframe::egui::{
    self, Align, Layout, RichText, ScrollArea, TextureHandle, TextureOptions, Ui, Vec2,
};
use platform::hotkeys::{Hotkey, KeyRouter, Registration, Source};
use platform::{CameraDevice, SettingsStore};

use crate::engine::{Clock, EngineCmd, EngineHandle, Snapshot};
use crate::options::{self, Facts, Tab};
use crate::preview::{self, Extras};
use crate::status::{self, Action, describe};
use crate::widgets::{self, Kind};
use crate::{icons, theme};

/// Wait this long after the last settings change before writing the file, so dragging a
/// slider doesn't write it on every frame.
pub const SAVE_DEBOUNCE_US: u64 = 400_000;

/// Window sizes from the approved lab (logical points).
pub const TRACKING_SIZE: [f32; 2] = [560.0, 620.0];
pub const OPTIONS_SIZE: [f32; 2] = [400.0, 620.0];

/// One line per hotkey: what it does and whether it works outside this app's windows (P13).
pub fn hotkey_line(key: Hotkey, status: &Registration) -> String {
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

/// Frame-rate estimate for the diagnostics overlay (EMA over frame intervals).
#[derive(Debug, Default, Clone, Copy)]
struct FpsMeter {
    last_seq: u64,
    last_us: Option<u64>,
    interval_us: Option<f32>,
}

impl FpsMeter {
    fn observe(&mut self, seq: u64, now_us: u64) {
        if seq == self.last_seq {
            return;
        }
        if let Some(prev) = self.last_us {
            let dt = now_us.saturating_sub(prev) as f32 / (seq - self.last_seq) as f32;
            self.interval_us = Some(match self.interval_us {
                Some(i) => i * 0.9 + dt * 0.1,
                None => dt,
            });
        }
        self.last_seq = seq;
        self.last_us = Some(now_us);
    }

    fn fps(&self) -> Option<f32> {
        self.interval_us.filter(|i| *i > 0.0).map(|i| 1e6 / i)
    }
}

pub struct Shell {
    engine: Box<dyn EngineHandle>,
    /// The latest engine state; refreshed at the start of every window pass.
    snap: Arc<Snapshot>,
    /// The UI's copy of the settings: validated here, sent to the engine, saved from here.
    settings: Settings,
    pub clock: Clock,
    /// Shown above the preview (settings-load fallback, save failure).
    pub notice: Option<String>,
    keys: KeyRouter,
    pub hotkey_lines: Vec<String>,
    /// Whether every global hotkey registered; if not, the lines show under the status.
    hotkeys_ok: bool,
    store: Option<SettingsStore>,
    dirty_since: Option<u64>,
    pub options_open: bool,
    pub tab: Tab,
    /// Set when Options opens; the app refreshes the camera list (slow, not in tests).
    pub wants_camera_list: bool,
    pub cameras: Vec<CameraDevice>,
    /// Requests only the platform can serve (pointer permission). Drained by the app.
    pub platform_actions: Vec<Action>,
    texture: Option<TextureHandle>,
    uploaded_seq: u64,
    /// How long the last preview upload took, for `OCM_PERF` (taken by `main`).
    pub last_upload: Option<std::time::Duration>,
    fps: FpsMeter,
    applied_style: Option<(Theme, bool)>,
}

impl std::fmt::Debug for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shell")
            .field("view", &self.snap.view)
            .field("clock", &self.clock)
            .field("notice", &self.notice)
            .field("options_open", &self.options_open)
            .field("tab", &self.tab)
            .finish_non_exhaustive()
    }
}

impl Shell {
    /// `settings` must be the ones the engine was made with; `clock` the engine's clock.
    pub fn new(
        settings: Settings,
        notice: Option<String>,
        hotkey_lines: Vec<String>,
        hotkeys_ok: bool,
        store: Option<SettingsStore>,
        mut engine: Box<dyn EngineHandle>,
        clock: Clock,
    ) -> Self {
        Self {
            snap: engine.snapshot(),
            engine,
            settings: settings.validated().unwrap_or_default(),
            clock,
            notice,
            keys: KeyRouter::new(),
            hotkey_lines,
            hotkeys_ok,
            store,
            dirty_since: None,
            options_open: false,
            tab: Tab::Pointer,
            wants_camera_list: false,
            cameras: Vec::new(),
            platform_actions: Vec::new(),
            texture: None,
            uploaded_seq: 0,
            last_upload: None,
            fps: FpsMeter::default(),
            applied_style: None,
        }
    }

    pub fn now_us(&self) -> u64 {
        self.clock.now_us()
    }

    /// Sends a controller input to the engine (commands; tests also send frames).
    pub fn input(&mut self, input: Input) {
        self.engine.send(EngineCmd::Input(input));
    }

    /// Asks the engine to re-check pointer permission ("Check again").
    pub fn check_access(&mut self) {
        self.engine.send(EngineCmd::CheckAccess);
    }

    /// Takes the engine's latest snapshot.
    pub fn refresh(&mut self) {
        self.snap = self.engine.snapshot();
    }

    /// The latest view (refreshed).
    #[cfg(test)]
    pub fn view(&mut self) -> View {
        self.refresh();
        self.snap.view.clone()
    }

    #[cfg(test)]
    pub fn state(&mut self) -> AppState {
        self.view().state
    }

    #[cfg(test)]
    pub fn pointer_owner(&mut self) -> PointerOwner {
        self.view().pointer
    }

    #[cfg(test)]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn command(&mut self, c: Command) {
        self.input(Input::Command(c));
    }

    pub fn hotkey(&mut self, key: Hotkey, source: Source) {
        if let Some(c) = self.keys.press(key, source, self.now_us()) {
            self.command(c);
        }
    }

    pub fn open_options(&mut self, tab: Option<Tab>) {
        if !self.options_open {
            self.wants_camera_list = true;
        }
        self.options_open = true;
        if let Some(t) = tab {
            self.tab = t;
        }
    }

    /// Validates new settings (R-A5, `app-core`), sends them to the engine and schedules a
    /// save. The engine applies them (a camera change reopens the camera).
    pub fn set_settings(&mut self, new: Settings) {
        match new.validated() {
            Ok(s) => {
                self.settings = s.clone();
                self.engine.send(EngineCmd::Settings(s));
                self.dirty_since = Some(self.now_us());
            }
            Err(e) => self.notice = Some(format!("That setting wasn't applied: {e}.")),
        }
    }

    /// Whether a save is waiting for the debounce.
    pub fn save_pending(&self) -> bool {
        self.dirty_since.is_some()
    }

    /// Writes the settings if they changed and the debounce passed (or `force`).
    pub fn save_if_due(&mut self, force: bool) {
        let Some(since) = self.dirty_since else {
            return;
        };
        if !force && self.now_us().saturating_sub(since) < SAVE_DEBOUNCE_US {
            return;
        }
        self.dirty_since = None;
        if let Some(store) = &self.store
            && let Err(e) = store.save(&self.settings)
        {
            self.notice = Some(format!("Settings couldn't be saved: {e}."));
        }
    }

    /// Applies theme and larger controls when they change.
    pub fn apply_style(&mut self, ctx: &egui::Context) {
        let s = &self.settings;
        let key = (s.theme, s.larger_controls);
        if self.applied_style == Some(key) {
            return;
        }
        self.applied_style = Some(key);
        let style = theme::style(s.theme, s.larger_controls);
        ctx.set_theme(match s.theme {
            Theme::Carbon => egui::Theme::Dark,
            Theme::Paper => egui::Theme::Light,
        });
        ctx.set_global_style(style);
    }

    /// Uploads the engine's preview image when a new frame arrived (`frame_seq`).
    fn upload_preview(&mut self, ctx: &egui::Context) {
        let seq = self.snap.view.frame_seq;
        if seq == self.uploaded_seq {
            return;
        }
        let started = std::time::Instant::now();
        let Some(image) = self.snap.preview.clone() else {
            self.texture = None;
            self.uploaded_seq = seq;
            return;
        };
        match &mut self.texture {
            Some(t) => t.set(image, TextureOptions::LINEAR),
            None => {
                self.texture = Some(ctx.load_texture("preview", image, TextureOptions::LINEAR));
            }
        }
        self.uploaded_seq = seq;
        self.last_upload = Some(started.elapsed());
    }

    fn act(&mut self, ui: &Ui, action: Action) {
        match action {
            Action::Start => self.command(Command::ToggleStart),
            Action::ChooseCamera => self.open_options(Some(Tab::Camera)),
            Action::CopyDetails => {
                let text = self.snap.view.error.clone().unwrap_or_default();
                ui.ctx().copy_text(text);
            }
            Action::OpenPointerSettings | Action::CheckPointerAccess => {
                self.platform_actions.push(action);
            }
        }
    }
}

/// Hotkey presses this window received while focused. Repeats are ignored: one press, one
/// command.
fn window_hotkeys(ui: &Ui) -> Vec<Hotkey> {
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

/// A button's accessible name with its key, e.g. "Pause  F10" (as `widgets::Button` names it).
#[cfg(test)]
pub fn with_key(label: &str, key: Hotkey) -> String {
    format!("{label}  {}", key.key_name())
}

fn diagnostics_lines(view: &app_core::View, fps: Option<f32>) -> Vec<String> {
    let opt = |v: Option<f32>, digits: usize| match v {
        Some(x) => format!("{x:.digits$}"),
        None => "—".to_owned(),
    };
    let status = view
        .track
        .map(|t| format!("{t:?}"))
        .unwrap_or_else(|| "—".to_owned());
    let redetect = if !view.loss_recovery {
        "off"
    } else if view.redetect_armed {
        "armed"
    } else {
        "not armed"
    };
    vec![
        format!("fps      {}", opt(fps, 1)),
        format!("zncc     {}", opt(view.diagnostics.score, 2)),
        format!("residual {}", opt(view.diagnostics.residual, 2)),
        format!("status   {status}"),
        format!("redetect {redetect}"),
    ]
}

/// The Tracking window (560 × 620): preview, status line, Start/Pause, Recenter, Options.
pub fn tracking_ui(ui: &mut Ui, shell: &mut Shell) {
    shell.apply_style(ui.ctx());
    for key in window_hotkeys(ui) {
        shell.hotkey(key, Source::Window);
    }
    // Deadlines (recenter, handover) are the engine's: it ticks on its own.
    shell.refresh();
    let snap = shell.snap.clone();
    let view = &snap.view;
    let now = shell.now_us();
    shell.fps.observe(view.frame_seq, now);
    shell.upload_preview(ui.ctx());
    let start_key = Hotkey::ToggleStart.key_name();
    let recenter_key = Hotkey::Recenter.key_name();
    let st = describe(view, start_key, recenter_key);
    let settings = &shell.settings;
    let extras = Extras {
        texture: shell.texture.as_ref(),
        mirrored: settings.mirror,
        diagnostics: settings
            .show_diagnostics
            .then(|| diagnostics_lines(view, shell.fps.fps())),
    };

    if let Some(n) = &shell.notice {
        ui.label(RichText::new(n).weak());
    }
    let out = preview::show(ui, view, &st, &extras);
    if let Some(p) = out.pick {
        shell.command(Command::SelectTarget(p));
    }
    if let Some(a) = out.action {
        shell.act(ui, a);
    }

    let controls_h = ui.spacing().interact_size.y + ui.spacing().item_spacing.y + 4.0;
    let status_h = (ui.available_height() - controls_h).max(40.0);
    let mut action = None;
    ScrollArea::vertical()
        .id_salt("status")
        .max_height(status_h)
        .min_scrolled_height(status_h)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                icons::show(ui, st.icon, 16.0);
                ui.label(RichText::new(&st.title).strong());
                ui.label(RichText::new("·").weak());
                ui.label(RichText::new(format!("Pointer: {}", st.pointer)).weak());
            });
            let body = RichText::new(&st.body);
            ui.label(if st.body_mono { body.monospace() } else { body });
            if let Some(h) = st.hint {
                ui.label(RichText::new(h).weak());
            }
            if !st.buttons.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    for b in &st.buttons {
                        let kind = if b.primary {
                            Kind::Primary
                        } else {
                            Kind::Secondary
                        };
                        if widgets::Button::new(b.label, kind)
                            .small()
                            .show(ui)
                            .clicked()
                        {
                            action = Some(b.action);
                        }
                    }
                });
            }
            if !shell.hotkeys_ok {
                for line in &shell.hotkey_lines {
                    ui.label(RichText::new(line).weak());
                }
            }
        });
    if let Some(a) = action {
        shell.act(ui, a);
    }

    let state = view.state;
    ui.horizontal(|ui| {
        let gap = ui.spacing().item_spacing.x;
        let options_w = 96.0;
        let w = ((ui.available_width() - options_w - 2.0 * gap) / 2.0).max(60.0);
        let start = ui
            .add_enabled_ui(status::start_enabled(state), |ui| {
                widgets::Button::primary(status::start_label(state))
                    .key(start_key)
                    .width(w)
                    .show(ui)
            })
            .inner;
        if start.clicked() {
            shell.command(Command::ToggleStart);
        }
        let recenter = ui
            .add_enabled_ui(status::recenter_enabled(state), |ui| {
                widgets::Button::secondary("Recenter")
                    .key(recenter_key)
                    .width(w)
                    .show(ui)
            })
            .inner;
        if recenter.clicked() {
            shell.command(Command::Recenter);
        }
        if widgets::Button::secondary("Options")
            .width(options_w)
            .show(ui)
            .clicked()
        {
            shell.open_options(None);
        }
    });
}

/// The Options window contents (400 × 620).
pub fn options_ui(ui: &mut Ui, shell: &mut Shell) {
    for key in window_hotkeys(ui) {
        shell.hotkey(key, Source::Window);
    }
    shell.refresh();
    let snap = shell.snap.clone();
    let mut s = shell.settings.clone();
    let close = {
        let facts = Facts {
            cameras: &shell.cameras,
            capture: snap.view.capture,
            hotkey_lines: &shell.hotkey_lines,
            displays: &snap.displays,
            primary: snap.primary,
        };
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            options::show(ui, &mut s, &mut shell.tab, &facts)
        })
        .inner
    };
    if s != shell.settings {
        shell.set_settings(s);
    }
    if close {
        shell.options_open = false;
        shell.save_if_due(true);
    }
}

/// The app icon PNG (ADR-0021). eframe sets the window / dock icon at runtime and would
/// otherwise show its own "e" — even inside the `.app`, where it overrides the `.icns`. So
/// macOS gets the Apple-grid variant (same as the bundle's), the others the full-bleed one.
#[cfg(target_os = "macos")]
const APP_ICON_PNG: &[u8] = include_bytes!("../../../assets/icon/macos/icon-512.png");
#[cfg(not(target_os = "macos"))]
const APP_ICON_PNG: &[u8] = include_bytes!("../../../assets/icon/png/icon-256.png");

/// The decoded app icon; `None` (eframe's default icon) only if the embedded PNG is broken,
/// which a test rules out.
pub fn app_icon() -> Option<egui::IconData> {
    eframe::icon_data::from_png_bytes(APP_ICON_PNG).ok()
}

/// Viewport builders for the two fixed-size windows. The Options window inherits the root
/// viewport's icon (eframe).
pub fn tracking_viewport() -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title("Open Camera Mouse")
        .with_inner_size(TRACKING_SIZE)
        .with_resizable(false)
        .with_maximize_button(false)
}

pub fn options_viewport(next_to: Option<egui::Rect>) -> egui::ViewportBuilder {
    let mut b = egui::ViewportBuilder::default()
        .with_title("Options")
        .with_inner_size(OPTIONS_SIZE)
        .with_resizable(false)
        .with_maximize_button(false)
        .with_minimize_button(false);
    // Best effort: Wayland can't position windows (PLATFORM_RISKS P10/P11).
    if let Some(r) = next_to {
        b = b.with_position(r.right_top() + Vec2::new(16.0, 0.0));
    }
    b
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
