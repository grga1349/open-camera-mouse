# Settings Reference

All settings are persisted to `config.json` in the platform config directory (see [RUNBOOK.md](RUNBOOK.md) for paths). Changes made via **Save** are written to disk and applied immediately.

---

## General

| Setting | Key | Default | Description |
|---------|-----|---------|-------------|
| Auto-start | `autoStart` | `false` | Start tracking automatically when the app launches. |
| Start/Pause hotkey | `startPause` | `F11` | Global hotkey that toggles tracking on/off, even while the app is in the background. |
| Recenter hotkey | `recenter` | `F12` | Global hotkey that recenters the tracker without pausing the camera. |

**Hotkey constraints:**
- Must be one of `F1`–`F20` — no modifier keys (Ctrl/Alt/Shift/Cmd) are supported, matching the `golang.design/x/hotkey` backend.
- `startPause` and `recenter` must be different keys.
- An invalid or duplicate value is rejected on save; a corrupted value in `config.json` falls back to the default (`F11`/`F12`) on load.
- Hotkeys are re-registered live when changed from Settings — no app restart required.

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

**Constants (not user-configurable):**
- Dwell radius = `30px` — cursor must stay within this radius while dwelling
