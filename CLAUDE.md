# Open Camera Mouse

Hands-free desktop application that turns your webcam into a mouse controller using template-matching for face/body marker tracking.

## Tech Stack

- **Backend**: Go 1.24+ with Wails v2 (desktop framework)
- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Computer Vision**: GoCV (OpenCV bindings)
- **Mouse Control**: robotgo
- **Platforms**: macOS, Windows, Linux

See [docs/STACK.md](docs/STACK.md) for versions and platform-specific build deps.

## Project Structure

```
.
├── main.go                 # Wails entry point
├── app.go                  # Wails bindings adapter (App struct)
├── frontend/               # React frontend
│   └── src/
│       ├── screens/        # Main and Settings screens
│       ├── components/     # Reusable UI components
│       ├── state/          # React state management
│       └── types/          # TypeScript types
├── internal/
│   ├── app/
│   │   ├── app.go          # Runtime loop + lifecycle (App struct)
│   │   └── commands.go     # Command types dispatched into the loop
│   ├── camera/
│   │   └── camera.go       # Webcam capture — Stream(ctx) → chan Frame
│   ├── tracking/
│   │   └── tracking.go     # Template-matching Tracker (no mutex)
│   ├── mouse/
│   │   └── mouse.go        # Cursor movement + dwell click (no mutex)
│   ├── preview/
│   │   └── preview.go      # JPEG encoder + TrackingOverlay type
│   ├── config/
│   │   └── config.go       # Flat Params struct + JSON persistence
│   └── hotkeys/
│       └── hotkeys.go      # Global hotkey handling (golang.design/x/hotkey)
└── .github/workflows/      # CI/CD
```

## Architecture Principles

1. **Single runtime goroutine owns all state** — tracker, mouse, and dwell state are only mutated from `app.run()`; no mutexes on the hot path
2. **Command channel for control plane** — Wails methods send commands into the loop; no direct mutation from outside
3. **Single Responsibility** — each package does one thing; no engine/manager/broker abstractions
4. **Interface-based abstractions** — use interfaces for testability (e.g., `mouse.Controller`)
5. **Platform isolation** — platform-specific code uses build tags (`_darwin.go`, `_windows.go`, `_linux.go`)
6. **Error handling** — wrap errors with context using `fmt.Errorf("context: %w", err)`

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the runtime loop diagram and mutex inventory.

## Data Flow

Two goroutines:

```
camera.Service (goroutine)
     │ chan camera.Frame  (buffer: 1)
     ▼
app.App.run() goroutine
     ├── select: ctx.Done | command | frame
     ├── tracking.Tracker.Update()
     ├── mouse.Mouse.Update()  →  robotgo Move + dwell Click
     └── preview.Encoder.Encode()  →  Wails EventsEmit("preview:frame")
```

The `run` loop in `internal/app/app.go` is the readable top-level algorithm.
See [docs/ALGORITHM.md](docs/ALGORITHM.md) for algorithm details.

## Key Components

### App Runtime (`internal/app/app.go`)
- `App.run(ctx)` — single select loop receiving frames and commands
- `handleFrame(frame)` — runs tracking, mouse, emits preview/status on change
- `handleCommand(cmd)` — applies queued control-plane updates between frames
- Only `app.mu` (lifecycle mutex) exists; all frame-path state is goroutine-local

### Tracker (`internal/tracking/tracking.go`)
- `Tracker` type with no mutex — owned exclusively by `app.run()` goroutine
- `Pick(frame, x, y)` — crops template from frame at given point
- `Update(frame)` — NCC template match; returns last known point as fallback when lost
- `HasTemplate() bool` — whether a pick has been made

### Mouse (`internal/mouse/mouse.go`)
- `Mouse` type with no mutex — owned exclusively by `app.run()` goroutine
- `Update(x, y, lost)` — applies gain/smoothing/deadzone to tracking delta, moves cursor, drives dwell
- Constants: `DeadzonePx`, `MaxSpeedPx`, `DwellRadiusPx` (not user-configurable)

### Preview (`internal/preview/preview.go`)
- `Encoder.Encode(frame, overlay)` — flips frame, encodes JPEG as data URL, rate-limited to ~15 fps
- Returns `Frame{DataURL, Width, Height, Tracking}` — React draws the tracking rectangle

## Wails API

```go
func (a *App) Start() error
func (a *App) Stop() error
func (a *App) PickPoint(x, y int)
func (a *App) Recenter()
func (a *App) ResetMouse()
func (a *App) ToggleTracking(enabled bool)
func (a *App) GetParams() config.Params
func (a *App) UpdateParams(params config.Params) error
```

## Events

| Event | Direction | Payload |
|-------|-----------|---------|
| `preview:frame` | Go → React | `{dataUrl, width, height, tracking}` |
| `status:update` | Go → React | `{running, lost}` (emitted only on change) |
| `service:running` | Go → React | `bool` |
| `recenter:hotkey` | Go → React | (none) |

## Build Commands

See [docs/RUNBOOK.md](docs/RUNBOOK.md) for all build, dev, format, and test commands.

## Configuration

See [docs/RUNBOOK.md](docs/RUNBOOK.md) for config file locations per platform.

## Coding Standards

- Do not emit comments in generated code unless explicitly requested

### Go
- Follow standard Go idioms (accept interfaces, return structs)
- Use `context.Context` for cancellation
- Prefer composition over inheritance
- Keep functions short (<50 lines ideally)
- Use meaningful variable names, avoid single letters except for loops
- Handle all errors explicitly

### TypeScript/React
- Functional components with hooks
- Types in `types/` directory
- State management via custom hooks in `state/`
- Use the shared `cn` helper for conditional class names (`frontend/src/lib/cn.ts`)

## Testing

- Unit tests alongside source files (`*_test.go`)
- Use table-driven tests for multiple cases
- Mock interfaces for isolation
- Run with race detector: `go test -race ./...`
