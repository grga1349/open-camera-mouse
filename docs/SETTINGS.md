# Settings Reference

All settings are persisted to `config.json` in the platform config directory (see [RUNBOOK.md](RUNBOOK.md) for paths). Changes made in the UI take effect immediately; only **Save** writes them to disk.

---

## General

| Setting | Key | Default | Description |
|---------|-----|---------|-------------|
| Auto-start | `autoStart` | `false` | Start tracking automatically when the app launches. Useful if you always want the camera mouse active on open. |
| Dwell on startup | `dwellOnStartup` | `false` | If true, dwell clicking is enabled when the app launches even if it was disabled when settings were last saved. Guards against accidentally launching with a click mode you didn't intend. |

---

## Tracking

Controls how the template-matching algorithm finds and follows your marker.

| Setting | Key | Default | Description |
|---------|-----|---------|-------------|
| Template size | `templateSizePx` | `30` | Side length (px) of the square patch extracted from the frame and used as the match template. **Larger** = more distinctive patch, more stable under partial occlusion, but slower to update and more sensitive to scale changes. **Smaller** = faster updates, more responsive, but easier to lose. |
| Search margin | `searchMarginPx` | `30` | How far (px) from the last known position to search for the template each frame. **Larger** handles fast head movement but makes the algorithm slower and increases false-match risk. **Smaller** is faster and more accurate but loses tracking if you move quickly. |
| Score threshold | `scoreThreshold` | `0.60` | Minimum normalized cross-correlation score (0–1) required to accept a match. **Higher** (e.g. 0.75) = fewer false positives, loses tracking more readily. **Lower** (e.g. 0.45) = follows through low-confidence frames, more prone to drifting onto wrong features. |
| Adaptive template | `adaptiveTemplate` | `true` | Blend the current matched region back into the template each frame so it gradually adapts to changes in lighting, expression, or marker appearance. Disable if tracking becomes unstable under fast appearance changes. |
| Template update alpha | `templateUpdateAlpha` | `0.20` | Blend weight for adaptive updates (0–1). `0.20` means each frame contributes 20% to the template. **Higher** = faster adaptation, less robust to mis-matches. **Lower** = very conservative, slow to adapt. Only used when `adaptiveTemplate` is true. |
| Marker shape | `markerShape` | `"circle"` | Shape of the overlay drawn on the preview. `"circle"` or `"square"`. Visual only — no effect on tracking. |

---

## Pointer

Controls how raw head-movement deltas are converted into cursor movement.

| Setting | Key | Default | Description |
|---------|-----|---------|-------------|
| Sensitivity | `sensitivity` | `30` | Overall cursor speed on a 1–100 scale. Internally maps to a gain and smoothing curve: higher sensitivity increases gain and decreases smoothing lag. This is the primary speed knob — use advanced settings to tune per-axis. |
| Deadzone | `deadzonePx` | `1` | Minimum head movement (px in camera space) required before the cursor moves. Movements smaller than this are ignored, which suppresses hand tremor and high-frequency noise. Increase if the cursor drifts when you hold still. |
| Max speed | `maxSpeedPx` | `25` | Hard cap on cursor displacement per frame (px in screen space). Prevents sudden large jumps caused by tracking errors or very fast head movement. Lower values make the cursor feel more controlled; too low will make it feel sluggish at high sensitivity. |

### Advanced pointer (optional overrides)

If set, these override the values derived from `sensitivity`. Set all three or none.

| Setting | Key | Description |
|---------|-----|-------------|
| X gain | `advanced.gainX` | Screen pixels moved per camera pixel of horizontal head movement. `0` means "use sensitivity-derived value". |
| Y gain | `advanced.gainY` | Screen pixels moved per camera pixel of vertical head movement. Separate from X so you can compensate for asymmetric camera placement. |
| Smoothing | `advanced.smoothing` | EMA factor for the smoothing filter (0–1). Applied to both axes. `0.35` is quite smooth/laggy; `0.15` is more responsive. Formula: `output = prev + (target - prev) * smoothing`. |

---

## Clicking

Controls dwell click — the auto-click that fires when the cursor stays still for a configured time.

| Setting | Key | Default | Description |
|---------|-----|---------|-------------|
| Dwell enabled | `dwellEnabled` | `false` | Master switch for dwell clicking. When disabled, no automatic clicks fire. |
| Dwell time | `dwellTimeMs` | `500` | How long (ms) the cursor must remain within the dwell radius before a click fires. Shorter = more responsive but accidental clicks are easier. Longer = more intentional but slower to use. |
| Dwell radius | `dwellRadiusPx` | `30` | If the cursor moves more than this many pixels from where it settled, the dwell timer resets. Larger radius is forgiving of tremor; too large makes it hard to aim precisely. |
| Click type | `clickType` | `"left"` | Which button the dwell fires: `"left"`, `"right"`, or `"double"`. Note: `"double"` currently fires a single left click (double-click is not yet implemented in the controller). |
| Right-click toggle | `rightClickToggle` | `false` | One-shot mode: the next dwell click fires as a right click, then this flag automatically resets to false and subsequent dwells return to the configured `clickType`. Useful for context menus without permanently switching click mode. |

---

## Hotkeys

Global keyboard shortcuts that work even when the app window is not focused. Only function keys (F1–F20) are supported.

| Setting | Key | Default | Description |
|---------|-----|---------|-------------|
| Start / pause | `startPause` | `"F11"` | Toggle tracking on and off. Same as the Start/Stop button in the UI. |
| Recenter | `recenter` | `"F12"` | Re-extract the template from the center of the current frame and move the cursor to the center of the screen. Use this when you've drifted too far or want to re-anchor after repositioning. The frontend handles the actual recenter call in response to this hotkey event. |

---

## How settings flow through the app

```
UI change
   ↓
App.UpdateParams(params)          — takes effect immediately (in-memory)
   ↓
Service.applyRuntimeParamsLocked()
   ├── Tracker.UpdateParams()     — updates match params (search margin, threshold, etc.)
   ├── CursorMover.SetMappingParams() — queued via dirty flag, applied next pipeline tick
   └── CursorMover.SetDwellParams()  — applied to DwellState directly

App.SaveParams(params)            — UpdateParams + write config.json to disk
```

Settings that affect the pipeline take effect within one frame of being applied (dirty flags are drained at the top of each pipeline tick).
