---
title: Troubleshooting
description: What to do when the app won't open, the camera or pointer doesn't work, tracking is unsteady, or the keys or clicks don't do what you expect.
order: 12
---

Find the problem below. If none of it helps,
[report it on GitHub](https://github.com/grga1349/open-camera-mouse/issues) and say which
system you use and what the status line under the preview says.

## The app won't open

- **macOS says the app can't be opened or is from an unidentified developer.** The app
  isn't notarized yet. Open it once, then click **Open Anyway** in **System Settings →
  Privacy & Security** ([Installation](../installation/#macos-apple-silicon)). If that
  button doesn't appear, you can clear the download flag in Terminal instead:
  `xattr -dr com.apple.quarantine "/Applications/Open Camera Mouse.app"`.
- **Windows shows a SmartScreen warning.** The installer isn't signed yet: click **More
  info → Run anyway** ([Installation](../installation/#windows-x64)).
- **The Linux AppImage doesn't start.** It needs FUSE 2 (for example the `libfuse2`
  package). Or use the `.tar.gz` instead ([Installation](../installation/#linux-x86_64-x11)).
- **It's an Intel Mac, a phone or a tablet.** These aren't supported
  ([System requirements](../system-requirements/)).

## The camera doesn't work

The status line and a card over the preview say what is wrong and offer a button to try
again; [First-time setup](../first-time-setup/#when-the-camera-isnt-available) lists each
one. In short:

- **Camera access needed:** allow the camera in your system's privacy settings, then press
  **Try again** ([Camera permission](../first-time-setup/#camera-permission)).
- **No camera found:** connect a webcam, or pick another in **Options → Camera**.
- **Camera disconnected:** plug it back in or close other apps that use the camera, then
  press **Reconnect**.
- **Something went wrong:** press **Restart camera**. **Copy details** copies the error
  for a bug report.
- **The wrong camera opens:** choose the right one in **Options → Camera**.

Cameras that only deliver MJPEG aren't supported, and external USB webcams haven't been
tested yet.

## The pointer doesn't move

Look at the status line: after "Pointer:" it says who moves the pointer.

- **"needs permission"** (macOS): turn **Open Camera Mouse** on in **System Settings →
  Privacy & Security → Accessibility**. The app notices within a few seconds
  ([Pointer permission](../first-time-setup/#pointer-permission)).
- **"not available"** (Linux): you are in a Wayland session, where the app can't move the
  pointer. Log in with an X11 session.
- **"mouse"**: the physical mouse has the pointer. Keep it still for 4 seconds
  ([Mouse handover](../mouse-handover/)). Moving the mouse or trackpad by more than a few
  pixels during the countdown starts it over.
- **Target lost** or **Select a target**: click a point on your face, or press
  **Recenter** ([Selecting a tracking point](../tracking-point/)).
- **Paused**: press **Resume** or the start key.

On **Windows**, the pointer can't control windows running as administrator, the User
Account Control prompt or the lock screen. Windows doesn't allow it.

## The pointer is shaky, slow or can't reach an edge

- **Shaky:** raise **Smoothing** in **Options → Pointer**.
- **Lags behind your head:** lower **Smoothing**.
- **Too fast, or can't reach an edge:** lower or raise the **speed**, or
  [recenter](../recentering/) in a position you can hold comfortably.
- **Stays away from an edge:** check **Screen limits** in **Options → Screen**; the
  default is 0 on every side.
- **Starts on the wrong monitor:** choose the **Main monitor** in **Options → Screen**
  ([Several monitors](../using-the-pointer/#several-monitors)).
- **Moves the wrong way:** check that **Mirror preview** in **Options → Camera** is on.

## Tracking stops or the target is lost often

- Pick a point with clear detail, such as the **tip of the nose** or an eyebrow, not a
  plain cheek ([Which point works well](../tracking-point/#which-point-works-well)).
- **Holding · moved too fast:** move a little slower; tracking continues by itself.
- **Holding · near the edge:** move back toward the middle of the picture.
- To have the app look for a lost target by itself, turn on **Find the target again** in
  **Options → Camera**.

## Dwell click doesn't click, or clicks too much

- **Doesn't click:** check that **Dwell to click** is on in **Options → Clicking**. It
  only clicks while the camera has the pointer and is tracking: not while holding, lost,
  paused, recentering or while the mouse has the pointer
  ([When it doesn't click](../clicking/#when-it-doesnt-click)).
- **Small movements keep restarting the count:** raise the **Hold-still area**.
- **Clicks before you are ready:** raise the **Dwell time**.
- **Clicks land on the wrong thing:** lower the **Hold-still area**.

## The keys don't work

- **Options → General → Keyboard** says whether each key works in any app or only in the
  app's window, and why. Inside the window the keys always work.
- On a Mac laptop keyboard, hold <kbd>Fn</kbd> with <kbd>F10</kbd> or <kbd>F12</kbd>.
- On macOS the start key is <kbd>F10</kbd>, not <kbd>F11</kbd>: macOS uses <kbd>F11</kbd>
  itself.
- On a Linux Wayland session the keys only work inside the window.

## A notice about settings

A line above the preview about your settings file means it couldn't be used as it was;
[Settings](../settings/#when-settings-cant-be-used) explains each notice.
