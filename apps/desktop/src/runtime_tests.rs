//! Runtime tests (task 0008 D7, acceptance criterion 2): the whole loop — camera link →
//! controller → pointer — with a scripted fake camera and a fake pointer, on the shell's
//! manual clock. Plus the threaded camera worker against a fake `CameraSource`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use app_core::ports::{
    CameraError, CameraFrame, CameraSource, CaptureFormat, CaptureRequest, Desktop, Point2,
};
use app_core::{AppState, Command, DesktopRect, PointerOwner, Settings};
use platform::hotkeys::{Hotkey, Registration};

use super::*;
use crate::app::{Clock, Shell, hotkey_line};
use crate::test_frames::{FLAT, FRAME_US, H, SCENE, W, shifted, with_color};

const DESKTOP: DesktopRect = DesktopRect {
    // Negative origin: a second display left of the primary (R-P5).
    x: -1920.0,
    y: 0.0,
    width: 3840.0,
    height: 1080.0,
};

// ---- fakes ----

#[derive(Debug)]
struct PointerState {
    access: PointerAccess,
    cursor: Point2,
    moves: Vec<Point2>,
    clicks: Vec<Point2>,
    access_checks: usize,
}

#[derive(Debug, Clone)]
struct FakePointer(Rc<RefCell<PointerState>>);

impl FakePointer {
    fn new(access: PointerAccess) -> Self {
        Self(Rc::new(RefCell::new(PointerState {
            access,
            cursor: Point2::new(100.0, 100.0),
            moves: Vec::new(),
            clicks: Vec::new(),
            access_checks: 0,
        })))
    }
}

impl PointerSink for FakePointer {
    fn access(&self) -> PointerAccess {
        let mut s = self.0.borrow_mut();
        s.access_checks += 1;
        s.access
    }

    fn move_to(&mut self, p: Point2) -> Result<(), PointerError> {
        let mut s = self.0.borrow_mut();
        // Like macOS without Accessibility: the move is dropped silently.
        if s.access == PointerAccess::Granted {
            s.cursor = p;
            s.moves.push(p);
        }
        Ok(())
    }

    fn click(&mut self, p: Point2) -> Result<(), PointerError> {
        self.0.borrow_mut().clicks.push(p);
        Ok(())
    }

    fn position(&self) -> Option<Point2> {
        Some(self.0.borrow().cursor)
    }

    fn desktop(&mut self) -> Result<Desktop, PointerError> {
        Ok(Desktop {
            bounds: DESKTOP,
            displays: vec![DESKTOP],
        })
    }
}

/// The camera as the script sets it: where the head is, what the scene is, whether it's
/// plugged in.
#[derive(Debug)]
struct Scene {
    head: (f32, f32),
    seed: u32,
    unplugged: bool,
    deny: bool,
    opens: Vec<Option<String>>,
    closes: usize,
    open: bool,
    ts: u64,
}

/// A synchronous [`CameraLink`]: one frame per poll while open, so the test owns time.
#[derive(Debug, Clone)]
struct ScriptedCamera(Rc<RefCell<Scene>>);

impl ScriptedCamera {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(Scene {
            head: (0.0, 0.0),
            seed: SCENE,
            unplugged: false,
            deny: false,
            opens: Vec::new(),
            closes: 0,
            open: false,
            ts: 0,
        })))
    }
}

impl CameraLink for ScriptedCamera {
    fn open(&mut self, device: Option<String>, _request: CaptureRequest) {
        let mut s = self.0.borrow_mut();
        s.opens.push(device);
        s.open = true;
    }

    fn close(&mut self) {
        let mut s = self.0.borrow_mut();
        s.closes += 1;
        s.open = false;
    }

