---
title: Accessibility options
description: Larger controls, the dark and light themes, the fixed keys, and how the app shows its state without relying on colour.
order: 11
---

The app is meant to be easy to see and to operate with a head-controlled pointer. These are
the options and behaviours that help.

## Larger controls

**Options → General → Larger controls** (off by default) makes the controls in both
windows bigger:

| Element | Normal | Larger |
|---|---|---|
| Text | 14 pt | 16 pt |
| Buttons and switches (height) | 36 pt | 48 pt |
| Borders | 1 pt | 2 pt |
| Focus ring | 2 pt | 4 pt |

Bigger targets are easier to hit with a head-controlled pointer and with
[dwell clicking](../clicking/). **Restore defaults** doesn't turn it off.

## Dark and light

**Options → General → Appearance** switches between **Dark** (the default) and **Light**.
Both are black, white and grey, with two accents: a green ring that marks the target in
the preview and yellow main buttons. **Restore defaults** keeps your choice.

## Status without colour

The app never tells you something by colour alone. Every state has an icon and a line of
text under the preview, such as **Tracking**, **Holding · moved too fast** or **Target
lost**, and the ring around the target changes shape as well as colour (dashed while
holding, a cross when lost). The table is in
[Selecting a tracking point](../tracking-point/#what-the-ring-shows).

## Keys

Two keys work without the mouse:

| Key | macOS | Windows and Linux |
|---|---|---|
| Start / pause | <kbd>F10</kbd> | <kbd>F11</kbd> |
| Recenter | <kbd>F12</kbd> | <kbd>F12</kbd> |

They work in any app where your system allows it and always inside the app's window;
**Options → General → Keyboard** shows which. They are fixed and can't be changed. On a
Mac laptop keyboard you may need to hold <kbd>Fn</kbd>.

## Hands-free clicking

[Dwell click](../clicking/) clicks when you hold the pointer still, so you can use the app
without pressing anything once it runs. Its timing and the size of the hold-still area are
adjustable.

## Known gaps

- The "Searching" animation in the preview doesn't follow the system's reduce-motion
  setting yet.
- Screen readers haven't been tested with the app on any system yet.
