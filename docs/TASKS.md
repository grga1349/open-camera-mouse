# Tasks — MVP Build Plan (Codex-ready)


## Phase 0 — Project bootstrap
- [x] Create Wails v2 React project
- [x] Set window 420×820, non-resizable
- [x] Install + configure Tailwind CSS (React UI)
- [x] Create MainScreen + SettingsScreen navigation


## Phase 1 — Types + state
- [x] Define TS types: AllParams, Telemetry
- [x] Implement app store (params + telemetry + actions)
- [x] Implement settings draft hook (snapshot/draft/dirty/save/cancel)


## Phase 2 — Backend skeleton
- [x] Create Go packages: app, config, camera, tracking, mouse, overlay, stream
- [x] Define Go structs mirroring AllParams
- [x] Implement config load/save to config.json (app data dir)


## Phase 3 — Camera capture
- [x] Implement camera open/read loop (device 0)
- [x] Add FPS measurement


## Phase 4 — Tracking engine
- [x] Implement SetPickPoint (extract template)
- [x] Implement Update (search rect, MatchTemplate, MinMaxLoc)
- [x] Implement LOST gating (no move, no drift)
- [x] Implement optional adaptive template update


## Phase 5 — Mouse mapping + OS input
- [x] Create MouseController interface (Move, Click variants, GetCursor)
- [x] Implement RobotGo controller (or equivalent)
- [x] Implement mapping: sensitivity→gain/smoothing, deadzone, clamp, lerp smoothing


## Phase 6 — Dwell clicking
- [x] Implement dwell state machine
- [x] Integrate clickType + rightClickToggle
- [x] Ensure dwell disabled when LOST


## Phase 7 — Preview + telemetry
- [x] Implement overlay drawing (marker, score, OK/LOST)
- [x] Implement preview encoding (JPEG/WebP base64) at 15–25 fps
- [x] Emit events: preview/frame and telemetry/state


## Phase 8 — Wails bindings
- [x] Expose methods:
  - Start/Stop
  - GetParams/UpdateParams/SaveParams
  - SetPickPoint
  - Recenter
  - ToggleTracking
  - (optional) ListCameras/SelectCamera


## Phase 9 — React wiring
- [x] Subscribe to preview/frame and telemetry/state
- [x] MainScreen:
  - preview click → SetPickPoint
  - recenter/start/pause/dwell toggle/right-click toggle
- [x] SettingsScreen:
  - tabs
  - sticky Save/Cancel
  - Save → SaveParams(draft)


## Phase 10 — MVP stabilization
- [x] Tune defaults
- [x] Add scoreThreshold control
- [ ] Verify 30-minute stability
- [x] Ensure config persists and reloads


## Phase 11 — Release prep (after MVP)
- [ ] README (build + usage)
- [ ] Demo GIF
- [ ] Basic CONTRIBUTING
