---
title: Settings
description: Every option in the Options window with its range and default, what Restore defaults keeps, and where your settings are saved.
order: 10
---

Press **Options** in the Tracking window to open the Options window. It has five tabs:
**Pointer**, **Clicking**, **Camera**, **Screen** and **General**. Every change applies at
once and is saved by itself; there is no Save button. **Close** closes the window.

## Pointer

| Setting | Range | Default |
|---|---|---|
| Horizontal speed | 1–60 | 20 |
| Vertical speed | 1–60 | 20 |
| Keep both speeds the same | On / Off | On |
| Smoothing | 0–0.95 | 0.20 |

Speed sets how far the pointer moves when you turn your head; smoothing makes it steadier
but slower to follow. More in [Using the pointer](../using-the-pointer/#speed-and-smoothing).

## Clicking

| Setting | Range | Default |
|---|---|---|
| Dwell to click | On / Off | Off |
| Dwell time | 0.3–3.0 s | 1.0 s |
| Pause after a click | 0.5–5.0 s | 2.0 s |
| Hold-still area | 1–15 % | 5 % |

The last three can only be changed while **Dwell to click** is on. What each one does:
[Clicking and dwell clicking](../clicking/#settings).

## Camera

| Setting | Range | Default |
|---|---|---|
| Camera | System default or a connected camera | System default |
| Mirror preview | On / Off | On |
| Find the target again | On / Off | Off |

- **Camera:** which webcam to use. A camera that was chosen before but isn't plugged in
  shows as "(not connected)".
- **Mirror preview:** with it on, the preview works like a mirror and moving your head
  right moves the pointer right.
- **Capture format** isn't a setting: it shows the picture size and frame rate the camera
  delivers, for example "640 × 480 · 30 fps", or "Camera not started". The app asks for
  640 × 480 at 30 frames per second and takes the nearest the camera offers.
- **Find the target again:** after the target is lost, look for it and continue by
  itself. See [Selecting a tracking point](../tracking-point/#when-the-target-is-lost).

## Screen

| Setting | Range | Default |
|---|---|---|
| Screen limits (left, right, top, bottom) | 0–1000 px | 0 px |
| Main monitor | Primary monitor, Centre of all monitors, or one monitor | Primary monitor |

**Main monitor** only appears when more than one monitor is connected. Both are explained
in [Using the pointer](../using-the-pointer/#screen-limits).

## General

| Setting | Range | Default |
|---|---|---|
| Appearance | Dark / Light | Dark |
| Larger controls | On / Off | Off |
| Show diagnostics | On / Off | Off |

- **Appearance** and **Larger controls:** see [Accessibility options](../accessibility/).
- **Keyboard** isn't a setting: it lists the start and recenter keys and whether each one
  works in any app or only in the app's window. The keys can't be changed.
- **Mouse takeover** isn't a setting either: it reminds you that moving the physical mouse
  takes control and the camera takes over again after 4 seconds of keeping it still
  ([Mouse handover](../mouse-handover/)).
- **Show diagnostics** writes the frame rate, the tracker's match score and its status on
  the preview. It is meant for bug reports and tuning; you don't need it for everyday use.

## Restore defaults

**Restore defaults**, at the bottom of the Options window, puts every setting back to its
default **except** the chosen camera, **Appearance** and **Larger controls**: those are
about your equipment and how the app looks, so they stay as they are.

## Where settings are saved

Settings are saved in a file named `settings.toml`, in this folder:

| System | Folder |
|---|---|
| macOS | `~/Library/Application Support/Open Camera Mouse` |
| Windows | `%APPDATA%\Open Camera Mouse` |
| Linux | `$XDG_CONFIG_HOME/open-camera-mouse`, or `~/.config/open-camera-mouse` |

Updating the app keeps them. To start over with every default, quit the app and delete
the file; the next launch starts like the first one.

## When settings can't be used

The app always starts, even with a bad settings file. If something was wrong, a one-line
notice above the preview says what happened:

- **"Some settings were out of range and were reset: …"** A value was outside its range
  (for example after editing the file by hand). Only the named settings went back to their
  defaults; the rest were kept.
- **"Your settings file was damaged, so defaults are in use."** The file couldn't be read
  as settings. It was kept in the same folder as `settings.toml.corrupt`; your next
  changes are saved to a new file.
- **"Your settings were saved by a newer version of Open Camera Mouse, so defaults are in
  use. Changes made now won't be saved."** You opened an older version after a newer one.
  The newer file is left alone; update the app to use it again.
- **"Your settings file couldn't be read, so defaults are in use. Changes made now won't
  be saved."** The system didn't let the app read the file, for example because of its
  permissions.
- **"Settings couldn't be saved: …"** Writing the file failed. The change still applies
  until you quit.
