# Product

## What it is

Open Camera Mouse is a hands-free desktop pointer for people who can't easily use a mouse.
The user clicks a distinctive spot on their face in the webcam preview (the tip of the nose
works well); the app tracks that spot frame to frame and moves the OS pointer with it, with an
optional dwell click. Everything runs on the device; no frame leaves the machine.

It started as a Go/Wails/GoCV app (v0.3.0, still in `main`'s history and the `v0.3.0` tag)
and was rewritten from scratch in Rust for v0.4.0 without OpenCV. Behaviour follows the
original **Camera Mouse 2018** where that is practical, and improves on it where it was
weak (ADR-0006). New behaviour is added as an option, off by default, unless the user
approves a new default.

## Principles

- Accurate, stable, low-latency tracking on ordinary webcams; zero cursor shake at rest.
- On-device and private: no cloud, no accounts, no telemetry.
- Accessibility first: big hit targets, status never shown by colour alone, keyboard use.
- Simple setup: one camera permission and one pointer permission, explained in the app.
- macOS, Windows and Linux (X11) are all release targets; don't assume one OS's behaviour.
- Deterministic, offline-testable core: measured changes over clever, unproven ones.

## Non-goals

Cloud features, face recognition, generic AI inference, medical certification claims,
OpenCV, a standalone tracking CLI, configurable hotkeys (fixed by user decision), automatic
recenter.

## Features and defaults

| Area | Behaviour | Default |
|---|---|---|
| Camera | Colour preview, mirrored; capture 640×480 @ 30 fps (nearest the device offers) | first camera, mirror on |
| Target | Click a point in the preview; recenter picks the frame centre after a 5-4-3-2-1 countdown | — |
| Tracker | Pyramidal Lucas–Kanade, 2018 preset plus an anchored appearance gate (ADR-0011, ADR-0018) | — |
| Loss | Lost target hands control back to the mouse; "Find the target again" re-detects it automatically | re-detection off |
| Pointer | Absolute mapping with per-axis gain 1–60, 1€ smoothing, automatic steady zone (0.2 camera px), screen limits with offset drift | gain 20 / 20 linked, smoothing 0.20 |
| Main monitor | Where recenter and pause put the cursor; every display stays reachable | OS primary |
| Handover | Moving the physical mouse takes over at once; the camera takes back over after 4 s still | on |
| Dwell click | One click after holding still; re-arms after moving away or the refractory time | off; 1 s dwell, 2 s refractory, radius 0.05 of the screen |
| Hotkeys | Start/pause **F10 on macOS** (macOS reserves F11), **F11** on Windows/Linux; **F12** recenter. Global where the OS allows, always in the window | fixed |
| Settings | Saved on change (TOML, schema-versioned); bad values reset per field with a notice | — |

Settings files: macOS `~/Library/Application Support/Open Camera Mouse`, Windows
`%APPDATA%\Open Camera Mouse`, Linux `$XDG_CONFIG_HOME/open-camera-mouse` (else
`~/.config/open-camera-mouse`).

## UI design

Approved reference: [`design-lab.html`](design-lab.html) (open it in a browser; it simulates
every state). Rules:

- **Two fixed-size windows:** Tracking (560×620: preview, status line, Start/Pause, Recenter,
  Options) and Options (400×620; tabs Pointer · Clicking · Camera · Screen · General).
  General ends with the app version in muted help text.
- **Monochrome chrome, colour preview.** Exactly two accent hues: the neon-green `#00ff00`
  thin ring marking the target (grey when lost) and yellow primary buttons. Nowhere else.
- **Themes:** Carbon (dark, default) and Paper (light), switched in Options; "Larger
  controls" scales sizes for motor accessibility.
- **Status** is an icon plus a text line, never colour alone. Every app state is visible:
  Stopped, Starting, No camera, Permission denied, No target, Tracking, Held, Lost, Searching,
  Recentering, Paused, Camera disconnected, Error; plus the pointer owner (camera, handover
  countdown, mouse, permission needed, unsupported).
- Controls show real numbers (gain 20, not "medium").

Design tokens (in `apps/desktop/src/theme.rs`):

| Token | Carbon | Paper | Used for |
|---|---|---|---|
| bg | `#000000` | `#ececec` | panel and window fill |
| surface / surface-2 | `#0a0a0a` / `#151515` | `#f8f8f8` / `#ffffff` | fields, buttons |
| line / line-strong | `#262626` / `#3f3f3f` | `#d6d6d6` / `#a8a8a8` | separators / control borders |
| text / muted | `#fafafa` / `#8f8f8f` | `#111111` / `#5a5a5a` | text / help text |
| marker | `#00ff00` | `#00ff00` | target ring only |
| yellow | `#ffd60a` | `#f5c518` | primary button fill only (text `#111111`) |
| radius | 8 frames / 6 widgets | same | |
| size (normal / larger) | font 14 / 16, hit 36 / 48, border 1 / 2, focus 2 / 4 | same | |
