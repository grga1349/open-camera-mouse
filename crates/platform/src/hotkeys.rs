//! Fixed hotkeys, task 0008 D5, ADR-0012 §"Global hotkeys". Not configurable (product
//! decision). Start / pause is F11 on Windows and Linux (2018 and legacy convention) and F10 on
//! macOS, where F11 is the system's Show Desktop shortcut and registering it fails (measured on
//! the dev Mac, ARCHITECTURE.md P13; user decision 2026-11-15). Recenter is F12 everywhere.
//!
//! Two sources feed the same commands:
//! - [`GlobalHotkeys`]: system-wide keys through `global-hotkey` (macOS Carbon hotkeys,
//!   Windows `RegisterHotKey`, X11 `XGrabKey`). Registration can fail (another app owns the
//!   key) or be unavailable (Wayland, ARCHITECTURE.md P12); the result per key is kept in a
//!   [`Registration`] so the UI can say so (P13).
//! - The app window's own key events, which the desktop shell forwards. They always work while
//!   the window has focus, whatever happened to the global registration.
//!
//! [`KeyRouter`] merges both so one physical press never toggles twice when an OS delivers it
//! to both the global hook and the focused window. It is pure, with an injected clock.

use std::sync::mpsc;
use std::time::Duration;

use app_core::Command;
use global_hotkey::hotkey::{Code, HotKey};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

/// One of the two fixed hotkeys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hotkey {
    /// F11 (F10 on macOS): start, pause, resume (through recenter).
    ToggleStart,
    /// F12: guided recenter.
    Recenter,
}

impl Hotkey {
    pub const ALL: [Hotkey; 2] = [Hotkey::ToggleStart, Hotkey::Recenter];

    /// The controller command this key sends.
    pub fn command(self) -> Command {
        match self {
            Hotkey::ToggleStart => Command::ToggleStart,
            Hotkey::Recenter => Command::Recenter,
        }
    }

    /// The key's name as shown to the user, and as `egui::Key::name` spells it, so the window
    /// can match its own key events against it.
    pub fn key_name(self) -> &'static str {
        self.key().1
    }

    /// The hotkey with this key name on this OS (`"F10"` → start / pause on macOS).
    pub fn from_key_name(name: &str) -> Option<Hotkey> {
        Hotkey::ALL.into_iter().find(|k| k.key_name() == name)
    }

    fn key(self) -> (Code, &'static str) {
        match self {
            Hotkey::ToggleStart => TOGGLE_START_KEY,
            Hotkey::Recenter => (Code::F12, "F12"),
        }
    }

    fn os_hotkey(self) -> HotKey {
        HotKey::new(None, self.key().0)
    }

    fn index(self) -> usize {
        match self {
            Hotkey::ToggleStart => 0,
            Hotkey::Recenter => 1,
        }
    }
}

#[cfg(target_os = "macos")]
const TOGGLE_START_KEY: (Code, &str) = (Code::F10, "F10");
#[cfg(not(target_os = "macos"))]
const TOGGLE_START_KEY: (Code, &str) = (Code::F11, "F11");

/// Where a press came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The system-wide hook.
    Global,
    /// The app window's own key event (focused window).
    Window,
}

/// What happened when a key was registered system-wide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Registration {
    /// The key works system-wide.
    Active,
    /// The OS refused the key (usually another app holds it). It still works in the window.
    Failed(String),
    /// This session has no global hotkeys at all (Wayland, no display). Window only.
    Unsupported(String),
}

impl Registration {
    pub fn is_active(&self) -> bool {
        matches!(self, Registration::Active)
    }
}

/// Presses of the same key from the *other* source within this window are the same physical
/// press delivered twice, and are dropped. Far shorter than any deliberate double press of a
/// toggle, far longer than delivery skew between the two paths.
pub const DUPLICATE_WINDOW: Duration = Duration::from_millis(300);

/// Merges global and in-window presses into commands, dropping cross-source duplicates.
#[derive(Debug, Default)]
pub struct KeyRouter {
    /// Last accepted press per key: (source, time in µs).
    last: [Option<(Source, u64)>; 2],
}

impl KeyRouter {
    pub fn new() -> Self {
        Self::default()
    }

    /// A key press (not a repeat, not a release) at `now_us` on the app's monotonic clock.
    /// Returns the command to send, or `None` if it duplicates a press just taken from the
    /// other source.
    pub fn press(&mut self, key: Hotkey, source: Source, now_us: u64) -> Option<Command> {
        let slot = &mut self.last[key.index()];
        if let Some((prev_source, prev_us)) = *slot {
            let window_us = DUPLICATE_WINDOW.as_micros() as u64;
            if prev_source != source && now_us.saturating_sub(prev_us) < window_us {
                return None;
            }
        }
        *slot = Some((source, now_us));
        Some(key.command())
    }
}

