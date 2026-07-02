# Architecture

## Runtime Loop

The application runs a single goroutine in `internal/app`. All mutable runtime state lives there — no mutexes on the hot path.

```
camera.Service (goroutine)
     │ chan camera.Frame  (buffer: 1)
     ▼
app.App.run() goroutine
     ├── select on: ctx.Done | command | frame
     ├── tracking.Tracker.Update()  → cursor movement via mouse.Mouse
     ├── mouse.Mouse.Update()       → robotgo.Move + dwell click
     └── preview.Encoder.Encode()   → "preview:frame" Wails event
```

Commands (pick point, recenter, set params, etc.) are sent via a buffered channel from Wails methods. The run goroutine drains them between frames.

Shutdown: `Stop()` cancels context and blocks on `<-done` until `run()` exits. The camera goroutine exits on `ctx.Done()`, closes `frames`, which causes `run` to return and close `done`.

The visible orchestration in `run`:
```go
func (a *App) run(ctx context.Context) {
    frames, _ := a.camera.Stream(ctx)
    for {
        select {
        case <-ctx.Done():
            return
        case cmd := <-a.commands:
            a.handleCommand(cmd)
        case frame, ok := <-frames:
            if !ok { return }
            a.handleFrame(frame)
        }
    }
}
```

## Package Responsibilities

| Package | Responsibility |
|---------|---------------|
| `internal/app` | Runtime loop, lifecycle (Start/Stop), command dispatch, param wiring |
| `internal/camera` | Webcam capture via GoCV; `Stream(ctx)` emits `Frame` to a buffered channel |
| `internal/tracking` | Template-matching tracker; no mutex — owned exclusively by the app goroutine |
| `internal/mouse` | Cursor movement (gain, smoothing, deadzone) + dwell click; no mutex |
| `internal/preview` | JPEG encoder; flips frame, wraps tracking coords, rate-limits to ~15 fps |
| `internal/config` | Flat `Params` struct; JSON persistence |
| `internal/hotkeys` | Global hotkey registration and dispatch |

## Architecture Principles

1. **Single runtime goroutine owns all state** — tracker, mouse, dwell, and frame state are only mutated from `app.run()`; no locking needed on the frame path
2. **Command channel for control plane** — Wails methods send commands into the loop instead of directly mutating state
3. **Single Responsibility** — each package does one thing; no engine/manager/broker abstractions
4. **No unnecessary abstractions** — packages call robotgo and GoCV directly; interfaces only where a second implementation exists today
5. **Platform isolation** — platform-specific code uses build tags

## Mutex Inventory

| Component | Mutex | Protects | Reason |
|-----------|-------|----------|--------|
| `app.App` | `sync.Mutex` | `running`, `params`, `cancel`, `done` | Lifecycle: Start/Stop called from any goroutine |

Components with **no mutex** (single-goroutine ownership):
- `tracking.Tracker` — owned by `app.run()` goroutine
- `mouse.Mouse` — owned by `app.run()` goroutine
- `preview.Encoder` — local to `handleFrame`
- `camera.Service` goroutine owns `vcap` exclusively

## See Also

- [Algorithm details](ALGORITHM.md) — per-frame loop and sub-algorithms
- [Tech stack and platform deps](STACK.md)
- [CI/CD pipeline](CI.md)
- [Dev runbook](RUNBOOK.md)
