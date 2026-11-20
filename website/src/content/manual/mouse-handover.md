---
title: Mouse handover
description: How the physical mouse and the camera share the pointer, and when the camera takes back over.
order: 9
---

You can use the physical mouse (or trackpad) at any time, even while the camera is
tracking. The two take turns; you don't need to pause first.

## The mouse takes over at once

Move the physical mouse by more than a few pixels and it has the pointer immediately. The
status reads **Mouse in control · camera resumes in 4 s** and counts down.

## The camera takes back over after 4 seconds

Keep the mouse still for 4 seconds and the camera takes the pointer back. Moving the mouse
during the countdown starts it over.

When the camera takes over, the pointer moves to the middle of the
[main monitor](../using-the-pointer/#several-monitors), and your head's current position
becomes the middle of the screen. This is the same as when tracking starts, so you never
have to find your old head position again.

## After picking a target

Clicking a point in the preview uses the mouse, so the same rule applies: after the click,
keep the mouse still for 4 seconds and the camera takes over. Recentering is the
exception: at the end of its countdown the camera takes the pointer at once.

## Dwell click during handover

While the mouse has the pointer, dwell click doesn't click. See
[Clicking and dwell clicking](../clicking/#when-it-doesnt-click).

The handover can't be turned off; **Options → General → Mouse takeover** describes it.
