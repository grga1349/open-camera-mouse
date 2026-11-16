//! The engine (ADR-0016): controller + camera + pointer on their own thread, so the cursor
//! never waits for the UI. Camera worker → engine → pointer; the UI sends [`EngineCmd`]s and
//! reads [`Snapshot`]s.
//!
//! # Threads
//!
//! - The camera runs on a worker thread ([`ThreadedCamera`]): opening can block (the macOS
//!   permission prompt, slow devices, P3) and `next_frame` blocks until a frame. Frames go
//!   through a newest-wins slot, so a stalled engine gets the newest frame next; status
//!   messages carry a session number, so anything from a camera already closed is dropped.
//! - [`EngineThread`] runs an [`Engine`] made on that thread by a factory (the macOS pointer
//!   isn't `Send`). It wakes on a frame, a command or [`WAIT`], steps, and publishes a
//!   snapshot when the view changed, then asks the UI to repaint.
//! - [`Engine`] itself is synchronous: tests drive it inline with fakes.
//!
//! # Time
//!
//! A frame is handled at its capture timestamp (clamped monotonic), and so is the cursor
//! observation before it, so smoothing and dwell follow the camera, not the UI. Commands and
//! idle ticks use the clock. Access and display polls use the controller's current time.
//!
//! # Order per step
//!
//! 1. Pointer permission every [`ACCESS_POLL_US`] (≈10 ms on macOS, never per frame) and the
//!    display layout every [`DESKTOP_POLL_US`].
//! 2. Camera messages in order, frames coalesced to the newest. Before each frame the OS
//!    cursor is observed (handover), then the frame goes in and its move comes out.
//! 3. No frame for [`IDLE_TICK_US`]: observe the cursor and `Tick`, so deadlines fire without
//!    frames (camera stalled, stopped, or not open).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use app_core::ports::{
    CameraError, CameraFrame, CameraSource, CaptureFormat, CaptureRequest, Desktop, PointerError,
    PointerSink,
};
use app_core::{Controller, DesktopRect, Effect, Input, PointerAccess, Settings, Step, View};
use eframe::egui::ColorImage;

use crate::perf::SharedPerf;
use crate::recorder::Recorder;

/// How often pointer permission is re-checked (a grant takes effect without a restart).
pub const ACCESS_POLL_US: u64 = 3_000_000;
/// How often the display layout is re-read (R-P5: monitors come and go).
pub const DESKTOP_POLL_US: u64 = 1_000_000;
/// Without a frame for this long, a step ticks the controller on the clock.
pub const IDLE_TICK_US: u64 = 100_000;
/// The engine thread's longest sleep: polls and deadlines run at least this often.
pub const WAIT: Duration = Duration::from_millis(100);
/// Effects can cause inputs that cause effects (a failed move reports back); never loop forever.
const MAX_EFFECT_ROUNDS: usize = 8;

/// The time source for the engine and the UI (same epoch as camera timestamps). Tests use a
/// shared manual clock so the UI, the engine and fake cameras agree.
#[derive(Debug, Clone)]
pub enum Clock {
    System(Instant),
    #[cfg_attr(not(test), allow(dead_code))] // Only tests set the clock by hand.
    Manual(Arc<AtomicU64>),
}

impl Clock {
    pub fn now_us(&self) -> u64 {
        match self {
            Clock::System(epoch) => epoch.elapsed().as_micros() as u64,
            Clock::Manual(t) => t.load(Ordering::SeqCst),
        }
    }

    #[cfg(test)]
    pub fn manual(t: u64) -> Self {
        Clock::Manual(Arc::new(AtomicU64::new(t)))
    }

    /// Moves a manual clock forward (no-op on the system clock).
    #[cfg(test)]
    pub fn advance(&self, us: u64) {
        if let Clock::Manual(t) = self {
            t.fetch_add(us, Ordering::SeqCst);
        }
    }
}

// ---- camera link ----

/// What the camera reports to the engine.
#[derive(Debug, Clone, PartialEq)]
pub enum CameraMsg {
    Opened(CaptureFormat),
    Failed(CameraError),
    Frame(CameraFrame),
}

