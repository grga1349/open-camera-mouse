# Refactor Plan

The goal: every package owns exactly one concern, `app` is pseudocode, root `app.go` is a thin Wails adapter. Delete everything that doesn't earn its place.

---

## Naming convention

Every package has one primary type. That type is named **`Service`** — not `Manager`, not a repetition of the package name. The package name already provides the domain context; the type name just needs to say "this is the thing you construct and use".

```
camera.Service      // was camera.Manager    ✓ done
hotkeys.Service     // was hotkeys.Manager   ✓ done
tracking.Service    // will be tracking.Tracker
cursor.Service      // new package
```

Files follow the same pattern: the main file in each package is `service.go`.

`Manager` is a Java-ism that adds no information inside a named package. `Tracker`, `Mover`, `Manager` — all mean the same thing and none of them say anything the package name doesn't already say.

---

## What is wrong today

### `internal/app` does too much

It contains the Service coordinator, the `CursorMover` state machine, the `FrameResult` type, the `track()` goroutine, the `process()` goroutine, and the param builder functions. "App" is not a domain — it is the junk drawer where things land when there is no obvious home.

- `track()` wraps `Tracker.Update()`. It belongs in the `tracking` package — the tracker should own its own streaming goroutine.
- `CursorMover` is a cursor-driving component. It belongs in its own `cursor` package.
- `process()` / `processFrame()` / `renderAndPublish()` drive the cursor and encode output. Once `CursorMover` has its own package, `process()` becomes `cursor.Mover.Run()` — a method on the thing it operates on.
- `FrameResult` is the output of the tracking stage. It belongs in `tracking`, not `app`.

After the moves, `internal/app` shrinks to: Service lifecycle + param builders. That is the right scope for a coordinator.

### `internal/stream` is a packaging accident

It holds two unrelated things:
- `PreviewEncoder` + `PreviewFrame` — a JPEG encoding step
- `Telemetry` — a plain frontend data struct

These can stay in a reduced `stream` package or move, but the name no longer means anything meaningful. The broker is already deleted. Rename the package to `output` or just keep the two small files where they are — this is low priority cosmetic work.

### Root `app.go` has implicit lifecycle coupling

`NewApp()` passes `app.emitParams` as a callback into `Service`. This means Service calls back into the Wails layer — an inside-out dependency. The cleaner direction is to treat param changes the same way preview and telemetry are treated: a channel `Service` writes to, which `startup()` subscribes to. No callbacks, no nil-ctx guards, same pattern everywhere.

---

## Target package structure

```
internal/
├── camera/
│   └── service.go          RENAMED (was manager.go) — Service.Stream(ctx) <-chan Frame
│
├── tracking/
│   ├── service.go          RENAMED (was tracker.go) — Service type, control plane methods
│   └── stream.go           NEW — Service.Stream(ctx, frames) <-chan FrameResult
│                           FrameResult type (moved from app.FrameResult)
│
├── cursor/                 NEW PACKAGE
│   └── service.go          CursorMover moved here, renamed to Service
│                           + Run(ctx, results) returns (previewCh, telemCh, done)
│                           + all of process()/processFrame()/renderAndPublish() inline
│
├── mouse/                  UNCHANGED
├── overlay/                UNCHANGED
├── stream/                 REDUCED — keep preview.go and telemetry.go as-is
├── config/                 UNCHANGED
├── hotkeys/                UNCHANGED
│
└── app/
    ├── service.go          ONLY: Service struct + Start/Stop/IsRunning + runPipeline
    └── params.go           UNCHANGED — config → domain type builders
```

**Deleted files:**
- `internal/app/pipeline.go` — contents split into `tracking/stream.go` and `cursor/mover.go`
- `internal/app/cursor_mover.go` — becomes `cursor/mover.go`

---

## Target code — what things should look like

### `internal/app/service.go` — the pseudocode

```go
func (s *Service) runPipeline(ctx context.Context) (<-chan stream.PreviewFrame, <-chan stream.Telemetry, error) {
    frames, err := s.camera.Stream(ctx)
    if err != nil {
        return nil, nil, err
    }
    results := s.tracker.Stream(ctx, frames)           // tracking.Service.Stream
    previewCh, telemCh, done := s.cursor.Run(ctx, results)  // cursor.Service.Run
    s.done = done
    return previewCh, telemCh, nil
}
```

Three lines that read as the algorithm: capture → track → process → return output.
No goroutine management, no types, no implementation details.

The Service struct shrinks to:

