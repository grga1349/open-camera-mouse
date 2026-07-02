# Algorithm

## Top-Level Loop

The application runs a per-frame pipeline inside a single goroutine. Each camera frame passes through tracking, cursor movement, and preview encoding.

```
[camera goroutine] → frames chan → [app.run goroutine]
                                        │
                                        ├─ tracking.Tracker.Update()
                                        ├─ mouse.Mouse.Update()
                                        └─ preview.Encoder.Encode()
```

**Per frame:**
1. **Capture** — read frame from webcam
2. **Apply pending commands** — pick point, recenter, set params (queued between frames)
3. **Track marker** — template match to locate the tracking point
4. **Move cursor** — translate tracking delta to cursor movement
5. **Dwell click** — click if cursor held still long enough
6. **Render preview** — flip frame, emit tracking overlay coords, encode JPEG, publish to UI

---

## Sub-Algorithms

### 1. Template Matching (`internal/tracking/tracking.go`)

Called once per frame. Uses OpenCV normalized cross-correlation.

```
INPUT:  grayscale frame, stored template patch (NxN pixels), last known templatePoint
OUTPUT: Result{Lost, X, Y}

1. Convert frame to grayscale
2. Compute search region:
     margin = templateSizePx * 2  (search margin constant)
     x = clamp(templatePoint.X ± margin, 0, frameWidth)
     y = clamp(templatePoint.Y ± margin, 0, frameHeight)
3. Run NCC template match on search region:
     gocv.MatchTemplate(searchRegion, template, TmCcoeffNormed)
4. Find peak (MinMaxLoc → maxVal, maxLoc)
5. If maxVal < 0.68 (score threshold) → return Lost=true, X/Y=last known point
6. Compute center = searchRect.Min + maxLoc + template.Size/2
7. Update templatePoint = center
8. Return Result{Lost=false, X=center.X, Y=center.Y}
```

**Fallback on loss:** the tracker returns the last known `templatePoint` when lost,
so downstream always has a valid position reference.

**Constants (not user-configurable):**
- Search margin = `templateSizePx × 2` — search region in each direction
- Score threshold = `0.68` — minimum NCC match quality to accept

---

### 2. Cursor Mapping (`internal/mouse/mouse.go`)

Called once per frame. Converts tracking pixel delta to cursor displacement.

```
INPUT:  tracking point (x, y), last tracking point, lost bool
OUTPUT: cursor moved by (moveX, moveY)

1. If lost or no previous point: record current point, return (no movement)
2. Compute delta:
     dx = lastPoint.X - point.X   (inverted: head right → cursor right)
     dy = point.Y - lastPoint.Y   (normal: head down → cursor down)
3. Apply deadzone (constant: 1px):
     if |dx| < DeadzonePx → dx = 0
     if |dy| < DeadzonePx → dy = 0
4. Clamp to max speed (constant: 35px):
     dx = clamp(dx, -MaxSpeedPx, +MaxSpeedPx)
     dy = clamp(dy, -MaxSpeedPx, +MaxSpeedPx)
5. Apply gain:
     targetX = dx * GainMultiplier
     targetY = dy * GainMultiplier
6. Apply smoothing (EMA):
     smoothX += (targetX - smoothX) * Smoothing
     smoothY += (targetY - smoothY) * Smoothing
7. Move cursor:
     newX = round(cursorX + smoothX)
     newY = round(cursorY + smoothY)
     robotgo.Move(newX, newY)
```

**Constants (not user-configurable):**
- `DeadzonePx = 1.0` — sub-pixel movements are ignored
- `MaxSpeedPx = 35.0` — per-frame displacement cap

**User-configurable:**
- `GainMultiplier` (1–30, default 8) — scales raw pixel delta to cursor displacement
- `Smoothing` (0–0.85, default 0.30) — EMA lerp coefficient; higher = more responsive

---

### 3. Dwell Click (`internal/mouse/mouse.go`)

Called once per frame (after cursor movement). Implements hover-to-click.

```
INPUT:  current cursor position (x, y) [post-move], lost bool
STATE:  dwellRefX/Y, dwellStart, dwellRefSet

1. If dwell disabled OR lost:
     dwellRefSet = false
     return

2. If ref not set:
     refX=x, refY=y, refSet=true, dwellStart=now
     return

3. dist = hypot(x-refX, y-refY)
   If dist > DwellRadiusPx (constant: 30px):
     refX=x, refY=y, dwellStart=now   (cursor moved — restart)
     return

4. If time.Since(dwellStart) >= DwellTime:
     robotgo.Click("left")
     dwellStart = now                  (restart timer)
```

**Constants (not user-configurable):**
- `DwellRadiusPx = 30` — cursor must stay within this radius

**User-configurable:**
- `DwellTimeMs` (200–1500ms, default 500ms)

---

### 4. Preview Rendering (`internal/preview/preview.go`)

Called once per frame, rate-limited to ~15 fps.

```
INPUT:  raw camera frame, TrackingOverlay{X, Y, TemplateSizePx, Lost}
OUTPUT: Frame{DataURL, Width, Height, Tracking} published via "preview:frame" Wails event

1. If less than 66ms since last encode: skip (rate limit)
2. Flip frame horizontally (mirror for natural webcam UX):
     gocv.Flip(frame, &display, 1)
3. Encode to JPEG (quality 80):
     gocv.IMEncodeWithParams(JPEGFileExt, display, quality=80)
4. Wrap as data URL: "data:image/jpeg;base64,<base64>"
5. Emit Frame{DataURL, Width, Height, Tracking} — React draws the rectangle overlay
```

**Overlay:** Go sends the tracking point coordinates (mirrored for display) and template size. React draws the bounding rectangle using CSS positioning over the `<img>` element.