/// The engine's handle on a camera: commands go out, messages come back.
pub trait CameraLink {
    /// Opens `device` (`Settings::camera`), closing any open camera first.
    fn open(&mut self, device: Option<String>, request: CaptureRequest);
    fn close(&mut self);
    /// Messages of the current session since the last call, in order, frames coalesced to the
    /// newest.
    fn poll(&mut self) -> Vec<CameraMsg>;
    /// Running total of frames replaced before the engine took them (it was behind).
    fn replaced(&self) -> u64 {
        0
    }
}

/// Makes the camera for a device on the worker thread, so it never has to be `Send`.
pub type CameraFactory = Box<dyn FnMut(Option<String>) -> Box<dyn CameraSource> + Send>;

enum Cmd {
    Open {
        session: u64,
        device: Option<String>,
        request: CaptureRequest,
    },
    Close,
    Quit,
}

/// The worker's newest-wins frame slot, shared with [`ThreadedCamera`].
#[derive(Debug, Default)]
struct FrameSlot {
    frame: Mutex<Option<(u64, CameraFrame)>>,
    replaced: AtomicU64,
}

/// [`CameraLink`] over a worker thread that owns the camera.
pub struct ThreadedCamera {
    cmds: Sender<Cmd>,
    msgs: Receiver<(u64, CameraMsg)>,
    slot: Arc<FrameSlot>,
    session: u64,
    worker: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for ThreadedCamera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadedCamera")
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}

impl ThreadedCamera {
    /// `wake` is called after every frame or message (the engine thread wakes on it).
    pub fn spawn(factory: CameraFactory, wake: impl Fn() + Send + 'static) -> Self {
        let (cmds, cmd_rx) = mpsc::channel();
        let (msg_tx, msgs) = mpsc::channel();
        let slot = Arc::new(FrameSlot::default());
        let worker_slot = slot.clone();
        let worker = thread::Builder::new()
            .name("camera-worker".into())
            .spawn(move || worker(factory, &cmd_rx, &msg_tx, &worker_slot, &wake))
            .ok();
        Self {
            cmds,
            msgs,
            slot,
            session: 0,
            worker,
        }
    }
}

impl CameraLink for ThreadedCamera {
    fn open(&mut self, device: Option<String>, request: CaptureRequest) {
        self.session += 1;
        let sent = self.cmds.send(Cmd::Open {
            session: self.session,
            device,
            request,
        });
        if sent.is_err() || self.worker.is_none() {
            // Reported on the next poll like any other failure.
            self.worker = None;
        }
    }

    fn close(&mut self) {
        self.session += 1;
        let _ = self.cmds.send(Cmd::Close);
    }

    fn poll(&mut self) -> Vec<CameraMsg> {
        // The slot first: a status message sent after this frame (a failure) is then already
        // in the channel, and an `Opened` sent before it certainly is.
        let frame = self
            .slot
            .frame
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .filter(|(s, _)| *s == self.session)
            .map(|(_, f)| CameraMsg::Frame(f));
        let (mut before, mut after) = (Vec::new(), Vec::new());
        let mut stopped = false;
        loop {
            match self.msgs.try_recv() {
                Ok((session, msg)) if session == self.session => match msg {
                    CameraMsg::Opened(_) => before.push(msg),
                    _ => after.push(msg),
                },
                Ok(_) => {}
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    stopped = true;
                    break;
                }
            }
        }
        before.extend(frame);
        before.append(&mut after);
        if stopped || self.worker.is_none() {
            before.push(CameraMsg::Failed(CameraError::Backend(
                "the camera thread stopped".into(),
            )));
        }
        before
    }

    fn replaced(&self) -> u64 {
        self.slot.replaced.load(Ordering::Relaxed)
    }
}

