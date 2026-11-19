//! What the Tracking window says about the current [`View`]: the status line (icon, title,
//! pointer owner, body, hint, actions), the preview overlay and which controls are enabled.
//! Pure, so every state's wording is unit-tested without a GUI. Wording follows the approved
//! design lab (`docs/design-lab.html` v3, `STATES`); it is draft-quality until the
//! Phase 8 user review.

use app_core::{AppState, HoldReason, MarkerKind, PointerOwner, TrackStatus, View};

/// A painted icon (see `icons.rs`). Status is always icon + text, never colour alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Play,
    Paused,
    Tracking,
    Held,
    NoTarget,
    Searching,
    Lost,
    Recenter,
    Mouse,
    Lock,
    PointerLock,
    CameraOff,
    Unplug,
    Error,
    Loading,
}

/// Something a status or card button asks for, beyond a plain controller command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// `Command::ToggleStart`: start, retry or reconnect the camera.
    Start,
    /// Open the OS page that grants pointer control (macOS Accessibility).
    OpenPointerSettings,
    /// Check pointer permission again (the runtime polls it, task 0008 D7).
    CheckPointerAccess,
    /// Open Options on the Camera tab.
    ChooseCamera,
    /// Copy the error text to the clipboard.
    CopyDetails,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    pub label: &'static str,
    pub action: Action,
    pub primary: bool,
}

const fn button(label: &'static str, action: Action, primary: bool) -> Button {
    Button {
        label,
        action,
        primary,
    }
}

/// A card over the preview for states without a live picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub icon: Icon,
    pub title: &'static str,
    pub body: String,
    pub buttons: Vec<Button>,
}

/// What the preview shows under the markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Camera {
    Live,
    /// Paused: the picture is dimmed with a pause glyph.
    Dimmed,
    /// Disconnected: the last frame, dimmed, "No signal".
    Frozen,
    /// No picture.
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub icon: Icon,
    pub title: String,
    /// Who moves the pointer, e.g. "camera", "mouse".
    pub pointer: &'static str,
    pub body: String,
    /// Monospace body (raw error text).
    pub body_mono: bool,
    pub hint: Option<&'static str>,
    pub buttons: Vec<Button>,
    pub card: Option<Card>,
    /// "Starting camera…" spinner over the preview.
    pub loading: bool,
    pub camera: Camera,
}

/// Seconds shown for a countdown: 5, 4, 3, 2, 1 (never 0 while it runs).
pub fn seconds(remaining_us: u64) -> u64 {
    remaining_us.div_ceil(1_000_000).max(1)
}

fn running(s: AppState) -> bool {
    matches!(
        s,
        AppState::NoTarget | AppState::Tracking | AppState::Lost | AppState::Recentering
    )
}

/// The primary Start / Pause / Resume button label, without the key.
pub fn start_label(s: AppState) -> &'static str {
    match s {
        AppState::Paused => "Resume",
        s if running(s) => "Pause",
        _ => "Start",
    }
}

/// Start is disabled while the camera is opening and where the preview card offers its own
/// retry button (lab).
pub fn start_enabled(s: AppState) -> bool {
    s == AppState::Stopped || s == AppState::Paused || running(s)
}

pub fn recenter_enabled(s: AppState) -> bool {
    matches!(
        s,
        AppState::NoTarget | AppState::Tracking | AppState::Lost | AppState::Paused
    )
}

/// A preview click selects a target (`Command::SelectTarget`) only in these states.
pub fn can_pick(s: AppState) -> bool {
    matches!(s, AppState::NoTarget | AppState::Tracking | AppState::Lost)
}

fn pointer_text(view: &View) -> &'static str {
    match view.pointer {
        PointerOwner::Camera if matches!(view.track, Some(TrackStatus::Held(_))) => "held still",
        PointerOwner::Camera => "camera",
        PointerOwner::Handover | PointerOwner::Mouse => "mouse",
        PointerOwner::PermissionNeeded => "needs permission",
        PointerOwner::Unsupported => "not available",
    }
}

fn camera_permission_body() -> &'static str {
    if cfg!(target_os = "macos") {
        "Allow it in System Settings > Privacy & Security > Camera, then try again."
    } else if cfg!(target_os = "windows") {
        "Allow it in Settings > Privacy & security > Camera, then try again."
    } else {
        "Check that you may use the camera and that no other app holds it, then try again."
    }
}

fn pointer_permission_body() -> &'static str {
    if cfg!(target_os = "macos") {
        "Tracking works, but the pointer can't move yet. Allow Open Camera Mouse under \
         Privacy & Security > Accessibility."
    } else {
        "Tracking works, but this system doesn't let the app move the pointer yet."
    }
}