```go
type Service struct {
    cfgManager   *config.Manager
    notifyParams func(config.AllParams)   // see root app.go cleanup below

    mu      sync.RWMutex
    params  config.AllParams

    camera  *camera.Manager
    tracker *tracking.Tracker
    cursor  *cursor.Mover

    cancel  context.CancelFunc
    done    <-chan struct{}
    running bool
}
```

### `internal/tracking/stream.go` — the track goroutine

```go
type FrameResult struct {
    Frame   camera.Frame
    Point   image.Point
    Score   float32
    Lost    bool
    Enabled bool    // was Tracking
    Params  Params
}

func (s *Service) Stream(ctx context.Context, frames <-chan camera.Frame) <-chan FrameResult {
    out := make(chan FrameResult, 1)
    go func() {
        defer close(out)
        defer t.Close()
        for {
            select {
            case <-ctx.Done():
                drainFrames(frames)
                return
            case frame, ok := <-frames:
                if !ok {
                    return
                }
                r := t.Update(frame)
                fr := FrameResult{
                    Frame:   frame,
                    Point:   r.Point,
                    Score:   r.Score,
                    Lost:    r.Lost,
                    Enabled: t.IsTrackingEnabled(),
                    Params:  t.Snapshot(),
                }
                select {
                case out <- fr:
                case <-ctx.Done():
                    frame.Mat.Close()
                    drainFrames(frames)
                    return
                }
            }
        }
    }()
    return out
}
```

`drainFrames` helper moves here from `pipeline.go`.

### `internal/cursor/mover.go` — cursor driving + pipeline output

```go
package cursor

const previewInterval = 66 * time.Millisecond

type Mover struct {
    controller mouse.Controller
    mapper     *mouse.Mapper
    dwell      *mouse.DwellState

    // pipeline-goroutine-owned
    lastPoint image.Point
    pointSet  bool

    // control-plane dirty flags
    mappingMu      sync.Mutex
    pendingMapping mouse.MappingParams
    mappingDirty   bool
    resetPending   bool
}

func (m *Mover) Run(ctx context.Context, results <-chan tracking.FrameResult) (<-chan stream.PreviewFrame, <-chan stream.Telemetry, <-chan struct{}) {
    previewCh := make(chan stream.PreviewFrame, 2)
    telemCh   := make(chan stream.Telemetry, 4)
    done      := make(chan struct{})
    go func() {
        defer close(done)
        defer close(previewCh)
        defer close(telemCh)
        enc := stream.NewPreviewEncoder(previewInterval)
        for {
            select {
            case <-ctx.Done():
                drain(results)
                return
            case result, ok := <-results:
                if !ok {
                    return
                }
                m.step(result, enc, previewCh, telemCh)
            }
        }
    }()
    return previewCh, telemCh, done
}
```

`step()` is `processFrame()` renamed, `renderAndPublish()` stays inline or as a private helper. `drain()` is `drainResults()` moved here.

### Root `app.go` — cleanup

The main smell is the `emitParams` callback passed into `NewService`. The fix is to treat param changes as a channel output from the Service, the same way preview and telemetry already work.

**Service gains a `paramsCh` field:**
```go
paramsCh chan config.AllParams  // closed on shutdown
```

**`emitParamsLocked()` writes to it (non-blocking):**
```go
func (s *Service) emitParamsLocked() {
    p := s.params
    select {
    case s.paramsCh <- p:
    default:
    }
}

func (s *Service) ParamChanges() <-chan config.AllParams {
    return s.paramsCh
}
```

**`NewApp()` no longer passes a callback:**
```go
svc, err := appsvc.NewService(cfg)
```

**`startup()` subscribes to all three output channels:**
```go
func (a *App) startup(ctx context.Context) {
    a.ctx = ctx
    go func() {
        for p := range a.service.ParamChanges() {
            runtime.EventsEmit(ctx, "params:update", p)
        }
    }()
    params := a.service.GetParams()
    a.applyHotkeys(params.Hotkeys)
    if params.General.AutoStart {
        go func() {
            if err := a.Start(); err != nil {
                a.logErrorf("autostart failed: %v", err)
            }
        }()
    }
}
```

The nil-ctx guard in `emitParams` disappears because the channel is always valid after construction.

---

## Dead code to delete alongside the moves

From the review — delete these while touching each file:

