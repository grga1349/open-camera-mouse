//! Controller state machine tests (task 0008 D0, pointer pipeline D1). Headless, deterministic: synthetic frames,
//! an injected clock, no camera and no OS cursor.

use std::collections::HashSet;

use app_core::ports::{
    CameraError, CameraFrame, CaptureFormat, CaptureRequest, Desktop, PointerError, RgbFrame,
};
use app_core::{
    AppState, Command, Controller, DesktopRect, DwellSettings, Effect, Event, Input, MarkerKind,
    PointerAccess, PointerOwner, RECENTER_US, STEADY_ZONE_CAM_PX, ScreenLimits, Settings, Step,
};
use pointer_core::metrics::{self, PathSample, Scale};
use tracking_core::{GrayFrame, LossReason, Point2, TrackStatus};

const W: u32 = 640;
const H: u32 = 480;
const FRAME_US: u64 = 33_333;
const HANDOVER_US: u64 = 4_000_000;

/// The default gain (test drive #8/#10).
fn gain() -> f32 {
    Settings::default().gain_x
}
const DESK: DesktopRect = DesktopRect {
    x: 0.0,
    y: 0.0,
    width: 1920.0,
    height: 1080.0,
};
const CENTER: Point2 = Point2::new(960.0, 540.0);
const SCENE: u32 = 7;
const OCCLUDER: u32 = 99;

// ---- synthetic scene ----

fn hash(i: i64, j: i64, seed: u32) -> f32 {
    let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (j as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ u64::from(seed).wrapping_mul(0xD6E8_FEB8_6659_FD93);
    z ^= z >> 29;
    z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 32;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

/// Non-periodic value noise in [0, 1] with 4 px cells. Fine on purpose: re-detection's
/// selection self-test only arms on texture whose ZNCC peak is narrower than the 21 px window
/// (8 px cells already fail its 0.1 margin at 640×480; 16 px cells score a 0.97–0.99
/// "second" peak). See task 0008 Progress, 2026-11-12.
fn noise(x: f32, y: f32, seed: u32) -> f32 {
    let (gx, gy) = (x / 4.0, y / 4.0);
    let (x0, y0) = (gx.floor(), gy.floor());
    let (fx, fy) = (gx - x0, gy - y0);
    let (i, j) = (x0 as i64, y0 as i64);
    let a = hash(i, j, seed) * (1.0 - fx) + hash(i + 1, j, seed) * fx;
    let b = hash(i, j + 1, seed) * (1.0 - fx) + hash(i + 1, j + 1, seed) * fx;
    a * (1.0 - fy) + b * fy
}

/// A raw camera frame (`w`×`h`) whose display-space (mirrored) content is the scene shifted
/// by `(dx, dy)`: `dx > 0` is the user's head moving to their right.
fn raw_frame(w: u32, h: u32, dx: f32, dy: f32, seed: u32, ts: u64) -> CameraFrame {
    raw_frame_with(w, h, dx, dy, seed, ts, 200.0, 0.0)
}

/// [`raw_frame`] with the texture's contrast (grey levels, 200 = strong) and per-pixel sensor
/// noise (uniform, ± `sensor` grey levels, different every frame), for faint targets like a
/// nose tip (test drive #2).
#[allow(clippy::too_many_arguments)]
fn raw_frame_with(
    w: u32,
    h: u32,
    dx: f32,
    dy: f32,
    seed: u32,
    ts: u64,
    contrast: f32,
    sensor: f32,
) -> CameraFrame {
    let mut data = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            // Raw is the mirror of display: raw x ↔ display w − 1 − x.
            let disp_x = (w - 1 - x) as f32;
            let v = if seed == OCCLUDER {
                // A flat occluder (hand, turned head): no texture, a certain loss.
                0.5
            } else {
                noise(disp_x - dx, y as f32 - dy, seed)
            };
            let n = if sensor > 0.0 {
                sensor * (2.0 * hash(i64::from(x), i64::from(y), (ts / 1000) as u32) - 1.0)
            } else {
                0.0
            };
            // Contrast 200 gives exactly the original `30 + 200·v` frames.
            data.push((130.0 - contrast / 2.0 + contrast * v + n) as u8);
        }
    }
    CameraFrame {
        luma: GrayFrame::from_vec(w, h, data).unwrap(),
        rgb: None,
        timestamp_us: ts,
    }
}

// ---- harness ----

struct Rig {
    c: Controller,
    now: u64,
    seen: HashSet<String>,
}

impl Rig {
    fn new(settings: Settings) -> Self {
        Self {
            c: Controller::new(settings, DESK).unwrap(),
            now: 1_000_000,
            seen: HashSet::new(),
        }
    }

    fn input(&mut self, i: Input) -> Step {
        let s = self.c.handle(i, self.now);
        self.seen.insert(format!("{:?}", self.c.state()));
        s
    }

    fn cmd(&mut self, c: Command) -> Step {
        self.input(Input::Command(c))
    }

    fn frame(&mut self, dx: f32, dy: f32, seed: u32) -> Step {
        self.now += FRAME_US;
        let f = raw_frame(W, H, dx, dy, seed, self.now);
        self.input(Input::Frame(f))
    }

    /// Frames of a still scene until `us` has passed. Returns all steps merged.
    fn hold(&mut self, us: u64, dx: f32, seed: u32) -> Step {
        let end = self.now + us;
        let mut all = Step::default();
        while self.now < end {
            let s = self.frame(dx, 0.0, seed);
            all.events.extend(s.events);
            all.effects.extend(s.effects);
        }
        all
    }

    fn started() -> Self {
        Self::started_with(Settings::default())
    }

    fn started_with(settings: Settings) -> Self {
        let mut r = Self::new(settings);
        r.cmd(Command::ToggleStart);
        r.input(Input::CameraOpened(CaptureFormat {
            width: W,
            height: H,
            fps: 30.0,
        }));
        r.frame(0.0, 0.0, SCENE);
        assert_eq!(r.c.state(), AppState::NoTarget);
        r
    }

    /// Selected at the frame centre, camera in control after the 4 s handover.
    fn tracking_with(settings: Settings) -> Self {
        let mut r = Self::started_with(settings);
        r.cmd(Command::SelectTarget(Point2::new(320.0, 240.0)));
        r.hold(HANDOVER_US, 0.0, SCENE);
        assert_eq!(r.c.state(), AppState::Tracking);
        assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
        r
    }

    fn tracking() -> Self {
        Self::tracking_with(Settings::default())
    }