    fn poll(&mut self) -> Vec<CameraMsg> {
        let mut s = self.0.borrow_mut();
        if !s.open {
            return Vec::new();
        }
        let mut out = Vec::new();
        if s.deny {
            s.open = false;
            return vec![CameraMsg::Failed(CameraError::PermissionDenied)];
        }
        if s.unplugged {
            s.open = false;
            return vec![CameraMsg::Failed(CameraError::Disconnected)];
        }
        if s.ts == 0 {
            out.push(CameraMsg::Opened(CaptureFormat {
                width: W,
                height: H,
                fps: 30.0,
            }));
        }
        s.ts += FRAME_US;
        let (dx, dy) = s.head;
        out.push(CameraMsg::Frame(with_color(shifted(
            W, H, dx, dy, s.seed, s.ts,
        ))));
        out
    }
}

struct Rig {
    shell: Shell,
    rt: Runtime<ScriptedCamera>,
    cam: ScriptedCamera,
    ptr: FakePointer,
}

impl Rig {
    fn new(access: PointerAccess, settings: Settings) -> Self {
        let lines = Hotkey::ALL
            .map(|k| hotkey_line(k, &Registration::Active))
            .to_vec();
        let mut shell = Shell::new(settings, None, lines, true, None);
        shell.clock = Clock::Manual(1_000_000);
        let cam = ScriptedCamera::new();
        let ptr = FakePointer::new(access);
        let rt = Runtime::new(cam.clone(), Ok(Box::new(ptr.clone())));
        Self {
            shell,
            rt,
            cam,
            ptr,
        }
    }

    fn granted() -> Self {
        Self::new(PointerAccess::Granted, Settings::default())
    }

    fn state(&self) -> AppState {
        self.shell.controller.state()
    }

    fn owner(&self) -> PointerOwner {
        self.shell.controller.pointer_owner()
    }

    fn cursor(&self) -> Point2 {
        self.ptr.0.borrow().cursor
    }

    /// One frame period: the clock moves, the runtime pumps.
    fn pump(&mut self) {
        if let Clock::Manual(t) = &mut self.shell.clock {
            *t += FRAME_US;
        }
        self.rt.pump(&mut self.shell);
    }

    fn pump_for(&mut self, us: u64) {
        let end = self.shell.now_us() + us;
        while self.shell.now_us() < end {
            self.pump();
        }
    }

    fn command(&mut self, c: Command) {
        self.shell.command(c);
        self.rt.flush(&mut self.shell);
    }

    /// Started, centre selected, and the 4 s handover over: the camera has the pointer.
    fn tracking(access: PointerAccess, settings: Settings) -> Self {
        let mut r = Self::new(access, settings);
        r.command(Command::ToggleStart);
        r.pump();
        assert_eq!(r.state(), AppState::NoTarget);
        r.command(Command::SelectTarget(Point2::new(320.0, 240.0)));
        r.pump_for(4_100_000);
        assert_eq!(r.state(), AppState::Tracking);
        r
    }
}

fn close(a: Point2, b: Point2, tol: f32) -> bool {
    (a.x - b.x).abs() <= tol && (a.y - b.y).abs() <= tol
}

// ---- the end-to-end flow ----

