//! Engine tests (task 0008 D7 criterion 2, task 0009 slice 2a / ADR-0016): the whole loop —
//! camera link → controller → pointer — with a scripted fake camera and a fake pointer, on a
//! manual clock. Plus the threaded camera worker and the engine thread against fakes.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use app_core::ports::Point2;
use app_core::{AppState, Command, PointerOwner};

use super::*;
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
            primary: Some(0),
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
    /// Delivers nothing (a stalled camera).
    stalled: bool,
    opens: Vec<Option<String>>,
    closes: usize,
    open: bool,
    sent_opened: bool,
    /// Capture timestamps: the clock's time by default; a fixed 30 fps sequence from here
    /// when set (to decouple capture time from arrival time).
    next_ts: Option<u64>,
}

/// A synchronous [`CameraLink`]: one frame per poll while open, so the test owns time.
#[derive(Debug, Clone)]
struct ScriptedCamera(Rc<RefCell<Scene>>, Clock);

impl ScriptedCamera {
    fn new(clock: Clock) -> Self {
        Self(
            Rc::new(RefCell::new(Scene {
                head: (0.0, 0.0),
                seed: SCENE,
                unplugged: false,
                deny: false,
                stalled: false,
                opens: Vec::new(),
                closes: 0,
                open: false,
                sent_opened: false,
                next_ts: None,
            })),
            clock,
        )
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
        if !s.open || s.stalled {
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
        if !s.sent_opened {
            s.sent_opened = true;
            out.push(CameraMsg::Opened(CaptureFormat {
                width: W,
                height: H,
                fps: 30.0,
            }));
        }
        let ts = match s.next_ts {
            Some(t) => {
                s.next_ts = Some(t + FRAME_US);
                t
            }
            None => self.1.now_us(),
        };
        let (dx, dy) = s.head;
        out.push(CameraMsg::Frame(with_color(shifted(
            W, H, dx, dy, s.seed, ts,
        ))));
        out
    }
}

struct Rig {
    engine: Engine<ScriptedCamera>,
    clock: Clock,
    cam: ScriptedCamera,
    ptr: FakePointer,
}

impl Rig {
    fn new(access: PointerAccess, settings: Settings) -> Self {
        let clock = Clock::manual(1_000_000);
        let cam = ScriptedCamera::new(clock.clone());
        let ptr = FakePointer::new(access);
        let engine = Engine::new(
            settings,
            clock.clone(),
            cam.clone(),
            Ok(Box::new(ptr.clone())),
        );
        Self {
            engine,
            clock,
            cam,
            ptr,
        }
    }

    fn granted() -> Self {
        Self::new(PointerAccess::Granted, Settings::default())
    }

    fn state(&self) -> AppState {
        self.engine.controller().state()
    }

    fn owner(&self) -> PointerOwner {
        self.engine.controller().pointer_owner()
    }

    fn cursor(&self) -> Point2 {
        self.ptr.0.borrow().cursor
    }

    fn moves(&self) -> Vec<Point2> {
        self.ptr.0.borrow().moves.clone()
    }

    /// One frame period: the clock moves, the engine steps.
    fn pump(&mut self) {
        self.clock.advance(FRAME_US);
        self.engine.step();
    }

    fn pump_for(&mut self, us: u64) {
        let end = self.clock.now_us() + us;
        while self.clock.now_us() < end {
            self.pump();
        }
    }

