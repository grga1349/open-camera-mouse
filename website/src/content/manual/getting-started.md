---
title: Getting started
description: What Open Camera Mouse does, what you need and how to start using it.
order: 1
---

Open Camera Mouse moves the mouse pointer with your head. Your webcam watches a spot on
your face that you pick, such as the tip of your nose, and the pointer follows it. If you
turn on dwell click, holding the pointer still clicks for you.

Everything runs on your computer: no camera image leaves it, and there are no accounts, no
cloud and no telemetry.

## What you need

- A Mac with Apple Silicon, a 64-bit Windows PC, or an x86_64 Linux PC running an X11
  session. The details are in [System requirements](../system-requirements/).
- A webcam. The camera built into a laptop works.

## First steps

1. [Install the app](../installation/) for your system.
2. Give it the permissions it asks for: the camera, and on macOS also Accessibility, so it
   can move the pointer. The installation steps say where to find them.
3. Press **Start** (or the start key) and click a spot on your face in the camera preview.
   The tip of the nose works well ([Selecting a tracking point](../tracking-point/)).
4. Keep the physical mouse still for 4 seconds: the camera takes over the pointer. Moving
   the mouse takes the pointer back at once, and 4 seconds of keeping it still hands it to
   the camera again.
5. To pick the target without clicking, press **Recenter** (<kbd>F12</kbd>). A circle
   appears in the middle of the preview and counts down 5-4-3-2-1. Look straight at the
   screen; the spot in the circle becomes the new target ([Recentering](../recentering/)).
6. **Options** has the pointer speed (gain), smoothing, dwell click, the camera, screen
   limits, the main monitor, the theme and larger controls. Changes are saved at once.

After the first session, [Using the pointer](../using-the-pointer/) and
[Clicking and dwell clicking](../clicking/) cover everyday use.

## Keys

The keys are fixed.

| Action | macOS | Windows and Linux |
|---|---|---|
| Start or pause | <kbd>F10</kbd> | <kbd>F11</kbd> |
| Recenter | <kbd>F12</kbd> | <kbd>F12</kbd> |

On a Mac laptop keyboard you may need to hold <kbd>Fn</kbd>. The keys work from any app
where the system allows it, and always inside the Open Camera Mouse window; the window
shows which ones work everywhere.

## Getting help

Check the known limitations in [System requirements](../system-requirements/#known-limitations).
If something doesn't work, please report it on
[GitHub Issues](https://github.com/grga1349/open-camera-mouse/issues).