    /// Flat frames until the tracker reports a loss (a flat frame is a hold or an appearance
    /// rejection first, then LK's texture test; tracking-core needs at most 3).
    fn occlude(&mut self) -> Step {
        let mut all = Step::default();
        for _ in 0..3 {
            let s = self.frame(0.0, 0.0, OCCLUDER);
            all.events.extend(s.events);
            all.effects.extend(s.effects);
            if self.c.state() == AppState::Lost {
                return all;
            }
        }
        panic!("not lost after 3 flat frames: {:?}", self.c.view(self.now));
    }
}

fn state_changes(s: &Step) -> Vec<(AppState, AppState)> {
    s.events
        .iter()
        .filter_map(|e| match e {
            Event::StateChanged { from, to } => Some((*from, *to)),
            _ => None,
        })
        .collect()
}

fn moves(s: &Step) -> Vec<Point2> {
    s.effects
        .iter()
        .filter_map(|e| match e {
            Effect::MovePointer(p) => Some(*p),
            _ => None,
        })
        .collect()
}

fn dist(a: Point2, b: Point2) -> f32 {
    (a.x - b.x).hypot(a.y - b.y)
}

// ---- lifecycle ----

#[test]
fn start_opens_the_camera_and_reaches_no_target() {
    let mut r = Rig::new(Settings::default());
    let s = r.cmd(Command::ToggleStart);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Stopped, AppState::Starting)]
    );
    assert_eq!(
        s.effects,
        vec![Effect::OpenCamera(CaptureRequest::default())]
    );
    // F11 again while starting changes nothing.
    assert_eq!(r.cmd(Command::ToggleStart), Step::default());
    let s = r.input(Input::CameraOpened(CaptureFormat {
        width: W,
        height: H,
        fps: 30.0,
    }));
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Starting, AppState::NoTarget)]
    );
    let v = r.c.view(r.now);
    assert_eq!(v.pointer, PointerOwner::Mouse);
    assert!(v.marker.is_none());
}

#[test]
fn a_first_frame_also_ends_starting() {
    let mut r = Rig::new(Settings::default());
    r.cmd(Command::ToggleStart);
    let s = r.frame(0.0, 0.0, SCENE);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Starting, AppState::NoTarget)]
    );
    assert_eq!(r.c.view(r.now).frame_size, Some((W, H)));
}

#[test]
fn camera_errors_map_to_visible_states_and_toggle_retries() {
    let cases = [
        (CameraError::NoDevice, AppState::NoCamera),
        (CameraError::PermissionDenied, AppState::PermissionDenied),
        (CameraError::Disconnected, AppState::CameraDisconnected),
        (CameraError::Backend("device busy".into()), AppState::Error),
    ];
    for (err, want) in cases {
        let mut r = Rig::tracking();
        let s = r.input(Input::CameraFailed(err.clone()));
        assert_eq!(
            state_changes(&s),
            vec![(AppState::Tracking, want)],
            "{err:?}"
        );
        assert!(s.effects.contains(&Effect::CloseCamera));
        assert!(s.events.contains(&Event::PointerChanged {
            from: PointerOwner::Camera,
            to: PointerOwner::Mouse
        }));
        let v = r.c.view(r.now);
        assert_eq!(v.error.as_deref(), Some(err.to_string().as_str()));
        assert!(v.marker.is_none());
        // Frames after a failure are ignored.
        assert_eq!(r.frame(0.0, 0.0, SCENE), Step::default());
        let s = r.cmd(Command::ToggleStart);
        assert_eq!(state_changes(&s), vec![(want, AppState::Starting)]);
        assert_eq!(r.c.view(r.now).error, None);
    }
}

#[test]
fn stop_releases_everything() {
    let mut r = Rig::tracking();
    let s = r.cmd(Command::Stop);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Tracking, AppState::Stopped)]
    );
    assert_eq!(s.effects, vec![Effect::CloseCamera]);
    assert_eq!(r.c.pointer_owner(), PointerOwner::Mouse);
    assert!(r.c.preview().is_none());
    assert_eq!(r.cmd(Command::Stop), Step::default());
}

#[test]
fn wide_frames_are_downsampled_to_640_and_mirrored() {
    let mut r = Rig::new(Settings::default());
    r.cmd(Command::ToggleStart);
    r.now += FRAME_US;
    let raw = raw_frame(1280, 960, 0.0, 0.0, SCENE, r.now);
    let raw_left = raw.luma.get(0, 100).unwrap();
    r.input(Input::Frame(raw));
    let f = r.c.preview().unwrap();
    assert_eq!((f.width(), f.height()), (640, 480));
    // Mirrored: the raw left edge is now on the right (box average of 2 px).
    assert!(f.get(639, 50).unwrap().abs_diff(raw_left) < 20);
}

// ---- selection, handover, mapping ----

#[test]
fn click_select_waits_for_a_still_mouse_then_centres_the_cursor() {
    let mut r = Rig::started();
    let s = r.cmd(Command::SelectTarget(Point2::new(320.0, 240.0)));
    assert!(s.events.contains(&Event::TargetSelected {
        at: Point2::new(320.0, 240.0)
    }));
    assert_eq!(
        state_changes(&s),
        vec![(AppState::NoTarget, AppState::Tracking)]
    );
    let v = r.c.view(r.now);
    assert_eq!(v.pointer, PointerOwner::Handover);
    assert_eq!(v.handover_remaining_us, Some(HANDOVER_US));
    assert_eq!(v.marker.unwrap().kind, MarkerKind::Tracking);
    assert!(
        moves(&s).is_empty(),
        "no pointer move until the camera takes over"
    );

    let s = r.hold(HANDOVER_US - 2 * FRAME_US, 0.0, SCENE);
    assert!(moves(&s).is_empty());
    assert_eq!(r.c.pointer_owner(), PointerOwner::Handover);
    // The countdown comes from the controller's deadline.
    let left = r.c.view(r.now).handover_remaining_us.unwrap();
    assert!(left > 0 && left <= 2 * FRAME_US, "{left}");

    let s = r.hold(2 * FRAME_US, 0.0, SCENE);
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    // Takeover maps the current head position to the desktop centre (2018 scheme 0).
    assert_eq!(moves(&s).first(), Some(&CENTER));
    assert!(s.events.contains(&Event::PointerChanged {
        from: PointerOwner::Handover,
        to: PointerOwner::Camera
    }));
}

