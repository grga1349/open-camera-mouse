---
title: FAQ
description: Short answers about privacy, cost, cameras, systems, clicking and the keys.
order: 13
---

## Does the camera picture leave my computer?

No. Open Camera Mouse processes the camera picture entirely on your computer and doesn't
record or upload video. There are no accounts, no cloud and no telemetry.

For debugging, developers can turn on a local diagnostic recording. It is off unless
explicitly enabled, stays on your computer, and is only shared if you choose to share it.

## What does it cost?

Nothing. Open Camera Mouse is free and open source; the code is on
[GitHub](https://github.com/grga1349/open-camera-mouse).

## Do I need a special camera?

No, an ordinary webcam works, including the one built into a laptop. Cameras that only
deliver MJPEG aren't supported, and external USB webcams haven't been tested yet
([System requirements](../system-requirements/#camera)).

## Which systems does it run on?

macOS on Apple Silicon, 64-bit Windows, and x86_64 Linux with an X11 session. Not Intel
Macs, Wayland sessions, phones or tablets ([System requirements](../system-requirements/)).

## Which point on my face should I pick?

The tip of the nose works well; the point between the eyebrows or an eyebrow works too.
Avoid plain areas with little detail
([Selecting a tracking point](../tracking-point/#which-point-works-well)).

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

It is inspired by the classic Camera Mouse, created at Boston College by James Gips and
Margrit Betke, but it is an independent project, written from scratch. It isn't affiliated
with or endorsed by Boston College or the original authors.
