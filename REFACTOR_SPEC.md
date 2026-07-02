# Open Camera Mouse – Simple Go Refactor Spec

## Goal

Refactor the Go backend by deleting old complexity and rebuilding it as a small, understandable real-time loop.

Keep OpenCV/GoCV for now.

Do not replace OpenCV in this refactor.

The goal is architecture cleanup, not algorithm replacement.

The app targets macOS and Windows.

---

## Main direction

The app should be:

```text
camera frame -> tracking -> mouse movement -> preview/status
```

Delete old overcomplicated code.

Avoid abstract architecture names.

Do not create packages named:

```text
engine
processor
manager
broker
runtime
```

Use simple domain packages.

---

## Package layout

Use this simple layout:

```text
internal/
  app/
    app.go
    commands.go        // only if needed

  camera/
    camera.go

  tracking/
    tracking.go

  mouse/
    mouse.go
    dwell.go           // only if needed

  preview/
    preview.go

  config/
    config.go

  hotkeys/
    hotkeys.go
```

Main file should usually match the package name:

```text
camera/camera.go
tracking/tracking.go
mouse/mouse.go
preview/preview.go
config/config.go
hotkeys/hotkeys.go
```

Only add extra files if the package becomes genuinely too large or has clearly separate logic.

Do not split files just for architecture ceremony.

---

## Responsibilities

### `app`

`app` coordinates everything.

It owns:

```text
start/stop lifecycle
command channel
main runtime loop
Wails bridge
config load/save
```

`app` is allowed to coordinate camera, tracking, mouse, and preview.

`app` should not contain OpenCV template matching code directly.

### `camera`

`camera` owns camera capture.

For now it may use GoCV internally.

It can return GoCV frames for now.

Do not over-abstract this in the first refactor.

### `tracking`

`tracking` owns template tracking.

For now use GoCV `MatchTemplate`.

Keep tracking logic here only.

No telemetry.

No UI config explosion.

### `mouse`

`mouse` owns cursor movement, gain, smoothing, deadzone, max speed, and dwell click.

No separate “cursor” package.

Mouse package can contain `dwell.go` if that makes the code cleaner.

### `preview`

`preview` owns preview encoding and optional overlay drawing.

Keep preview because React needs it for selecting the tracking point.

### `config`

`config` owns persisted app settings.

Remove old advanced settings.

### `hotkeys`

`hotkeys` owns global hotkey setup if already working.

Do not redesign hotkeys in this refactor.

---

## Runtime loop

Use one main runtime loop in `app`.

The loop receives:

```text
camera frames
commands from Wails/UI/hotkeys
context cancellation
```

The loop owns runtime mutable state.

That means tracker, mouse mapper, dwell state, latest frame, selected point, and lost state should only be mutated from this loop.

External methods should send commands into the loop.

Example shape:

```go
func (a *App) run(ctx context.Context) {
    frames, err := a.camera.Stream(ctx)
    if err != nil {
        a.emitStatus(true)
        return
    }

    for {
        select {
        case <-ctx.Done():
            return

        case cmd := <-a.commands:
            a.handleCommand(cmd)

        case frame, ok := <-frames:
            if !ok {
                return
            }

            a.handleFrame(frame)
        }
    }
}
```

This is the core architecture.

---

## Commands

Use a command channel instead of direct mutation from many methods.

Example:

```go
type CommandKind int

const (
    CmdPickPoint CommandKind = iota
    CmdRecenter
    CmdSetParams
    CmdSetTrackingEnabled
    CmdResetMouse
    CmdStop
)

type Command struct {
    Kind    CommandKind
    X       int
    Y       int
    Params  config.Params
    Enabled bool
}
```

Wails methods should usually only send commands.

Example:

```go
func (a *App) PickPoint(x, y int) error {
    return a.sendCommand(Command{
        Kind: CmdPickPoint,
        X: x,
        Y: y,
    })
}
```

