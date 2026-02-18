# Go Architecture Review Overview (No Code Changes)

## Summary
This review uses `golang-patterns` and evaluates your five points against current code.  
Current design is functional, but key runtime paths are callback-driven and lock-heavy in places where channel pipelines would simplify flow control, improve isolation, and make top-level orchestration more visible.

## Findings (ordered by severity)
1. **High: broker callbacks are synchronous and can stall frame processing**  
`internal/stream/broker.go:31`, `internal/stream/broker.go:41`, `internal/app/frame_processor.go:95`, `internal/app/frame_processor.go:97`, `app.go:51`, `app.go:54`  
`EmitPreview`/`EmitTelemetry` execute handlers inline on the processing path. Any slow handler (UI event emit, future subscriber) directly back-pressures camera/tracking loop.

2. **High: dwell state has unsynchronized concurrent mutation risk**  
`internal/app/cursor_mover.go:74`, `internal/app/cursor_mover.go:99`, `internal/mouse/dwell.go:30`, `internal/mouse/dwell.go:35`  
`UpdateDwell()` and `SetDwellParams()` can run concurrently; `DwellState` has mutable fields without locking/serialization.

3. **Medium: hotkey manager can create unbounded action goroutines**  
`internal/hotkeys/manager.go:72`, `internal/hotkeys/manager.go:73`  
Each keydown spawns `go action()`, so rapid key repeats can overlap start/stop/recenter actions and increase race pressure.

4. **Medium: algorithmic core is too monolithic for “pseudo-code-like” readability**  
`internal/app/frame_processor.go:59`, `internal/tracking/tracker.go:90`  
`Process()` and `Tracker.Update()` blend state snapshotting, CV ops, fallback logic, rendering, and publishing in single flows.

5. **Medium: macro orchestration is fragmented across callback wiring**  
`app.go:46`, `internal/app/app.go:82`, `internal/camera/manager.go:40`  
Top-level flow is not represented as one explicit pipeline; callback chains hide runtime choreography.

## Recommended Target Architecture
Use a **channel-based pub/sub event bus** with explicit bounded queues and drop policies.

Core runtime pipeline:
1. `camera.Manager` publishes `camera.Frame` to `framesCh`.
2. `FrameProcessor` consumes `framesCh`, emits `TrackingResult`, `PreviewFrame`, `Telemetry`.
3. `CursorMover` consumes `TrackingResult`.
4. UI bridge consumes preview/telemetry subscriptions from broker.
5. Control plane (start/stop/recenter/params) uses command channels to serialize mutable state transitions.

## Important Public API / Interface Changes
1. Replace callback broker API:
- from:
  - `SubscribePreview(handler func(PreviewFrame))`
  - `SubscribeTelemetry(handler func(Telemetry))`
  - `EmitPreview`, `EmitTelemetry`
- to:
  - `SubscribePreview(ctx context.Context, buffer int) <-chan stream.PreviewFrame`
  - `SubscribeTelemetry(ctx context.Context, buffer int) <-chan stream.Telemetry`
  - `PublishPreview(frame stream.PreviewFrame)`
  - `PublishTelemetry(t stream.Telemetry)`

2. Introduce broker policy configuration:
- `type BrokerPolicy struct { PreviewBuffer int; TelemetryBuffer int; DropPreviewIfSlow bool; DropTelemetryIfSlow bool }`

3. Optional (recommended) camera API evolution:
- from `Start(ctx, handler FrameHandler)`  
- to `Start(ctx) (<-chan camera.Frame, error)` for explicit pipeline wiring in app orchestration.

## Algorithm Decomposition Plan (pseudo-code style)
1. Split `FrameProcessor.Process` into:
- `snapshotRuntimeState()`
- `trackPoint(frame, state)`
- `applyMarkerFallback(result, state)`
- `renderPreview(frame, result, state)`
- `publishOutputs(frame, result, state)`

2. Split `Tracker.Update` into:
- `toGray(frame)`
- `computeSearchRect(gray)`
- `matchTemplate(searchMat)`
- `buildTrackingResult(match)`
- `applyAdaptiveTemplateIfEnabled(searchMat, match)`

3. Split `Service` orchestration responsibilities:
- lifecycle (`Start/Stop`)
- control commands (`SetPickPoint/Recenter/ToggleTracking`)
- params updates (`ApplyParams/SaveParams`)
- event wiring (`wireUIStreams`)

## App-Level Orchestration Visibility
Make orchestration explicit in `internal/app/app.go` via a single `runPipeline(ctx)` showing:
1. channel creation,
2. goroutine start order,
3. cancellation handling,
4. broker publish points,
5. shutdown sequence (`cancel -> wait -> close`).

## Refactor/Tidy Opportunities
1. Replace shared mutable state with single-owner goroutines where possible (broker, dwell, hotkey dispatch).
2. Keep mutexes only for short-lived snapshot access and avoid locking around external I/O calls.
3. Move param mapping helpers from `internal/app/app.go` into dedicated mapper file for cleaner service surface.
4. Standardize error strategy for tracking misses (`ErrNoTemplate`/lost state) vs normal no-detection path.

## Test Cases and Scenarios
1. Broker slow subscriber does not block producer.
2. Broker unsubscribe via context cancels and removes subscriber cleanly.
3. Preview drop policy works under backpressure; telemetry policy behaves as configured.
4. Dwell param update during active tracking has no data races (`go test -race` target).
5. Hotkey bursts do not spawn unbounded parallel start/stop transitions.
6. Frame pipeline shutdown is leak-free (all goroutines exit on cancel).
7. `Process` decomposition preserves marker fallback and mirror coordinate behavior.
8. `Tracker.Update` decomposition preserves score threshold and adaptive template behavior.

## Acceptance Criteria
1. No data races under `go test -race ./...`.
2. No frame-loop stalls caused by UI subscribers.
3. Macro runtime flow is readable from one orchestration function/file.
4. Frame/tracker logic is understandable at high level with small implementation helpers.

## Assumptions and Defaults
1. Keep current behavior semantics (tracking, dwell, preview cadence, UI events) unchanged.
2. Default broker policy: bounded buffers with preview drop-on-slow enabled.
3. Prefer channels for cross-component coordination; keep mutexes only inside component-local state where single-thread ownership is not practical.
4. No code changes performed in this review turn.