/// The runtime's order (task 0008 D7): observe the cursor, then the frame, every frame. An
/// observation taken just before the takeover deadline must not cancel the takeover by being
/// compared with the takeover's own move (found by the D7 runtime test).
#[test]
fn an_observation_before_a_due_takeover_does_not_cancel_it() {
    let mut r = Rig::started();
    let mouse = Point2::new(100.0, 100.0);
    r.input(Input::PointerObserved(Some(mouse)));
    r.cmd(Command::SelectTarget(Point2::new(320.0, 240.0)));
    let mut cursor = mouse;
    let end = r.now + HANDOVER_US + 10 * FRAME_US;
    while r.now < end {
        r.now += FRAME_US;
        r.input(Input::PointerObserved(Some(cursor)));
        let f = raw_frame(W, H, 0.0, 0.0, SCENE, r.now);
        let s = r.input(Input::Frame(f));
        if let Some(p) = s.pointer_target() {
            cursor = p;
        }
    }
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    assert_eq!(cursor, CENTER);
}

#[test]
fn head_right_moves_the_cursor_right() {
    let mut r = Rig::tracking();
    // Mapping direction per frame; the steady zone (which holds sub-0.5 px steps) is off.
    r.c.set_steady_zone_cam_px(0.0);
    let mut last = CENTER;
    for i in 1..=5 {
        let s = r.frame(i as f32, 0.0, SCENE);
        let p = s
            .pointer_target()
            .expect("camera moves the pointer every tracked frame");
        assert!(p.x > last.x, "frame {i}: {p:?} should be right of {last:?}");
        assert!((p.y - CENTER.y).abs() < 3.0, "{p:?}");
        last = p;
    }
    // Filtering lags a little; once the head is still the cursor settles at 5 px of head
    // motion × the default gain × (1920 / 640), within the filter's residual lag.
    let last = r.hold(500_000, 5.0, SCENE).pointer_target().unwrap();
    let short = CENTER.x + 5.0 * gain() * 3.0 - last.x;
    assert!((-1.5..3.0).contains(&short), "{last:?}");
    let t = r.c.view(r.now).marker.unwrap();
    assert!((t.position.x - 325.0).abs() < 0.5, "{t:?}");
}

#[test]
fn moving_the_mouse_takes_control_and_stillness_gives_it_back() {
    let mut r = Rig::tracking();
    let s = r.frame(0.0, 0.0, SCENE);
    let cmd = s.pointer_target().unwrap();
    // Within 5 px: still the camera's.
    r.input(Input::PointerObserved(Some(Point2::new(
        cmd.x + 3.0,
        cmd.y + 4.0,
    ))));
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    // The user grabs the mouse.
    let grabbed = Point2::new(cmd.x + 40.0, cmd.y);
    let s = r.input(Input::PointerObserved(Some(grabbed)));
    assert!(s.events.contains(&Event::PointerChanged {
        from: PointerOwner::Camera,
        to: PointerOwner::Handover
    }));
    let s = r.frame(2.0, 0.0, SCENE);
    assert!(moves(&s).is_empty(), "the mouse has control");

    // Movement after 1 s restarts the 4 s timer.
    r.hold(1_000_000, 0.0, SCENE);
    r.input(Input::PointerObserved(Some(Point2::new(
        grabbed.x + 1.0,
        grabbed.y,
    ))));
    let moved_at = r.now;
    let s = r.hold(HANDOVER_US - 100_000, 0.0, SCENE);
    assert!(moves(&s).is_empty());
    assert_eq!(r.c.pointer_owner(), PointerOwner::Handover);
    r.now = moved_at + HANDOVER_US;
    let s = r.input(Input::Tick);
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    assert_eq!(s.pointer_target(), Some(CENTER));
}

#[test]
fn a_selection_outside_the_frame_is_rejected() {
    let mut r = Rig::started();
    let p = Point2::new(700.0, 10.0);
    let s = r.cmd(Command::SelectTarget(p));
    assert_eq!(s.events, vec![Event::SelectionRejected { at: p }]);
    assert_eq!(r.c.state(), AppState::NoTarget);
}

#[test]
fn a_frame_size_change_drops_the_target() {
    let mut r = Rig::tracking();
    r.now += FRAME_US;
    let s = r.input(Input::Frame(raw_frame(320, 240, 0.0, 0.0, SCENE, r.now)));
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Tracking, AppState::NoTarget)]
    );
    assert_eq!(r.c.pointer_owner(), PointerOwner::Mouse);
}

// ---- recenter and pause ----

#[test]
fn recenter_counts_down_5s_then_tracks_the_frame_centre() {
    let mut r = Rig::started();
    let t0 = r.now;
    let s = r.cmd(Command::Recenter);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::NoTarget, AppState::Recentering)]
    );
    let v = r.c.view(t0);
    let rc = v.recenter.unwrap();
    assert_eq!(
        (rc.center, rc.remaining_us, rc.seconds()),
        (Point2::new(320.0, 240.0), RECENTER_US, 5)
    );
    assert_eq!(r.c.view(t0 + 500_000).recenter.unwrap().seconds(), 5);
    assert_eq!(r.c.view(t0 + 1_000_000).recenter.unwrap().seconds(), 4);
    assert_eq!(r.c.view(t0 + 4_500_000).recenter.unwrap().seconds(), 1);
    assert_eq!(v.pointer, PointerOwner::Mouse);

    // Clicks don't select during the countdown.
    let s = r.cmd(Command::SelectTarget(Point2::new(100.0, 100.0)));
    assert_eq!(s, Step::default());

    r.now = t0 + RECENTER_US - 1;
    assert!(state_changes(&r.input(Input::Tick)).is_empty());
    r.now = t0 + RECENTER_US;
    let s = r.input(Input::Tick);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Recentering, AppState::Tracking)]
    );
    assert!(s.events.contains(&Event::TargetSelected {
        at: Point2::new(320.0, 240.0)
    }));
    // Recenter takes over at once (2018 forced start): no 4 s handover.
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    assert_eq!(s.pointer_target(), Some(CENTER));
    assert!(r.c.view(r.now).recenter.is_none());
}

#[test]
fn recenter_again_restarts_the_countdown() {
    let mut r = Rig::started();
    r.cmd(Command::Recenter);
    r.now += 3_000_000;
    r.cmd(Command::Recenter);
    assert_eq!(r.c.view(r.now).recenter.unwrap().remaining_us, RECENTER_US);
}

#[test]
fn recenter_without_a_frame_waits_for_the_next_one() {
    let mut r = Rig::new(Settings::default());
    r.cmd(Command::ToggleStart);
    r.input(Input::CameraOpened(CaptureFormat {
        width: W,
        height: H,
        fps: 30.0,
    }));
    r.cmd(Command::Recenter);
    r.now += RECENTER_US + 10;
    r.input(Input::Tick);
    assert_eq!(r.c.state(), AppState::Recentering);
    assert_eq!(r.c.view(r.now).recenter.unwrap().remaining_us, 0);
    let s = r.frame(0.0, 0.0, SCENE);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Recentering, AppState::Tracking)]
    );
}

