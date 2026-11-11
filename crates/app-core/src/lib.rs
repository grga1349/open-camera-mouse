//! UI-less application controller (ADR-0003/ADR-0007).
//!
//! Phase 2 skeleton: the user-visible state set from BEHAVIOR_SPEC R-A1, a command enum and a
//! controller whose every transition is emitted as an event. The full state machine is
//! Phase 6 work.

pub mod ports;

/// Every app state is user-visible (R-A1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Stopped,
    Starting,
    NoCamera,
    PermissionDenied,
    NoTarget,
    Tracking,
    Lost,
    Recentering,
    Paused,
    CameraDisconnected,
}

/// Commands the UI (or a test) sends. The F11/F12 hotkeys map to `ToggleStart` and
/// `Recenter` (ADR-0006).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    ToggleStart,
    Recenter,
}

/// Events the controller emits. A UI renders state only from these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    StateChanged { from: AppState, to: AppState },
}

/// The application controller.
#[derive(Debug)]
pub struct Controller {
    state: AppState,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller {
    pub fn new() -> Self {
        Self {
            state: AppState::Stopped,
        }
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    /// Applies a command. Returns the events it caused, in order. No transition is silent.
    pub fn handle(&mut self, command: Command) -> Vec<Event> {
        let next = match (command, self.state) {
            (Command::ToggleStart, AppState::Stopped) => AppState::Starting,
            (Command::ToggleStart, _) => AppState::Stopped,
            // Recenter is wired in Phase 6; until then it changes nothing.
            (Command::Recenter, s) => s,
        };
        self.transition(next)
    }

    fn transition(&mut self, to: AppState) -> Vec<Event> {
        if to == self.state {
            return Vec::new();
        }
        let from = std::mem::replace(&mut self.state, to);
        vec![Event::StateChanged { from, to }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ports::{CameraError, CameraFrame, CameraSource, CaptureFormat, CaptureRequest};
    use tracking_core::GrayFrame;

    #[test]
    fn toggle_start_emits_every_transition() {
        let mut c = Controller::new();
        assert_eq!(
            c.handle(Command::ToggleStart),
            vec![Event::StateChanged {
                from: AppState::Stopped,
                to: AppState::Starting
            }]
        );
        assert_eq!(
            c.handle(Command::ToggleStart),
            vec![Event::StateChanged {
                from: AppState::Starting,
                to: AppState::Stopped
            }]
        );
    }

    #[test]
    fn noop_command_emits_nothing() {
        let mut c = Controller::new();
        assert!(c.handle(Command::Recenter).is_empty());
        assert_eq!(c.state(), AppState::Stopped);
    }

    /// A webcam-free fake, proving the port is implementable without hardware.
    struct FakeCamera {
        frames: u64,
    }

    impl CameraSource for FakeCamera {
        fn open(&mut self, r: CaptureRequest) -> Result<CaptureFormat, CameraError> {
            Ok(CaptureFormat {
                width: r.width,
                height: r.height,
                fps: r.fps as f32,
            })
        }
        fn next_frame(&mut self) -> Result<CameraFrame, CameraError> {
            self.frames += 1;
            Ok(CameraFrame {
                luma: GrayFrame::filled(640, 480, 128).unwrap(),
                timestamp_us: self.frames * 33_333,
            })
        }
        fn close(&mut self) {}
    }

    #[test]
    fn fake_camera_honours_default_request() {
        let mut cam = FakeCamera { frames: 0 };
        let fmt = cam.open(CaptureRequest::default()).unwrap();
        assert_eq!((fmt.width, fmt.height), (640, 480));
        let f = cam.next_frame().unwrap();
        assert_eq!(f.luma.width(), 640);
        assert_eq!(f.timestamp_us, 33_333);
    }
}
