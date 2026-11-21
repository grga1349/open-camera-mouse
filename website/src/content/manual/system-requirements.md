---
title: System requirements
description: Which computers, systems and cameras Open Camera Mouse runs on, and its known limitations.
order: 2
---

Open Camera Mouse is a desktop app for macOS, Windows and Linux. It doesn't run on phones,
tablets or Chromebooks.

## Computer

| System | Supported | Not supported |
|---|---|---|
| macOS | Apple Silicon (M-series) Macs, macOS 11 or later | Intel Macs |
| Windows | 64-bit Windows on x64 | |
| Linux | x86_64 with an X11 session | Wayland sessions |

## Camera

A webcam that your system recognises, such as the one built into a laptop. The app asks
the camera for 640×480 at 30 frames per second and uses the nearest format it offers.

Cameras that only deliver MJPEG aren't supported. External USB webcams haven't been tested
yet.

## Permissions

- **macOS:** Camera, and Accessibility to move the pointer.
- **Windows:** access to the camera for desktop apps, if Windows asks.
- **Linux:** your user needs access to the camera, usually through the `video` group.

[Installation](../installation/) shows where each one is granted.

## Known limitations

| System | Limitation |
|---|---|
| macOS | Not notarized yet, so macOS blocks the first launch until you allow it ([how](../installation/#macos-apple-silicon)). Apple Silicon only. |
| Windows | Not signed yet, so SmartScreen shows a warning. The pointer can't control windows that run as administrator, the User Account Control prompt or the lock screen. |
| Linux | X11 only. On Wayland the app can't move the pointer or catch the start and recenter keys outside its window; inside the window the keys still work. |
| All | The "Searching" animation in the preview doesn't follow the system's reduce-motion setting. |

If you run into a problem, report it on
[GitHub Issues](https://github.com/grga1349/open-camera-mouse/issues).