#[test]
fn pause_releases_and_centres_and_resume_goes_through_recenter() {
    let mut r = Rig::tracking();
    let s = r.cmd(Command::ToggleStart);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Tracking, AppState::Paused)]
    );
    assert_eq!(moves(&s), vec![CENTER]);
    assert_eq!(r.c.pointer_owner(), PointerOwner::Mouse);
    // Paused: preview keeps updating, nothing is tracked or moved.
    let seq = r.c.view(r.now).frame_seq;
    let s = r.frame(10.0, 0.0, SCENE);
    assert!(moves(&s).is_empty());
    assert_eq!(r.c.view(r.now).frame_seq, seq + 1);
    assert!(r.c.view(r.now).marker.is_none());
    let s = r.cmd(Command::SelectTarget(Point2::new(320.0, 240.0)));
    assert_eq!(s, Step::default(), "no selection while paused");

    let s = r.cmd(Command::ToggleStart);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Paused, AppState::Recentering)]
    );
}

#[test]
fn recenter_ends_a_pause() {
    let mut r = Rig::tracking();
    r.cmd(Command::ToggleStart);
    let s = r.cmd(Command::Recenter);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Paused, AppState::Recentering)]
    );
    r.hold(RECENTER_US, 0.0, SCENE);
    assert_eq!(r.c.state(), AppState::Tracking);
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
}

#[test]
fn toggle_during_recenter_pauses() {
    let mut r = Rig::started();
    r.cmd(Command::Recenter);
    let s = r.cmd(Command::ToggleStart);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Recentering, AppState::Paused)]
    );
    r.now += RECENTER_US;
    assert!(state_changes(&r.input(Input::Tick)).is_empty());
}

// ---- loss ----

#[test]
fn loss_without_recovery_waits_for_reselection() {
    let mut r = Rig::tracking();
    let s = r.occlude();
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Tracking, AppState::Lost)]
    );
    assert_eq!(r.c.pointer_owner(), PointerOwner::Mouse);
    let v = r.c.view(r.now);
    let reason = match v.track {
        Some(TrackStatus::Lost(reason)) => reason,
        other => panic!("{other:?}"),
    };
    assert_ne!(reason, LossReason::Searching);
    assert_eq!(v.marker.unwrap().kind, MarkerKind::Lost);
    assert!(!v.loss_recovery && !v.redetect_armed);

    // The target comes back: 2018 doesn't look for it. The loss reason stays on show.
    let s = r.hold(2_000_000, 0.0, SCENE);
    assert!(moves(&s).is_empty());
    assert_eq!(r.c.state(), AppState::Lost);
    assert_eq!(r.c.view(r.now).track, Some(TrackStatus::Lost(reason)));
    // No automatic takeover while lost.
    assert_eq!(r.c.pointer_owner(), PointerOwner::Mouse);

    let s = r.cmd(Command::SelectTarget(Point2::new(320.0, 240.0)));
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Lost, AppState::Tracking)]
    );
    assert_eq!(r.c.pointer_owner(), PointerOwner::Handover);
}

fn with_recovery() -> Settings {
    Settings {
        loss_recovery: true,
        ..Settings::default()
    }
}

#[test]
fn recovery_on_searches_and_resumes_within_the_jump_bound() {
    let mut r = Rig::tracking_with(with_recovery());
    assert!(r.c.view(r.now).redetect_armed);
    let before = r.frame(0.0, 0.0, SCENE).pointer_target().unwrap();
    r.occlude();
    r.frame(0.0, 0.0, OCCLUDER);
    let v = r.c.view(r.now);
    assert_eq!(v.marker.unwrap().kind, MarkerKind::Searching);
    assert!(matches!(v.track, Some(TrackStatus::Lost(reason)) if reason != LossReason::Searching));
    assert_eq!(v.pointer, PointerOwner::Mouse);

    // Back where it was: two finds aren't enough (three consecutive confirm, test drive
    // #3), the third resumes and control returns at once, without a jump.
    for _ in 0..2 {
        let s = r.frame(0.0, 0.0, SCENE);
        assert!(state_changes(&s).is_empty(), "not confirmed yet");
        assert_eq!(r.c.state(), AppState::Lost);
    }
    let s = r.frame(0.0, 0.0, SCENE);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Lost, AppState::Tracking)]
    );
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    let p = s.pointer_target().unwrap();
    assert!(dist(p, before) < 5.0, "{p:?} vs {before:?}");
}

#[test]
fn recovery_needs_consecutive_finds_in_one_place() {
    let mut r = Rig::tracking_with(with_recovery());
    r.occlude();
    // Found, then gone, then found: the count starts over, no resume after 3 frames.
    r.frame(0.0, 0.0, SCENE);
    r.frame(0.0, 0.0, OCCLUDER);
    r.frame(0.0, 0.0, SCENE);
    r.frame(0.0, 0.0, SCENE);
    assert_eq!(r.c.state(), AppState::Lost);
    r.frame(0.0, 0.0, SCENE);
    assert_eq!(r.c.state(), AppState::Tracking);
}

#[test]
fn recovery_beyond_the_jump_bound_goes_through_handover() {
    let mut r = Rig::tracking_with(with_recovery());
    let before = r.frame(0.0, 0.0, SCENE).pointer_target().unwrap();
    r.input(Input::PointerObserved(Some(before)));
    r.occlude();
    r.frame(0.0, 0.0, OCCLUDER);
    // Reappears 40 px to the right: 40 × 18 = 720 screen px, far over the 54 px bound.
    // Confirmed on the third consecutive find.
    r.frame(40.0, 0.0, SCENE);
    r.frame(40.0, 0.0, SCENE);
    let s = r.frame(40.0, 0.0, SCENE);
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Lost, AppState::Tracking)]
    );
    assert!(moves(&s).is_empty(), "R-P4: no jump on re-acquisition");
    assert_eq!(r.c.pointer_owner(), PointerOwner::Handover);
    // Then the normal takeover: still mouse for 4 s → cursor to centre.
    let s = r.hold(HANDOVER_US, 40.0, SCENE);
    assert_eq!(moves(&s).first(), Some(&CENTER));
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
}

// ---- pointer permission ----

