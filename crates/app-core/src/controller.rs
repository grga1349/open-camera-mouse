//! The application state machine (BEHAVIOR.md R-A1..R-A6, ADR-0006). Pure: no I/O, no
//! threads, no clock. Every call passes `now_us` (one monotonic clock owned by the runtime),
//! and every side effect comes back as an [`Effect`] for the runtime to perform.
//!
//! # States
//!
//! [`AppState`] is R-A1 plus `Error` (a camera backend failure that is none of the named
//! cases). Who drives the OS cursor is an orthogonal sub-state, [`PointerOwner`]: pointer
//! permission and mouse handover don't stop tracking, so they aren't top-level states
//! (decided in task 0008 D0, recorded in BEHAVIOR.md R-A1).
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
//!
//! # Pointer pipeline (task 0008 D1)
//!
//! Each tracked frame while the camera is in control: absolute map (ADR-0006) → dt-aware
//! smoothing (R-P2) → screen limits, all edges including the desktop origin → offset drift
//! while pinned → dwell (R-K1, only when enabled). Dwell never runs while the target is
//! held or lost or the mouse has control (R-K2): those reset it.

use pointer_core::dwell::{Dwell, DwellParams};
use pointer_core::handover::{
    HandoverParams, InactivityTimer, REACQUIRE_MAX_JUMP_FRAC, mouse_moved_away, within_jump_bound,
};
use pointer_core::limits::{Limits, apply_limits, bounds, drift_offset};
use pointer_core::one_euro::{OneEuro, OneEuroParams};
use pointer_core::smoothing::Smoother;
use pointer_core::steady::{SteadyZone, approach_edge};
use pointer_core::{DesktopRect, FrameSize, Gain, map_raw_scaled, scale_reference};
use tracking_core::lk::{LkParams, LkTracker};
use tracking_core::{
    Diagnostics, GrayFrame, LossReason, Point2, TrackError, TrackOutput, TrackStatus, Tracker,
    downsample_to_width, ingest,
};

use crate::color::{RgbFrame, downsample_rgb_to_width, ingest_rgb};
use crate::ports::{
    CameraError, CameraFrame, CaptureFormat, CaptureRequest, Desktop, PointerError,
};
use crate::settings::{MainMonitor, Settings, SettingsError};

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
    DesktopChanged(Desktop),
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
    /// A dwell completed: left-click at this position (R-K1). Always after the move.
    Click(Point2),
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
    /// Progress `0..=1` of a running dwell, when dwell clicking is on and armed.
    pub dwell_progress: Option<f32>,
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
    /// The displays, sorted left to right then top to bottom (the "Main monitor" order).
    displays: Vec<DesktopRect>,
    /// Index into `displays` of the OS primary display (#14).
    primary: Option<usize>,
    handover: HandoverParams,
    state: AppState,
    control: Control,
    access: PointerAccess,
    tracker: LkTracker,
    frame: Option<GrayFrame>,
    /// Colour twin of `frame` for the preview, same canonical space and size.
    color: Option<RgbFrame>,
    frame_seq: u64,
    track: Option<TrackOutput>,
    recenter_deadline_us: Option<u64>,
    /// Image-space offset of the absolute mapping (ADR-0006), reset on camera takeover.
    offset: Point2,
    last_commanded: Option<Point2>,
    last_observed: Option<Point2>,
    /// Camera-space 1€ filter on the tracked point, before mapping (ADR-0017).
    one_euro: OneEuro,
    smoother: Smoother,
    steady: SteadyZone,
    /// Steady-zone radius in camera px of head movement ([`STEADY_ZONE_CAM_PX`]).
    steady_cam_px: f32,
    dwell: Dwell,
    capture: Option<CaptureFormat>,
    error: Option<String>,
}

/// Steady zone (test drive #5, #11, #13; ADR-0015/-0017 amendments): head movement smaller than
/// this, in camera px, doesn't move the cursor. The zone in camera px gives the same result at
/// every gain. A hold costs precision: the cursor sits up to the radius from where the head
/// points. 0.2 chosen on the device (user, Phase 6d) from
/// the S1 sweep: 0.1 visible (≥ 2 screen px) reversals/s at rest, hold error 0.26 cam px vs
/// 0.43 at the former 0.5, lag ≈ 30 ms lower.
pub const STEADY_ZONE_CAM_PX: f32 = 0.2;

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

fn limits(settings: &Settings) -> Limits {
    let l = settings.limits;
    Limits {
        left: l.left as f32,
        right: l.right as f32,
        top: l.top as f32,
        bottom: l.bottom as f32,
    }
}

