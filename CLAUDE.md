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
├── app.go                  # Wails bindings layer
├── frontend/               # React frontend
│   └── src/
│       ├── screens/        # Main and Settings screens
│       ├── components/     # Reusable UI components
│       ├── state/          # React state management
│       └── types/          # TypeScript types
├── internal/
│   ├── app/                # Core application logic
│   │   ├── app.go          # Service coordinator + runPipeline
│   │   ├── pipeline.go     # Pipeline stage functions: track(), process()
│   │   ├── params.go       # Param builder functions
│   │   └── cursor_mover.go # Cursor movement state machine
│   ├── camera/             # Webcam capture — Stream(ctx) → chan Frame
│   ├── tracking/           # Template matching tracker (state machine)
│   ├── mouse/              # Mouse control abstraction
│   │   ├── controller.go   # Interface
│   │   ├── robotgo.go      # Implementation
│   │   ├── mapping.go      # Gain/smoothing/deadzone mapper
│   │   └── dwell.go        # Dwell click state machine
│   ├── config/             # Settings persistence
│   ├── stream/             # Broker (channel-based pub/sub), preview encoder, telemetry
│   ├── overlay/            # Marker rendering
│   └── hotkeys/            # Global hotkey handling (golang.design/x/hotkey)
└── .github/workflows/      # CI/CD
```

## Architecture Principles

1. **Channel pipeline** — runtime data flows through channels; goroutines own their state exclusively
2. **State machines with getters/setters** — only components with control-plane updates carry a mutex; pipeline-owned runtime state needs no synchronization
3. **Single Responsibility** — each package/struct does one thing well
4. **Interface-based abstractions** — use interfaces for testability (e.g., `mouse.Controller`)
5. **Platform isolation** — platform-specific code uses build tags (`_darwin.go`, `_windows.go`, `_linux.go`)
6. **Error handling** — wrap errors with context using `fmt.Errorf("context: %w", err)`

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full pipeline diagram and mutex inventory.

## Data Flow

Three goroutines connected by channels:

```
camera.Manager (goroutine)
     │ chan camera.Frame
     ▼
track() goroutine
     │ chan FrameResult
     ▼
process() goroutine
     ├── CursorMover → mouse.Controller (robotgo)
     ├── → broker.PublishPreview → chan → Wails EventsEmit
     └── → broker.PublishTelemetry → chan → Wails EventsEmit
```

`runPipeline` in `internal/app/app.go` is the readable top-level algorithm.
See [docs/ALGORITHM.md](docs/ALGORITHM.md) for algorithm details.

## Key Components

### Pipeline Stage Functions (`internal/app/pipeline.go`)
- `track(ctx, frames, tracker)` — consumes camera frames, runs template matching, emits `FrameResult`
- `process(ctx, results, cursor, broker)` — moves cursor, drives dwell, renders and publishes preview/telemetry

### Tracker (`internal/tracking/tracker.go`)
- State machine: template, params, last frame — protected by `sync.RWMutex`
- Control plane: `SetPickPoint`, `Recenter`, `UpdateParams`, `SetTrackingEnabled`
- Pipeline: `Update(frame)` always returns a result (uses last known point as fallback when lost)

### CursorMover (`internal/app/cursor_mover.go`)
- Runtime state (`lastPoint`, `pointSet`, `mapper`, `dwell`) owned by `process()` goroutine — no mutex
- Control plane uses dirty flags under `mappingMu` to safely pass new params to the pipeline goroutine

### Broker (`internal/stream/broker.go`)
- Channel-based pub/sub; each subscriber gets its own buffered channel
- `BrokerPolicy` controls buffer depth and drop-on-slow behavior per stream

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