#[test]
fn denied_pointer_permission_keeps_tracking_without_moving() {
    let mut r = Rig::started();
    let s = r.input(Input::PointerAccess(PointerAccess::Denied));
    assert!(s.events.contains(&Event::PointerChanged {
        from: PointerOwner::Mouse,
        to: PointerOwner::PermissionNeeded
    }));
    r.cmd(Command::SelectTarget(Point2::new(320.0, 240.0)));
    let s = r.hold(HANDOVER_US + 500_000, 0.0, SCENE);
    assert!(moves(&s).is_empty());
    let s = r.frame(3.0, 0.0, SCENE);
    assert!(moves(&s).is_empty());
    assert_eq!(r.c.state(), AppState::Tracking);
    assert_eq!(r.c.view(r.now).pointer, PointerOwner::PermissionNeeded);
    assert_eq!(r.c.view(r.now).handover_remaining_us, None);

    // Granted after "Check again": the camera already holds control.
    r.input(Input::PointerAccess(PointerAccess::Granted));
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    assert!(r.frame(4.0, 0.0, SCENE).pointer_target().is_some());
}

#[test]
fn a_failed_move_reports_the_permission_state() {
    let mut r = Rig::tracking();
    r.input(Input::PointerFailed(PointerError::PermissionDenied));
    assert_eq!(r.c.pointer_owner(), PointerOwner::PermissionNeeded);
    let mut r = Rig::tracking();
    r.input(Input::PointerFailed(PointerError::Unsupported(
        "wayland".into(),
    )));
    assert_eq!(r.c.pointer_owner(), PointerOwner::Unsupported);
    let mut r = Rig::tracking();
    r.input(Input::PointerFailed(PointerError::Backend("x".into())));
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    assert!(r.c.view(r.now).error.is_some());
}

// ---- settings ----

#[test]
fn invalid_settings_are_refused_and_change_nothing() {
    let mut r = Rig::tracking();
    let bad = Settings {
        smoothing: 1.0,
        ..Settings::default()
    };
    let err = r.c.update_settings(bad, r.now).unwrap_err();
    assert_eq!(err.field, "smoothing");
    assert_eq!(r.c.settings(), &Settings::default());
    assert!(
        Controller::new(
            Settings {
                gain_x: f32::NAN,
                ..Settings::default()
            },
            DESK
        )
        .is_err()
    );
}

#[test]
fn changing_mirroring_drops_the_target() {
    let mut r = Rig::tracking();
    let s =
        r.c.update_settings(
            Settings {
                mirror: false,
                ..Settings::default()
            },
            r.now,
        )
        .unwrap();
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Tracking, AppState::NoTarget)]
    );
    assert!(
        r.c.preview().is_none(),
        "no selection on a stale-orientation frame"
    );
}

#[test]
fn toggling_recovery_rebuilds_the_tracker_and_keeps_tracking() {
    let mut r = Rig::tracking();
    assert!(!r.c.view(r.now).redetect_armed);
    let s = r.c.update_settings(with_recovery(), r.now).unwrap();
    assert!(state_changes(&s).is_empty());
    assert!(r.c.view(r.now).redetect_armed);
    assert_eq!(r.c.state(), AppState::Tracking);
    // Beyond the 0.5 camera px steady zone, so the cursor follows.
    let s = r.hold(500_000, 2.0, SCENE);
    assert!(s.pointer_target().unwrap().x > CENTER.x);
    let s = r.c.update_settings(Settings::default(), r.now).unwrap();
    assert!(state_changes(&s).is_empty());
    assert!(!r.c.view(r.now).redetect_armed);
}

#[test]
fn changing_the_camera_reopens_it() {
    let mut r = Rig::tracking();
    let s =
        r.c.update_settings(
            Settings {
                camera: Some("usb-1".into()),
                ..Settings::default()
            },
            r.now,
        )
        .unwrap();
    assert_eq!(
        s.effects,
        vec![
            Effect::CloseCamera,
            Effect::OpenCamera(CaptureRequest::default())
        ]
    );
    assert_eq!(
        state_changes(&s),
        vec![(AppState::Tracking, AppState::Starting)]
    );
}

// ---- coverage of R-A1 ----

#[test]
fn every_app_state_is_reachable() {
    let mut seen = HashSet::new();
    let mut r = Rig::tracking_with(with_recovery());
    r.cmd(Command::ToggleStart); // Paused
    r.cmd(Command::Recenter); // Recentering
    r.hold(RECENTER_US, 0.0, SCENE); // Tracking
    r.occlude(); // Lost
    r.cmd(Command::Stop); // Stopped
    seen.extend(r.seen.drain());
    for err in [
        CameraError::NoDevice,
        CameraError::PermissionDenied,
        CameraError::Disconnected,
        CameraError::Backend("x".into()),
    ] {
        let mut r = Rig::started(); // Starting, NoTarget
        r.input(Input::CameraFailed(err));
        seen.extend(r.seen.drain());
    }
    let all = [
        AppState::Stopped,
        AppState::Starting,
        AppState::NoCamera,
        AppState::PermissionDenied,
        AppState::Error,
        AppState::NoTarget,
        AppState::Tracking,
        AppState::Lost,
        AppState::Recentering,
        AppState::Paused,
        AppState::CameraDisconnected,
    ];
    for s in all {
        assert!(
            seen.contains(&format!("{s:?}")),
            "{s:?} not reached: {seen:?}"
        );
    }
}

// ---- pointer pipeline (D1): smoothing, limits, drift, dwell ----

fn clicks(s: &Step) -> Vec<Point2> {
    s.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Click(p) => Some(*p),
            _ => None,
        })
        .collect()
}

/// Moves the head from its current `from` to `to` (display-space px) in 2 px steps.
fn sweep(r: &mut Rig, from: f32, to: f32) -> Step {
    let mut all = Step::default();
    let mut dx = from;
    while (to - dx).abs() > 1e-3 {
        dx += (to - dx).clamp(-2.0, 2.0);
        let s = r.frame(dx, 0.0, SCENE);
        all.events.extend(s.events);
        all.effects.extend(s.effects);
    }
    all
}

/// Smoothing 0 turns off the 2018 smoother only; the 1€ filter (ADR-0017) still runs, and
/// settles a 2 px step (2 × the default gain × 1920/640) within half a second.
#[test]
fn zero_smoothing_settles_a_step_within_half_a_second() {
    let mut r = Rig::tracking_with(Settings {
        smoothing: 0.0,
        ..Settings::default()
    });
    // The filters, not the steady zone, are under test here.
    r.c.set_steady_zone_cam_px(0.0);
    let first = r.frame(2.0, 0.0, SCENE).pointer_target().unwrap();
    assert!(first.x > CENTER.x, "moves on the first frame: {first:?}");
    let p = r.hold(500_000, 2.0, SCENE).pointer_target().unwrap();
    let short = CENTER.x + 2.0 * gain() * 3.0 - p.x;
    assert!((-1.5..3.0).contains(&short), "{p:?}");
}