/// select → track → head right moves the cursor right → lose → pause → resume through
/// recenter → tracking again (task 0008 D7).
#[test]
fn end_to_end_select_track_lose_pause_recenter_resume() {
    let mut r = Rig::tracking(PointerAccess::Granted, Settings::default());
    assert_eq!(r.owner(), PointerOwner::Camera);
    // Camera takeover puts the cursor at the desktop centre (R-A2), negative origin honoured.
    assert!(close(r.cursor(), DESKTOP.center(), 2.0), "{:?}", r.cursor());
    assert_eq!(r.cam.0.borrow().opens, vec![None]);

    let mut last = r.cursor();
    for i in 1..=5 {
        r.cam.0.borrow_mut().head = (i as f32, 0.0);
        r.pump();
        let p = r.cursor();
        assert!(p.x > last.x, "frame {i}: {p:?} should be right of {last:?}");
        last = p;
    }

    r.cam.0.borrow_mut().seed = FLAT;
    for _ in 0..3 {
        r.pump();
    }
    assert_eq!(r.state(), AppState::Lost);
    assert_eq!(r.owner(), PointerOwner::Mouse);

    r.command(Command::ToggleStart);
    assert_eq!(r.state(), AppState::Paused);
    assert!(
        close(r.cursor(), DESKTOP.center(), 0.5),
        "pause centres the cursor"
    );
    let moves = r.ptr.0.borrow().moves.len();
    r.pump_for(1_000_000);
    assert_eq!(r.ptr.0.borrow().moves.len(), moves, "no moves while paused");

    // Resume goes through the 5 s recenter, then the frame centre is the target.
    {
        let mut s = r.cam.0.borrow_mut();
        s.seed = SCENE;
        s.head = (0.0, 0.0);
    }
    r.command(Command::ToggleStart);
    assert_eq!(r.state(), AppState::Recentering);
    r.pump_for(4_900_000);
    assert_eq!(r.state(), AppState::Recentering);
    r.pump_for(200_000);
    assert_eq!(r.state(), AppState::Tracking);
    assert_eq!(r.owner(), PointerOwner::Camera);
    assert!(close(r.cursor(), DESKTOP.center(), 2.0), "{:?}", r.cursor());
    // The camera stayed open throughout.
    assert_eq!(r.cam.0.borrow().opens.len(), 1);
    assert_eq!(r.cam.0.borrow().closes, 0);
}

#[test]
fn physical_mouse_takes_over_and_hands_back_after_4_s() {
    let mut r = Rig::tracking(PointerAccess::Granted, Settings::default());
    // The user grabs the mouse: the cursor is somewhere the camera didn't put it.
    r.ptr.0.borrow_mut().cursor = Point2::new(-500.0, 200.0);
    r.pump();
    assert_eq!(r.owner(), PointerOwner::Handover);
    let moves = r.ptr.0.borrow().moves.len();
    r.pump_for(3_800_000);
    assert_eq!(r.owner(), PointerOwner::Handover, "still within 4 s");
    assert_eq!(r.ptr.0.borrow().moves.len(), moves, "the camera keeps off");
    r.pump_for(400_000);
    assert_eq!(r.owner(), PointerOwner::Camera);
    assert!(close(r.cursor(), DESKTOP.center(), 2.0));
}

#[test]
fn dwell_clicks_once_where_the_cursor_rests() {
    let mut settings = Settings::default();
    settings.dwell.enabled = true;
    let mut r = Rig::tracking(PointerAccess::Granted, settings);
    let dwell_us = (f64::from(r.shell.controller.settings().dwell.dwell_s) * 1e6) as u64;
    r.pump_for(dwell_us + 200_000);
    let clicks = r.ptr.0.borrow().clicks.clone();
    assert_eq!(clicks.len(), 1, "{clicks:?}");
    assert!(close(clicks[0], r.cursor(), 1.0));
    r.pump_for(dwell_us / 2);
    assert_eq!(r.ptr.0.borrow().clicks.len(), 1, "fires once");
}

#[test]
fn unplugging_the_camera_shows_disconnected_and_releases_the_pointer() {
    let mut r = Rig::tracking(PointerAccess::Granted, Settings::default());
    r.cam.0.borrow_mut().unplugged = true;
    r.pump();
    assert_eq!(r.state(), AppState::CameraDisconnected);
    assert_eq!(r.owner(), PointerOwner::Mouse);
    assert_eq!(r.cam.0.borrow().closes, 1, "the runtime closed it");
    // Plugged back in: Start retries.
    r.cam.0.borrow_mut().unplugged = false;
    r.command(Command::ToggleStart);
    r.pump();
    assert_eq!(r.state(), AppState::NoTarget);
}