impl Drop for ThreadedCamera {
    /// Closes the camera before the app exits (the camera light goes off). Bounded: a stuck
    /// read times out after `FRAME_TIMEOUT`, a close after `CLOSE_WAIT` (platform::camera).
    fn drop(&mut self) {
        let _ = self.cmds.send(Cmd::Quit);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

fn worker(
    mut factory: CameraFactory,
    cmds: &Receiver<Cmd>,
    msgs: &Sender<(u64, CameraMsg)>,
    slot: &FrameSlot,
    wake: &dyn Fn(),
) {
    let send = |session: u64, msg: CameraMsg| {
        let ok = msgs.send((session, msg)).is_ok();
        wake();
        ok
    };
    let mut open: Option<(u64, Box<dyn CameraSource>)> = None;
    loop {
        // Idle: block for a command. Streaming: take commands between frames.
        let cmd = if open.is_some() {
            match cmds.try_recv() {
                Ok(c) => Some(c),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => break,
            }
        } else {
            match cmds.recv() {
                Ok(c) => Some(c),
                Err(_) => break,
            }
        };
        if let Some(cmd) = cmd {
            if let Some((_, mut cam)) = open.take() {
                cam.close();
            }
            match cmd {
                Cmd::Open {
                    session,
                    device,
                    request,
                } => {
                    let mut cam = factory(device);
                    match cam.open(request) {
                        Ok(format) => {
                            if !send(session, CameraMsg::Opened(format)) {
                                break;
                            }
                            open = Some((session, cam));
                        }
                        Err(e) => {
                            if !send(session, CameraMsg::Failed(e)) {
                                break;
                            }
                        }
                    }
                }
                Cmd::Close => {}
                Cmd::Quit => break,
            }
            continue;
        }
        let Some((session, cam)) = open.as_mut() else {
            continue;
        };
        let session = *session;
        match cam.next_frame() {
            Ok(f) => {
                let old = slot
                    .frame
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .replace((session, f));
                if old.is_some() {
                    slot.replaced.fetch_add(1, Ordering::Relaxed);
                }
                wake();
            }
            Err(e) => {
                if let Some((_, mut cam)) = open.take() {
                    cam.close();
                }
                if !send(session, CameraMsg::Failed(e)) {
                    break;
                }
            }
        }
    }
    if let Some((_, mut cam)) = open.take() {
        cam.close();
    }
}

// ---- engine ----

/// UI → engine.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineCmd {
    /// A controller input: commands from clicks and hotkeys (tests also send frames).
    Input(Input),
    /// Settings already validated by the UI (`Settings::validated`).
    Settings(Settings),
    /// Re-check pointer permission now ("Check again").
    CheckAccess,
}

/// Engine → UI: everything the windows render.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub view: View,
    /// The displays in "Main monitor" order.
    pub displays: Vec<DesktopRect>,
    /// Index of the OS primary display in `displays`, if known (#14).
    pub primary: Option<usize>,
    /// The latest canonical frame (mirrored) as an egui image, built off the UI thread.
    /// Changes when `view.frame_seq` does.
    pub preview: Option<Arc<ColorImage>>,
}

/// The UI's side of the engine: the app's [`EngineThread`], or an inline engine in tests.
pub trait EngineHandle {
    fn send(&mut self, cmd: EngineCmd);
    /// The latest published state.
    fn snapshot(&mut self) -> Arc<Snapshot>;
}

/// Performs effects and feeds observations. Generic over the camera link so tests can run
/// the whole loop synchronously.
pub struct Engine<C: CameraLink> {
    controller: Controller,
    clock: Clock,
    /// The controller's time: monotonic, frames at their capture time (module doc).
    t_us: u64,
    camera: C,
    /// `None` when the platform has no pointer backend (the controller is told `Unsupported`).
    pointer: Option<Box<dyn PointerSink>>,
    pointer_error: Option<PointerError>,
    access_due_us: Option<u64>,
    desktop_due_us: Option<u64>,
    desktop: Option<Desktop>,
    last_frame_us: u64,
    preview: Option<Arc<ColorImage>>,
    preview_seq: u64,
    recorder: Option<Recorder>,
    perf: Option<SharedPerf>,
}

impl<C: CameraLink> std::fmt::Debug for Engine<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("controller", &self.controller)
            .field("t_us", &self.t_us)
            .field("desktop", &self.desktop)
            .field("pointer_error", &self.pointer_error)
            .finish_non_exhaustive()
    }
}