#[test]
fn default_smoothing_lags_then_settles() {
    let mut r = Rig::tracking();
    r.c.set_steady_zone_cam_px(0.0);
    let first = r.frame(2.0, 0.0, SCENE).pointer_target().unwrap();
    assert!(
        first.x > CENTER.x && first.x < CENTER.x + 2.0 * gain() * 3.0 - 6.0,
        "{first:?}"
    );
    let settled = r.hold(500_000, 2.0, SCENE).pointer_target().unwrap();
    let short = CENTER.x + 2.0 * gain() * 3.0 - settled.x;
    assert!((-1.5..3.0).contains(&short), "{settled:?}");
}

/// Limits keep the cursor inside the insets; while pinned, the offset drifts back so the
/// cursor comes off the edge (2018 §2.6, as a rate).
#[test]
fn screen_limits_pin_the_cursor_and_drift_recentres_the_mapping() {
    let mut r = Rig::tracking_with(Settings {
        smoothing: 0.0,
        limits: ScreenLimits {
            right: 200,
            ..ScreenLimits::default()
        },
        ..Settings::default()
    });
    let edge = DESK.width - 1.0 - 200.0;
    // Raw x = 960 + dx × 18 passes the 1719 px edge from dx ≈ 42.
    let s = sweep(&mut r, 0.0, 50.0);
    let m = moves(&s);
    assert!(
        m.iter().all(|p| p.x <= edge),
        "cursor past the limit: {m:?}"
    );
    assert_eq!(m.last().unwrap().x, edge, "pinned at the right limit");
    // Holding the head there: the offset drifts ~14.3 image px/s toward the centre until
    // the target is back inside the limit. The cursor then stays within the steady zone of
    // the edge (ADR-0017): a held head doesn't move it.
    let s = r.hold(2_000_000, 50.0, SCENE);
    let last = s.pointer_target().unwrap();
    assert!(last.x <= edge && last.x > edge - 40.0, "{last:?}");
    // Head back to centre: the cursor is now left of the desktop centre (mapping shifted).
    let back = sweep(&mut r, 50.0, 0.0).pointer_target().unwrap();
    assert!(back.x < CENTER.x - 50.0, "{back:?}");
}

fn with_dwell() -> Settings {
    Settings {
        dwell: DwellSettings {
            enabled: true,
            ..DwellSettings::default()
        },
        ..Settings::default()
    }
}

#[test]
fn dwell_is_off_by_default() {
    let mut r = Rig::tracking();
    assert!(clicks(&r.hold(4_000_000, 0.0, SCENE)).is_empty());
    assert_eq!(r.c.view(r.now).dwell_progress, None);
}

/// R-K1: one click after 1 s still, then nothing during the 2 s refractory.
#[test]
fn dwell_fires_once_when_enabled() {
    let mut r = Rig::tracking_with(with_dwell());
    let s = r.hold(500_000, 0.0, SCENE);
    assert!(clicks(&s).is_empty());
    let p = r.c.view(r.now).dwell_progress.unwrap();
    assert!((0.3..0.6).contains(&p), "{p}");
    let s = r.hold(2_000_000, 0.0, SCENE);
    let c = clicks(&s);
    assert_eq!(c.len(), 1, "{c:?}");
    assert!(dist(c[0], CENTER) < 3.0, "{c:?}");
    // The click follows the move in the same step.
    let idx = s
        .effects
        .iter()
        .position(|e| matches!(e, Effect::Click(_)))
        .unwrap();
    assert!(matches!(s.effects[idx - 1], Effect::MovePointer(_)));
}

/// R-K2: no click while the mouse has control or the target is lost.
#[test]
fn dwell_never_fires_in_handover_or_when_lost() {
    let mut r = Rig::tracking_with(with_dwell());
    r.hold(800_000, 0.0, SCENE);
    // The user grabs the mouse: handover, for 3 s (< 4 s inactivity).
    let cmd = r.c.view(r.now);
    assert_eq!(cmd.pointer, PointerOwner::Camera);
    r.input(Input::PointerObserved(Some(Point2::new(100.0, 100.0))));
    assert_eq!(r.c.pointer_owner(), PointerOwner::Handover);
    assert!(clicks(&r.hold(3_000_000, 0.0, SCENE)).is_empty());
    assert_eq!(r.c.view(r.now).dwell_progress, None);

    let mut r = Rig::tracking_with(with_dwell());
    r.hold(800_000, 0.0, SCENE);
    r.occlude();
    let mut all = Step::default();
    for _ in 0..60 {
        let s = r.frame(0.0, 0.0, OCCLUDER);
        all.effects.extend(s.effects);
    }
    assert!(clicks(&all).is_empty());
}

#[test]
fn dwell_does_not_click_when_pointer_permission_is_denied() {
    let mut r = Rig::tracking_with(with_dwell());
    r.input(Input::PointerAccess(PointerAccess::Denied));
    assert!(clicks(&r.hold(2_000_000, 0.0, SCENE)).is_empty());
}

// ---- colour preview (D3, ADR-0008 amendment) ----

/// A raw 1280×720 frame whose colour encodes the raw column in red and the row in green.
fn colour_frame(ts: u64, rgb_w: u32) -> CameraFrame {
    let (w, h) = (1280u32, 720u32);
    let luma = GrayFrame::filled(w, h, 128).unwrap();
    let mut d = Vec::with_capacity((rgb_w * h * 3) as usize);
    for y in 0..h {
        for x in 0..rgb_w {
            d.extend([(x / 8) as u8, (y / 4) as u8, 200]);
        }
    }
    CameraFrame {
        luma,
        rgb: Some(RgbFrame::from_vec(rgb_w, h, d).unwrap()),
        timestamp_us: ts,
    }
}

#[test]
fn colour_preview_is_mirrored_and_downsampled_with_the_luma() {
    let mut r = Rig::started();
    r.now += FRAME_US;
    r.input(Input::Frame(colour_frame(r.now, 1280)));
    let (g, c) = (r.c.preview().unwrap(), r.c.preview_rgb().unwrap());
    assert_eq!((c.width(), c.height()), (g.width(), g.height()));
    assert_eq!((c.width(), c.height()), (640, 360));
    // Mirrored (default): display column 0 is raw columns 1278..1280, i.e. red 159.
    assert_eq!(c.get(0, 0), Some([159, 0, 200]));
    assert_eq!(c.get(639, 0), Some([0, 0, 200]));
    // Rows 718..720 have green 179.
    assert_eq!(c.get(320, 359).map(|p| p[1]), Some(179));
}

