---
title: Clicking and dwell clicking
description: How to click hands-free by holding the pointer still, and the settings that control it.
order: 8
---

With **dwell click** on, holding the pointer still clicks for you: one left click where
the pointer rests. It is **off by default**; turn it on in **Options → Clicking → Dwell
to click**.

## How a dwell click works

1. Move the pointer to what you want to click and hold your head still.
2. A thin green arc around the target ring in the preview fills up while you hold still.
3. When it is full, after the dwell time (1 second by default), the app clicks once.
4. It doesn't click again in the same place. The next click needs you to move away and
   come to rest again, or to stay still until the pause after a click (2 seconds by
   default) has passed and then for the dwell time again.

"Still" means the pointer stays inside a small area around where it came to rest, so a
little wobble doesn't restart the count.

## When it doesn't click

Dwell click only works while the camera has the pointer and is tracking. It doesn't click:

- while the target is lost or **Holding**;
- while the mouse has the pointer, including the 4 seconds before the camera takes over
  again (see [Mouse handover](../mouse-handover/));
- while paused or recentering.

## Settings

In **Options → Clicking**, available when **Dwell to click** is on.

| Setting | Range | Default |
|---|---|---|
| Dwell time | 0.3–3.0 s | 1.0 s |
| Pause after a click | 0.5–5.0 s | 2.0 s |
| Hold-still area | 1–15 % | 5 % |

- **Dwell time:** how long to hold still before a click.
- **Pause after a click:** no second click until you move away or this time passes.
- **Hold-still area:** how far the pointer may wander and still count as still, as a share
  of the screen (of its shorter side). Raise it if small movements keep restarting the
  count; lower it if clicks land on the wrong thing.

Dwell click makes left clicks only.
