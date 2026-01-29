# Engine — Template Matching Tracker + Mouse Mapping
- if adaptiveTemplate:
- newPatch = crop(gray, centered at lastPos, templateSize)
- template = (1-a)*template + a*newPatch


## Pointer mapping
### Sensitivity → gain/smoothing (MVP mapping)
- gain = map sensitivity 1..100 → [1.2 .. 5.0] then multiply by 4 for the final cursor gain
- smoothing = map sensitivity 1..100 → [0.35 .. 0.15]
- gainX=gainY=gain unless advanced enabled


### Deadzone + clamp
- if abs(dx) < deadzonePx → dx=0
- if abs(dy) < deadzonePx → dy=0
- dx = clamp(dx, -maxSpeedPx, +maxSpeedPx)
- dy = clamp(dy, -maxSpeedPx, +maxSpeedPx)


### Smoothing (exponential)
- targetX = cursorX + dx*gainX
- targetY = cursorY + dy*gainY
- cursorX = lerp(cursorX, targetX, smoothing)
- cursorY = lerp(cursorY, targetY, smoothing)
- MoveMouse(round(cursorX), round(cursorY))


## Dwell clicking
If dwellEnabled and NOT lost:
- If ref not set: ref=(cursor), dwellStart=now
- dist = hypot(cursor-ref)
- if dist <= dwellRadiusPx:
- if now - dwellStart >= dwellTimeMs: click(); dwellStart=now
- else:
- ref=(cursor), dwellStart=now


### click() selection
- if rightClickToggle: right click
- else use clickType


## Telemetry output
Emit a state object at 10–20 Hz:
- fps
- score
- lost
- trackingOn
- posX,posY (image coords)


## Preview output
Emit encoded preview at 15–25 Hz:
- base64 image
- width,height


## Recommended defaults (start)
- templateSizePx=20
- searchMarginPx=30
- scoreThreshold=0.60
- adaptiveTemplate=true, templateUpdateAlpha=0.20
- sensitivity=30
- deadzonePx=1
- maxSpeedPx=25
- dwellEnabled=false
- dwellTimeMs=500
- dwellRadiusPx=30
