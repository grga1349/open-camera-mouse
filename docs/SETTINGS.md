# Settings Reference

All settings are persisted to `config.json` in the platform config directory (see [RUNBOOK.md](RUNBOOK.md) for paths). Changes made via **Save** are written to disk and applied immediately.

---

## General

| Setting | Key | Default | Description |
|---------|-----|---------|-------------|
| Auto-start | `autoStart` | `false` | Start tracking automatically when the app launches. |

**Fixed shortcuts (not configurable):**
- `F11` — toggle start/stop
- `F12` — recenter tracker and reset cursor position

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
