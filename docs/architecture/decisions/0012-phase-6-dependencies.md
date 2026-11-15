# ADR-0012: Phase 6 dependencies and settings persistence

Status: Accepted (persistence D2, camera D3, pointer D4, hotkeys D5)
Date: 2026-11-12
Task: 0008 slice D2. Amends ADR-0007 (what `app-core` may depend on).

## Context

Phase 6 needs its first crates outside egui: settings persistence now (D2), then a camera
backend (D3), pointer injection (D4) and global hotkeys (D5). ADR-0007 keeps the cores free of
I/O, OS and GUI crates, but it doesn't say whether `app-core` may use `serde` to describe the
settings it already owns. `deny.toml` allows permissive licences only (no MPL, no GPL).

Versions and licences re-queried from the crates.io API on 2026-11-12. Licences of every
crate newly added to `Cargo.lock` were checked against `deny.toml`.

## Options considered

**Where the serde derives live**
1. `#[derive(Serialize, Deserialize)]` on `app_core::Settings`, with `serde` in `app-core`. One
   struct, so field names on disk can't drift from the code. `serde` is pure data mapping (no
   I/O, no OS, no GUI) and builds on every target.
2. A mirror DTO in `platform` with hand-written conversions. Keeps `app-core` to `thiserror`
   only, but every new setting has to be added in two places, and a missed field is silently
   dropped on save.

**Config directory**
1. `directories` 6.0.0. Pulls in `dirs-sys` → `option-ext` 0.2.0, which is **MPL-2.0**, so
   `cargo deny` fails. Allowing it would mean a licence exception for one path lookup.
2. A small hand-written function in `platform` (three OS rules, tested on every OS with
   injected environment values). No new dependency.

**Format**: TOML (`toml` 1.2.1) vs JSON. TOML is hand-editable with comments, and the file is
small and flat. `toml` was already in the tree through eframe's dependencies.

## Decision

- `app-core` may depend on `serde` (`derive` + `std`, no other features) for `Settings` and its
  nested types, besides `thiserror` and the two cores. No I/O crates, no `toml`, no OS or GUI
  crates. `scripts/check-core-deps.sh` now checks `app-core` against that allowlist too.
- Persistence lives in `platform::settings_store` (`toml`, `std::fs`). The config directory is
  computed by hand-written code, not `directories`:
  macOS `~/Library/Application Support/Open Camera Mouse/`, Windows
  `%APPDATA%\Open Camera Mouse\`, Linux/Unix `$XDG_CONFIG_HOME/open-camera-mouse/` or
  `~/.config/open-camera-mouse/`. Relative environment values are ignored.
- File: `settings.toml`, `schema_version` on the first line, then `Settings` fields. Missing
  fields take defaults, unknown fields are ignored (within one schema version). Writes are
  atomic (temporary file + rename).
- Loading never fails; each fallback comes with a user-facing notice (R-A5):

| File | Settings | Store |
|---|---|---|
| missing | defaults, no notice | writable |
| unparsable, wrong types, missing or older `schema_version` | defaults; file moved to `settings.toml.corrupt` | writable |
| newer `schema_version` | defaults | read-only (an older build never overwrites a newer file) |
| values out of range | only those fields reset (`Settings::repaired`) | writable |
| unreadable (permissions, I/O) | defaults | read-only |

### Crates

| Crate | Version | Licence | Where | Status |
|---|---|---|---|---|
| `serde` | 1.0.229 | MIT OR Apache-2.0 | app-core (derive) | **adopted** |
| `toml` | 1.2.1 | MIT OR Apache-2.0 | platform | **adopted** |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | platform (dev) | **adopted** |
| `directories` | 6.0.0 | MIT OR Apache-2.0, but `option-ext` is MPL-2.0 | — | **rejected** |
| `nokhwa` (`input-native` only, no `decoding`) | 0.10.11 | Apache-2.0 (51 new transitive crates, all permissive, checked 2026-11-15) | platform (all OS) | **adopted** (D3, measured below) |
| `objc2-av-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | — | **fallback**, not built (D3): `nokhwa` met every gate; would need `unsafe` + an ADR |
| `objc2-core-graphics` (+ `objc2-core-foundation` for `CGPoint`) | 0.3.2 | Zlib OR Apache-2.0 OR MIT | platform (macOS) | **adopted** (D4): posted events, Accessibility check, display list |
| `objc2-application-services` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | — | **not needed** (D4): `CGPreflightPostEventAccess` in CoreGraphics checks exactly the permission posting needs |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | platform (Windows) | **adopted** (D4), compiled in CI; device validation Phase 7 |
| `x11rb` (`xtest`, `randr`, pure-Rust connection) | 0.13.2, not 0.14.0 | MIT OR Apache-2.0 | platform (Linux) | **adopted** (D4) at 0.13.2, the version winit already pulls in, so no second copy; compiled in CI; device validation Phase 7 |
| `global-hotkey` (default features off) | 0.10.0 | Apache-2.0 OR MIT (3 new lockfile crates: itself, `crossbeam-channel` 0.5.17, `keyboard-types` 0.8.3, both MIT OR Apache-2.0; checked 2026-11-15) | platform (all OS) | **adopted** (D5, measured below) |
| `enigo` | 0.6.1 | MIT | — | **not used** (D4): we'd still need per-OS code for position, permission, displays and the suppression setting |