impl<C: CameraLink> Engine<C> {
    pub fn new(
        settings: Settings,
        clock: Clock,
        camera: C,
        pointer: Result<Box<dyn PointerSink>, PointerError>,
    ) -> Self {
        // The pointer adapter reports the real desktop on the first step.
        let desktop = DesktopRect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        // Loaded settings are already repaired; fall back rather than panic regardless.
        let mut controller = Controller::new(settings, desktop)
            .or_else(|_| Controller::new(Settings::default(), desktop))
            .expect("default settings are valid");
        // Debug builds: `OCM_DEV_STEADY=<camera px>` tries another steady zone on the device
        // (test drive #11) without a rebuild; 0 turns it off.
        if cfg!(debug_assertions)
            && let Some(v) = std::env::var("OCM_DEV_STEADY")
                .ok()
                .and_then(|v| v.parse::<f32>().ok())
        {
            controller.set_steady_zone_cam_px(v);
        }
        let (pointer, pointer_error) = match pointer {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e)),
        };
        let t_us = clock.now_us();
        Self {
            controller,
            clock,
            t_us,
            camera,
            pointer,
            pointer_error,
            access_due_us: None,
            desktop_due_us: None,
            desktop: None,
            last_frame_us: t_us,
            preview: None,
            preview_seq: 0,
            recorder: None,
            perf: None,
        }
    }

    /// Records raw frames and controller events (dev / diagnostics, `recorder`).
    pub fn with_recorder(mut self, mut recorder: Option<Recorder>) -> Self {
        if let Some(r) = &mut recorder {
            r.settings(self.controller.settings());
        }
        self.recorder = recorder;
        self
    }

    /// Frame-path timing (`OCM_PERF=1`, `perf`).
    pub fn with_perf(mut self, perf: Option<SharedPerf>) -> Self {
        self.perf = perf;
        self
    }

    #[cfg(test)]
    pub fn controller(&self) -> &Controller {
        &self.controller
    }

    /// The controller's time moved to `t` if that is later (frames, commands).
    fn at(&mut self, t: u64) -> u64 {
        self.t_us = self.t_us.max(t);
        self.t_us
    }

    /// Applies a UI command at once, effects included.
    pub fn apply(&mut self, cmd: EngineCmd) {
        match cmd {
            EngineCmd::Input(Input::Frame(f)) => self.frame(f),
            EngineCmd::Input(i) => {
                let t = self.at(self.clock.now_us());
                self.input(i, t);
            }
            EngineCmd::Settings(s) => {
                let t = self.at(self.clock.now_us());
                // The UI validated them; a rejection here would be a bug, so keep the old ones.
                if let Ok(step) = self.controller.update_settings(s, t) {
                    if let Some(r) = &mut self.recorder {
                        r.settings(self.controller.settings());
                    }
                    self.perform(step);
                }
            }
            EngineCmd::CheckAccess => self.check_access(),
        }
    }

    /// One pass of the loop (module doc, "Order per step").
    pub fn step(&mut self) {
        let now = self.clock.now_us();
        if let Some(p) = &self.perf {
            p.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .report_if_due(self.camera.replaced());
        }
        if self.access_due_us.is_none_or(|t| now >= t) {
            self.check_access();
        }
        if self.desktop_due_us.is_none_or(|t| now >= t) {
            self.desktop_due_us = Some(now.saturating_add(DESKTOP_POLL_US));
            self.poll_desktop();
        }
        for msg in self.camera.poll() {
            match msg {
                CameraMsg::Frame(f) => self.frame(f),
                CameraMsg::Opened(fmt) => {
                    let t = self.at(self.clock.now_us());
                    self.input(Input::CameraOpened(fmt), t);
                }
                CameraMsg::Failed(e) => {
                    let t = self.at(self.clock.now_us());
                    self.input(Input::CameraFailed(e), t);
                }
            }
        }
        let now = self.clock.now_us();
        if now.saturating_sub(self.last_frame_us) >= IDLE_TICK_US {
            self.tick();
        }
    }

    /// Observes the cursor and ticks at the clock's time, so deadlines fire without frames.
    pub fn tick(&mut self) {
        let t = self.at(self.clock.now_us());
        self.observe(t);
        self.input(Input::Tick, t);
    }

    /// The state for the UI, with the preview image rebuilt when a new frame arrived.
    pub fn snapshot(&mut self) -> Snapshot {
        let view = self.controller.view(self.clock.now_us().max(self.t_us));
        if view.frame_seq != self.preview_seq {
            self.preview_seq = view.frame_seq;
            self.preview = preview_image(&self.controller).map(Arc::new);
        }
        Snapshot {
            view,
            displays: self.controller.displays().to_vec(),
            primary: self.controller.primary_display(),
            preview: self.preview.clone(),
        }
    }

    fn frame(&mut self, f: CameraFrame) {
        let captured = f.timestamp_us;
        let t = self.at(captured);
        self.last_frame_us = self.clock.now_us();
        // Handover: the cursor as it is *before* this frame's move.
        self.observe(t);
        if let Some(r) = &mut self.recorder {
            r.frame(&f);
        }
        let received = self.clock.now_us();
        let t0 = Instant::now();
        let step = self.controller.handle(Input::Frame(f), t);
        let t1 = Instant::now();
        self.perform(step);
        if let Some(p) = &self.perf {
            p.lock().unwrap_or_else(PoisonError::into_inner).frame(
                received.saturating_sub(captured),
                t1 - t0,
                t1.elapsed(),
                self.clock.now_us().saturating_sub(captured),
            );
        }
    }

    fn input(&mut self, input: Input, t: u64) {
        let step = self.controller.handle(input, t);
        self.perform(step);
    }

    /// Re-checks pointer permission now (also the "Check again" button).
    fn check_access(&mut self) {
        self.access_due_us = Some(self.clock.now_us().saturating_add(ACCESS_POLL_US));
        let access = match (&self.pointer, &self.pointer_error) {
            (Some(p), _) => p.access(),
            (None, Some(PointerError::PermissionDenied)) => PointerAccess::Denied,
            (None, _) => PointerAccess::Unsupported,
        };
        self.input(Input::PointerAccess(access), self.t_us);
    }

    fn poll_desktop(&mut self) {
        let Some(p) = &mut self.pointer else { return };
        let input = match p.desktop() {
            Ok(d) if self.desktop.as_ref() != Some(&d) => {
                if let Some(r) = &mut self.recorder {
                    r.desktop(self.t_us, &d);
                }
                self.desktop = Some(d.clone());
                Input::DesktopChanged(d)
            }
            Ok(_) => return,
            Err(e) => Input::PointerFailed(e),
        };
        self.input(input, self.t_us);
    }

    fn observe(&mut self, t: u64) {
        if let Some(p) = &self.pointer {
            let at = p.position();
            self.input(Input::PointerObserved(at), t);
        }
    }

    /// Performs a step's effects, and those of any inputs they cause (bounded).
    fn perform(&mut self, mut step: Step) {
        for _ in 0..MAX_EFFECT_ROUNDS {
            if let Some(r) = &mut self.recorder {
                r.events(self.t_us, &step.events);
            }
            if step.effects.is_empty() {
                return;
            }
            let mut next = Step::default();
            for e in std::mem::take(&mut step.effects) {
                let failed = match e {
                    Effect::OpenCamera(request) => {
                        let device = self.controller.settings().camera.clone();
                        self.camera.open(device, request);
                        None
                    }
                    Effect::CloseCamera => {
                        self.camera.close();
                        None
                    }
                    Effect::MovePointer(p) => {
                        self.pointer.as_mut().and_then(|s| s.move_to(p).err())
                    }
                    Effect::Click(p) => self.pointer.as_mut().and_then(|s| s.click(p).err()),
                };
                if let Some(e) = failed {
                    let s = self.controller.handle(Input::PointerFailed(e), self.t_us);
                    next.events.extend(s.events);
                    next.effects.extend(s.effects);
                }
            }
            step = next;
        }
    }
}

