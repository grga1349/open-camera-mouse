---
title: First-time setup and permissions
description: What you see on the first launch, the two permissions the app needs and what the app says when one is missing.
order: 4
---

Open Camera Mouse needs two things from your system: the **camera**, to see your face,
and permission to **move the pointer**. [Installation](../installation/) shows where each
one is granted. This page describes what the app shows before and after you grant them.

## The first launch

The Tracking window opens with the camera off. A card over the preview says what the app
does and which keys start and recenter it. Press **Start camera** (or **Start**, or the
start key: <kbd>F10</kbd> on macOS, <kbd>F11</kbd> on Windows and Linux).

The status line under the preview says "Starting camera…" while the camera opens, which
usually takes a second or two. When the picture appears, the status reads **Select a
target**: click a point on your face, as described in
[Selecting a tracking point](../tracking-point/).

The video is processed on your computer and isn't uploaded anywhere.

## Camera permission

On macOS the system asks for the camera the first time you start it; on Windows it may
ask too. If access is refused, the app shows **Camera access needed**, says where to allow
it, and offers **Try again**:

- **macOS:** System Settings → Privacy & Security → Camera.
- **Windows:** Settings → Privacy & security → Camera.
- **Linux:** check that your user may use the camera (usually the `video` group) and that
  no other app holds it.

## Pointer permission

On **macOS** the app needs **Accessibility** to move the pointer and click. Until it has
it, tracking still works but the pointer stays put, and the status reads **Pointer control
needs permission** with two buttons:

- **Open Accessibility settings** opens the right page in System Settings. Turn **Open
  Camera Mouse** on there.
- **Check again** checks at once. The app also checks by itself every few seconds, so you
  don't need to restart it.

On **Windows** and on **Linux with X11** no extra permission is needed. On a **Wayland**
session the app can't move the pointer at all; tracking still works and the status says
so. Use an X11 session instead (see [System requirements](../system-requirements/)).

## When the camera isn't available

| Status | What to do |
|---|---|
| No camera found | Connect a webcam and press **Try again**, or **Choose camera** to pick another one. |
| Camera disconnected | Plug it back in or close other apps that use it, then press **Reconnect**. The preview keeps the last picture, marked "No signal". |
| Something went wrong | Press **Restart camera**. **Copy details** copies the error text for a bug report. |

**Choose camera** opens Options on the Camera tab, where you can pick the camera. The
first time, the app uses the system's default camera.

## Checking the keys

The start and recenter keys work in any app where your system allows it, and always inside
the Open Camera Mouse window. **Options → General → Keyboard** lists each key with "works
in any app" or "works only in this window" and the reason. If a key only works in the
window, the status area under the preview says so too.
