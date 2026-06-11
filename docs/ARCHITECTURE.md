# Architecture

## Pipeline

Three goroutines connected by buffered channels:

```
camera.Manager (goroutine)
     │ chan camera.Frame  (buffer: 1)
     ▼
track() goroutine                          ← internal/app/pipeline.go
     │ chan FrameResult  (buffer: 1)
     ▼
process() goroutine                        ← internal/app/pipeline.go
     ├── CursorMover.Update + UpdateDwell → mouse.Controller (robotgo)
     ├── renderAndPublish → broker.PublishPreview → chan PreviewFrame → Wails EventsEmit
     └── renderAndPublish → broker.PublishTelemetry → chan Telemetry → Wails EventsEmit
```

The visible orchestration in `runPipeline`:
```go
func (s *Service) runPipeline(ctx context.Context) error {
    frames, err := s.camera.Stream(ctx)
    if err != nil { return err }
    results := track(ctx, frames, s.tracker)
    s.done = make(chan struct{})
    go func() {
        defer close(s.done)
        process(ctx, results, s.cursorMover, s.broker)
    }()
    return nil
}
```

Shutdown: `cancel()` → camera goroutine exits → closes `frames` channel → `track` goroutine exits, calls `tracker.Close()` → closes `results` channel → `process` goroutine exits → closes `done` → `Stop()` unblocks.

## Package Responsibilities

| Package | Responsibility |
|---------|---------------|
| `internal/app` | Service coordinator: lifecycle, component wiring, param distribution; `pipeline.go` contains `track()` and `process()` stage functions |
| `internal/camera` | Webcam capture via GoCV; emits frames to a channel via `Stream(ctx)` |
| `internal/tracking` | Template matching tracker — state machine with params, template, last frame |
| `internal/mouse` | Mouse control interface, gain/smoothing/deadzone mapping, dwell click state |
| `internal/config` | Config persistence (JSON) and param types |
| `internal/stream` | Channel-based pub/sub broker, JPEG preview encoder, telemetry types |
| `internal/overlay` | Marker rendering onto preview frames |
| `internal/hotkeys` | Global hotkey registration and dispatch |

## Architecture Principles

1. **Channel pipeline** — runtime data flows through channels; goroutines own their state exclusively
2. **State machines with getters/setters** — only components whose state is updated from the control plane (UI events, param changes) carry a mutex; all others are stateless or single-goroutine-owned
3. **Single Responsibility** — each package/struct does one thing well
4. **Interface-based abstractions** — use interfaces for testability (e.g., `mouse.Controller`)
5. **Platform isolation** — platform-specific code uses build tags (`_darwin.go`, `_windows.go`, `_linux.go`)
6. **Error handling** — wrap errors with context using `fmt.Errorf("context: %w", err)`

## Mutex Inventory

| Component | Mutex | Protects | Reason |
|-----------|-------|----------|--------|
| `Service` | `sync.RWMutex` | `running`, `params`, `cancel`, `done` | Control-plane state machine |
| `Tracker` | `sync.RWMutex` | `params`, `template`, `templatePoint`, `lastFrame`, `trackingEnabled`, `lost` | Params + pick point set from control plane; Update called from pipeline |
| `CursorMover` | `sync.Mutex` (`mappingMu`) | `pendingMapping`, `mappingDirty`, `resetPending` | Params set from control plane; applied lazily by pipeline goroutine |
| `DwellState` | `sync.Mutex` | `params`, `refX/Y`, `refSet`, `dwellStart` | Params set from control plane; Update called from pipeline |
| `Broker` | `sync.Mutex` | subscriber channel slices | Registry updated from any goroutine |
| `hotkeys.manager` | `sync.Mutex` | entries list | Updated from control plane |

Components with **no mutex** (single-goroutine ownership):
- `camera.Manager` — goroutine owns `cap` exclusively
- `PreviewEncoder` — local variable inside `process()` goroutine
- `CursorMover.lastPoint`, `pointSet`, `mapper`, `dwell` — owned by `process()` goroutine

## See Also

- [Algorithm details](ALGORITHM.md) — top-level loop and 4 sub-algorithms
- [Tech stack and platform deps](STACK.md)
- [CI/CD pipeline](CI.md)
- [Dev runbook](RUNBOOK.md)