#[test]
fn a_colour_image_of_the_wrong_size_is_dropped() {
    let mut r = Rig::started();
    r.now += FRAME_US;
    r.input(Input::Frame(colour_frame(r.now, 1278)));
    assert!(r.c.preview().is_some());
    assert!(r.c.preview_rgb().is_none());
}

#[test]
fn stopping_clears_the_colour_preview() {
    let mut r = Rig::started();
    r.now += FRAME_US;
    r.input(Input::Frame(colour_frame(r.now, 1280)));
    assert!(r.c.preview_rgb().is_some());
    r.cmd(Command::Stop);
    assert!(r.c.preview_rgb().is_none());
}

// ---- "Main monitor": recenter on a chosen display (test drive #4, ADR-0014) ----

/// Two displays side by side: the main one (1920×1080 at the origin) and a larger one to its
/// right. Bounding box 4480×1440, centre (2240, 720).
const LEFT: DesktopRect = DesktopRect {
    x: 0.0,
    y: 0.0,
    width: 1920.0,
    height: 1080.0,
};
const RIGHT: DesktopRect = DesktopRect {
    x: 1920.0,
    y: 0.0,
    width: 2560.0,
    height: 1440.0,
};
const BOTH: DesktopRect = DesktopRect {
    x: 0.0,
    y: 0.0,
    width: 4480.0,
    height: 1440.0,
};

fn two_displays(recenter_display: Option<u32>) -> Rig {
    let mut r = Rig::started_with(Settings {
        recenter_display,
        ..Settings::default()
    });
    // The adapter's order is arbitrary; the controller numbers displays left to right.
    r.input(Input::DesktopChanged(Desktop {
        bounds: BOTH,
        displays: vec![RIGHT, LEFT],
    }));
    r
}

fn recentered(r: &mut Rig) -> Step {
    r.cmd(Command::Recenter);
    r.now += RECENTER_US;
    r.input(Input::Tick)
}

#[test]
fn displays_are_numbered_left_to_right() {
    let r = two_displays(None);
    assert_eq!(r.c.displays(), &[LEFT, RIGHT]);
}

#[test]
fn recenter_without_a_display_choice_uses_the_centre_of_all_displays() {
    let mut r = two_displays(None);
    let s = recentered(&mut r);
    assert_eq!(s.pointer_target(), Some(BOTH.center()));
}

#[test]
fn recenter_on_a_display_starts_at_its_centre_and_still_reaches_the_others() {
    let mut r = two_displays(Some(1));
    let s = recentered(&mut r);
    assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
    assert_eq!(s.pointer_target(), Some(RIGHT.center()));

    // The scale comes from the Main monitor's size (ADR-0017), shifted to its centre: head
    // 5 px left moves the cursor 5 · gain · 2560 / 640 px left of the home point.
    let s = r.hold(2_000_000, -5.0, SCENE);
    let last = *moves(&s).last().unwrap();
    let want = Point2::new(RIGHT.center().x - 5.0 * gain() * 4.0, RIGHT.center().y);
    // The steady zone scales with the Main monitor too: 0.5 · gain · 2560 / 640.
    let zone = STEADY_ZONE_CAM_PX * gain() * 2560.0 / 640.0;
    assert!(dist(last, want) < zone + 3.0, "{last:?} vs {want:?}");

    // Far enough left, the cursor crosses onto the left display (head turned in 2 px steps):
    // 25 px · 88 = 2200 px left of 3200 is on the left display.
    for i in 1..=10 {
        r.frame(-5.0 - 2.0 * i as f32, 0.0, SCENE);
    }
    let s = r.hold(2_000_000, -25.0, SCENE);
    assert_eq!(r.c.state(), AppState::Tracking);
    let last = *moves(&s).last().unwrap();
    assert!(last.x < LEFT.width, "{last:?} still on the right display");
}

#[test]
fn pause_puts_the_cursor_on_the_chosen_display() {
    let mut r = two_displays(Some(0));
    recentered(&mut r);
    let s = r.cmd(Command::ToggleStart);
    assert_eq!(r.c.state(), AppState::Paused);
    assert_eq!(moves(&s), vec![LEFT.center()]);
}

#[test]
fn recenter_on_an_unplugged_display_falls_back_to_the_centre_of_all() {
    let mut r = two_displays(Some(1));
    r.input(Input::DesktopChanged(Desktop {
        bounds: LEFT,
        displays: vec![LEFT],
    }));
    let s = recentered(&mut r);
    assert_eq!(s.pointer_target(), Some(LEFT.center()));
    // The setting is kept for when the display comes back.
    assert_eq!(r.c.settings().recenter_display, Some(1));
}

// ---- steady zone (test drive #5, ADR-0015) ----

#[test]
fn steady_zone_holds_the_cursor_still_against_head_jitter() {
    let mut r = Rig::tracking();
    // Settle, then sub-pixel head jitter: ±0.2 camera px ≈ ±13 screen px at the default gain.
    r.hold(500_000, 0.0, SCENE);
    let mut targets = Vec::new();
    for i in 0..90 {
        let j = if i % 2 == 0 { 0.2 } else { -0.2 };
        let jy = if i % 3 == 0 { 0.15 } else { -0.1 };
        targets.extend(moves(&r.frame(j, jy, SCENE)));
    }
    let first = targets[0];
    assert!(
        targets.iter().all(|&p| p == first),
        "cursor moved while the head only jittered: {targets:?}"
    );

    // A deliberate turn (head 5 px right ≈ 90 screen px) is followed.
    for i in 1..=5 {
        r.frame(i as f32, 0.0, SCENE);
    }
    let s = r.hold(1_000_000, 5.0, SCENE);
    let last = *moves(&s).last().unwrap();
    assert!(last.x - first.x > 75.0, "{last:?} vs {first:?}");
}

#[test]
fn without_steady_zone_head_jitter_moves_the_cursor() {
    // Control for the test above: with the zone off, filtered jitter still reaches the
    // cursor; the zone is what makes it exactly zero.
    let mut r = Rig::tracking();
    r.c.set_steady_zone_cam_px(0.0);
    r.hold(500_000, 0.0, SCENE);
    let mut targets = Vec::new();
    for i in 0..90 {
        let j = if i % 2 == 0 { 0.2 } else { -0.2 };
        targets.extend(moves(&r.frame(j, 0.0, SCENE)));
    }
    let xs: Vec<f32> = targets.iter().map(|p| p.x).collect();
    let spread =
        xs.iter().cloned().fold(f32::MIN, f32::max) - xs.iter().cloned().fold(f32::MAX, f32::min);
    assert!(spread > 0.0, "spread {spread}");
}

