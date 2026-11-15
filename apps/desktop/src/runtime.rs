//! The runtime (task 0008 D7): performs the controller's effects and feeds it what the
//! platform sees. Camera thread → controller → pointer.
//!
//! # Threads
//!
//! - The camera runs on its own worker thread ([`ThreadedCamera`]): opening can block (the
//!   macOS permission prompt, slow devices, P3) and `next_frame` blocks until a frame.
//!   Messages carry a session number, so frames or failures from a camera that was already
//!   closed are dropped. The worker wakes the UI (`request_repaint`) after each message.
//! - The controller and the pointer stay on the main thread: the macOS pointer isn't `Send`.
//!   [`Runtime::pump`] runs from `eframe::App::logic`, which eframe calls even while the
//!   window is minimised or hidden (then at most every 100 ms, eframe's throttle).
//!
//! # Order per pump
//!
//! 1. Effects queued by UI commands since the last pump (open camera, pause → centre, …).
//! 2. Pointer permission every [`ACCESS_POLL_US`] (≈10 ms on macOS, never per frame) and the
//!    desktop rectangle every [`DESKTOP_POLL_US`].
//! 3. Camera messages in order, frames coalesced to the newest. Before each frame the OS
//!    cursor is observed (handover), then the frame goes in and its move comes out.
//! 4. A `Tick`, so deadlines fire without frames.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

use app_core::ports::{
    CameraError, CameraFrame, CameraSource, CaptureFormat, CaptureRequest, Desktop, PointerError,
    PointerSink,
};
use app_core::{Effect, Input, PointerAccess};

use crate::app::Shell;
use crate::perf::Perf;
use crate::recorder::Recorder;

/// How often pointer permission is re-checked (a grant takes effect without a restart).
pub const ACCESS_POLL_US: u64 = 3_000_000;
/// How often the display layout is re-read (R-P5: monitors come and go).
pub const DESKTOP_POLL_US: u64 = 1_000_000;
/// Frames sent by the worker but not yet taken by [`CameraLink::poll`]. Beyond this the
/// worker drops new frames, so a stalled UI can't pile up memory.
const MAX_FRAMES_IN_FLIGHT: usize = 3;
/// Effects can cause inputs that cause effects (a failed move reports back); never loop forever.
const MAX_EFFECT_ROUNDS: usize = 8;

/// What the camera reports to the runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum CameraMsg {
    Opened(CaptureFormat),
    Failed(CameraError),
    Frame(CameraFrame),
}

