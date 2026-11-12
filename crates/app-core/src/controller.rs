//! The application state machine (BEHAVIOR_SPEC R-A1..R-A6, ADR-0006). Pure: no I/O, no
//! threads, no clock. Every call passes `now_us` (one monotonic clock owned by the runtime),
//! and every side effect comes back as an [`Effect`] for the runtime to perform.
//!
//! # States
//!
//! [`AppState`] is R-A1 plus `Error` (a camera backend failure that is none of the named
//! cases). Who drives the OS cursor is an orthogonal sub-state, [`PointerOwner`]: pointer
//! permission and mouse handover don't stop tracking, so they aren't top-level states
//! (decided in task 0008 D0, recorded in BEHAVIOR_SPEC R-A1).
//!
//! # Control handover (R-A2, 2018 §2.7)
//!
//! - Selecting a target by clicking the preview leaves the mouse in control
//!   ([`PointerOwner::Handover`]). Once the mouse has been still for 4 s the camera takes
//!   over: the current head position is mapped to the desktop centre and the cursor moves
//!   there (2018 `AutoCenterScheme 0`).
//! - The end of a recenter takes over at once, the same way (2018 forced start).
//! - While the camera is in control, an OS cursor more than 5 px from the last commanded
//!   position means the user grabbed the mouse: back to `Handover`.
//! - A loss releases control ([`PointerOwner::Mouse`]). With "Find the target again" on, a
//!   re-acquisition gives control back at once only if the cursor would move at most the
//!   R-P4 bound; otherwise it goes through `Handover`.

use pointer_core::handover::{
    HandoverParams, InactivityTimer, REACQUIRE_MAX_JUMP_FRAC, mouse_moved_away, within_jump_bound,
};
use pointer_core::{DesktopRect, FrameSize, Gain, map_absolute};
use tracking_core::lk::{LkParams, LkTracker};
use tracking_core::{
    Diagnostics, GrayFrame, LossReason, Point2, TrackError, TrackOutput, TrackStatus, Tracker,
    downsample_to_width, ingest,
};

use crate::ports::{CameraError, CameraFrame, CaptureFormat, CaptureRequest, PointerError};
use crate::settings::{Settings, SettingsError};

/// Guided recenter countdown (R-A3, user decision 2026-11-12: 5-4-3-2-1).
pub const RECENTER_US: u64 = 5_000_000;

/// Canonical display-space width cap (ADR-0008): wider frames are box-downsampled first.
pub const MAX_FRAME_WIDTH: u32 = 640;

/// Every app state is user-visible (R-A1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Stopped,
    /// The camera was asked to open; waiting for it.
    Starting,
    NoCamera,
    PermissionDenied,
    /// The camera failed in a way that is none of the cases above. Details in [`View::error`].
    Error,
    /// Preview running, nothing selected.
    NoTarget,
    /// A target is tracked. The tracker may be holding (`TrackStatus::Held`), see [`View`].
    Tracking,
    /// The target was lost. With "Find the target again" on and armed, the tracker searches.
    Lost,
    /// 5 s countdown with the centre circle; at 0 the frame centre becomes the target.
    Recentering,
    Paused,
    CameraDisconnected,
}

/// Who moves the OS cursor (sub-state of every [`AppState`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerOwner {
    /// The camera moves the cursor.
    Camera,
    /// The mouse has control; the camera takes over when it has been still for 4 s. The
    /// countdown is [`View::handover_remaining_us`].
    Handover,
    /// The mouse has control and the camera won't take over by itself (stopped, no target,
    /// lost, recentering, paused).
    Mouse,
    /// The OS denies pointer control (macOS Accessibility). Tracking still works.
    PermissionNeeded,
    /// Pointer injection is unsupported in this session (e.g. Wayland).
    Unsupported,
}

/// What the runtime knows about pointer-control permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerAccess {
    /// Not checked yet: moves are attempted, a failure reports back.
    Unknown,
    Granted,
    Denied,
    Unsupported,
}

/// Commands from the UI, the hotkeys (F11 = `ToggleStart`, F12 = `Recenter`) or a test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    /// Start (or retry) the camera; pause while running; resume from pause through recenter.
    ToggleStart,
    /// Start the guided recenter. Also ends a pause. Restarts a running countdown.
    Recenter,
    /// A preview click, already converted to canonical display-space frame coordinates
    /// through the drawing rect (R-C2).
    SelectTarget(Point2),
    /// Stop the camera and release everything.
    Stop,
}

