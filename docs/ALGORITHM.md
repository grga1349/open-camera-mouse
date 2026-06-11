# Algorithm

## Top-Level Loop

The application runs a per-frame pipeline. Each camera frame passes through three goroutines in sequence:

```
[camera goroutine] → frames chan → [track goroutine] → results chan → [process goroutine]
```

**Per frame:**
1. **Capture** — read frame from webcam
2. **Track marker** — template match to locate the tracking point
3. **Move cursor** — translate tracking delta to cursor movement
4. **Dwell click** — click if cursor held still long enough
5. **Render preview** — draw overlay, encode JPEG, publish to UI
6. **Publish telemetry** — send FPS, score, position to UI

---

## Sub-Algorithms

### 1. Template Matching (`internal/tracking/tracker.go`)

Called once per frame by the `track` goroutine. Uses OpenCV normalized cross-correlation.

```
INPUT:  grayscale frame, stored template patch (NxN pixels), last known templatePoint
OUTPUT: Result{Point, Score, Lost}

1. Convert frame to grayscale
2. Compute search region:
     x = clamp(templatePoint.X ± SearchMargin, 0, frameWidth)
     y = clamp(templatePoint.Y ± SearchMargin, 0, frameHeight)
3. Run NCC template match on search region:
     gocv.MatchTemplate(searchRegion, template, TmCcoeffNormed)
4. Find peak (MinMaxLoc → maxVal, maxLoc)
5. If maxVal < ScoreThreshold → return Lost=true, Point=templatePoint (fallback)
6. Compute center = searchRect.Min + maxLoc + template.Size/2
7. If AdaptiveTemplate:
     blend current patch into template:
     template = alpha*currentPatch + (1-alpha)*template
8. Update templatePoint = center
9. Return Result{Point=center, Score=maxVal, Lost=false}
```

**Fallback on loss:** the tracker always returns the last known `templatePoint` when tracking is lost, so downstream stages always have a valid position reference.

---

### 2. Cursor Mapping (`internal/mouse/mapping.go`, `internal/app/cursor_mover.go`)

Called once per frame by the `process` goroutine. Converts tracking pixel delta to cursor displacement.

```
INPUT:  tracking point (x, y), last tracking point
OUTPUT: cursor moved by (moveX, moveY)

1. Compute delta:
     dx = lastPoint.X - point.X   (inverted: head right → cursor right)
     dy = point.Y - lastPoint.Y   (normal: head down → cursor down)
2. Apply deadzone (nullify sub-threshold movement):
     if |dx| < DeadzonePx → dx = 0
     if |dy| < DeadzonePx → dy = 0
3. Clamp to max speed:
     dx = clamp(dx, -MaxSpeedPx, +MaxSpeedPx)
     dy = clamp(dy, -MaxSpeedPx, +MaxSpeedPx)
4. Apply gain:
     targetX = dx * GainX
     targetY = dy * GainY
5. Apply smoothing (exponential moving average):
     smoothedX = prevX + (targetX - prevX) * Smoothing
     smoothedY = prevY + (targetY - prevY) * Smoothing
     prevX, prevY = smoothedX, smoothedY
6. Move cursor:
     newX = round(cursorX + smoothedX)
     newY = round(cursorY + smoothedY)
     controller.Move(newX, newY)
```

**GainX / GainY** scale is derived from the `Sensitivity` setting (1–100 → gain 4.8–20).
**Smoothing** is the lerp coefficient (0.15–0.35): lower = more smoothing, higher = more responsive.

---

### 3. Dwell Click (`internal/mouse/dwell.go`)

Called once per frame by the `process` goroutine. Implements hover-to-click.

```
INPUT:  current cursor position (x, y), trackingLost bool
STATE:  refX, refY (reference position), dwellStart (timer), refSet bool

1. If dwell disabled OR trackingLost:
     reset(refX=x, refY=y, refSet=false)
     return

2. If ref not set:
     refX=x, refY=y, refSet=true, dwellStart=now
     return

3. dist = hypot(x-refX, y-refY)
   If dist > RadiusPx:
     refX=x, refY=y, dwellStart=now   (cursor moved — restart)
     return

4. If time.Since(dwellStart) >= DwellTime:
     controller.Click(ClickButton)     (trigger click)
     dwellStart = now                  (restart timer for repeat)
```

**RadiusPx** is the pixel radius within which cursor must stay. Default: 30px.
**DwellTime** is the hold duration before click. Default: 500ms.

---

### 4. Preview Rendering (`internal/app/pipeline.go`)

Called once per frame (rate-limited to ~15 fps) by the `process` goroutine.

```
INPUT:  raw camera frame, TrackingResult, Params (MarkerShape, TemplateSize)
OUTPUT: base64 JPEG published to broker → Wails EventsEmit → frontend

1. Clone frame mat
2. Flip horizontally (mirror for natural webcam UX):
     gocv.Flip(display, &display, 1)
3. Mirror marker x-coordinate to match flipped display:
     mirroredX = frame.Cols - point.X
4. Choose marker color:
     tracking disabled → white
     lost              → red
     tracking OK       → green
5. Draw marker overlay (circle or square) + score text
6. Encode to JPEG:
     gocv.IMEncode(JPEGFileExt, display)
7. Base64-encode bytes
8. Publish PreviewFrame{Data, Width, Height, Timestamp} to broker channel
9. Publish Telemetry{FPS, Score, Lost, Tracking, PosX, PosY} to broker channel
```

**Rate limiting:** the `PreviewEncoder` skips encoding if the previous frame was sent less than 66ms ago (~15 fps ceiling), decoupling preview cadence from the camera frame rate.
