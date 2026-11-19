//! Headless UI tests (`egui_kittest`, task 0008 D6). Every control is driven by its
//! AccessKit label; time is the shell's manual clock, so countdowns come from controller
//! deadlines; frames are synthetic (no camera).

use super::*;
use app_core::ports::{CameraError, CameraFrame, CaptureFormat, Point2};
use app_core::{AppState, MarkerKind, PointerAccess, PointerOwner};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

use crate::engine::{CameraLink, CameraMsg, Clock, Engine, InlineEngine};
use crate::preview::{PREVIEW_LABEL, fit_rect, to_screen};

use crate::test_frames::{FLAT, FRAME_US, H, SCENE, W, shifted};

fn raw_frame(w: u32, h: u32, seed: u32, ts: u64) -> CameraFrame {
    shifted(w, h, 0.0, 0.0, seed, ts)
}

// ---- shell rigs ----

/// No camera: the UI tests feed frames themselves (`Input::Frame`).
#[derive(Debug)]
struct NullCamera;

impl CameraLink for NullCamera {
    fn open(&mut self, _device: Option<String>, _request: app_core::ports::CaptureRequest) {}
    fn close(&mut self) {}
    fn poll(&mut self) -> Vec<CameraMsg> {
        Vec::new()
    }
}

/// A shell over an inline engine (no camera, no pointer) on a manual clock at `t`.
fn shell_at(
    t: u64,
    settings: Settings,
    lines: Vec<String>,
    ok: bool,
    store: Option<SettingsStore>,
) -> Shell {
    let clock = Clock::manual(t);
    let pointer = Err(app_core::ports::PointerError::Backend("tests".into()));
    let engine = Engine::new(settings.clone(), clock.clone(), NullCamera, pointer);
    Shell::new(
        settings,
        None,
        lines,
        ok,
        store,
        Box::new(InlineEngine(engine)),
        clock,
    )
}

fn advance(sh: &mut Shell, us: u64) {
    sh.clock.advance(us);
}

fn frame_sized(sh: &mut Shell, w: u32, h: u32, seed: u32) {
    advance(sh, FRAME_US);
    let ts = sh.now_us();
    sh.input(Input::Frame(raw_frame(w, h, seed, ts)));
}

fn frame(sh: &mut Shell, seed: u32) {
    frame_sized(sh, W, H, seed);
}

fn hold(sh: &mut Shell, us: u64) {
    let end = sh.now_us() + us;
    while sh.now_us() < end {
        frame(sh, SCENE);
    }
}

fn stopped_with(settings: Settings) -> Shell {
    shell_at(1_000_000, settings, test_lines(), true, None)
}

fn stopped() -> Shell {
    stopped_with(Settings::default())
}

fn test_lines() -> Vec<String> {
    Hotkey::ALL
        .map(|k| hotkey_line(k, &Registration::Active))
        .to_vec()
}

fn opened(sh: &mut Shell, w: u32, h: u32) {
    sh.command(Command::ToggleStart);
    sh.input(Input::CameraOpened(CaptureFormat {
        width: w,
        height: h,
        fps: 30.0,
    }));
}

fn started_with(settings: Settings) -> Shell {
    let mut sh = stopped_with(settings);
    opened(&mut sh, W, H);
    frame(&mut sh, SCENE);
    assert_eq!(sh.state(), AppState::NoTarget);
    sh
}

fn started() -> Shell {
    started_with(Settings::default())
}

/// Selected at the centre; still in the 4 s mouse handover.
fn selected_with(settings: Settings) -> Shell {
    let mut sh = started_with(settings);
    sh.command(Command::SelectTarget(Point2::new(320.0, 240.0)));
    frame(&mut sh, SCENE);
    assert_eq!(sh.state(), AppState::Tracking);
    sh
}

fn tracking_with(settings: Settings) -> Shell {
    let mut sh = selected_with(settings);
    hold(&mut sh, 4_000_000);
    assert_eq!(sh.pointer_owner(), PointerOwner::Camera);
    sh
}

fn tracking() -> Shell {
    tracking_with(Settings::default())
}

fn lost_with(settings: Settings) -> Shell {
    let mut sh = tracking_with(settings);
    for _ in 0..3 {
        frame(&mut sh, FLAT);
        if sh.state() == AppState::Lost {
            return sh;
        }
    }
    panic!("not lost after 3 flat frames");
}