| File | What to delete |
|------|---------------|
| `tracking/tracker.go` | `Lost()` method, `t.lost` field |
| `tracking/tracker.go` (new `result.go`) | `Result.Timestamp` field |
| `mouse/dwell.go` | **Fix deadlock**: release `d.mu` before calling `afterClick()` and before `controller.Click()` |
| `mouse/dwell.go` | `controller.Click()` called under lock — move outside |
| `app/params.go` | `ClickTypeDouble` maps to `ClickLeft` — add `ClickDouble` to mouse or remove from config |
| `camera/manager.go` | Rename `cap` → `vcap` (shadows builtin) |
| `camera/manager.go` | EMA-smooth the FPS calculation |
| `tracking/tracker.go` | `errors.New("below threshold")` in hot path → package-level var |
| `tracking/tracker.go` | `matchTemplate` re-computes resultCols/resultRows already validated by `computeSearchRect` |

---

## Tracker mutex — the remaining hot-path lock

After the moves above, the one remaining architectural debt is `Tracker.Update()` holding a write lock for the entire template-matching operation (~10ms). The fix uses the same dirty-flag pattern as `cursor.Mover`:

- `pendingParams` + `paramsDirty` flag — drained at the top of `Stream()`'s goroutine loop, before calling `Update()`
- `pendingPickPoint` + `pickDirty` flag — same drain point; `extractTemplate` happens from inside the goroutine using the current incoming frame instead of `lastFrame`
- `trackingEnabled` → `atomic.Bool` (single field, no struct lock needed)
- `lastFrame` is removed — the goroutine uses the incoming `frame` directly; `SetPickPoint` queues a pending pick that the goroutine services on next frame

With those changes the only lock left in Tracker is the write lock in `SetPickPoint`/`Recenter` to enqueue the pending point — a very short critical section with no CV work inside it.

This is the most complex change because GoCV Mat ownership has to be reasoned through carefully. Do it last, after the package moves are clean.

---

## Migration order

Each step compiles and passes `go build ./...` before moving to the next.

1. **Rename `tracking/tracker.go` → `tracking/service.go`**, type `Tracker` → `Service`; update all references
2. **Add `tracking.FrameResult` type and `Service.Stream()`** (new file `tracking/stream.go`) — no deletions yet, both `track()` and `Stream()` exist
3. **Create `internal/cursor` package** — move `CursorMover` as `cursor.Service`, add `Run()` method (content of `process()`); both `cursor.Service` and `app.CursorMover` exist
4. **Update `internal/app/service.go`** — swap `*CursorMover` → `*cursor.Service`, use `s.tracker.Stream()` and `s.cursor.Run()` in `runPipeline`; imports updated
5. **Delete `internal/app/pipeline.go` and `internal/app/cursor_mover.go`**
6. **Dead code cleanup** — `Lost()`, `t.lost`, `Result.Timestamp`, `matchTemplate` redundancy, sentinel error, `cap` rename, FPS EMA
7. **Fix dwell deadlock** — release `d.mu` before calling `afterClick()` and `controller.Click()`
8. **Root `app.go` cleanup** — replace callback with `paramsCh`, update `startup()`
9. **Tracker dirty-flag refactor** — last and most complex; convert hot-path lock to dirty flags + atomic

---

---

## React frontend

### The god object — `useAppStore.tsx`

`useAppStore` puts four categories of state in a single context:

| State | Update rate | Who cares |
|-------|------------|-----------|
| `params` | Rare (save/load) | SettingsScreen, ClickModeControls, MainScreen |
| `isRunning` | Rare (start/stop) | PrimaryActions, StatusHeader |
| `telemetry` | ~30fps | StatusHeader only |
| `preview` | ~15fps | CameraPreview only |

Because it's one context, a 30fps telemetry tick re-renders every component that calls `useAppStore()` — including `ClickModeControls` and `PrimaryActions`, which read only from `params` and `isRunning`. This is a continuous render storm.

**Fix: split into four focused contexts/hooks:**

```
src/state/
  useParams.ts       — params + setParams (slow, shared widely)
  useRunning.ts      — isRunning + setRunning (slow)
  useTelemetry.ts    — telemetry + setTelemetry (30fps, only StatusHeader)
  usePreview.ts      — preview + setPreview (15fps, only CameraPreview)
```

Each hook exposes its own `Context` + `Provider`. `App.tsx` composes the four providers. Components subscribe only to what they need.

The `StoreActions` exported type and the `AppState` mega-type both disappear. `AppProvider` is replaced by four small providers.

`PreviewFrame` type moves from `useAppStore.tsx` to `types/preview.ts` — it's a domain type, not a store concern.

---

### `MainScreen.tsx` has too much inline logic

At 176 lines, `MainScreen` owns:
- Recenter orchestration (pause → recenter → countdown → resume)
- Dwell hover auto-enable (timeout on mouse-enter)
- Preview click → pick-point mapping
- Clicking param optimistic updates