---

## Mutex policy

Remove mutexes from the frame hot path.

No mutexes in:

```text
tracking
mouse mapping
dwell state
per-frame preview path
```

A small mutex or atomic in `app` for Start/Stop lifecycle is acceptable.

The important rule:

```text
only the app runtime loop mutates runtime state
```

So tracking and mouse do not need mutexes.

---

## Runtime params

Keep only the parameters the user actually needs.

```go
type Params struct {
    TemplateSizePx int     `json:"templateSizePx"`
    GainMultiplier float64 `json:"gainMultiplier"`
    Smoothing      float64 `json:"smoothing"`

    DwellEnabled bool `json:"dwellEnabled"`
    DwellTimeMs  int  `json:"dwellTimeMs"`
}
```

Remove the old advanced parameter set.

---

## Constants

Move technical tuning values into constants.

Example:

```go
const (
    DefaultTemplateSizePx = 45
    MinTemplateSizePx     = 25
    MaxTemplateSizePx     = 75

    DefaultGainMultiplier = 8.0
    MinGainMultiplier     = 1.0
    MaxGainMultiplier     = 30.0

    DefaultSmoothing = 0.30
    MinSmoothing     = 0.0
    MaxSmoothing     = 0.85

    SearchMarginMultiplier = 2
    ScoreThreshold         = 0.68

    DeadzonePx = 1.0
    MaxSpeedPx = 35.0

    DwellRadiusPx = 30

    PreviewFPS   = 15
    PreviewJPEGQ = 80
)
```

Template size remains user configurable.

Search margin is derived:

```go
searchMargin := templateSize * SearchMarginMultiplier
```

Threshold is constant.

Adaptive template should be removed or hardcoded off.

---

## Tracking

Keep the algorithm simple.

Use GoCV for now.

Tracking flow:

```text
1. User picks a point on preview.
2. Crop template around selected point.
3. For each frame:
   - convert to grayscale
   - search near previous point
   - run normalized template matching
   - choose best match
   - if score below threshold, mark lost
   - else update tracked point
```

Tracking package should expose a simple type.

Example:

```go
type Params struct {
    TemplateSizePx int
}

type Result struct {
    OK    bool
    Lost  bool
    X     int
    Y     int
    Score float64
}

type Tracker struct {
    // no mutex
}

func New(params Params) *Tracker
func (t *Tracker) SetParams(params Params)
func (t *Tracker) Pick(frame gocv.Mat, x, y int) error
func (t *Tracker) Update(frame gocv.Mat) Result
func (t *Tracker) Reset()
func (t *Tracker) Close()
```

`Score` can exist internally or in the result, but do not emit it as telemetry to React.

Do not expose search margin, threshold, adaptive template, or alpha in config.

---

## Mouse

Mouse package should own pointer movement and dwell.

Use a large gain multiplier.

Example:

```go
type Params struct {
    GainMultiplier float64
    Smoothing      float64
    DwellEnabled   bool
    DwellTimeMs    int
}

type Mouse struct {
    // no mutex
}

func New(params Params) *Mouse
func (m *Mouse) SetParams(params Params)
func (m *Mouse) Reset()
func (m *Mouse) Update(x, y int, lost bool)
```

Use constants for:

```text
deadzone
max speed
dwell radius
```

Do not expose gain X/gain Y/deadzone/max speed separately.

One gain multiplier is enough.

---

## Preview

Keep preview.

Remove telemetry.

Preview should emit only what React needs to display the camera image and optionally overlay the tracking rectangle.

Acceptable Go-side preview event:

```go
type Frame struct {
    DataURL string `json:"dataUrl"`
    Width   int    `json:"width"`
    Height  int    `json:"height"`

    Tracking *TrackingOverlay `json:"tracking,omitempty"`
}

type TrackingOverlay struct {
    X              int  `json:"x"`
    Y              int  `json:"y"`
    TemplateSizePx int  `json:"templateSizePx"`
    Lost           bool `json:"lost"`
}
```