fn tracking_window(sh: Shell) -> Harness<'static, Shell> {
    tracking_window_sized(sh, Vec2::from(TRACKING_SIZE))
}

fn tracking_window_sized(sh: Shell, size: Vec2) -> Harness<'static, Shell> {
    let mut h = Harness::builder()
        .with_size(size)
        .build_ui_state(|ui, sh: &mut Shell| tracking_ui(ui, sh), sh);
    h.run_steps(4);
    h
}

fn options_window(sh: Shell) -> Harness<'static, Shell> {
    options_window_sized(sh, Vec2::from(OPTIONS_SIZE))
}

fn options_window_sized(mut sh: Shell, size: Vec2) -> Harness<'static, Shell> {
    sh.options_open = true;
    let mut h = Harness::builder().with_size(size).build_ui_state(
        |ui, sh: &mut Shell| {
            sh.apply_style(ui.ctx());
            options_ui(ui, sh);
        },
        sh,
    );
    h.run_steps(4);
    h
}

fn has_text(h: &Harness<'_, Shell>, text: &str) -> bool {
    h.query_all_by_label(text).next().is_some()
}

#[track_caller]
fn assert_text(h: &Harness<'_, Shell>, text: &str) {
    assert!(has_text(h, text), "no node labelled {text:?}");
}

fn start_label(state_label: &str) -> String {
    with_key(state_label, Hotkey::ToggleStart)
}

fn recenter_label() -> String {
    with_key("Recenter", Hotkey::Recenter)
}

fn state(h: &mut Harness<'_, Shell>) -> AppState {
    h.state_mut().state()
}

// ---- every state renders icon + text (criterion 3) ----

#[test]
fn status_text_per_state() {
    let cases: Vec<(Shell, &str)> = vec![
        (stopped(), "Camera off"),
        (
            {
                let mut s = stopped();
                s.command(Command::ToggleStart);
                s
            },
            "Starting camera…",
        ),
        (
            {
                let mut s = stopped();
                s.command(Command::ToggleStart);
                s.input(Input::CameraFailed(CameraError::NoDevice));
                s
            },
            "No camera found",
        ),
        (
            {
                let mut s = stopped();
                s.command(Command::ToggleStart);
                s.input(Input::CameraFailed(CameraError::PermissionDenied));
                s
            },
            "Camera access needed",
        ),
        (
            {
                let mut s = stopped();
                s.command(Command::ToggleStart);
                s.input(Input::CameraFailed(CameraError::Backend(
                    "device busy".into(),
                )));
                s
            },
            "Something went wrong",
        ),
        (started(), "Select a target"),
        (tracking(), "Tracking"),
        (lost_with(Settings::default()), "Target lost"),
        (
            {
                let mut s = started();
                s.command(Command::Recenter);
                s
            },
            "Recentering · 5",
        ),
        (
            {
                let mut s = started();
                s.command(Command::ToggleStart);
                s
            },
            "Paused",
        ),
        (
            {
                let mut s = started();
                s.input(Input::CameraFailed(CameraError::Disconnected));
                s
            },
            "Camera disconnected",
        ),
        // Pointer sub-states.
        (
            selected_with(Settings::default()),
            "Mouse in control · camera resumes in 4 s",
        ),
        (
            {
                let mut s = tracking();
                s.input(Input::PointerAccess(PointerAccess::Denied));
                s
            },
            "Pointer control needs permission",
        ),
        (
            {
                let mut s = tracking();
                s.input(Input::PointerAccess(PointerAccess::Unsupported));
                s
            },
            "Pointer: not available",
        ),
    ];
    let mut seen = std::collections::HashSet::new();
    for (mut sh, title) in cases {
        seen.insert(format!("{:?}", sh.state()));
        let h = tracking_window(sh);
        assert_text(&h, title);
    }
    // All eleven R-A1 states (+ Error) were covered.
    assert_eq!(seen.len(), 11, "{seen:?}");
}

#[test]
fn searching_shows_when_find_again_is_on() {
    let settings = Settings {
        loss_recovery: true,
        ..Settings::default()
    };
    let sh = lost_with(settings);
    let mut sh = sh;
    let view = sh.view();
    assert_eq!(view.marker.map(|m| m.kind), Some(MarkerKind::Searching));
    let h = tracking_window(sh);
    assert_text(&h, "Looking for the target…");
}