D3, D4 and D5 each update their row here (adopted or rejected, with the measurement) and add
the crate to `platform` per target. Check every new transitive licence before adopting:
`cargo deny` runs only in CI.

## Camera backend (task 0008 D3, 2026-11-15)

Measured on the dev Mac (macOS 14.7.3 arm64, built-in FaceTime HD Camera), first in a
throwaway spike, then through the adapter on a worker thread
(`crates/platform/tests/camera_device.rs`, `#[ignore]`d, run with `--ignored`):

| Check | Result |
|---|---|
| Opens | yes, permission already granted to the terminal |
| 640×480 negotiated | yes, YUYV (`yuvs`), reported 30 fps |
| Real FPS (timestamps, 120 frames after 30 warm-up) | 30.09 (adapter), 30.00–30.02 (spike) |
| Open → first frame | cold 1.2–1.4 s, warm 0.11–0.21 s |
| Close (stop stream + drop) | 76–97 ms. Camera LED going off (nokhwa #224) not checked by the agent: D7 manual QA |
| Re-open after close in one process | yes, same format and FPS |
| Panics | none on this camera; the P1 panics are reported on external webcams (C922), not reproducible here |

Findings that shaped the adapter:
- `nokhwa` 0.10.11 on macOS maps an **NV12** request to a 10-bit format the FaceTime camera
  refuses ("Cannot fulfill request"), and maps every 4:2:0 output to `YUYV` by mistake. The
  adapter asks for YUYV first and asks for NV12 only off macOS.
- `compatible_camera_formats()` returns an empty list on macOS, so the adapter can't pick from
  a list. It tries requests in order: YUYV 640×480@30, NV12 (not macOS), highest frame rate at
  640×480, driver default.
- `frame()` blocks with no timeout. Disconnect detection therefore lives on the reader side
  (no frame for 3 s → `Disconnected`), and `close` waits at most 2 s for the capture thread.
- `decoding` (JPEG via `mozjpeg`) stays off. A camera that only offers MJPEG gets a `Backend`
  error naming it. If Phase 7 hardware needs MJPEG, add a pure-Rust decoder through an ADR.
- `nokhwa`'s macOS bindings pull in `block` 0.1.6, which raises a future-incompatibility
  warning (not an error) on the pinned toolchain. Watch it on toolchain bumps. It's the
  strongest argument for the `objc2-av-foundation` fallback later.

Why not the thin `objc2-av-foundation` adapter now: `nokhwa` passed every gate above and covers
Windows and Linux with the same code, while the objc2 adapter would need `unsafe`
(the workspace denies it), a delegate class and its own Windows/Linux counterparts. It stays the
fallback if P1 panics or the `block` warning become real problems in Phase 7.

## Pointer adapter (task 0008 D4, 2026-11-15)

All four pointer crates were already in `Cargo.lock` through eframe/winit at the same
versions: D4 added no crate to the lockfile, only edges.

Measured on the dev Mac (macOS 14.7.3 arm64, two displays: 1440×900 main at (0, 0) and
2560×1440 at (1440, −540)), with a throwaway spike, then the adapter
(`crates/platform/tests/pointer_device.rs`, `#[ignore]`d):

| Check | `CGWarpMouseCursorPosition` | Posted `MouseMoved` (`CGEventPost`, HID tap) |
|---|---|---|
| Needs Accessibility | **no**, works without | **yes**: dropped silently without it, no error |
| Lands | synchronously, rounded to whole points | asynchronously: 0.3–0.9 ms typical, ~38 ms worst of 60, sub-point precision |
| Cost per call | ~40 µs | ~3 µs (first call ~25 ms) |
| Physical mouse while the cursor is held at 30 Hz (user moving the mouse, 6 s) | **0 of 162** samples show the movement: the physical mouse is dead | default source (0.25 s suppression): 5 of 162. **Source with suppression 0: 89 of 163** |
| Target in a dead zone or off every display | clamped by macOS | clamped by macOS, oddly: (5000, 100) → (1440, 100), not the right display's edge |

Other measurements: `CGPreflightPostEventAccess` costs **≈10.6 ms per call**; reading the
position (`CGEventCreate` + location) 0.12 µs; listing displays 0.15 ms.

Decision:
- **macOS: posted events** from a `CGEventSource` (HID system state) with local-event
  suppression set to 0. Warping is rejected: it needs no permission, but it blocks the physical
  mouse, so handover (ADR-0006) can't work. Accessibility is therefore required to move the
  cursor at all (as in 2018). `access()` = `CGPreflightPostEventAccess`; `request_access()` =
  `CGRequestPostEventAccess` (the system prompt); `open_access_settings()` opens
  `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`. A click
  checks access first (rare, so the 10 ms is affordable) and reports `PermissionDenied`;
  moves don't (too slow per frame). The runtime (D7) checks `access()` at start and polls it
  every few seconds while running or waiting for a grant, never per frame. Revoking access
  mid-run otherwise looks like the user grabbing the mouse until that poll runs.
- **Windows: `SetCursorPos`** for moves (exact physical pixels, no normalisation to 0–65535
  needed), `SendInput` left down/up for clicks, `GetCursorPos`, `EnumDisplayMonitors` for
  displays. No permission; UIPI-blocked or secure-desktop calls become `Backend` errors.
  Assumes per-monitor DPI awareness (winit's default): Phase 7 validates.
- **Linux: XTest** (`xtest_fake_input` motion and button 1) over `x11rb`'s pure-Rust
  connection, `QueryPointer` for the position, RandR 1.5 monitors (root window if RandR can't
  say). A Wayland session (`XDG_SESSION_TYPE=wayland`, or `WAYLAND_DISPLAY` set with no X11
  session type) and a missing X display or XTEST extension → `PointerError::Unsupported`
  from `SystemPointer::new()`. The runtime turns that into `PointerAccess::Unsupported`.
- **Shared, pure, all OS** (`platform::pointer::geometry`): the controller's desktop is the
  displays' bounding box (R-P5, negative origins tested). Targets are snapped to the nearest
  display before they reach the OS, because the OS clamps dead-zone targets in surprising
  places. The adapter reports positions it caused itself as the requested point (`Echo`):
  the latest snapped target always, plus earlier targets and the pre-move cursor position
  for 100 ms. Otherwise a snapped target or a move that lands later than one frame would
  differ from the commanded point by more than 5 px and trigger a false handover. A user's
  movement doesn't match those points, so it passes through.
- `PointerSink` (app-core port) gains `access()`, `click()` and `desktop()`. The runtime must
  read `position()` **before** the next `move_to` (observe → controller → move), so each
  observation judges the previous move.
- `unsafe`: `platform` opts out locally per ADR-0007: one block for `CGGetActiveDisplayList`
  on macOS, and the Windows module (Win32 calls only), each with a `SAFETY` comment.

Not checked here: the click path on device (D7 manual QA; an automated click would hit
whatever is under the cursor), Windows and X11 on device (Phase 7), and the macOS
Accessibility grant for a `.app` rather than the terminal (D7, together with P2).

## Global hotkeys (task 0008 D5, 2026-11-15)

`global-hotkey` 0.10.0 in `platform::hotkeys`: macOS Carbon `RegisterEventHotKey` (exclusive),
Windows `RegisterHotKey`, X11 `XGrabKey`. Its other deps (`objc2-app-kit`, `windows-sys` 0.59,
`x11rb` 0.13, `xkeysym`, `once_cell`) were already in the lockfile.

Measured on the dev Mac (macOS 14, built-in + default shortcuts):

| Key | Registration | Why |
|---|---|---|
| F11 | **refused**, `AlreadyRegistered` | System "Show Desktop" holds it (PLATFORM_RISKS P13, now VERIFIED) |
| F10 | active | — |
| F12 | active | — |

Decision (user, 2026-11-15): start / pause is **F10 on macOS**, F11 on Windows and Linux (the
2018/legacy users' key); recenter F12 everywhere. Still fixed, not configurable. Amends
ADR-0006's hotkey row and BEHAVIOR_SPEC R-A4.

Design:
- `GlobalHotkeys::register` never fails. Each key gets a `Registration` (`Active`,
  `Failed(reason)`, `Unsupported(reason)`) and the window shows one line per key: "works in any
  app" or "works only in this window: …". A taken key is reported in plain words ("another app
  or a system shortcut already uses F12").
- It must be created on the event-loop thread (macOS: main thread; Windows: the hidden hotkey
  window lives on the calling thread): the eframe app-creator closure. The crate's event
  handler (set once per process) forwards presses into a channel and calls
  `egui::Context::request_repaint`, so presses arrive while the window is in the background.
- On X11 the crate's `register` returns `Ok` even when its connection thread failed to start,
  so `platform` itself treats Wayland (same test as the pointer adapter) and a missing
  `DISPLAY` as `Unsupported` without calling it (P12).
- The window's own key events (`egui::Event::Key`, pressed, not repeat) always work as the
  fallback. `KeyRouter` (pure, injected clock) merges both sources: a press of the same key from
  the other source within 300 ms is the same physical press and is dropped, so a focused window
  never toggles twice.

Not checked here: delivery of a press from the background on device (registration only; D7
manual QA), Windows and X11 on device (Phase 7), `Fn` needed on Mac laptop keyboards (the
default media-key layout; document in D7 QA / Phase 8 help).

## Consequences

- One `Settings` type is the in-memory model, the validation and the on-disk schema. Renaming
  a field is a schema change: bump `Settings::SCHEMA_VERSION` and add a migration in
  `settings_store::decode`, which now rejects older versions as corrupt because none have
  shipped.
- A hand-edited file with one bad value keeps every other setting.
- Running an older build after a newer one shows defaults and saves nothing until the newer
  build is back. That's preferable to losing the newer settings.
- The hand-written Windows path reads `%APPDATA%` instead of calling
  `SHGetKnownFolderPath`. If Phase 7 finds a setup where the variable is missing or wrong, the
  fix is that API through `windows-sys` (already a candidate), not `directories`.
- Saving on change is wired with the Options window (D6). D2 loads at startup and shows the
  notice in the skeleton window.