/// System-wide start / pause and recenter keys. Keep it alive for as long as the keys should work; dropping it
/// unregisters them.
pub struct GlobalHotkeys {
    manager: Option<GlobalHotKeyManager>,
    status: [Registration; 2],
    /// Ids handed out by `global-hotkey` for our two keys.
    ids: [u32; 2],
    presses: mpsc::Receiver<u32>,
}

impl std::fmt::Debug for GlobalHotkeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalHotkeys")
            .field("active", &self.manager.is_some())
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl GlobalHotkeys {
    /// Registers both keys system-wide. Never fails: per-key problems are in
    /// [`Self::status`].
    ///
    /// Call it once per process, on the thread that runs the window event loop (on macOS that
    /// must be the main thread; on Windows the hotkey window lives on the calling thread). In
    /// eframe that is the app-creator closure. `wake` is called from whatever thread the OS
    /// delivers the press on, so the UI can repaint while it is in the background (e.g.
    /// `egui::Context::request_repaint`).
    pub fn register(wake: impl Fn() + Send + Sync + 'static) -> Self {
        let ids = Hotkey::ALL.map(|k| k.os_hotkey().id());
        let (tx, presses) = mpsc::channel();
        if let Some(reason) = unsupported_session() {
            return Self {
                manager: None,
                status: [
                    Registration::Unsupported(reason.clone()),
                    Registration::Unsupported(reason),
                ],
                ids,
                presses,
            };
        }
        // Set once per process (the crate keeps one global handler). A second `register`
        // would keep forwarding to the first instance's channel; the app creates one.
        GlobalHotKeyEvent::set_event_handler(Some(move |e: GlobalHotKeyEvent| {
            if e.state == HotKeyState::Pressed && tx.send(e.id).is_ok() {
                wake();
            }
        }));
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                let reason = format!("global hotkeys could not start: {e}");
                return Self {
                    manager: None,
                    status: [
                        Registration::Failed(reason.clone()),
                        Registration::Failed(reason),
                    ],
                    ids,
                    presses,
                };
            }
        };
        let status = Hotkey::ALL.map(|k| match manager.register(k.os_hotkey()) {
            Ok(()) => Registration::Active,
            Err(e) => Registration::Failed(refusal_reason(k, &e)),
        });
        Self {
            manager: Some(manager),
            status,
            ids,
            presses,
        }
    }

    /// How registering `key` went.
    pub fn status(&self, key: Hotkey) -> &Registration {
        &self.status[key.index()]
    }

    /// Presses since the last call, oldest first. Never blocks.
    pub fn drain(&self) -> Vec<Hotkey> {
        self.presses
            .try_iter()
            .filter_map(|id| key_for_id(&self.ids, id))
            .collect()
    }
}

impl Drop for GlobalHotkeys {
    fn drop(&mut self) {
        if let Some(m) = &self.manager {
            for (key, status) in Hotkey::ALL.iter().zip(&self.status) {
                if status.is_active() {
                    let _ = m.unregister(key.os_hotkey());
                }
            }
        }
    }
}

/// Plain words for a refused key. `AlreadyRegistered` is what macOS returned for F11 while
/// Show Desktop owned it (dev Mac, task 0008 D5) and what Windows/X11 return when another app
/// grabbed the key first.
fn refusal_reason(key: Hotkey, e: &global_hotkey::Error) -> String {
    match e {
        global_hotkey::Error::AlreadyRegistered(_) => format!(
            "another app or a system shortcut already uses {}",
            key.key_name()
        ),
        other => other.to_string(),
    }
}

fn key_for_id(ids: &[u32; 2], id: u32) -> Option<Hotkey> {
    Hotkey::ALL.into_iter().find(|k| ids[k.index()] == id)
}