#[test]
fn camera_error_details_are_shown_in_full() {
    let mut sh = stopped();
    sh.command(Command::ToggleStart);
    sh.input(Input::CameraFailed(CameraError::Backend(
        "device busy".into(),
    )));
    let h = tracking_window(sh);
    assert_text(&h, "camera backend error: device busy");
}

// ---- countdowns come from controller deadlines (criterion 7) ----

#[test]
fn recenter_countdown_follows_the_injected_clock() {
    let mut sh = started();
    sh.command(Command::Recenter);
    let mut h = tracking_window(sh);
    assert_text(&h, "Recentering · 5");
    advance(h.state_mut(), 2_100_000);
    h.run_steps(4);
    assert_text(&h, "Recentering · 3");
    advance(h.state_mut(), 3_000_000);
    h.run_steps(4);
    // At 0 the frame centre was selected, without any new frame or UI timer.
    assert_eq!(state(&mut h), AppState::Tracking);
}

#[test]
fn handover_countdown_follows_frames_and_clock() {
    let mut sh = selected_with(Settings::default());
    hold(&mut sh, 1_500_000);
    let h = tracking_window(sh);
    assert_text(&h, "Mouse in control · camera resumes in 3 s");
}

// ---- controls, by label (criterion 4) ----

#[test]
fn start_button_starts_the_camera() {
    let mut h = tracking_window(stopped());
    h.get_by_label(&start_label("Start")).click();
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Starting);
    assert!(
        h.get_by_label(&start_label("Start"))
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn pause_and_resume_through_recenter() {
    let mut h = tracking_window(tracking());
    h.get_by_label(&start_label("Pause")).click();
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Paused);
    h.get_by_label(&start_label("Resume")).click();
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Recentering);
}

#[test]
fn recenter_button_starts_the_countdown_and_is_disabled_when_stopped() {
    let h = tracking_window(stopped());
    assert!(
        h.get_by_label(&recenter_label())
            .accesskit_node()
            .is_disabled()
    );
    let mut h = tracking_window(started());
    h.get_by_label(&recenter_label()).click();
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Recentering);
}

#[test]
fn options_button_opens_options() {
    let mut h = tracking_window(stopped());
    h.get_by_label("Options").click();
    h.run_steps(4);
    assert!(h.state().options_open);
    assert!(h.state().wants_camera_list);
}

#[test]
fn preview_click_selects_a_target() {
    let mut h = tracking_window(started());
    h.get_by_label(PREVIEW_LABEL).click();
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Tracking);
}

#[test]
fn card_buttons() {
    let mut h = tracking_window(stopped());
    h.get_by_label("Start camera").click();
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Starting);

    let mut sh = stopped();
    sh.command(Command::ToggleStart);
    sh.input(Input::CameraFailed(CameraError::NoDevice));
    let mut h = tracking_window(sh);
    h.get_by_label("Choose camera").click();
    h.run_steps(4);
    assert!(h.state().options_open);
    assert_eq!(h.state().tab, Tab::Camera);
    h.get_by_label("Try again").click();
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Starting);

    let mut sh = started();
    sh.input(Input::CameraFailed(CameraError::Disconnected));
    let mut h = tracking_window(sh);
    h.get_by_label("Reconnect").click();
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Starting);
}

#[test]
fn copy_details_copies_the_error() {
    let mut sh = stopped();
    sh.command(Command::ToggleStart);
    sh.input(Input::CameraFailed(CameraError::Backend(
        "device busy".into(),
    )));
    let mut h = tracking_window(sh);
    h.get_by_label("Restart camera");
    h.get_by_label("Copy details").click();
    h.step();
    let copied = h.output().platform_output.commands.iter().any(|c| {
        matches!(c, egui::OutputCommand::CopyText(t) if t == "camera backend error: device busy")
    });
    assert!(copied, "{:?}", h.output().platform_output.commands);
}

