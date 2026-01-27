# Tasks — MVP Build Plan (Codex-ready)


## Phase 0 — Project bootstrap
- [ ] Create Wails v2 React project
- [ ] Set window 420×820, non-resizable
- [ ] Create MainScreen + SettingsScreen navigation


## Phase 1 — Types + state
- [ ] Define TS types: AllParams, Telemetry
- [ ] Implement app store (params + telemetry + actions)
- [ ] Implement settings draft hook (snapshot/draft/dirty/save/cancel)


## Phase 2 — Backend skeleton
- [ ] Create Go packages: app, config, camera, tracking, mouse, overlay, stream
- [ ] Define Go structs mirroring AllParams
- [ ] Implement config load/save to config.json (app data dir)


## Phase 3 — Camera capture
- [ ] Implement camera open/read loop (device 0)
- [ ] Add FPS measurement


## Phase 4 — Tracking engine
- [ ] Implement SetPickPoint (extract template)
- [ ] Implement Update (search rect, MatchTemplate, MinMaxLoc)
- [ ] Implement LOST gating (no move, no drift)
- [ ] Implement optional adaptive template update


## Phase 5 — Mouse mapping + OS input
- [ ] Create MouseController interface (Move, Click variants, GetCursor)
- [ ] Implement RobotGo controller (or equivalent)
- [ ] Implement mapping: sensitivity→gain/smoothing, deadzone, clamp, lerp smoothing


## Phase 6 — Dwell clicking
- [ ] Implement dwell state machine
- [ ] Integrate clickType + rightClickToggle
- [ ] Ensure dwell disabled when LOST


## Phase 7 — Preview + telemetry
- [ ] Implement overlay drawing (marker, score, OK/LOST)
- [ ] Implement preview encoding (JPEG/WebP base64) at 15–25 fps
- [ ] Emit events: preview/frame and telemetry/state


## Phase 8 — Wails bindings
- [ ] Expose methods:
- Start/Stop
- GetParams/UpdateParams/SaveParams
- SetPickPoint
- Recenter
- ToggleTracking
- (optional) ListCameras/SelectCamera


## Phase 9 — React wiring
- [ ] Subscribe to preview/frame and telemetry/state
- [ ] MainScreen:
- preview click → SetPickPoint
- recenter/start/pause/dwell toggle/right-click toggle
- [ ] SettingsScreen:
- tabs
- sticky Save/Cancel
- Save → SaveParams(draft)


## Phase 10 — MVP stabilization
- [ ] Tune defaults
- [ ] Add scoreThreshold control
- [ ] Verify 30-minute stability
- [ ] Ensure config persists and reloads


## Phase 11 — Release prep (after MVP)
- [ ] README (build + usage)
- [ ] Demo GIF
- [ ] Basic CONTRIBUTING