/// The controller's latest canonical frame as an egui image: colour when the camera gave
/// colour, else grey.
fn preview_image(c: &Controller) -> Option<ColorImage> {
    if let Some(rgb) = c.preview_rgb() {
        Some(ColorImage::from_rgb(
            [rgb.width() as usize, rgb.height() as usize],
            rgb.as_bytes(),
        ))
    } else {
        c.preview().map(|luma| {
            ColorImage::from_gray(
                [luma.width() as usize, luma.height() as usize],
                luma.as_bytes(),
            )
        })
    }
}

// ---- engine thread ----

enum Msg {
    Cmd(EngineCmd),
    /// The camera has something: step.
    Camera,
    Quit,
}

/// Runs an [`Engine`] on its own thread (ADR-0016).
pub struct EngineThread {
    tx: Sender<Msg>,
    latest: Arc<Mutex<Arc<Snapshot>>>,
    thread: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for EngineThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineThread").finish_non_exhaustive()
    }
}

/// Makes the engine on its thread. The argument wakes the engine; give it to the camera.
pub type EngineFactory = Box<dyn FnOnce(Box<dyn Fn() + Send>) -> Engine<ThreadedCamera> + Send>;

impl EngineThread {
    /// Starts the thread and waits for its first snapshot. `repaint` is called whenever a
    /// new snapshot is published.
    pub fn spawn(make: EngineFactory, repaint: impl Fn() + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        let wake_tx = tx.clone();
        let wake: Box<dyn Fn() + Send> = Box::new(move || {
            let _ = wake_tx.send(Msg::Camera);
        });
        let (first_tx, first_rx) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("engine".into())
            .spawn(move || {
                let mut engine = make(wake);
                let snap = Arc::new(engine.snapshot());
                let latest = Arc::new(Mutex::new(snap));
                if first_tx.send(latest.clone()).is_ok() {
                    run(&mut engine, &rx, &latest, &repaint);
                }
            })
            .expect("spawn the engine thread");
        let latest = first_rx.recv().expect("the engine thread started");
        Self {
            tx,
            latest,
            thread: Some(thread),
        }
    }
}