#[test]
fn pointer_permission_buttons_reach_the_platform() {
    let mut sh = tracking();
    sh.input(Input::PointerAccess(PointerAccess::Denied));
    let mut h = tracking_window(sh);
    h.get_by_label("Check again").click();
    h.run_steps(4);
    assert_eq!(h.state().platform_actions, vec![Action::CheckPointerAccess]);
    if cfg!(target_os = "macos") {
        h.get_by_label("Open Accessibility settings").click();
        h.run_steps(4);
        assert!(
            h.state()
                .platform_actions
                .contains(&Action::OpenPointerSettings)
        );
    }
}

/// The start / pause key on this OS (F10 on macOS, F11 elsewhere).
fn start_key() -> egui::Key {
    egui::Key::from_name(Hotkey::ToggleStart.key_name()).expect("an egui key")
}

#[test]
fn in_window_keys_toggle_start_and_recenter() {
    let mut h = tracking_window(stopped());
    h.key_press(start_key());
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Starting);

    let mut h = tracking_window(started());
    h.key_press(egui::Key::F12);
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Recentering);
}

#[test]
fn in_window_start_key_after_a_global_press_is_the_same_press() {
    let mut sh = stopped();
    sh.hotkey(Hotkey::ToggleStart, Source::Global);
    let mut h = tracking_window(sh);
    assert_eq!(state(&mut h), AppState::Starting);
    h.key_press(start_key());
    h.run_steps(4);
    assert_eq!(state(&mut h), AppState::Starting);
}

/// Labels of every node that gets keyboard focus while tabbing through the window.
fn tab_order(h: &mut Harness<'_, Shell>, presses: usize) -> Vec<String> {
    let mut seen = Vec::new();
    for _ in 0..presses {
        h.key_press(egui::Key::Tab);
        h.run_steps(4);
        if let Some(n) = h.root().children_recursive().find(|n| n.is_focused())
            && let Some(l) = n.accesskit_node().label()
            && !seen.contains(&l)
        {
            seen.push(l);
        }
    }
    seen
}

#[test]
fn tracking_controls_are_reachable_by_tab() {
    let mut h = tracking_window(tracking());
    let order = tab_order(&mut h, 8);
    for want in [
        PREVIEW_LABEL.to_owned(),
        start_label("Pause"),
        recenter_label(),
        "Options".to_owned(),
    ] {
        assert!(order.contains(&want), "{want:?} not in {order:?}");
    }
}

#[test]
fn settings_notice_is_shown() {
    let text = "Your settings file was damaged, so defaults are in use.";
    let mut sh = stopped();
    sh.notice = Some(text.to_owned());
    let h = tracking_window(sh);
    assert_text(&h, text);
}

#[test]
fn failed_hotkeys_are_explained_in_the_tracking_window() {
    let lines = vec![
        hotkey_line(Hotkey::ToggleStart, &Registration::Active),
        hotkey_line(
            Hotkey::Recenter,
            &Registration::Unsupported("global hotkeys aren't available on Wayland".into()),
        ),
    ];
    let sh = shell_at(0, Settings::default(), lines, false, None);
    let h = tracking_window(sh);
    assert_text(
        &h,
        "F12: recenter, works only in this window (global hotkeys aren't available on Wayland).",
    );
    // All registered: no lines in the Tracking window (they're in Options → General).
    let h = tracking_window(stopped());
    assert!(!has_text(&h, &test_lines()[1]));
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
    let line = hotkey_line(
        Hotkey::ToggleStart,
        &Registration::Failed("macOS uses this key for something else".into()),
    );
    assert!(line.contains(": macOS uses"), "{line}");
}

// ---- R-C2: click mapping through the drawing rect, with letterboxing (criterion 6) ----

fn click_at(h: &mut Harness<'_, Shell>, pos: egui::Pos2) {
    h.hover_at(pos);
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    h.run_steps(4);
}

#[test]
fn letterboxed_click_selects_the_point_under_the_cursor() {
    let (fw, fh) = (640, 360);
    let mut sh = stopped();
    opened(&mut sh, fw, fh);
    frame_sized(&mut sh, fw, fh, SCENE);
    assert_eq!(sh.state(), AppState::NoTarget);
    let mut h = tracking_window(sh);
    let area = h.get_by_label(PREVIEW_LABEL).rect();
    let drawn = fit_rect(area, (fw, fh));
    assert!(drawn.min.y > area.min.y + 10.0, "letterboxed");

    // On the bar above the picture: nothing is selected.
    click_at(&mut h, egui::pos2(area.center().x, area.min.y + 4.0));
    assert_eq!(state(&mut h), AppState::NoTarget);

    let want = Point2::new(200.0, 120.0);
    click_at(&mut h, to_screen(want, drawn, (fw, fh)));
    assert_eq!(state(&mut h), AppState::Tracking);
    let got = h.state_mut().view().marker.unwrap().position;
    assert!(
        (got.x - want.x).abs() < 1.0 && (got.y - want.y).abs() < 1.0,
        "{got:?}"
    );
}