#[test]
fn camera_permission_denied_reaches_the_ui_state() {
    let mut r = Rig::granted();
    r.cam.0.borrow_mut().deny = true;
    r.command(Command::ToggleStart);
    r.pump();
    assert_eq!(r.state(), AppState::PermissionDenied);
}

// ---- pointer permission and polling ----

#[test]
fn denied_pointer_tracks_without_moving_and_a_later_grant_is_picked_up() {
    let mut r = Rig::new(PointerAccess::Denied, Settings::default());
    r.command(Command::ToggleStart);
    r.pump();
    r.command(Command::SelectTarget(Point2::new(320.0, 240.0)));
    r.pump_for(4_100_000);
    assert_eq!(r.state(), AppState::Tracking);
    assert_eq!(r.owner(), PointerOwner::PermissionNeeded);
    assert!(r.ptr.0.borrow().moves.is_empty());

    // The user grants Accessibility; the next poll (≤ 3 s) notices without a restart.
    r.ptr.0.borrow_mut().access = PointerAccess::Granted;
    r.pump_for(ACCESS_POLL_US + 100_000);
    assert_ne!(r.owner(), PointerOwner::PermissionNeeded);
}

#[test]
fn access_is_polled_every_few_seconds_not_per_frame() {
    let mut r = Rig::granted();
    r.command(Command::ToggleStart);
    r.pump_for(10_000_000); // ≈ 300 frames
    let checks = r.ptr.0.borrow().access_checks;
    assert!((3..=5).contains(&checks), "{checks} checks in 10 s");
}

#[test]
fn the_desktop_from_the_pointer_reaches_the_controller() {
    let mut r = Rig::granted();
    r.pump();
    // Pause moves to the controller's desktop centre: it must be the fake's, not the default.
    r.command(Command::ToggleStart);
    r.pump();
    r.command(Command::ToggleStart);
    assert_eq!(r.state(), AppState::Paused);
    assert!(close(r.cursor(), DESKTOP.center(), 0.5), "{:?}", r.cursor());
}

#[test]
fn no_pointer_backend_is_unsupported_and_tracking_still_runs() {
    let lines = Hotkey::ALL
        .map(|k| hotkey_line(k, &Registration::Active))
        .to_vec();
    let mut shell = Shell::new(Settings::default(), None, lines, true, None);
    shell.clock = Clock::Manual(1_000_000);
    let cam = ScriptedCamera::new();
    let mut rt = Runtime::new(cam, Err(PointerError::Unsupported("Wayland".into())));
    shell.command(Command::ToggleStart);
    rt.pump(&mut shell);
    assert_eq!(shell.controller.pointer_owner(), PointerOwner::Unsupported);
    assert_eq!(shell.controller.state(), AppState::NoTarget);
}

#[test]
fn changing_the_camera_reopens_with_the_new_device() {
    let mut r = Rig::granted();
    r.command(Command::ToggleStart);
    r.pump();
    let mut s = r.shell.controller.settings().clone();
    s.camera = Some("usb-cam".into());
    r.shell.set_settings(s);
    r.rt.flush(&mut r.shell);
    let cam = r.cam.0.borrow();
    assert_eq!(cam.opens, vec![None, Some("usb-cam".into())]);
    assert_eq!(cam.closes, 1);
}

// ---- the threaded camera worker ----

/// A camera source on the worker thread: `frames` frames, then the given error.
struct FakeSource {
    open_result: Result<(), CameraError>,
    frames: u32,
    then: CameraError,
    sent: u32,
    log: Arc<Mutex<Vec<String>>>,
}

impl CameraSource for FakeSource {
    fn open(&mut self, _r: CaptureRequest) -> Result<CaptureFormat, CameraError> {
        self.log.lock().unwrap().push("open".into());
        self.open_result.clone().map(|()| CaptureFormat {
            width: 8,
            height: 8,
            fps: 30.0,
        })
    }