    fn command(&mut self, c: Command) {
        self.engine.apply(EngineCmd::Input(Input::Command(c)));
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

    // The steady zone (`STEADY_ZONE_CAM_PX`, #11) may hold the first small step; the cursor never
    // moves left and ends up right of where it started.
    let start = r.cursor();
    let mut last = start;
    for i in 1..=5 {
        r.cam.0.borrow_mut().head = (i as f32, 0.0);
        r.pump();
        let p = r.cursor();
        assert!(p.x >= last.x, "frame {i}: {p:?} left of {last:?}");
        last = p;
    }
    assert!(last.x > start.x + 50.0, "{last:?} vs {start:?}");

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
    let dwell_us = (f64::from(r.engine.controller().settings().dwell.dwell_s) * 1e6) as u64;
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
    let clock = Clock::manual(1_000_000);
    let cam = ScriptedCamera::new(clock.clone());
    let mut e = Engine::new(
        Settings::default(),
        clock.clone(),
        cam,
        Err(PointerError::Unsupported("Wayland".into())),
    );
    e.apply(EngineCmd::Input(Input::Command(Command::ToggleStart)));
    clock.advance(FRAME_US);
    e.step();
    assert_eq!(e.controller().pointer_owner(), PointerOwner::Unsupported);
    assert_eq!(e.controller().state(), AppState::NoTarget);
}

#[test]
fn changing_the_camera_reopens_with_the_new_device() {
    let mut r = Rig::granted();
    r.command(Command::ToggleStart);
    r.pump();
    let mut s = r.engine.controller().settings().clone();
    s.camera = Some("usb-cam".into());
    r.engine.apply(EngineCmd::Settings(s));
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

// ---- ADR-0016: the engine doesn't depend on the UI, frames run on camera time ----

/// Head moving right a little each frame, so every frame moves the cursor somewhere new.
fn sway(r: &mut Rig, i: usize) {
    r.cam.0.borrow_mut().head = ((i % 20) as f32 * 0.5, (i % 7) as f32 * 0.3);
}

#[test]
fn reading_snapshots_changes_neither_the_number_nor_the_positions_of_moves() {
    let run = |read: bool| {
        let mut r = Rig::tracking(PointerAccess::Granted, Settings::default());
        let before = r.moves().len();
        for i in 0..60 {
            sway(&mut r, i);
            r.pump();
            if read {
                // A UI pass (or ten): reads only.
                for _ in 0..10 {
                    let _ = r.engine.snapshot();
                }
            }
        }
        r.moves()[before..].to_vec()
    };
    let quiet = run(false);
    assert_eq!(quiet.len(), 60, "one move per frame");
    assert_eq!(run(true), quiet);
}

#[test]
fn regular_capture_times_give_the_same_path_whatever_the_arrival_jitter() {
    let run = |jitter: bool| {
        let mut r = Rig::tracking(PointerAccess::Granted, Settings::default());
        let start = r.clock.now_us();
        r.cam.0.borrow_mut().next_ts = Some(start + FRAME_US);
        let before = r.moves().len();
        for i in 0..60u64 {
            sway(&mut r, i as usize);
            // Frame `i` was captured at start + (i + 1) frames; it arrives 0–25 ms later.
            let arrival =
                start + (i + 1) * FRAME_US + if jitter { (i * 7_919) % 25_000 } else { 0 };
            let now = r.clock.now_us();
            r.clock.advance(arrival.saturating_sub(now));
            r.engine.step();
        }
        r.moves()[before..].to_vec()
    };
    let steady = run(false);
    assert_eq!(steady.len(), 60);
    assert_eq!(run(true), steady);
}

#[test]
fn deadlines_fire_with_no_frames_and_no_ui() {
    let mut r = Rig::tracking(PointerAccess::Granted, Settings::default());
    r.cam.0.borrow_mut().stalled = true;
    r.command(Command::Recenter);
    assert_eq!(r.state(), AppState::Recentering);
    // Only the engine's idle steps, as when minimised with a stalled camera.
    let step = |r: &mut Rig, us: u64| {
        let end = r.clock.now_us() + us;
        while r.clock.now_us() < end {
            r.clock.advance(WAIT.as_micros() as u64);
            r.engine.step();
        }
    };
    step(&mut r, 4_800_000);
    assert_eq!(r.state(), AppState::Recentering);
    step(&mut r, 400_000);
    assert_eq!(
        r.state(),
        AppState::Tracking,
        "the countdown finished on the last frame"
    );
}

/// A camera source producing numbered frames as fast as it can (2 ms each).
struct Counting {
    produced: Arc<AtomicU64>,
}

impl CameraSource for Counting {
    fn open(&mut self, _r: CaptureRequest) -> Result<CaptureFormat, CameraError> {
        Ok(CaptureFormat {
            width: 8,
            height: 8,
            fps: 500.0,
        })
    }

    fn next_frame(&mut self) -> Result<CameraFrame, CameraError> {
        std::thread::sleep(Duration::from_millis(2));
        let n = self.produced.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(shifted(8, 8, 0.0, 0.0, SCENE, n))
    }

    fn close(&mut self) {}
}

#[test]
fn a_stalled_engine_gets_the_newest_frame_next() {
    let produced = Arc::new(AtomicU64::new(0));
    let p = produced.clone();
    let factory: CameraFactory = Box::new(move |_| {
        Box::new(Counting {
            produced: p.clone(),
        }) as Box<dyn CameraSource>
    });
    let mut cam = ThreadedCamera::spawn(factory, || {});
    cam.open(None, CaptureRequest::default());
    poll_until(&mut cam, |m| {
        m.iter().any(|m| matches!(m, CameraMsg::Frame(_)))
    });
    // The engine is busy while ≥ 20 frames are produced. Counted, not timed: a 2 ms sleep
    // takes ~15 ms on Windows runners and a fixed 100 ms gave < 11 frames there (#7).
    let start = produced.load(Ordering::SeqCst);
    let waited = Instant::now();
    while produced.load(Ordering::SeqCst) < start + 20 {
        assert!(waited.elapsed() < Duration::from_secs(5), "camera stopped");
        std::thread::sleep(Duration::from_millis(1));
    }
    let newest = produced.load(Ordering::SeqCst);
    let frames: Vec<u64> = cam
        .poll()
        .into_iter()
        .filter_map(|m| match m {
            CameraMsg::Frame(f) => Some(f.timestamp_us),
            _ => None,
        })
        .collect();
    assert_eq!(frames.len(), 1);
    // The frame in the slot is the newest one, or the one just being written.
    assert!(frames[0] + 1 >= newest, "got {} of {newest}", frames[0]);
    assert!(
        cam.replaced() > 10,
        "older frames were replaced, not queued"
    );
}

/// A pointer that can live on the engine thread and be read from the test.
#[derive(Debug, Clone, Default)]
struct SharedMoves(Arc<Mutex<Vec<Point2>>>);

impl PointerSink for SharedMoves {
    fn access(&self) -> PointerAccess {
        PointerAccess::Granted
    }
    fn move_to(&mut self, p: Point2) -> Result<(), PointerError> {
        self.0.lock().unwrap().push(p);
        Ok(())
    }
    fn click(&mut self, _p: Point2) -> Result<(), PointerError> {
        Ok(())
    }
    fn position(&self) -> Option<Point2> {
        self.0.lock().unwrap().last().copied()
    }
    fn desktop(&mut self) -> Result<Desktop, PointerError> {
        Ok(Desktop {
            bounds: DESKTOP,
            displays: vec![DESKTOP],
            primary: Some(0),
        })
    }
}

/// Textured frames stamped with the (manual) clock, one every 2 ms of real time.
struct ClockedScene {
    clock: Clock,
}

impl CameraSource for ClockedScene {
    fn open(&mut self, _r: CaptureRequest) -> Result<CaptureFormat, CameraError> {
        Ok(CaptureFormat {
            width: W,
            height: H,
            fps: 30.0,
        })
    }

    fn next_frame(&mut self) -> Result<CameraFrame, CameraError> {
        std::thread::sleep(Duration::from_millis(2));
        Ok(shifted(W, H, 0.0, 0.0, SCENE, self.clock.now_us()))
    }

    fn close(&mut self) {}
}

fn wait_for(what: &str, done: impl Fn() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "timed out: {what}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn the_engine_thread_moves_the_pointer_with_no_ui_pass() {
    let clock = Clock::manual(1_000_000);
    // Simulated time runs ≈ 16× real time, so the 4 s handover takes ≈ 250 ms.
    let ticking = Arc::new(AtomicBool::new(true));
    let ticker = {
        let (clock, ticking) = (clock.clone(), ticking.clone());
        std::thread::spawn(move || {
            while ticking.load(Ordering::SeqCst) {
                clock.advance(FRAME_US);
                std::thread::sleep(Duration::from_millis(2));
            }
        })
    };
    let moves = SharedMoves::default();
    let (c, m) = (clock.clone(), moves.clone());
    let make: EngineFactory = Box::new(move |wake| {
        let cc = c.clone();
        let factory: CameraFactory = Box::new(move |_| {
            Box::new(ClockedScene { clock: cc.clone() }) as Box<dyn CameraSource>
        });
        let camera = ThreadedCamera::spawn(factory, wake);
        Engine::new(Settings::default(), c, camera, Ok(Box::new(m)))
    });
    let repaints = Arc::new(AtomicU64::new(0));
    let rp = repaints.clone();
    let mut engine = EngineThread::spawn(make, move || {
        rp.fetch_add(1, Ordering::SeqCst);
    });
    engine.send(EngineCmd::Input(Input::Command(Command::ToggleStart)));
    wait_for("a frame to select on", || {
        let v = engine_view(&engine);
        v.state == AppState::NoTarget && v.frame_seq > 0
    });
    engine.send(EngineCmd::Input(Input::Command(Command::SelectTarget(
        Point2::new(320.0, 240.0),
    ))));
    wait_for("camera takes over", || !moves.0.lock().unwrap().is_empty());
    // From here on nobody reads a snapshot or paints: the moves keep coming.
    let n = moves.0.lock().unwrap().len();
    wait_for("20 more moves", || moves.0.lock().unwrap().len() >= n + 20);
    assert!(
        repaints.load(Ordering::SeqCst) > 0,
        "the UI was told to repaint"
    );
    ticking.store(false, Ordering::SeqCst);
    ticker.join().unwrap();
    drop(engine);
}

/// The engine thread's latest published view, read without `&mut` (unlike the trait).
fn engine_view(e: &EngineThread) -> View {
    e.latest
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .view
        .clone()
}