// ---- fixed windows fit at zoom 1.3 with larger controls (criterion 5) ----

fn inside(h: &Harness<'_, Shell>, label: &str, size: Vec2) {
    let r = h.get_by_label(label).rect();
    assert!(
        r.min.x >= 0.0 && r.min.y >= 0.0 && r.max.x <= size.x + 0.5 && r.max.y <= size.y + 0.5,
        "{label:?} at {r:?} outside {size:?}"
    );
}

#[test]
fn tracking_window_fits_at_130_percent_with_larger_controls() {
    let settings = Settings {
        larger_controls: true,
        ..Settings::default()
    };
    // Zoom 1.3 leaves 1/1.3 of the points; text_scale itself is applied as egui zoom.
    let size = Vec2::from(TRACKING_SIZE) / 1.3;
    let mut sh = tracking_with(settings);
    sh.input(Input::PointerAccess(PointerAccess::Denied)); // the longest status (body + two buttons)
    let h = tracking_window_sized(sh, size);
    for l in [start_label("Pause"), recenter_label(), "Options".to_owned()] {
        inside(&h, &l, size);
    }
}

#[test]
fn options_window_fits_at_130_percent_with_larger_controls() {
    let settings = Settings {
        larger_controls: true,
        ..Settings::default()
    };
    let size = Vec2::from(OPTIONS_SIZE) / 1.3;
    let mut h = options_window_sized(stopped_with(settings), size);
    for tab in ["Pointer", "Clicking", "Camera", "Screen", "General"] {
        h.get_by_role_and_label(egui::accesskit::Role::Tab, tab)
            .click();
        h.run_steps(4);
        for l in ["Restore defaults", "Close", "Pointer", "General"] {
            inside(&h, l, size);
        }
    }
}

// ---- Options window controls, by label ----

fn settings(h: &Harness<'_, Shell>) -> Settings {
    h.state().settings().clone()
}

#[test]
fn pointer_tab_controls() {
    let mut h = options_window(stopped());
    h.get_by_label("Horizontal speed").set_value_accesskit(10.0);
    h.run_steps(4);
    let s = settings(&h);
    assert_eq!((s.gain_x, s.gain_y), (10.0, 10.0), "linked by default");

    h.get_by_label("Keep both speeds the same").click();
    h.run_steps(4);
    assert!(!settings(&h).link_gain);
    h.get_by_label("Vertical speed").set_value_accesskit(3.0);
    h.run_steps(4);
    let s = settings(&h);
    assert_eq!((s.gain_x, s.gain_y), (10.0, 3.0));

    h.get_by_label("Smoothing").set_value_accesskit(0.5);
    h.run_steps(4);
    assert_eq!(settings(&h).smoothing, 0.5);

    assert!(
        h.query_by_label("Steady zone").is_none(),
        "automatic, not an option (#11)"
    );
    assert!(
        h.query_by_label("Screen limits").is_none(),
        "limits live on the Screen tab (#9)"
    );
}

/// #9: screen limits and the Main monitor have their own tab, before General.
#[test]
fn screen_tab_controls() {
    let mut h = options_window(stopped());
    h.get_by_role_and_label(egui::accesskit::Role::Tab, "Screen")
        .click();
    h.run_steps(4);
    assert_eq!(h.state().tab, Tab::Screen);
    for (label, want) in [("Left", 10), ("Right", 20), ("Top", 30), ("Bottom", 40)] {
        h.get_by_label(label).set_value_accesskit(f64::from(want));
        h.run_steps(4);
    }
    let l = settings(&h).limits;
    assert_eq!((l.left, l.right, l.top, l.bottom), (10, 20, 30, 40));
}