The recenter timer is already extracted (`useRecenterCountdown`), but `handleRecenter` itself — which sequences pause/recenter/resume — is inline. The dwell hover timer (`dwellHoverRef`) is also inline. Both should be hooks.

**Extract:**
```
screens/main/hooks/
  useRecenterCountdown.ts   — already exists ✓
  useRecenter.ts            — NEW: pause → Recenter() → countdown → resume
  useDwellHover.ts          — NEW: hover timeout → enableDwell
  usePickPoint.ts           — NEW: click event → SetPickPoint(x, y)
```

After extraction `MainScreen` becomes wiring-only: compose hooks, pass props to leaf components.

---

### Repeated section-updater boilerplate in every tab

Every tab defines the same pattern:

```ts
const updateTracking = (changes: Partial<typeof tracking>) =>
  updateDraft((current) => ({ ...current, tracking: { ...current.tracking, ...changes } }));
```

Four tabs × ~5 lines each = ~20 lines of copy-paste. Replace with a shared factory in `TabProps` or a utility:

```ts
// shared util (tabs/utils.ts or inline in TabProps)
function makeUpdater<K extends keyof AllParams>(
  updateDraft: TabProps['updateDraft'],
  key: K,
) {
  return (changes: Partial<AllParams[K]>) =>
    updateDraft((curr) => ({ ...curr, [key]: { ...curr[key], ...changes } }));
}

// usage in tab:
const updateTracking = makeUpdater(updateDraft, 'tracking');
```

---

### `useSettingsDraft.ts` — dead export and duplicate clone

`saveDraft` is exported in `SettingsDraft` but never called (SettingsScreen uses the `onSave` callback instead). Remove it.

`cloneParams` inside `useSettingsDraft.ts` and `normalizeParams` inside `App.tsx` are the same `JSON.parse(JSON.stringify(...))` one-liner with different names. Extract to `src/lib/clone.ts`:

```ts
export const deepClone = <T>(value: T): T => JSON.parse(JSON.stringify(value));
```

---

### `useMemo` on JSX in `SettingsScreen.tsx`

```ts
const activeTabContent = useMemo(() => {
  const Component = TAB_COMPONENTS[activeTab];
  return <Component draft={draft} updateDraft={updateDraft} />;
}, [activeTab, draft, updateDraft]);
```

`useMemo` on JSX is not useful — `React.createElement` is cheap; the memo doesn't prevent child renders. If the goal is to avoid re-creating the element when other SettingsScreen state changes, this is redundant (there is no other state). Remove it; render directly.

---

### `App.tsx` — PascalCase event field fallbacks

```ts
const data = frame?.Data ?? frame?.data;
const fps = payload?.FPS ?? payload?.fps ?? 0;
```

Wails serializes Go struct fields using their JSON tags (camelCase), so these PascalCase fallbacks are never hit. The `normalizeParams` JSON round-trip already handles the struct serialization correctly. The telemetry/preview event payloads go through the same Wails serializer — trust the camelCase and drop the `?? PascalCase` chains.

---

### `GeneralTab.tsx` — duplicate inline defaults

```ts
const general = draft.general ?? { autoStart: false, dwellOnStartup: false };
```

Appears twice. `draft.general` is always present per `AllParams` type — the `??` is dead code caused by mistrust of the type. Remove the fallbacks; they hide a real type issue if they're ever needed.

---

### `ScreenShell.tsx` — comment violates coding standard

Remove the JSDoc comment. The name and props are self-explanatory.

---

### Priority

| # | Severity | Location |
|---|----------|----------|
| 1 | **Perf / arch** | `useAppStore.tsx` → split into 4 focused hooks |
| 2 | **Structure** | `MainScreen.tsx` → extract 3 hooks |
| 3 | Code quality | Tab boilerplate → `makeUpdater` util |
| 4 | Dead code | `saveDraft`, duplicate `cloneParams`/`normalizeParams` |
| 5 | Correctness | `useMemo` on JSX in `SettingsScreen` |
| 6 | Cleanup | Wails PascalCase fallbacks, `GeneralTab` dead defaults, comment |

---

## What does NOT change

- The three-goroutine topology (camera → track → process) — correct, stays
- Buffered channels of size 1 between stages (back-pressure)
- All of `mouse/`, `overlay/`, `config/`, `hotkeys/` — already clean
- `stream/preview.go` and `stream/telemetry.go` — already just data, no logic worth moving
- The dirty-flag pattern in `cursor.Mover` for control-plane mapping params
- `go build ./...` passes at every step
