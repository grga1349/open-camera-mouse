---
title: FAQ
description: Short answers about privacy, cost, cameras, systems, clicking and the keys.
order: 13
---

## Does the camera picture leave my computer?

No. The video is processed on your computer and never recorded or sent. There are no
accounts, no cloud and no telemetry.

## What does it cost?

Nothing. Open Camera Mouse is free and open source; the code is on
[GitHub](https://github.com/grga1349/open-camera-mouse).

## Do I need a special camera?

No, an ordinary webcam works, including the one built into a laptop. Cameras that only
deliver MJPEG aren't supported, and external USB webcams haven't been tested yet
([System requirements](../system-requirements/#camera)).

## Which systems does it run on?

macOS on Apple Silicon, 64-bit Windows, and x86_64 Linux with an X11 session. Not Intel
Macs, Wayland sessions, phones or tablets. Windows and Linux builds are tested
automatically but not yet on real devices ([System requirements](../system-requirements/)).

## Which point on my face should I pick?

The tip of the nose works well; the point between the eyebrows or an eyebrow works too.
Avoid plain areas with little detail
([Selecting a tracking point](../tracking-point/#which-spot-works-well)).

## How do I click without a mouse?

Turn on **Dwell to click** in **Options → Clicking**. Then holding the pointer still for a
second makes one left click ([Clicking and dwell clicking](../clicking/)).

## Can it right-click, double-click or drag?

No. Dwell click makes left clicks only.

## Can I still use my mouse?

Yes, at any time. Moving it takes the pointer at once; after 4 seconds of keeping it still
the camera takes over again ([Mouse handover](../mouse-handover/)).

## Can I change the keys?

No. Start / pause is <kbd>F10</kbd> on macOS and <kbd>F11</kbd> on Windows and Linux;
recenter is <kbd>F12</kbd> everywhere. The keys are fixed on purpose.

## Does it recenter by itself?

No. Recentering only happens when you press **Recenter** or <kbd>F12</kbd>, or when you
resume after a pause ([Recentering](../recentering/)).

## How do I go back to the default settings?

Press **Restore defaults** at the bottom of the Options window. It keeps your camera,
theme and larger controls ([Settings](../settings/#restore-defaults)).

## Is it related to the original Camera Mouse?

Open Camera Mouse is an independent open-source project inspired by the classic Camera
Mouse. It isn't affiliated with or endorsed by the original authors.