fn run(
    engine: &mut Engine<ThreadedCamera>,
    rx: &Receiver<Msg>,
    latest: &Mutex<Arc<Snapshot>>,
    repaint: &dyn Fn(),
) {
    let mut published = engine.snapshot();
    loop {
        let first = match rx.recv_timeout(WAIT) {
            Ok(m) => Some(m),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => return,
        };
        for msg in first
            .into_iter()
            .chain(std::iter::from_fn(|| rx.try_recv().ok()))
        {
            match msg {
                Msg::Cmd(c) => engine.apply(c),
                Msg::Camera => {}
                Msg::Quit => return,
            }
        }
        engine.step();
        let snap = engine.snapshot();
        if snap.view != published.view
            || snap.displays != published.displays
            || snap.primary != published.primary
        {
            published = snap.clone();
            *latest.lock().unwrap_or_else(PoisonError::into_inner) = Arc::new(snap);
            repaint();
        }
    }
}

impl EngineHandle for EngineThread {
    fn send(&mut self, cmd: EngineCmd) {
        let _ = self.tx.send(Msg::Cmd(cmd));
    }

    fn snapshot(&mut self) -> Arc<Snapshot> {
        self.latest
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Drop for EngineThread {
    /// Stops the engine; dropping it closes the camera (`ThreadedCamera::drop`).
    fn drop(&mut self) {
        let _ = self.tx.send(Msg::Quit);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// The engine run inline, for UI tests: commands apply at once, and every snapshot ticks
/// first, as the engine thread would have meanwhile.
#[cfg(test)]
#[derive(Debug)]
pub struct InlineEngine<C: CameraLink>(pub Engine<C>);

#[cfg(test)]
impl<C: CameraLink> EngineHandle for InlineEngine<C> {
    fn send(&mut self, cmd: EngineCmd) {
        self.0.apply(cmd);
    }

    fn snapshot(&mut self) -> Arc<Snapshot> {
        self.0.tick();
        Arc::new(self.0.snapshot())
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