/// Why this session can't have global hotkeys, if it can't. On X11 `global-hotkey` reports
/// success even when its connection fails (it registers on a thread that has already exited),
/// so the cases it can't see are caught here.
fn unsupported_session() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let var = |k| std::env::var(k).ok();
        unsupported_linux(
            var("XDG_SESSION_TYPE").as_deref(),
            var("WAYLAND_DISPLAY").as_deref(),
            var("DISPLAY").as_deref(),
        )
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn unsupported_linux(
    session_type: Option<&str>,
    wayland_display: Option<&str>,
    display: Option<&str>,
) -> Option<String> {
    if crate::pointer::is_wayland(session_type, wayland_display) {
        Some("global hotkeys aren't available on Wayland".to_owned())
    } else if display.is_none_or(|d| d.trim().is_empty()) {
        Some("no X11 display, so global hotkeys aren't available".to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: u64 = 1_000;

    #[test]
    fn keys_map_to_the_fixed_commands() {
        assert_eq!(Hotkey::ToggleStart.command(), Command::ToggleStart);
        assert_eq!(Hotkey::Recenter.command(), Command::Recenter);
        let start = if cfg!(target_os = "macos") {
            "F10"
        } else {
            "F11"
        };
        assert_eq!(Hotkey::ToggleStart.key_name(), start);
        assert_eq!(Hotkey::Recenter.key_name(), "F12");
    }

    #[test]
    fn key_names_resolve_back() {
        for k in Hotkey::ALL {
            assert_eq!(Hotkey::from_key_name(k.key_name()), Some(k));
        }
        assert_eq!(Hotkey::from_key_name("F1"), None);
        let other_start = if cfg!(target_os = "macos") {
            "F11"
        } else {
            "F10"
        };
        assert_eq!(Hotkey::from_key_name(other_start), None);
    }

    #[test]
    fn os_ids_are_distinct_and_resolve_back() {
        let ids = Hotkey::ALL.map(|k| k.os_hotkey().id());
        assert_ne!(ids[0], ids[1]);
        for k in Hotkey::ALL {
            assert_eq!(key_for_id(&ids, ids[k.index()]), Some(k));
        }
        let other = HotKey::new(None, Code::F1).id();
        assert_eq!(key_for_id(&ids, other), None);
    }

    #[test]
    fn one_press_from_each_source_is_one_command() {
        let mut r = KeyRouter::new();
        let t = 10_000 * MS;
        assert_eq!(
            r.press(Hotkey::ToggleStart, Source::Global, t),
            Some(Command::ToggleStart)
        );
        assert_eq!(
            r.press(Hotkey::ToggleStart, Source::Window, t + 40 * MS),
            None
        );
        // And the other order.
        let t = 20_000 * MS;
        assert_eq!(
            r.press(Hotkey::Recenter, Source::Window, t),
            Some(Command::Recenter)
        );
        assert_eq!(r.press(Hotkey::Recenter, Source::Global, t + 5 * MS), None);
    }

    #[test]
    fn repeated_presses_from_one_source_all_count() {
        for src in [Source::Global, Source::Window] {
            let mut r = KeyRouter::new();
            for i in 0..3 {
                assert_eq!(
                    r.press(Hotkey::ToggleStart, src, 50_000 * MS + i * 50 * MS),
                    Some(Command::ToggleStart),
                    "{src:?} press {i}"
                );
            }
        }
    }

    #[test]
    fn a_later_press_from_the_other_source_counts() {
        let mut r = KeyRouter::new();
        assert!(r.press(Hotkey::ToggleStart, Source::Global, 0).is_some());
        assert!(
            r.press(Hotkey::ToggleStart, Source::Window, 300 * MS)
                .is_some()
        );
        assert!(
            r.press(Hotkey::ToggleStart, Source::Global, 299 * MS + 300 * MS)
                .is_none()
        );
    }

    #[test]
    fn keys_are_independent() {
        let mut r = KeyRouter::new();
        assert!(r.press(Hotkey::ToggleStart, Source::Global, 0).is_some());
        assert!(r.press(Hotkey::Recenter, Source::Window, MS).is_some());
    }

    #[test]
    fn a_taken_key_is_explained_in_plain_words() {
        let e = global_hotkey::Error::AlreadyRegistered(Hotkey::Recenter.os_hotkey());
        assert_eq!(
            refusal_reason(Hotkey::Recenter, &e),
            "another app or a system shortcut already uses F12"
        );
    }

    #[test]
    fn linux_sessions_without_global_hotkeys() {
        assert!(unsupported_linux(Some("wayland"), Some("wayland-0"), Some(":0")).is_some());
        assert!(unsupported_linux(None, Some("wayland-0"), Some(":0")).is_some());
        assert!(unsupported_linux(Some("x11"), None, None).is_some());
        assert!(unsupported_linux(Some("x11"), None, Some("")).is_some());
        assert!(unsupported_linux(Some("x11"), None, Some(":0")).is_none());
        assert!(unsupported_linux(None, None, Some(":1")).is_none());
    }
}
