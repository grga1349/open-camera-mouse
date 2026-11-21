---
title: Using the pointer
description: How head movement maps to the pointer, how to pause, and the options for speed, smoothing, screen edges and several monitors.
order: 6
---

Once the camera has the pointer, the pointer follows your head: turn or move your head
right and the pointer moves right, up and it moves up. The preview is mirrored, so it
moves the same way you do.

## How movement maps to the screen

Each position of your head corresponds to one position on the screen. When the camera
takes over, the position your head is in at that moment becomes the middle of the screen;
from there, the further you move, the further the pointer goes. If you keep your head
still, the pointer stays still: small jitter is ignored.

If you can't comfortably reach an edge, raise the speed or
[recenter](../recentering/) in a more comfortable position.

## Pausing

Press **Pause** or the start key (<kbd>F10</kbd> on macOS, <kbd>F11</kbd> on Windows and
Linux) to give the pointer back to the mouse. The pointer moves to the middle of the main
monitor, the preview dims and the status reads **Paused**.

Press **Resume**, the start key or **Recenter** to continue. Resuming always
[recenters](../recentering/) first, so you line up with the centre circle again.

## Speed and smoothing

In **Options → Pointer**. The values are numbers, as in the app.

| Setting | Range | Default |
|---|---|---|
| Horizontal speed | 1–60 | 20 |
| Vertical speed | 1–60 | 20 |
| Smoothing | 0–0.95 | 0.20 |

- **Speed** sets how far the pointer moves when you turn your head. **Keep both speeds the
  same** (on by default) links the two; turn it off to set them separately.
- **Smoothing** makes the pointer steadier but slower to follow. Lower it if the pointer
  lags behind you; raise it if it is shaky.

Changes apply and are saved at once.

## Screen limits

**Options → Screen → Screen limits** keeps the pointer this many pixels away from each
screen edge (left, right, top, bottom; 0–1000, default 0). Use it to keep the pointer
away from an edge you never need, such as a hidden dock or taskbar.

## Several monitors

With more than one monitor, **Options → Screen** also shows **Main monitor**: where
recentering, pausing and the camera taking over put the pointer. Choose **Primary
monitor** (the default, the one your system calls primary), **Centre of all monitors**,
or one monitor by number; monitors are numbered left to right.

The main monitor only sets the starting point: the pointer can still reach every monitor.