Preferred: draw rectangle in React using overlay coordinates.

That avoids extra Go drawing logic.

Go only encodes the preview image.

---

## Events

Remove telemetry events completely.

Delete:

```text
telemetry:state
FPS display
score display
position display
```

Keep:

```text
preview:frame
status:update
service:running
```

Status should be minimal:

```go
type Status struct {
    Running bool `json:"running"`
    Lost    bool `json:"lost"`
}
```

Emit status only when it changes.

---

## Delete old code

Delete or stop using:

```text
FrameProcessor
Broker
Telemetry
old advanced params
old duplicated pipeline pieces
old tracker snapshots
old cursor mover mutex/pending mapping logic
old dwell mutex logic
```

Do not leave unused old paths in the codebase.

---

## Config cleanup

Remove old config fields:

```text
SearchMarginPx
ScoreThreshold
AdaptiveTemplate
TemplateUpdateAlpha
MarkerShape
Amplification
DeadzonePx
MaxSpeedPx
GainX
GainY
ClickTypeDouble
RightClickToggle
DwellRadiusPx
DwellOnStartup
telemetry settings
advanced tracking settings
advanced mouse settings
```

Keep:

```text
TemplateSizePx
GainMultiplier
Smoothing
DwellEnabled
DwellTimeMs
Camera device if currently supported
Hotkey if currently supported
```

If old config files contain removed fields, ignore them.

Do not build a complex migration system unless necessary.

---

## React cleanup

Do not fully redesign React yet.

Only remove things that no longer exist.

Remove UI/state/listeners for:

```text
telemetry
FPS
tracking score
position x/y
search margin
score threshold
adaptive template
template alpha
deadzone
max speed
gain x/y
amplification
marker shape
double click type
right click toggle
dwell radius
dwell on startup
advanced tracking panel
advanced mouse panel
```

Keep UI for:

```text
Start/Stop
camera preview
pick tracking point
template size
pointer speed / gain multiplier
smoothing
dwell enabled
dwell time
reset/recenter
```

React can draw tracking rectangle over preview using the `TrackingOverlay` values from preview event.

---

## Wails API

Simplify public API to roughly:

```go
func (a *App) Start() error
func (a *App) Stop() error
func (a *App) PickPoint(x int, y int) error
func (a *App) Recenter() error
func (a *App) ResetMouse() error
func (a *App) GetParams() config.Params
func (a *App) UpdateParams(params config.Params) error
```

Avoid many tiny setters.

Use one `UpdateParams`.

---

## Do not do

Do not:

```text
remove OpenCV
replace camera capture
write custom matcher
add face detection
add AI tracking
add landmarks
redesign the entire React UI
add new telemetry/debug panel
create engine/manager/broker packages
split every tiny helper into its own file
```

This refactor is only about simplifying the existing app.

---

## Acceptance criteria

The refactor is successful when:

```text
1. App builds.
2. Camera preview works.
3. User can pick a point on preview.
4. Tracking works.
5. Mouse moves from tracked point.
6. Template size is configurable.
7. Gain multiplier is configurable.
8. Smoothing is configurable.
9. Dwell enabled/time are configurable.
10. Telemetry is gone.
11. Advanced params are gone.
12. Tracking has no mutex.
13. Mouse/dwell hot path has no mutex.
14. FrameProcessor/Broker/Telemetry are deleted.
15. Technical tuning is in constants.
16. React no longer references removed events or fields.
17. No new abstract architecture packages are introduced.
```

Final desired shape:

```text
app runtime loop
  receives camera frames
  receives UI commands
  calls tracking
  calls mouse
  emits preview/status

camera = camera capture
tracking = template matching
mouse = cursor movement/dwell
preview = preview encoding
config = tiny persisted params
hotkeys = hotkey handling
```