#[test]
fn main_monitor_lists_the_monitors_left_to_right() {
    let mut sh = stopped();
    let left = app_core::DesktopRect {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };
    let right = app_core::DesktopRect {
        x: 1920.0,
        y: 0.0,
        width: 2560.0,
        height: 1440.0,
    };
    sh.input(Input::DesktopChanged(app_core::ports::Desktop {
        bounds: app_core::DesktopRect {
            width: 4480.0,
            height: 1440.0,
            ..left
        },
        displays: vec![right, left],
        primary: Some(1),
    }));
    sh.tab = Tab::Screen;
    let mut h = options_window(sh);
    assert_eq!(
        settings(&h).recenter_display,
        app_core::MainMonitor::Primary,
        "default: the primary monitor (#14)"
    );
    h.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Main monitor")
        .click();
    h.run_steps(4);
    h.get_by_label("Monitor 1 · 1920 × 1080 · primary");
    h.get_by_label("Monitor 2 · 2560 × 1440").click();
    h.run_steps(4);
    assert_eq!(
        settings(&h).recenter_display,
        app_core::MainMonitor::Display(1)
    );
}

#[test]
fn main_monitor_is_hidden_with_one_monitor() {
    let mut sh = stopped();
    sh.tab = Tab::Screen;
    let h = options_window(sh);
    assert!(h.query_by_label("Screen limits").is_some());
    assert!(h.query_by_label("Main monitor").is_none());
}

#[test]
fn clicking_tab_controls() {
    let mut h = options_window(stopped());
    h.get_by_label("Clicking").click();
    h.run_steps(4);
    assert_eq!(h.state().tab, Tab::Clicking);
    h.get_by_label("Dwell to click").click();
    h.run_steps(4);
    assert!(settings(&h).dwell.enabled);
    h.get_by_label("Dwell time").set_value_accesskit(2.0);
    h.run_steps(4);
    h.get_by_label("Pause after a click")
        .set_value_accesskit(3.0);
    h.run_steps(4);
    h.get_by_label("Hold-still area").set_value_accesskit(10.0);
    h.run_steps(4);
    let d = settings(&h).dwell;
    assert_eq!((d.dwell_s, d.refractory_s), (2.0, 3.0));
    assert!((d.radius_frac - 0.10).abs() < 1e-6, "{}", d.radius_frac);
}

#[test]
fn camera_tab_controls() {
    let mut sh = stopped();
    sh.cameras = vec![
        CameraDevice {
            id: "0".into(),
            name: "FaceTime HD Camera".into(),
        },
        CameraDevice {
            id: "1".into(),
            name: "USB Webcam".into(),
        },
    ];
    sh.tab = Tab::Camera;
    let mut h = options_window(sh);
    assert_text(&h, "Camera not started");
    h.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Camera")
        .click();
    h.run_steps(4);
    h.get_by_label("USB Webcam").click();
    h.run_steps(4);
    assert_eq!(settings(&h).camera.as_deref(), Some("1"));
    h.get_by_label("Mirror preview").click();
    h.run_steps(4);
    assert!(!settings(&h).mirror);
    let find = h.get_by_label("Find the target again");
    assert_eq!(
        find.accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::False),
        "off by default"
    );
    find.click();
    h.run_steps(4);
    assert!(settings(&h).loss_recovery);
}

#[test]
fn general_tab_controls() {
    let mut sh = stopped();
    sh.tab = Tab::General;
    let mut h = options_window(sh);
    h.get_by_label("Light").click();
    h.run_steps(4);
    assert_eq!(settings(&h).theme, Theme::Paper);
    h.get_by_label("Dark").click();
    h.run_steps(4);
    assert_eq!(settings(&h).theme, Theme::Carbon);
    assert!(h.query_by_label("Text size").is_none(), "removed (#10)");
    h.get_by_label("Larger controls").click();
    h.run_steps(4);
    assert!(settings(&h).larger_controls);
    // Below the fold of the fixed window: scroll, as a keyboard or mouse user would.
    h.get_by_label("Show diagnostics").scroll_to_me();
    h.run_steps(4);
    h.get_by_label("Show diagnostics").click();
    h.run_steps(4);
    assert!(settings(&h).show_diagnostics);
    assert_text(&h, &test_lines()[0]);
}