    fn next_frame(&mut self) -> Result<CameraFrame, CameraError> {
        std::thread::sleep(Duration::from_millis(2));
        if self.sent >= self.frames {
            return Err(self.then.clone());
        }
        self.sent += 1;
        Ok(shifted(8, 8, 0.0, 0.0, SCENE, u64::from(self.sent)))
    }

    fn close(&mut self) {
        self.log.lock().unwrap().push("close".into());
    }
}

fn spawn_fake(
    open_result: Result<(), CameraError>,
    frames: u32,
    then: CameraError,
) -> (ThreadedCamera, Arc<Mutex<Vec<String>>>) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let factory: CameraFactory = Box::new(move |_device| {
        Box::new(FakeSource {
            open_result: open_result.clone(),
            frames,
            then: then.clone(),
            sent: 0,
            log: l.clone(),
        }) as Box<dyn CameraSource>
    });
    (ThreadedCamera::spawn(factory, || {}), log)
}

/// Polls until `done` says so or 5 s pass.
fn poll_until(cam: &mut ThreadedCamera, done: impl Fn(&[CameraMsg]) -> bool) -> Vec<CameraMsg> {
    let start = Instant::now();
    let mut all = Vec::new();
    while start.elapsed() < Duration::from_secs(5) {
        all.extend(cam.poll());
        if done(&all) {
            return all;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("timed out; got {} messages", all.len());
}

#[test]
fn worker_opens_streams_and_reports_a_disconnect() {
    let (mut cam, log) = spawn_fake(Ok(()), 20, CameraError::Disconnected);
    cam.open(None, CaptureRequest::default());
    let msgs = poll_until(&mut cam, |m| {
        m.iter().any(|m| matches!(m, CameraMsg::Failed(_)))
    });
    assert!(matches!(msgs[0], CameraMsg::Opened(_)), "{:?}", msgs[0]);
    assert!(msgs.iter().any(|m| matches!(m, CameraMsg::Frame(_))));
    assert_eq!(
        msgs.last(),
        Some(&CameraMsg::Failed(CameraError::Disconnected))
    );
    drop(cam);
    assert_eq!(*log.lock().unwrap(), vec!["open", "close"]);
}

#[test]
fn worker_reports_an_open_failure() {
    let (mut cam, _log) = spawn_fake(Err(CameraError::PermissionDenied), 0, CameraError::NoDevice);
    cam.open(None, CaptureRequest::default());
    let msgs = poll_until(&mut cam, |m| !m.is_empty());
    assert_eq!(msgs, vec![CameraMsg::Failed(CameraError::PermissionDenied)]);
}

#[test]
fn worker_drops_messages_from_a_closed_session() {
    let (mut cam, log) = spawn_fake(Ok(()), u32::MAX, CameraError::Disconnected);
    cam.open(None, CaptureRequest::default());
    poll_until(&mut cam, |m| {
        m.iter().any(|m| matches!(m, CameraMsg::Frame(_)))
    });
    cam.close();
    std::thread::sleep(Duration::from_millis(50));
    assert!(cam.poll().is_empty(), "nothing after close");
    // Closed on the worker, before the next open.
    let start = Instant::now();
    while !log.lock().unwrap().contains(&"close".to_owned()) {
        assert!(start.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn worker_coalesces_frames_to_the_newest() {
    let (mut cam, _log) = spawn_fake(Ok(()), u32::MAX, CameraError::Disconnected);
    cam.open(None, CaptureRequest::default());
    poll_until(&mut cam, |m| {
        m.iter().any(|m| matches!(m, CameraMsg::Opened(_)))
    });
    std::thread::sleep(Duration::from_millis(60)); // ≈ 30 frames produced, ≤ 3 in flight
    let msgs = cam.poll();
    let frames: Vec<_> = msgs
        .iter()
        .filter_map(|m| match m {
            CameraMsg::Frame(f) => Some(f.timestamp_us),
            _ => None,
        })
        .collect();
    assert_eq!(frames.len(), 1, "{msgs:?}");
}