fn pointer_permission_buttons() -> Vec<Button> {
    let mut b = Vec::new();
    if cfg!(target_os = "macos") {
        b.push(button(
            "Open Accessibility settings",
            Action::OpenPointerSettings,
            false,
        ));
    }
    b.push(button("Check again", Action::CheckPointerAccess, false));
    b
}

/// The status for a view. `start_key` is the start / pause key name on this OS (F10 on
/// macOS, F11 elsewhere, task 0008 D5), `recenter_key` is F12.
pub fn describe(view: &View, start_key: &str, recenter_key: &str) -> Status {
    let mut st = Status {
        icon: Icon::Tracking,
        title: String::new(),
        pointer: pointer_text(view),
        body: String::new(),
        body_mono: false,
        hint: None,
        buttons: Vec::new(),
        card: None,
        loading: false,
        camera: Camera::Off,
    };
    let set = |st: &mut Status, icon, title: &str, body: &str| {
        st.icon = icon;
        st.title = title.to_owned();
        st.body = body.to_owned();
    };
    match view.state {
        AppState::Stopped => {
            set(
                &mut st,
                Icon::Play,
                "Camera off",
                &format!("Press Start or {start_key} to turn on the camera."),
            );
            st.card = Some(Card {
                icon: Icon::Tracking,
                title: "Open Camera Mouse",
                body: format!(
                    "Move the pointer with your head. Start the camera, then click a point on \
                     your face. {start_key} starts and pauses, {recenter_key} recenters."
                ),
                buttons: vec![button("Start camera", Action::Start, true)],
            });
        }
        AppState::Starting => {
            set(
                &mut st,
                Icon::Loading,
                "Starting camera…",
                "This usually takes a second or two.",
            );
            st.loading = true;
        }
        AppState::NoCamera => {
            set(
                &mut st,
                Icon::CameraOff,
                "No camera found",
                "Connect a camera, or choose another in Options > Camera.",
            );
            st.card = Some(Card {
                icon: Icon::CameraOff,
                title: "No camera found",
                body: "Connect a webcam, then try again.".to_owned(),
                buttons: vec![
                    button("Try again", Action::Start, true),
                    button("Choose camera", Action::ChooseCamera, false),
                ],
            });
        }
        AppState::PermissionDenied => {
            set(
                &mut st,
                Icon::Lock,
                "Camera access needed",
                camera_permission_body(),
            );
            st.card = Some(Card {
                icon: Icon::Lock,
                title: "Allow camera access",
                body: "Video stays on this computer. Nothing is recorded or sent.".to_owned(),
                buttons: vec![button("Try again", Action::Start, true)],
            });
        }
        AppState::Error => {
            let details = view
                .error
                .clone()
                .unwrap_or_else(|| "unknown camera error".to_owned());
            set(&mut st, Icon::Error, "Something went wrong", &details);
            st.body_mono = true;
            st.card = Some(Card {
                icon: Icon::Error,
                title: "Something went wrong",
                body: "The camera reported an error. Restarting it usually helps.".to_owned(),
                buttons: vec![
                    button("Restart camera", Action::Start, true),
                    button("Copy details", Action::CopyDetails, false),
                ],
            });
        }
        AppState::CameraDisconnected => {
            set(
                &mut st,
                Icon::Unplug,
                "Camera disconnected",
                "Check the cable or close other apps using the camera.",
            );
            st.camera = Camera::Frozen;
            st.card = Some(Card {
                icon: Icon::Unplug,
                title: "Camera disconnected",
                body: "Plug it back in, then reconnect.".to_owned(),
                buttons: vec![
                    button("Reconnect", Action::Start, true),
                    button("Choose camera", Action::ChooseCamera, false),
                ],
            });
        }
        AppState::NoTarget => {
            set(
                &mut st,
                Icon::NoTarget,
                "Select a target",
                "Click a point on your face. The tip of the nose or between the eyebrows works \
                 well.",
            );
        }
        AppState::Tracking => match view.track {
            Some(TrackStatus::Held(HoldReason::MotionOutlier)) => set(
                &mut st,
                Icon::Held,
                "Holding · moved too fast",
                "Move a little slower. Tracking continues by itself.",
            ),
            Some(TrackStatus::Held(HoldReason::NearEdgeMovingOut)) => set(
                &mut st,
                Icon::Held,
                "Holding · near the edge",
                "Move back toward the middle of the picture.",
            ),
            _ => set(
                &mut st,
                Icon::Tracking,
                "Tracking",
                "Move your head to move the pointer. Click the preview to choose another point.",
            ),
        },
        AppState::Lost => {
            if view.marker.map(|m| m.kind) == Some(MarkerKind::Searching) {
                set(
                    &mut st,
                    Icon::Searching,
                    "Looking for the target…",
                    "Face the camera. The mouse has control until the target is found.",
                );
            } else if view.loss_recovery && !view.redetect_armed {
                set(
                    &mut st,
                    Icon::Lost,
                    "Target lost",
                    "Click a point on your face again to continue.",
                );
                st.hint = Some(
                    "That spot has too little detail to be found again automatically. Try the \
                     tip of your nose or an eyebrow.",
                );
            } else {
                set(
                    &mut st,
                    Icon::Lost,
                    "Target lost",
                    &format!(
                        "Click the same point on your face again, or press {recenter_key} to \
                         recenter."
                    ),
                );
            }
        }
        AppState::Recentering => {
            let n = view.recenter.map(|r| seconds(r.remaining_us)).unwrap_or(1);
            set(
                &mut st,
                Icon::Recenter,
                &format!("Recentering · {n}"),
                "Look straight at the screen and put the tip of your nose (or the point you want \
                 to track) inside the white circle. Tracking starts when the countdown ends.",
            );
        }
        AppState::Paused => {
            set(
                &mut st,
                Icon::Paused,
                "Paused",
                &format!(
                    "The pointer is yours. Press {start_key}, Resume or Recenter to continue. You \
                     line up with the centre circle first."
                ),
            );
            st.camera = Camera::Dimmed;
        }
    }
    if running(view.state) {
        st.camera = Camera::Live;
    }

    // Pointer sub-state (orthogonal to the app state, BEHAVIOR.md R-A1).
    if can_pick(view.state) {
        match view.pointer {
            PointerOwner::PermissionNeeded => {
                set(
                    &mut st,
                    Icon::PointerLock,
                    "Pointer control needs permission",
                    pointer_permission_body(),
                );
                st.hint = None;
                st.buttons = pointer_permission_buttons();
            }
            PointerOwner::Handover => {
                let n = view.handover_remaining_us.map(seconds).unwrap_or(1);
                set(
                    &mut st,
                    Icon::Mouse,
                    &format!("Mouse in control · camera resumes in {n} s"),
                    "The camera takes over again when the mouse is still.",
                );
            }
            PointerOwner::Unsupported => {
                st.hint = Some(
                    "This session can't move the pointer (for example on Wayland). Tracking \
                     still works.",
                );
            }
            PointerOwner::Camera | PointerOwner::Mouse => {}
        }
    }
    st
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::ports::Point2;
    use app_core::{Diagnostics, LossReason, Marker, RecenterView};

    fn view(state: AppState, pointer: PointerOwner) -> View {
        View {
            state,
            pointer,
            track: None,
            marker: None,
            recenter: None,
            handover_remaining_us: None,
            dwell_progress: None,
            frame_size: Some((640, 480)),
            frame_seq: 1,
            diagnostics: Diagnostics::default(),
            loss_recovery: false,
            redetect_armed: false,
            capture: None,
            error: None,
        }
    }

    fn d(v: &View) -> Status {
        describe(v, "F10", "F12")
    }

    #[test]
    fn every_app_state_has_an_icon_and_a_title() {
        use AppState as S;
        let cases = [
            (S::Stopped, Icon::Play, "Camera off"),
            (S::Starting, Icon::Loading, "Starting camera…"),
            (S::NoCamera, Icon::CameraOff, "No camera found"),
            (S::PermissionDenied, Icon::Lock, "Camera access needed"),
            (S::Error, Icon::Error, "Something went wrong"),
            (S::NoTarget, Icon::NoTarget, "Select a target"),
            (S::Tracking, Icon::Tracking, "Tracking"),
            (S::Lost, Icon::Lost, "Target lost"),
            (S::Paused, Icon::Paused, "Paused"),
            (S::CameraDisconnected, Icon::Unplug, "Camera disconnected"),
        ];
        for (state, icon, title) in cases {
            let s = d(&view(state, PointerOwner::Mouse));
            assert_eq!((s.icon, s.title.as_str()), (icon, title), "{state:?}");
            assert!(!s.body.is_empty(), "{state:?}");
        }
    }

    #[test]
    fn recentering_title_counts_down_from_the_controller_deadline() {
        let mut v = view(AppState::Recentering, PointerOwner::Mouse);
        v.recenter = Some(RecenterView {
            center: Point2::new(320.0, 240.0),
            remaining_us: 4_200_000,
        });
        assert_eq!(d(&v).title, "Recentering · 5");
        v.recenter.as_mut().unwrap().remaining_us = 999_999;
        assert_eq!(d(&v).title, "Recentering · 1");
        assert_eq!(d(&v).icon, Icon::Recenter);
    }

    #[test]
    fn held_states_say_why() {
        let mut v = view(AppState::Tracking, PointerOwner::Camera);
        v.track = Some(TrackStatus::Held(HoldReason::MotionOutlier));
        let s = d(&v);
        assert_eq!(
            (s.icon, s.title.as_str(), s.pointer),
            (Icon::Held, "Holding · moved too fast", "held still")
        );
        v.track = Some(TrackStatus::Held(HoldReason::NearEdgeMovingOut));
        assert_eq!(d(&v).title, "Holding · near the edge");
    }

    #[test]
    fn lost_wording_follows_find_the_target_again() {
        let mut v = view(AppState::Lost, PointerOwner::Mouse);
        v.track = Some(TrackStatus::Lost(LossReason::LowConfidence));
        v.marker = Some(Marker {
            position: Point2::new(100.0, 100.0),
            kind: MarkerKind::Lost,
        });
        let off = d(&v);
        assert_eq!(off.title, "Target lost");
        assert!(off.body.contains("F12"), "{}", off.body);
        assert_eq!(off.hint, None);

        v.loss_recovery = true;
        v.redetect_armed = false;
        assert!(d(&v).hint.unwrap().contains("too little detail"));

        v.redetect_armed = true;
        v.marker.as_mut().unwrap().kind = MarkerKind::Searching;
        let s = d(&v);
        assert_eq!(
            (s.icon, s.title.as_str()),
            (Icon::Searching, "Looking for the target…")
        );
    }

    #[test]
    fn handover_shows_the_controller_countdown() {
        let mut v = view(AppState::Tracking, PointerOwner::Handover);
        v.handover_remaining_us = Some(2_500_000);
        let s = d(&v);
        assert_eq!(s.icon, Icon::Mouse);
        assert_eq!(s.title, "Mouse in control · camera resumes in 3 s");
        assert_eq!(s.pointer, "mouse");
    }

    #[test]
    fn pointer_permission_overrides_running_states_only() {
        let v = view(AppState::Tracking, PointerOwner::PermissionNeeded);
        let s = d(&v);
        assert_eq!(s.icon, Icon::PointerLock);
        assert_eq!(s.title, "Pointer control needs permission");
        assert_eq!(s.pointer, "needs permission");
        assert!(
            s.buttons
                .iter()
                .any(|b| b.action == Action::CheckPointerAccess)
        );

        let paused = d(&view(AppState::Paused, PointerOwner::PermissionNeeded));
        assert_eq!(paused.title, "Paused");
    }

    #[test]
    fn unsupported_pointer_is_a_hint() {
        let s = d(&view(AppState::Tracking, PointerOwner::Unsupported));
        assert_eq!(s.title, "Tracking");
        assert_eq!(s.pointer, "not available");
        assert!(s.hint.unwrap().contains("can't move the pointer"));
    }

    #[test]
    fn error_body_is_the_raw_error_in_monospace() {
        let mut v = view(AppState::Error, PointerOwner::Mouse);
        v.error = Some("camera backend error: device busy".to_owned());
        let s = d(&v);
        assert_eq!(s.body, "camera backend error: device busy");
        assert!(s.body_mono);
        let card = s.card.unwrap();
        assert!(card.buttons.iter().any(|b| b.action == Action::CopyDetails));
    }

    #[test]
    fn cards_and_camera_modes() {
        use AppState as S;
        for (state, card, camera) in [
            (S::Stopped, true, Camera::Off),
            (S::Starting, false, Camera::Off),
            (S::NoCamera, true, Camera::Off),
            (S::PermissionDenied, true, Camera::Off),
            (S::Error, true, Camera::Off),
            (S::CameraDisconnected, true, Camera::Frozen),
            (S::NoTarget, false, Camera::Live),
            (S::Tracking, false, Camera::Live),
            (S::Lost, false, Camera::Live),
            (S::Recentering, false, Camera::Live),
            (S::Paused, false, Camera::Dimmed),
        ] {
            let s = d(&view(state, PointerOwner::Mouse));
            assert_eq!((s.card.is_some(), s.camera), (card, camera), "{state:?}");
        }
        assert!(d(&view(S::Starting, PointerOwner::Mouse)).loading);
    }

    #[test]
    fn keys_in_the_wording_come_from_the_caller() {
        let s = describe(&view(AppState::Paused, PointerOwner::Mouse), "F11", "F12");
        assert!(s.body.contains("Press F11"), "{}", s.body);
    }

    #[test]
    fn control_enablement_matches_the_lab() {
        use AppState as S;
        assert_eq!(start_label(S::Tracking), "Pause");
        assert_eq!(start_label(S::Paused), "Resume");
        assert_eq!(start_label(S::Stopped), "Start");
        assert!(!start_enabled(S::Starting));
        assert!(!start_enabled(S::NoCamera));
        assert!(start_enabled(S::Recentering));
        assert!(recenter_enabled(S::Paused));
        assert!(!recenter_enabled(S::Recentering));
        assert!(!recenter_enabled(S::Stopped));
        assert!(can_pick(S::Lost));
        assert!(!can_pick(S::Recentering));
    }
}
