# Open Camera Mouse

Hands-free desktop application that turns your webcam into a mouse controller using template-matching for face/body marker tracking.

## Tech Stack

- **Backend**: Go 1.24+ with Wails v2 (desktop framework)
- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Computer Vision**: GoCV (OpenCV bindings)
- **Mouse Control**: robotgo
- **Platforms**: macOS, Windows, Linux

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
│   │   ├── app.go          # Thin coordinator/lifecycle
│   │   ├── cursor_mover.go # Mouse movement + dwell logic
│   │   └── frame_processor.go # Tracking + overlay pipeline
│   ├── camera/             # Webcam capture (GoCV)
│   ├── tracking/           # Template matching tracker
│   ├── mouse/              # Mouse control abstraction
│   │   ├── controller.go   # Interface
│   │   ├── robotgo.go      # Implementation
│   │   ├── mapping.go      # Gain/smoothing
│   │   └── dwell.go        # Dwell click state
│   ├── config/             # Settings persistence
│   ├── stream/             # Preview/telemetry streaming
│   ├── overlay/            # Marker rendering
│   └── hotkeys/            # Global hotkey handling (golang.design/x/hotkey)
└── .github/workflows/      # CI/CD
```

## Architecture Principles

1. **Single Responsibility**: Each package/struct does one thing well
2. **Interface-based Abstractions**: Use interfaces for testability (e.g., `mouse.Controller`)
3. **Platform Isolation**: Platform-specific code uses build tags (`_darwin.go`, `_windows.go`, `_linux.go`)
4. **Thread Safety**: Use `sync.RWMutex` for shared state, minimize lock scope
5. **Error Handling**: Wrap errors with context using `fmt.Errorf("context: %w", err)`

## Data Flow

```
Camera → FrameProcessor → CursorMover → Mouse
              ↓                ↓
         PreviewEncoder    DwellState
              ↓
           Broker → Frontend (via Wails events)
```

## Key Components

### FrameProcessor
- Receives camera frames
- Runs template matching via Tracker
- Renders overlay markers
- Encodes preview for frontend
- Emits telemetry

### CursorMover
- Receives tracking results from FrameProcessor
- Applies gain, smoothing, deadzone
- Moves system cursor
- Manages dwell click state

### App (Coordinator)
- Manages lifecycle (start/stop)
- Wires components together
- Distributes parameter updates
- Thin layer - delegates to components

## Build Commands

Install the Wails CLI and golines locally:

```bash
go install github.com/wailsapp/wails/v2/cmd/wails@latest
go install github.com/segmentio/golines@latest
```

Use the Makefile for repeatable workflows:

```bash
make wails-dev       # wails dev
make wails-build     # wails build -clean
make format          # golines (Go) + Prettier (frontend)
make format-go       # golines only
make format-frontend # Prettier only
make frontend-install
make frontend-build
make frontend-dev

go test ./...        # Run backend tests
go vet ./...         # Vet backend
```

## Configuration

Config stored at:
- macOS: `~/Library/Application Support/open-camera-mouse/config.json`
- Windows: `%APPDATA%/open-camera-mouse/config.json`
- Linux: `~/.config/open-camera-mouse/config.json`

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

## Platform-Specific Notes

### macOS
- Requires Accessibility permissions for mouse control (robotgo)
- Hotkeys use `golang.design/x/hotkey` (Carbon) — no Accessibility permissions required

### Windows
- Hotkeys use `golang.design/x/hotkey`
- Requires MinGW OpenCV for builds

### Linux
- Hotkeys use `golang.design/x/hotkey` (X11 only, no Wayland support)
- Requires `libx11-dev` for builds
