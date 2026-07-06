# Settings Reference

All settings are persisted to `config.json` in the platform config directory (see [RUNBOOK.md](RUNBOOK.md) for paths). Changes made via **Save** are written to disk and applied immediately.

---

## General

| Setting | Key | Default | Description |
|---------|-----|---------|-------------|
| Auto-start | `autoStart` | `false` | Start tracking automatically when the app launches. |

**Fixed shortcuts (not configurable):**
- `F11` — toggle start/stop
- `F12` — recenter tracker and reset cursor position (see [Recenter flow](#recenter-flow) below)

---

## Tracking

| Setting | Key | Default | Range | Description |
|---------|-----|---------|-------|-------------|
| Template size | `templateSizePx` | `45` | 30 / 45 / 60 | Side length (px) of the patch extracted from the frame and used as the match template. Larger = more distinctive, more stable. Smaller = faster updates. |

**Constants (not user-configurable):**
- Search margin = `templateSizePx × 2` — derived automatically
- Score threshold = `0.68` — minimum NCC score to accept a match
- Adaptive template = disabled

---

## Pointer

| Setting | Key | Default | Range | Description |
|---------|-----|---------|-------|-------------|
| Gain | `gainMultiplier` | `8.0` | 1–30 | Multiplier applied to raw pixel delta. Higher = more cursor movement per head movement. |
| Smoothing | `smoothing` | `0.30` | 0.05–1.0 | EMA lerp coefficient. Higher = more responsive, less smooth. Lower = smoother, more lag. Values ≤ 0 or > 1 are reset to the default on load. |

**Constants (not user-configurable):**
- Deadzone = `1px` — sub-pixel deltas are ignored
- Max speed = `35px` — per-frame displacement cap

---

## Clicking

| Setting | Key | Default | Range | Description |
|---------|-----|---------|-------|-------------|
| Dwell enabled | `dwellEnabled` | `false` | on/off | Enable hover-to-click. Toggled from the main screen. |
| Dwell time | `dwellTimeMs` | `500` | 200–1500ms | How long the cursor must stay still before a click fires. |
| Right click | `rightClickEnabled` | `false` | on/off | When on, dwell fires a right click instead of a left click. Toggled from the main screen. |

**Constants (not user-configurable):**
- Dwell radius = `30px` — cursor must stay within this radius while dwelling

---

## Recenter flow

Triggered from the main screen's **Recenter** button or the `F12` hotkey — both go through the same guided flow:

1. Tracking and cursor movement pause immediately; the tracking overlay hides.
2. The camera preview stays live. A **white** square (sized to `templateSizePx`) appears at the exact center of the frame — position the desired tracking point (e.g. your nose) inside it.
3. After a 3-second countdown, the frame's center pixel is picked as the new tracking target.
4. Tracking resumes and the normal green/red overlay returns.

Recenter requires tracking to already be running (`Start`/`F11` first) — otherwise it fails visibly rather than silently no-op-ing.