/// The runtime's handle on a camera: commands go out, messages come back.
pub trait CameraLink {
    /// Opens `device` (`Settings::camera`), closing any open camera first.
    fn open(&mut self, device: Option<String>, request: CaptureRequest);
    fn close(&mut self);
    /// Messages of the current session since the last call, in order, frames coalesced to the
    /// newest between other messages.
    fn poll(&mut self) -> Vec<CameraMsg>;
    /// Running totals for diagnostics: frames dropped because the UI was behind, and frames
    /// coalesced away in `poll`.
    fn dropped(&self) -> (u64, u64) {
        (0, 0)
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

/// [`CameraLink`] over a worker thread that owns the camera.
pub struct ThreadedCamera {
    cmds: Sender<Cmd>,
    msgs: Receiver<(u64, CameraMsg)>,
    session: u64,
    in_flight: Arc<AtomicUsize>,
    dropped: Arc<AtomicU64>,
    coalesced: u64,
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
    /// `wake` is called after every message (the app requests a repaint with it).
    pub fn spawn(factory: CameraFactory, wake: impl Fn() + Send + 'static) -> Self {
        let (cmds, cmd_rx) = mpsc::channel();
        let (msg_tx, msgs) = mpsc::channel();
        let in_flight = Arc::new(AtomicUsize::new(0));
        let counter = in_flight.clone();
        let dropped = Arc::new(AtomicU64::new(0));
        let drops = dropped.clone();
        let worker = thread::Builder::new()
            .name("camera-worker".into())
            .spawn(move || worker(factory, &cmd_rx, &msg_tx, &counter, &drops, &wake))
            .ok();
        Self {
            cmds,
            msgs,
            session: 0,
            in_flight,
            dropped,
            coalesced: 0,
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
        let mut out = Vec::new();
        let mut frame = None;
        loop {
            match self.msgs.try_recv() {
                Ok((session, msg)) => {
                    if matches!(msg, CameraMsg::Frame(_)) {
                        self.in_flight.fetch_sub(1, Ordering::Relaxed);
                    }
                    if session != self.session {
                        continue;
                    }
                    match msg {
                        CameraMsg::Frame(_) => {
                            if frame.replace(msg).is_some() {
                                self.coalesced += 1;
                            }
                        }
                        other => {
                            out.extend(frame.take());
                            out.push(other);
                        }
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    out.extend(frame.take());
                    out.push(CameraMsg::Failed(CameraError::Backend(
                        "the camera thread stopped".into(),
                    )));
                    return out;
                }
            }
        }
        out.extend(frame);
        out
    }

    fn dropped(&self) -> (u64, u64) {
        (self.dropped.load(Ordering::Relaxed), self.coalesced)
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
    in_flight: &AtomicUsize,
    dropped: &AtomicU64,
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
                if in_flight.load(Ordering::Relaxed) >= MAX_FRAMES_IN_FLIGHT {
                    dropped.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                in_flight.fetch_add(1, Ordering::Relaxed);
                if !send(session, CameraMsg::Frame(f)) {
                    break;
                }
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

/// Performs effects and feeds observations. Generic over the camera link so tests can run
/// the whole loop synchronously.
pub struct Runtime<C: CameraLink> {
    camera: C,
    /// `None` when the platform has no pointer backend (the controller is told `Unsupported`).
    pointer: Option<Box<dyn PointerSink>>,
    pointer_error: Option<PointerError>,
    access_due_us: Option<u64>,
    desktop_due_us: Option<u64>,
    desktop: Option<Desktop>,
    recorder: Option<Recorder>,
    perf: Option<Perf>,
}

impl<C: CameraLink> std::fmt::Debug for Runtime<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
            .field("desktop", &self.desktop)
            .field("pointer_error", &self.pointer_error)
            .finish_non_exhaustive()
    }
}

impl<C: CameraLink> Runtime<C> {
    pub fn new(camera: C, pointer: Result<Box<dyn PointerSink>, PointerError>) -> Self {
        let (pointer, pointer_error) = match pointer {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e)),
        };
        Self {
            camera,
            pointer,
            pointer_error,
            access_due_us: None,
            desktop_due_us: None,
            desktop: None,
            recorder: None,
            perf: None,
        }
    }

    /// Records raw frames and controller events (dev / diagnostics, `recorder`).
    pub fn with_recorder(mut self, recorder: Option<Recorder>) -> Self {
        self.recorder = recorder;
        self
    }

    /// Frame-path timing (`OCM_PERF=1`, `perf`).
    pub fn with_perf(mut self, perf: Option<Perf>) -> Self {
        self.perf = perf;
        self
    }

    pub fn perf_mut(&mut self) -> Option<&mut Perf> {
        self.perf.as_mut()
    }

    /// One pass of the loop (module doc, "Order per pump").
    pub fn pump(&mut self, shell: &mut Shell) {
        if let Some(p) = &mut self.perf {
            let (dropped, coalesced) = self.camera.dropped();
            p.logic(dropped, coalesced);
        }
        self.perform(shell);
        let now = shell.now_us();
        if self.access_due_us.is_none_or(|t| now >= t) {
            self.check_access(shell);
        }
        if self.desktop_due_us.is_none_or(|t| now >= t) {
            self.desktop_due_us = Some(now.saturating_add(DESKTOP_POLL_US));
            self.poll_desktop(shell);
        }
        let mut observed = false;
        for msg in self.camera.poll() {
            match msg {
                CameraMsg::Frame(f) => {
                    // Handover: the cursor as it is *before* this frame's move.
                    self.observe(shell);
                    observed = true;
                    if let Some(r) = &mut self.recorder {
                        r.frame(&f);
                    }
                    let age_us = shell.now_us().saturating_sub(f.timestamp_us);
                    let t0 = std::time::Instant::now();
                    shell.input(Input::Frame(f));
                    let t1 = std::time::Instant::now();
                    self.perform(shell);
                    if let Some(p) = &mut self.perf {
                        p.frame(age_us, t1 - t0, t1.elapsed());
                    }
                    continue;
                }
                CameraMsg::Opened(fmt) => shell.input(Input::CameraOpened(fmt)),
                CameraMsg::Failed(e) => shell.input(Input::CameraFailed(e)),
            }
            self.perform(shell);
        }
        if !observed {
            // Keeps the handover inactivity timer honest when frames stall.
            self.observe(shell);
        }
        shell.input(Input::Tick);
        self.perform(shell);
    }

    /// Re-checks pointer permission now (also the "Check again" button).
    pub fn check_access(&mut self, shell: &mut Shell) {
        let now = shell.now_us();
        self.access_due_us = Some(now.saturating_add(ACCESS_POLL_US));
        let access = match (&self.pointer, &self.pointer_error) {
            (Some(p), _) => p.access(),
            (None, Some(PointerError::PermissionDenied)) => PointerAccess::Denied,
            (None, _) => PointerAccess::Unsupported,
        };
        shell.pointer_access(access);
        self.perform(shell);
    }

    fn poll_desktop(&mut self, shell: &mut Shell) {
        let Some(p) = &mut self.pointer else { return };
        match p.desktop() {
            Ok(d) if self.desktop.as_ref() != Some(&d) => {
                self.desktop = Some(d.clone());
                shell.input(Input::DesktopChanged(d));
            }
            Ok(_) => {}
            Err(e) => shell.input(Input::PointerFailed(e)),
        }
    }

    fn observe(&mut self, shell: &mut Shell) {
        if let Some(p) = &self.pointer {
            shell.input(Input::PointerObserved(p.position()));
        }
    }

    /// Performs effects queued by UI clicks in this pass, without polling anything.
    pub fn flush(&mut self, shell: &mut Shell) {
        self.perform(shell);
    }

    /// Performs queued effects until none are left (bounded).
    fn perform(&mut self, shell: &mut Shell) {
        if let Some(r) = &mut self.recorder {
            r.events(shell.now_us(), &shell.take_events());
        } else {
            shell.take_events();
        }
        for _ in 0..MAX_EFFECT_ROUNDS {
            let effects = shell.take_effects();
            if effects.is_empty() {
                return;
            }
            for e in effects {
                match e {
                    Effect::OpenCamera(request) => {
                        let device = shell.controller.settings().camera.clone();
                        self.camera.open(device, request);
                    }
                    Effect::CloseCamera => self.camera.close(),
                    Effect::MovePointer(p) => {
                        if let Some(sink) = &mut self.pointer
                            && let Err(e) = sink.move_to(p)
                        {
                            shell.input(Input::PointerFailed(e));
                        }
                    }
                    Effect::Click(p) => {
                        if let Some(sink) = &mut self.pointer
                            && let Err(e) = sink.click(p)
                        {
                            shell.input(Input::PointerFailed(e));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