#[test]
fn restore_defaults_and_close() {
    let sh = stopped_with(Settings {
        gain_x: 12.0,
        gain_y: 12.0,
        ..Settings::default()
    });
    let mut h = options_window(sh);
    h.get_by_label("Restore defaults").click();
    h.run_steps(4);
    assert_eq!(settings(&h).gain_x, 20.0);
    h.get_by_label("Close").click();
    h.run_steps(4);
    assert!(!h.state().options_open);
}

#[test]
fn tabs_move_with_arrow_keys() {
    let mut h = options_window(stopped());
    h.get_by_label("Pointer").focus();
    h.run_steps(4);
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(4);
    assert_eq!(h.state().tab, Tab::Clicking);
    h.key_press(egui::Key::End);
    h.run_steps(4);
    assert_eq!(h.state().tab, Tab::General);
}

#[test]
fn options_controls_are_reachable_by_tab() {
    let mut sh = stopped();
    sh.tab = Tab::General;
    let mut h = options_window(sh);
    let order = tab_order(&mut h, 14);
    for want in [
        "Pointer",
        "Dark",
        "Light",
        "Larger controls",
        "Show diagnostics",
        "Restore defaults",
        "Close",
    ] {
        assert!(order.iter().any(|l| l == want), "{want:?} not in {order:?}");
    }
}

#[test]
fn diagnostics_overlay_lines() {
    let mut sh = tracking();
    let view = sh.view();
    let lines = diagnostics_lines(&view, Some(30.0));
    assert_eq!(lines[0], "fps      30.0");
    assert_eq!(lines[3], "status   Tracking");
    assert_eq!(lines[4], "redetect off");
}

// ---- persistence (criterion 8) ----

#[test]
fn options_changes_are_saved_after_the_debounce() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.toml");
    let mut sh = shell_at(
        1_000_000,
        Settings::default(),
        test_lines(),
        true,
        Some(SettingsStore::at(&path)),
    );
    sh.tab = Tab::General;
    let mut h = options_window(sh);
    h.get_by_label("Light").click();
    h.run_steps(4);
    assert!(h.state().save_pending());
    h.state_mut().save_if_due(false);
    assert!(!path.exists(), "not written before the debounce");

    advance(h.state_mut(), SAVE_DEBOUNCE_US);
    h.state_mut().save_if_due(false);
    assert!(!h.state().save_pending());
    let loaded = SettingsStore::at(&path).load();
    assert_eq!(loaded.settings.theme, Theme::Paper);
    assert_eq!(loaded.notice, None);
}

#[test]
fn closing_options_saves_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.toml");
    let mut sh = shell_at(
        1_000_000,
        Settings::default(),
        test_lines(),
        true,
        Some(SettingsStore::at(&path)),
    );
    sh.tab = Tab::Camera;
    let mut h = options_window(sh);
    h.get_by_label("Find the target again").click();
    h.run_steps(4);
    h.get_by_label("Close").click();
    h.run_steps(4);
    assert!(SettingsStore::at(&path).load().settings.loss_recovery);
}

#[test]
fn theme_setting_restyles_the_ui() {
    let sh = stopped_with(Settings {
        theme: Theme::Paper,
        ..Settings::default()
    });
    let h = tracking_window(sh);
    let v = h.ctx.global_style().visuals.clone();
    assert!(!v.dark_mode);
    assert_eq!(v.panel_fill, theme::PAPER.bg);
}

#[test]
fn preview_texture_follows_frames() {
    let mut sh = started();
    let mut h = tracking_window(std::mem::replace(&mut sh, stopped()));
    assert!(h.state().texture.is_some());
    let seq = h.state().uploaded_seq;
    frame(h.state_mut(), SCENE);
    h.run_steps(4);
    assert_eq!(h.state().uploaded_seq, seq + 1);
}

// ---- packaging ----

#[test]
fn the_embedded_app_icon_decodes_to_a_square_opaque_centre() {
    let icon = app_icon().expect("embedded icon PNG decodes");
    assert_eq!(icon.width, icon.height);
    assert!(icon.width >= 256, "icon too small for the dock / task bar");
    assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
    let c = (icon.height / 2 * icon.width + icon.width / 2) as usize * 4;
    assert_eq!(icon.rgba[c + 3], 255, "icon centre must be opaque");
}