// ---- test drive failure points (task 0009, diagnosed on local recordings 2026-11-15) ----
//
// Each test reproduces one failure seen on the user's camera with synthetic frames. The
// recordings themselves are never committed; these tests are the record.

/// A faint target like a nose tip: texture contrast 13 grey levels plus ±3 levels of sensor
/// noise puts LK's texture measure (`min_eigen`, threshold 1.0) at ≈ 1.0–1.25, where the user's
/// recording (S2) sat while the selected point matched its reference at ZNCC 0.93–0.98.
const FAINT_CONTRAST: f32 = 13.0;
const SENSOR_NOISE: f32 = 3.0;

impl Rig {
    fn faint_frame(&mut self, dx: f32, dy: f32) -> Step {
        self.now += FRAME_US;
        let f = raw_frame_with(W, H, dx, dy, SCENE, self.now, FAINT_CONTRAST, SENSOR_NOISE);
        self.input(Input::Frame(f))
    }
}

fn loss_count(states: &[AppState]) -> usize {
    states
        .windows(2)
        .filter(|w| w[0] != AppState::Lost && w[1] == AppState::Lost)
        .count()
}

/// #2 (S2): with the head still, the faint target flipped Lost ↔ Tracking every 2–3 frames
/// (with recovery on; without recovery the first dip is a permanent loss). A still, matching
/// target must stay tracked.
#[test]
fn a_faint_still_target_is_tracked_without_losses() {
    for recovery in [false, true] {
        let mut r = Rig::new(Settings {
            loss_recovery: recovery,
            ..Settings::default()
        });
        r.cmd(Command::ToggleStart);
        r.faint_frame(0.0, 0.0);
        r.cmd(Command::SelectTarget(Point2::new(320.0, 240.0)));
        assert_eq!(r.c.state(), AppState::Tracking);
        let mut states = vec![AppState::Tracking];
        for _ in 0..90 {
            r.faint_frame(0.0, 0.0);
            states.push(r.c.state());
        }
        let lost = states.iter().filter(|s| **s == AppState::Lost).count();
        assert_eq!(
            (loss_count(&states), lost),
            (0, 0),
            "recovery {recovery}: losses / lost frames over 3 s still"
        );
        let m = r.c.view(r.now).marker.unwrap().position;
        assert!(
            dist(m, Point2::new(320.0, 240.0)) < 1.0,
            "marker drifted to {m:?}"
        );
    }
}

/// The user's desktop: a 1440×900 laptop screen and a 2560×1440 display to its right,
/// 540 px higher (bounding box 4000×1440).
fn users_desktop() -> Desktop {
    let laptop = DesktopRect {
        x: 0.0,
        y: 0.0,
        width: 1440.0,
        height: 900.0,
    };
    let external = DesktopRect {
        x: 1440.0,
        y: -540.0,
        width: 2560.0,
        height: 1440.0,
    };
    Desktop {
        bounds: DesktopRect {
            x: 0.0,
            y: -540.0,
            width: 4000.0,
            height: 1440.0,
        },
        displays: vec![laptop, external],
    }
}

/// Head micro-movement at rest as measured in S1: ≈ 0.08 px rms per frame in camera pixels
/// (uniform in ±0.14), deterministic.
fn rest_tremor(i: u64) -> (f32, f32) {
    let u = |seed| 0.28 * (hash(i as i64, 0, seed) - 0.5);
    (u(501), u(502))
}

/// #5 (S1): sitting still, the cursor shook ≈ 2.6 px rms frame to frame on the user's two
/// displays. At rest the cursor must not move at all (user, 2026-11-15: "jitter has to be 0").
/// ADR-0017: 1€ filter in camera space, scale by the main display, steady zone. #11: the zone
/// is 0.5 camera px of head movement at every gain, so it holds at **every** speed 1–60, and a
/// deliberate 1 camera px move still moves the cursor.
#[test]
fn the_cursor_does_not_move_at_rest_on_a_wide_desktop() {
    still_at_rest(1..=15);
}

// Every gain 1–60, split so the test harness runs the parts in parallel.
#[test]
fn the_cursor_does_not_move_at_rest_gains_16_to_30() {
    still_at_rest(16..=30);
}

#[test]
fn the_cursor_does_not_move_at_rest_gains_31_to_45() {
    still_at_rest(31..=45);
}

#[test]
fn the_cursor_does_not_move_at_rest_gains_46_to_60() {
    still_at_rest(46..=60);
}

fn still_at_rest(gains: std::ops::RangeInclusive<u32>) {
    for gain in gains {
        let gain = gain as f32;
        let mut r = Rig::new(Settings {
            gain_x: gain,
            gain_y: gain,
            ..Settings::default()
        });
        r.input(Input::DesktopChanged(users_desktop()));
        r.cmd(Command::ToggleStart);
        r.frame(0.0, 0.0, SCENE);
        r.cmd(Command::SelectTarget(Point2::new(320.0, 240.0)));
        r.hold(HANDOVER_US, 0.0, SCENE);
        assert_eq!(r.c.pointer_owner(), PointerOwner::Camera);
        // The largest display (2560 × 1440) sets the scale.
        let scale = Scale {
            x: f64::from(gain * 2560.0 / 640.0),
            y: f64::from(gain * 1440.0 / 480.0),
        };
        let mut path = Vec::new();
        for i in 0..150 {
            let (dx, dy) = rest_tremor(i);
            let s = r.frame(dx, dy, SCENE);
            // The first second settles; the next 4 s are measured.
            if i >= 30 {
                let cursor = s.pointer_target();
                assert!(
                    cursor.is_some(),
                    "camera moves the pointer every tracked frame"
                );
                path.push(PathSample {
                    t_us: r.now,
                    cursor,
                });
            }
        }
        // Shared metric definitions (ADR-0020): no creep, no shake.
        let m = metrics::rest(&path, scale);
        assert_eq!(
            (m.moves, m.reversals),
            (0, 0),
            "gain {gain}: cursor moved at rest: {m:?}"
        );

        // A deliberate 1 camera px move reaches the cursor (past the 0.5 px zone).
        let before = path.last().and_then(|p| p.cursor).unwrap();
        let after = r.hold(1_000_000, 1.0, SCENE).pointer_target().unwrap();
        assert!(
            metrics::reached(before, after, (1.0, 0.0), scale),
            "gain {gain}: {before:?} → {after:?}"
        );
    }
}