/// Everything the controller reacts to.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Command(Command),
    /// A raw camera frame (native orientation; mirrored at ingest here, ADR-0008).
    Frame(CameraFrame),
    CameraOpened(CaptureFormat),
    CameraFailed(CameraError),
    /// The OS cursor position, observed after the previous `MovePointer` was applied. `None`
    /// if the platform can't report it (no handover detection then).
    PointerObserved(Option<Point2>),
    /// Result of a pointer-permission check.
    PointerAccess(PointerAccess),
    /// A `MovePointer` effect failed.
    PointerFailed(PointerError),
    DesktopChanged(DesktopRect),
    /// Time passed; lets deadlines (recenter, handover) fire without a frame.
    Tick,
}

/// Emitted for every change. A UI renders from these and [`View`], nothing else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    StateChanged {
        from: AppState,
        to: AppState,
    },
    PointerChanged {
        from: PointerOwner,
        to: PointerOwner,
    },
    TargetSelected {
        at: Point2,
    },
    /// A selection outside the frame (or with no frame yet) was refused.
    SelectionRejected {
        at: Point2,
    },
}

/// Side effects for the runtime to perform, in order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    /// Open the camera from `Settings::camera` with this request (closing any open one).
    OpenCamera(CaptureRequest),
    CloseCamera,
    /// Move the OS cursor (virtual-desktop coordinates).
    MovePointer(Point2),
}

/// The result of one [`Controller::handle`] call.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Step {
    pub events: Vec<Event>,
    pub effects: Vec<Effect>,
}

impl Step {
    /// The last pointer move in this step, if any.
    pub fn pointer_target(&self) -> Option<Point2> {
        self.effects.iter().rev().find_map(|e| match e {
            Effect::MovePointer(p) => Some(*p),
            _ => None,
        })
    }

    fn append(&mut self, other: Step) {
        self.events.extend(other.events);
        self.effects.extend(other.effects);
    }
}