fn dwell_params(settings: &Settings) -> DwellParams {
    let d = settings.dwell;
    let us = |s: f32| (f64::from(s) * 1e6).round() as u64;
    DwellParams {
        dwell_us: us(d.dwell_s),
        radius_frac: d.radius_frac,
    }
}

/// Left to right, then top to bottom: the order a user reads their monitors in.
fn sorted_displays(mut d: Vec<DesktopRect>) -> Vec<DesktopRect> {
    d.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    d
}

/// The primary display's index in the sorted list: the adapter's report, otherwise the
/// display at the global origin (the OS primary on macOS and Windows by definition).
fn primary_index(d: &Desktop, sorted: &[DesktopRect]) -> Option<usize> {
    let at_origin = || d.displays.iter().find(|r| r.x == 0.0 && r.y == 0.0);
    let rect = d
        .primary
        .and_then(|i| d.displays.get(i))
        .or_else(at_origin)?;
    sorted.iter().position(|r| r == rect)
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
            displays: Vec::new(),
            primary: None,
            handover: HandoverParams::default(),
            state: AppState::Stopped,
            control: Control::Mouse(None),
            access: PointerAccess::Unknown,
            frame: None,
            color: None,
            frame_seq: 0,
            track: None,
            recenter_deadline_us: None,
            offset: Point2::default(),
            last_commanded: None,
            last_observed: None,
            one_euro: OneEuro::default(),
            smoother: Smoother::default(),
            steady: SteadyZone::default(),
            steady_cam_px: STEADY_ZONE_CAM_PX,
            dwell: Dwell::default(),
            capture: None,
            error: None,
        })
    }

    /// Overrides the steady-zone radius in camera px (≥ 0; 0 = off). Not a user setting
    /// (test drive #11): for tests.
    pub fn set_steady_zone_cam_px(&mut self, cam_px: f32) {
        self.steady_cam_px = if cam_px.is_finite() {
            cam_px.max(0.0)
        } else {
            0.0
        };
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    /// The last tracker output while a target exists (Tracking or Lost), as [`View::track`]
    /// shows it: while searching, the status keeps the reason of the loss itself. Cheap, for
    /// per-frame diagnostics.
    pub fn track(&self) -> Option<TrackOutput> {
        let has_target = matches!(self.state, AppState::Tracking | AppState::Lost);
        self.track.filter(|_| has_target)
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The displays in "Main monitor" order (left to right, then top to bottom).
    pub fn displays(&self) -> &[DesktopRect] {
        &self.displays
    }

    /// The OS primary display's index in [`Self::displays`], if known (#14).
    pub fn primary_display(&self) -> Option<usize> {
        self.primary
    }

    /// The "Main monitor" in effect, as an index into [`Self::displays`]: the primary display,
    /// the chosen one, or `None` for the centre of all (chosen, or the display is gone).
    pub fn main_display(&self) -> Option<usize> {
        match self.settings.recenter_display {
            MainMonitor::Primary => self.primary,
            MainMonitor::All => None,
            MainMonitor::Display(i) => Some(i as usize).filter(|&i| i < self.displays.len()),
        }
    }

    /// Where recenter and pause put the cursor: the main display's centre, or the centre of
    /// the whole desktop (2018) when it is "all" or unknown (ADR-0014).
    pub fn home(&self) -> Point2 {
        self.main_display()
            .and_then(|i| self.displays.get(i))
            .map_or_else(|| self.desktop.center(), DesktopRect::center)
    }

    /// The latest canonical (mirrored, ≤ 640 wide) luma frame, the one the tracker sees.
    pub fn preview(&self) -> Option<&GrayFrame> {
        self.frame.as_ref()
    }

    /// The colour image of the latest canonical frame, for the preview texture. Same size
    /// and orientation as [`Self::preview`]; `None` if the camera gave no colour.
    pub fn preview_rgb(&self) -> Option<&RgbFrame> {
        self.color.as_ref()
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
        // An observation was taken before anything this call commands: firing a due takeover
        // first would compare it with the takeover's own move and hand control straight back
        // to the mouse. Deadlines still fire after it, below.
        if !matches!(input, Input::PointerObserved(_)) {
            self.fire_deadlines(now_us, &mut step);
        }
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
            Input::DesktopChanged(d) => {
                self.desktop = d.bounds;
                let sorted = sorted_displays(d.displays.clone());
                self.primary = primary_index(&d, &sorted);
                self.displays = sorted;
            }
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
            self.clear_frame();
            step.effects.push(Effect::CloseCamera);
            step.effects
                .push(Effect::OpenCamera(CaptureRequest::default()));
            self.set_state(AppState::Starting, &mut step);
        } else if old.mirror != self.settings.mirror {
            // Stale frame is in the old orientation; never select on it.
            self.clear_frame();
            if matches!(self.state, AppState::Tracking | AppState::Lost) {
                self.drop_target();
                self.set_state(AppState::NoTarget, &mut step);
            }
        }
        if old.loss_recovery != self.settings.loss_recovery {
            // In place: the selection is kept, so a loss is searched for once it's on
            // (ADR-0010 amendment 2026-11-25).
            self.tracker
                .set_redetect(tracker_params(&self.settings).redetect);
        }
        if old.dwell != self.settings.dwell {
            self.dwell.reset();
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
        let track = self.track();
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
        let dwell_progress = (self.settings.dwell.enabled
            && self.state == AppState::Tracking
            && self.pointer_owner() == PointerOwner::Camera)
            .then(|| self.dwell.progress(now_us, &dwell_params(&self.settings)))
            .flatten();
        View {
            state: self.state,
            pointer: self.pointer_owner(),
            track: track.map(|t| t.status),
            marker,
            recenter,
            handover_remaining_us,
            dwell_progress,
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
                    self.clear_frame();
                    self.capture = None;
                    step.effects.push(Effect::CloseCamera);
                    self.set_state(S::Stopped, step);
                }
            }
        }
    }

    fn on_frame(&mut self, raw: CameraFrame, now_us: u64, step: &mut Step) {
        use AppState as S;
        let (raw_w, raw_h) = (raw.luma.width(), raw.luma.height());
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
        // A colour image that doesn't match the luma would misalign the preview: drop it.
        self.color = raw
            .rgb
            .filter(|c| (c.width(), c.height()) == (raw_w, raw_h))
            .map(|c| {
                ingest_rgb(
                    downsample_rgb_to_width(c, MAX_FRAME_WIDTH),
                    self.settings.mirror,
                )
            });
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
                    self.dwell.reset();
                }
            }
            Control::Mouse(Some(t)) => t.observe(p, now_us, &self.handover),
            Control::Mouse(None) => {}
        }
    }

    // ---- transitions ----

    fn clear_frame(&mut self) {
        self.frame = None;
        self.color = None;
    }

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
        self.smoother.reset();
        self.one_euro.reset();
        self.steady.reset();
        self.dwell.reset();
    }

    /// 2018 pause: release control, invalidate tracking, cursor to the home point (desktop
    /// centre unless "Main monitor" picks a display).
    fn pause(&mut self, step: &mut Step) {
        self.drop_target();
        self.move_pointer(self.home(), step);
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
                self.dwell.reset();
                self.set_state(AppState::Lost, step);
            }
            TrackStatus::Tracking | TrackStatus::Held(_) => {
                self.track = Some(out);
                if was_lost {
                    self.reacquired(out.position, now_us, step);
                } else if self.control == Control::Camera && out.status == TrackStatus::Tracking {
                    self.drive(out.position, now_us, step);
                } else {
                    // Held, or the mouse has control: no dwell (R-K2).
                    self.dwell.reset();
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
            self.smoother.reset_to(target, now_us);
            self.one_euro.reset();
            self.steady.reset_to(target);
            self.dwell.reset();
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
            self.finish_recenter(now_us, &mut s);
            step.append(s);
        }
        if let Control::Mouse(Some(t)) = self.control
            && t.expired(now_us)
            && self.state == AppState::Tracking
            && let Some(track) = self.track
        {
            self.take_over(track.position, now_us, step);
        }
    }

    /// At 0 the point inside the circle (the frame centre) becomes the target and the camera
    /// takes control (R-A3). Without a frame yet, wait for the next one.
    fn finish_recenter(&mut self, now_us: u64, step: &mut Step) {
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
        self.take_over(c, now_us, step);
    }

    /// Camera takes control (2018 `AutoCenterScheme 0`): `position` maps to the home point
    /// (the desktop centre, or the "Main monitor" display's centre, ADR-0014) and the cursor
    /// moves there. The mapping still spans the whole desktop: only the offset shifts.
    fn take_over(&mut self, position: Point2, now_us: u64, step: &mut Step) {
        let home = self.home();
        if let Some(frame) = &self.frame {
            let c = frame_center(frame);
            let size = frame_size(frame);
            let d = self.desktop.center();
            // Frame px per screen px of the mapping (inverse of `map_raw`'s scale).
            let inv = |gain: f32, screen: f32, frame: f32| {
                let s = gain * screen / frame;
                if s > 0.0 { 1.0 / s } else { 0.0 }
            };
            let r = self.scale_reference();
            let ix = inv(self.settings.gain_x, r.width, size.width);
            let iy = inv(self.settings.gain_y, r.height, size.height);
            self.offset = Point2::new(
                c.x - position.x + (home.x - d.x) * ix,
                c.y - position.y + (home.y - d.y) * iy,
            );
        }
        self.control = Control::Camera;
        self.smoother.reset_to(home, now_us);
        self.one_euro.reset();
        self.steady.reset_to(home);
        self.dwell.reset();
        self.move_pointer(home, step);
    }

    /// The steady-zone radius in screen px: [`STEADY_ZONE_CAM_PX`] of head movement times the
    /// mapping's screen px per camera px on the faster axis, so it grows with the speed and
    /// the Main monitor's size (test drive #11).
    fn steady_radius(&self) -> f32 {
        self.scale()
            .map_or(0.0, |(sx, sy)| self.steady_cam_px * sx.max(sy))
    }

    /// Screen px per camera px on each axis (gain × Main monitor / frame), if a frame is known.
    fn scale(&self) -> Option<(f32, f32)> {
        let size = frame_size(self.frame.as_ref()?);
        let r = self.scale_reference();
        Some((
            self.settings.gain_x * r.width / size.width,
            self.settings.gain_y * r.height / size.height,
        ))
    }

    /// The display whose size sets the mapping scale (ADR-0017).
    fn scale_reference(&self) -> DesktopRect {
        scale_reference(&self.displays, self.main_display(), self.desktop)
    }

    fn map_unlimited(&self, p: Point2) -> Point2 {
        let Some(frame) = &self.frame else {
            return self.home();
        };
        map_raw_scaled(
            p,
            self.offset,
            frame_size(frame),
            self.desktop,
            self.scale_reference(),
            Gain {
                x: self.settings.gain_x,
                y: self.settings.gain_y,
            },
        )
    }

    /// The unsmoothed cursor position for `p`, inside the screen limits.
    fn map(&self, p: Point2) -> Point2 {
        apply_limits(self.map_unlimited(p), self.desktop, limits(&self.settings)).position
    }

    /// One camera-controlled pointer update: map → smooth → limits → drift → steady zone →
    /// dwell. The steady zone only holds what is sent out; the smoother keeps following.
    fn drive(&mut self, p: Point2, now_us: u64, step: &mut Step) {
        let last_us = self.smoother.last_us().unwrap_or(now_us);
        let p = self.one_euro.update(p, now_us, &OneEuroParams::default());
        let raw = self.map_unlimited(p);
        let smoothed = self.smoother.update(raw, now_us, self.settings.smoothing);
        let limited = apply_limits(smoothed, self.desktop, limits(&self.settings));
        // 2018 keeps the clamped position as the filter state.
        self.smoother.set_current(limited.position);
        // How far the head is past each limit, in camera px: the drift stops there.
        let (lo, hi) = bounds(self.desktop, limits(&self.settings));
        let (sx, sy) = self.scale().unwrap_or((1.0, 1.0));
        let past = |v: f32, lo: f32, hi: f32, s: f32| ((v - hi).max(lo - v)).max(0.0) / s;
        let past = Point2::new(past(raw.x, lo.x, hi.x, sx), past(raw.y, lo.y, hi.y, sy));
        self.offset = drift_offset(self.offset, &limited, now_us.saturating_sub(last_us), past);
        let radius = self.steady_radius();
        let mut cursor = self.steady.update(limited.position, radius);
        // The edge itself must be reachable (menu bar, corners, ADR-0017), but snapping a
        // pinned axis onto it released the zone's trail in one frame (test drive #12,
        // ADR-0020): near a limit the trail shortens with the target's distance to it.
        let t = limited.position;
        cursor.x = approach_edge(cursor.x, t.x, lo.x, hi.x);
        cursor.y = approach_edge(cursor.y, t.y, lo.y, hi.y);
        self.steady.reset_to(cursor);
        self.move_pointer(cursor, step);
        if !self.settings.dwell.enabled {
            return;
        }
        let params = dwell_params(&self.settings);
        // The click lands where the cursor is, not on the dwell's anchor (up to the dwell
        // radius away): clicking there jumped the cursor and back, missed small targets and
        // read as the user grabbing the mouse (v0.4.1 device check).
        if self
            .dwell
            .update(cursor, now_us, true, &params, self.desktop)
            .is_some()
            && !matches!(
                self.access,
                PointerAccess::Denied | PointerAccess::Unsupported
            )
        {
            step.effects.push(Effect::Click(cursor));
        }
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