/// How the UI draws the tracked point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerKind {
    /// Solid ring.
    Tracking,
    /// Dashed ring: the tracker holds the last position.
    Held,
    /// Lost, re-detection searching around the last position.
    Searching,
    /// Lost: grey ring + X at the last position.
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Marker {
    /// Display-space frame coordinates.
    pub position: Point2,
    pub kind: MarkerKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecenterView {
    /// The circle centre, display-space frame coordinates.
    pub center: Point2,
    pub remaining_us: u64,
}

impl RecenterView {
    /// The number shown in the badge: 5, 4, 3, 2, 1.
    pub fn seconds(&self) -> u64 {
        self.remaining_us.div_ceil(1_000_000).max(1)
    }
}

/// Everything the UI needs to render, computed for a given time.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub state: AppState,
    pub pointer: PointerOwner,
    /// Last tracker status while a target exists (Tracking or Lost).
    pub track: Option<TrackStatus>,
    pub marker: Option<Marker>,
    pub recenter: Option<RecenterView>,
    /// Time left before the camera takes over, while [`PointerOwner::Handover`].
    pub handover_remaining_us: Option<u64>,
    /// Size of the canonical frame; the preview texture and click mapping use it (R-C2).
    pub frame_size: Option<(u32, u32)>,
    /// Increments on every ingested frame, so the UI knows when to re-upload the texture.
    pub frame_seq: u64,
    pub diagnostics: Diagnostics,
    pub loss_recovery: bool,
    /// Re-detection is armed for the current selection (ADR-0011). `false` while lost with
    /// `loss_recovery` on means "can't find this spot again, re-select" (lab `lost-noauto`).
    pub redetect_armed: bool,
    pub capture: Option<CaptureFormat>,
    /// Last camera or pointer error text.
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Control {
    Camera,
    /// `None`: no automatic takeover.
    Mouse(Option<InactivityTimer>),
}

/// The controller. Owns the tracker (ADR-0011) and the pointer mapping state.
#[derive(Debug)]
pub struct Controller {
    settings: Settings,
    desktop: DesktopRect,
    handover: HandoverParams,
    state: AppState,
    control: Control,
    access: PointerAccess,
    tracker: LkTracker,
    frame: Option<GrayFrame>,
    frame_seq: u64,
    track: Option<TrackOutput>,
    recenter_deadline_us: Option<u64>,
    /// Image-space offset of the absolute mapping (ADR-0006), reset on camera takeover.
    offset: Point2,
    last_commanded: Option<Point2>,
    last_observed: Option<Point2>,
    capture: Option<CaptureFormat>,
    error: Option<String>,
}

/// Tracker parameters for the settings: the recommended tracker, with re-detection only when
/// "Find the target again" is on (ADR-0011 amendment).
pub fn tracker_params(settings: &Settings) -> LkParams {
    let mut p = LkParams::recommended();
    if !settings.loss_recovery {
        p.redetect = None;
    }
    p
}

fn frame_size(f: &GrayFrame) -> FrameSize {
    FrameSize {
        width: f.width() as f32,
        height: f.height() as f32,
    }
}

fn frame_center(f: &GrayFrame) -> Point2 {
    Point2::new(f.width() as f32 / 2.0, f.height() as f32 / 2.0)
}

impl Controller {
    /// `settings` are validated here too (R-A5).
    pub fn new(settings: Settings, desktop: DesktopRect) -> Result<Self, SettingsError> {
        let settings = settings.validated()?;
        Ok(Self {
            tracker: LkTracker::new(tracker_params(&settings)),
            settings,
            desktop,
            handover: HandoverParams::default(),
            state: AppState::Stopped,
            control: Control::Mouse(None),
            access: PointerAccess::Unknown,
            frame: None,
            frame_seq: 0,
            track: None,
            recenter_deadline_us: None,
            offset: Point2::default(),
            last_commanded: None,
            last_observed: None,
            capture: None,
            error: None,
        })
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The latest canonical (mirrored, ≤ 640 wide) frame for the preview.
    pub fn preview(&self) -> Option<&GrayFrame> {
        self.frame.as_ref()
    }

    pub fn pointer_owner(&self) -> PointerOwner {
        match self.access {
            PointerAccess::Denied => PointerOwner::PermissionNeeded,
            PointerAccess::Unsupported => PointerOwner::Unsupported,
            PointerAccess::Unknown | PointerAccess::Granted => match self.control {
                Control::Camera => PointerOwner::Camera,
                Control::Mouse(Some(_)) => PointerOwner::Handover,
                Control::Mouse(None) => PointerOwner::Mouse,
            },
        }
    }

    /// Applies one input at time `now_us`. Returns the events and effects it caused, in
    /// order. No transition is silent (R-A1).
    pub fn handle(&mut self, input: Input, now_us: u64) -> Step {
        let owner_before = self.pointer_owner();
        let mut step = Step::default();
        self.fire_deadlines(now_us, &mut step);
        match input {
            Input::Command(c) => self.on_command(c, now_us, &mut step),
            Input::Frame(f) => self.on_frame(f, now_us, &mut step),
            Input::CameraOpened(fmt) => {
                self.capture = Some(fmt);
                if self.state == AppState::Starting {
                    self.error = None;
                    self.set_state(AppState::NoTarget, &mut step);
                }
            }
            Input::CameraFailed(e) => self.on_camera_failed(e, &mut step),
            Input::PointerObserved(p) => self.on_pointer_observed(p, now_us),
            Input::PointerAccess(a) => self.access = a,
            Input::PointerFailed(e) => {
                match e {
                    PointerError::PermissionDenied => self.access = PointerAccess::Denied,
                    PointerError::Unsupported(_) => self.access = PointerAccess::Unsupported,
                    PointerError::Backend(_) => {}
                }
                self.error = Some(e.to_string());
            }
            Input::DesktopChanged(d) => self.desktop = d,
            Input::Tick => {}
        }
        // A deadline may have become due through this input (e.g. the first frame after the
        // recenter deadline).
        self.fire_deadlines(now_us, &mut step);
        let owner_after = self.pointer_owner();
        if owner_after != owner_before {
            step.events.push(Event::PointerChanged {
                from: owner_before,
                to: owner_after,
            });
        }
        step
    }

    /// Replaces the settings after validating them (R-A5). A change that invalidates the
    /// tracker re-initialises it (R-A6): mirroring flips display space, so the target is
    /// dropped; "Find the target again" rebuilds the tracker and re-selects the current
    /// point; a camera change reopens the camera.
    pub fn update_settings(&mut self, new: Settings, now_us: u64) -> Result<Step, SettingsError> {
        let new = new.validated()?;
        let old = std::mem::replace(&mut self.settings, new);
        let owner_before = self.pointer_owner();
        let mut step = Step::default();
        self.fire_deadlines(now_us, &mut step);

        if old.camera != self.settings.camera && self.camera_active() {
            self.drop_target();
            self.frame = None;
            step.effects.push(Effect::CloseCamera);
            step.effects
                .push(Effect::OpenCamera(CaptureRequest::default()));
            self.set_state(AppState::Starting, &mut step);
        } else if old.mirror != self.settings.mirror {
            // Stale frame is in the old orientation; never select on it.
            self.frame = None;
            if matches!(self.state, AppState::Tracking | AppState::Lost) {
                self.drop_target();
                self.set_state(AppState::NoTarget, &mut step);
            }
        }
        if old.loss_recovery != self.settings.loss_recovery {
            self.rebuild_tracker(&mut step);
        }

        let owner_after = self.pointer_owner();
        if owner_after != owner_before {
            step.events.push(Event::PointerChanged {
                from: owner_before,
                to: owner_after,
            });
        }
        Ok(step)
    }

    /// Render data at `now_us`.
    pub fn view(&self, now_us: u64) -> View {
        let has_target = matches!(self.state, AppState::Tracking | AppState::Lost);
        let track = self.track.filter(|_| has_target);
        let marker = track.map(|t| Marker {
            position: t.position,
            kind: match t.status {
                TrackStatus::Tracking => MarkerKind::Tracking,
                TrackStatus::Held(_) => MarkerKind::Held,
                TrackStatus::Lost(_) if self.tracker.redetect_armed() => MarkerKind::Searching,
                TrackStatus::Lost(_) => MarkerKind::Lost,
            },
        });
        let recenter = match (self.state, self.recenter_deadline_us) {
            (AppState::Recentering, Some(d)) => Some(RecenterView {
                center: self
                    .frame
                    .as_ref()
                    .map(frame_center)
                    .unwrap_or(Point2::new(320.0, 240.0)),
                remaining_us: d.saturating_sub(now_us),
            }),
            _ => None,
        };
        let handover_remaining_us = match (self.pointer_owner(), self.control) {
            (PointerOwner::Handover, Control::Mouse(Some(t))) => Some(t.remaining_us(now_us)),
            _ => None,
        };
        View {
            state: self.state,
            pointer: self.pointer_owner(),
            track: track.map(|t| t.status),
            marker,
            recenter,
            handover_remaining_us,
            frame_size: self.frame.as_ref().map(|f| (f.width(), f.height())),
            frame_seq: self.frame_seq,
            diagnostics: track.map(|t| t.diagnostics).unwrap_or_default(),
            loss_recovery: self.settings.loss_recovery,
            redetect_armed: self.tracker.redetect_armed(),
            capture: self.capture,
            error: self.error.clone(),
        }
    }

    // ---- inputs ----

    fn on_command(&mut self, c: Command, now_us: u64, step: &mut Step) {
        use AppState as S;
        match c {
            Command::ToggleStart => match self.state {
                S::Stopped
                | S::NoCamera
                | S::PermissionDenied
                | S::Error
                | S::CameraDisconnected => self.start_camera(step),
                S::Starting => {}
                S::NoTarget | S::Tracking | S::Lost | S::Recentering => self.pause(step),
                // R-A3: resume always goes through recenter.
                S::Paused => self.start_recenter(now_us, step),
            },
            Command::Recenter => match self.state {
                S::NoTarget | S::Tracking | S::Lost | S::Recentering | S::Paused => {
                    self.start_recenter(now_us, step)
                }
                _ => {}
            },
            Command::SelectTarget(p) => match self.state {
                S::NoTarget | S::Tracking | S::Lost => self.select(p, now_us, step),
                _ => {}
            },
            Command::Stop => {
                if self.state != S::Stopped {
                    self.drop_target();
                    self.frame = None;
                    self.capture = None;
                    step.effects.push(Effect::CloseCamera);
                    self.set_state(S::Stopped, step);
                }
            }
        }
    }

    fn on_frame(&mut self, raw: CameraFrame, now_us: u64, step: &mut Step) {
        use AppState as S;
        match self.state {
            // Frames prove the camera is open even if `CameraOpened` hasn't arrived.
            S::Starting => self.set_state(S::NoTarget, step),
            S::NoTarget | S::Tracking | S::Lost | S::Recentering | S::Paused => {}
            S::Stopped | S::NoCamera | S::PermissionDenied | S::Error | S::CameraDisconnected => {
                return;
            }
        }
        let frame = ingest(
            downsample_to_width(raw.luma, MAX_FRAME_WIDTH),
            self.settings.mirror,
        );
        self.frame_seq += 1;
        self.frame = Some(frame);
        match self.state {
            S::Tracking => self.track_step(now_us, step),
            // Without re-detection there's nothing to step: keep the loss reason on show.
            S::Lost if self.tracker.redetect_armed() => self.track_step(now_us, step),
            _ => {}
        }
    }

    fn on_camera_failed(&mut self, e: CameraError, step: &mut Step) {
        if self.state == AppState::Stopped {
            return;
        }
        let to = match &e {
            CameraError::NoDevice => AppState::NoCamera,
            CameraError::PermissionDenied => AppState::PermissionDenied,
            CameraError::Disconnected => AppState::CameraDisconnected,
            CameraError::Backend(_) => AppState::Error,
        };
        self.error = Some(e.to_string());
        self.drop_target();
        // Keep the last frame: the disconnected preview shows it frozen.
        step.effects.push(Effect::CloseCamera);
        self.set_state(to, step);
    }

    fn on_pointer_observed(&mut self, p: Option<Point2>, now_us: u64) {
        self.last_observed = p;
        let Some(p) = p else { return };
        match &mut self.control {
            Control::Camera => {
                if let Some(c) = self.last_commanded
                    && mouse_moved_away(c, p, &self.handover)
                {
                    let mut t = InactivityTimer::start(now_us, &self.handover);
                    t.observe(p, now_us, &self.handover);
                    self.control = Control::Mouse(Some(t));
                    self.last_commanded = None;
                }
            }
            Control::Mouse(Some(t)) => t.observe(p, now_us, &self.handover),
            Control::Mouse(None) => {}
        }
    }

    // ---- transitions ----

    fn set_state(&mut self, to: AppState, step: &mut Step) {
        if to != self.state {
            let from = std::mem::replace(&mut self.state, to);
            step.events.push(Event::StateChanged { from, to });
        }
        if to != AppState::Recentering {
            self.recenter_deadline_us = None;
        }
    }

    fn camera_active(&self) -> bool {
        !matches!(
            self.state,
            AppState::Stopped
                | AppState::NoCamera
                | AppState::PermissionDenied
                | AppState::Error
                | AppState::CameraDisconnected
        )
    }

    fn start_camera(&mut self, step: &mut Step) {
        self.error = None;
        step.effects
            .push(Effect::OpenCamera(CaptureRequest::default()));
        self.set_state(AppState::Starting, step);
    }

    /// Drops the target and releases the pointer to the mouse.
    fn drop_target(&mut self) {
        self.tracker.reset();
        self.track = None;
        self.control = Control::Mouse(None);
        self.last_commanded = None;
    }

    /// 2018 pause: release control, invalidate tracking, cursor to desktop centre.
    fn pause(&mut self, step: &mut Step) {
        self.drop_target();
        self.move_pointer(self.desktop.center(), step);
        self.last_commanded = None;
        self.set_state(AppState::Paused, step);
    }

    fn start_recenter(&mut self, now_us: u64, step: &mut Step) {
        self.drop_target();
        self.set_state(AppState::Recentering, step);
        self.recenter_deadline_us = Some(now_us.saturating_add(RECENTER_US));
    }

    fn select(&mut self, p: Point2, now_us: u64, step: &mut Step) {
        let Some(frame) = &self.frame else {
            step.events.push(Event::SelectionRejected { at: p });
            return;
        };
        match self.tracker.select(frame, p) {
            Ok(()) => {
                self.track = Some(TrackOutput {
                    position: p,
                    status: TrackStatus::Tracking,
                    diagnostics: Diagnostics::default(),
                });
                // The user just used the mouse to click: the camera takes over once it is
                // still (R-A2), not immediately.
                let mut t = InactivityTimer::start(now_us, &self.handover);
                if let Some(o) = self.last_observed {
                    t.observe(o, now_us, &self.handover);
                }
                self.control = Control::Mouse(Some(t));
                self.last_commanded = None;
                step.events.push(Event::TargetSelected { at: p });
                self.set_state(AppState::Tracking, step);
            }
            Err(_) => step.events.push(Event::SelectionRejected { at: p }),
        }
    }

    fn rebuild_tracker(&mut self, step: &mut Step) {
        self.tracker = LkTracker::new(tracker_params(&self.settings));
        if self.state == AppState::Tracking
            && let (Some(frame), Some(t)) = (&self.frame, self.track)
            && self.tracker.select(frame, t.position).is_ok()
        {
            return;
        }
        if self.state == AppState::Tracking {
            self.drop_target();
            self.set_state(AppState::NoTarget, step);
        }
        // Lost stays Lost; the new tracker isn't searching, so the user re-selects.
    }

    fn track_step(&mut self, now_us: u64, step: &mut Step) {
        let Some(frame) = &self.frame else { return };
        let out = match self.tracker.step(frame) {
            Ok(out) => out,
            Err(TrackError::FrameSizeChanged { .. } | TrackError::SelectionOutOfFrame { .. }) => {
                self.drop_target();
                self.set_state(AppState::NoTarget, step);
                return;
            }
        };
        let was_lost = self.state == AppState::Lost;
        match out.status {
            TrackStatus::Lost(reason) => {
                // While searching, keep the reason of the loss itself on show.
                let keep_reason = was_lost && reason == LossReason::Searching;
                let status = match (keep_reason, self.track) {
                    (true, Some(prev)) => prev.status,
                    _ => out.status,
                };
                self.track = Some(TrackOutput { status, ..out });
                // `last_commanded` is kept: it is the R-P4 reference if the cursor position
                // can't be observed.
                self.control = Control::Mouse(None);
                self.set_state(AppState::Lost, step);
            }
            TrackStatus::Tracking | TrackStatus::Held(_) => {
                self.track = Some(out);
                if was_lost {
                    self.reacquired(out.position, now_us, step);
                } else if self.control == Control::Camera && out.status == TrackStatus::Tracking {
                    let target = self.map(out.position);
                    self.move_pointer(target, step);
                }
            }
        }
    }

    /// Re-detection found the target (R-P4): control comes back at once only if the cursor
    /// would jump at most the bound; otherwise through the normal handover.
    fn reacquired(&mut self, position: Point2, now_us: u64, step: &mut Step) {
        self.set_state(AppState::Tracking, step);
        let target = self.map(position);
        let cursor = self.last_observed.or(self.last_commanded);
        if cursor
            .is_none_or(|c| within_jump_bound(c, target, self.desktop, REACQUIRE_MAX_JUMP_FRAC))
        {
            self.control = Control::Camera;
            self.move_pointer(target, step);
        } else {
            let mut t = InactivityTimer::start(now_us, &self.handover);
            if let Some(o) = cursor {
                t.observe(o, now_us, &self.handover);
            }
            self.control = Control::Mouse(Some(t));
        }
    }

    fn fire_deadlines(&mut self, now_us: u64, step: &mut Step) {
        if self.state == AppState::Recentering
            && self.recenter_deadline_us.is_some_and(|d| now_us >= d)
        {
            let mut s = Step::default();
            self.finish_recenter(&mut s);
            step.append(s);
        }
        if let Control::Mouse(Some(t)) = self.control
            && t.expired(now_us)
            && self.state == AppState::Tracking
            && let Some(track) = self.track
        {
            self.take_over(track.position, step);
        }
    }

    /// At 0 the point inside the circle (the frame centre) becomes the target and the camera
    /// takes control (R-A3). Without a frame yet, wait for the next one.
    fn finish_recenter(&mut self, step: &mut Step) {
        let Some(frame) = &self.frame else { return };
        let c = frame_center(frame);
        if self.tracker.select(frame, c).is_err() {
            return;
        }
        self.track = Some(TrackOutput {
            position: c,
            status: TrackStatus::Tracking,
            diagnostics: Diagnostics::default(),
        });
        step.events.push(Event::TargetSelected { at: c });
        self.set_state(AppState::Tracking, step);
        self.take_over(c, step);
    }

    /// Camera takes control (2018 `AutoCenterScheme 0`): `position` maps to the desktop
    /// centre and the cursor moves there.
    fn take_over(&mut self, position: Point2, step: &mut Step) {
        if let Some(frame) = &self.frame {
            let c = frame_center(frame);
            self.offset = Point2::new(c.x - position.x, c.y - position.y);
        }
        self.control = Control::Camera;
        self.move_pointer(self.desktop.center(), step);
    }

    fn map(&self, p: Point2) -> Point2 {
        let Some(frame) = &self.frame else {
            return self.desktop.center();
        };
        map_absolute(
            p,
            self.offset,
            frame_size(frame),
            self.desktop,
            Gain {
                x: self.settings.gain_x,
                y: self.settings.gain_y,
            },
        )
    }

    fn move_pointer(&mut self, p: Point2, step: &mut Step) {
        if matches!(
            self.access,
            PointerAccess::Denied | PointerAccess::Unsupported
        ) {
            return;
        }
        self.last_commanded = Some(p);
        step.effects.push(Effect::MovePointer(p));
    }
}
