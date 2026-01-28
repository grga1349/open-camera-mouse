# Handover — Tailwind Bootstrap

## Completed
- Configured Tailwind CSS (postcss config, theme tokens, npm deps) and removed legacy App.css styling.
- Added placeholder MainScreen + SettingsScreen components using Tailwind utility classes and navigation shell in `App.tsx`.
- Set window size to 420×820 with resizing disabled in `main.go`.
- Documented Tailwind requirement inside `docs/TASKS.md`.
- Simplified styling to pure Tailwind palette classes (zinc neutrals + emerald CTA) with a shared `Button` component for consistent shape/hover.
- Added `frontend/src/types/params.ts` and `frontend/src/types/telemetry.ts` to define AllParams + Telemetry contracts for upcoming state work.
- Implemented `useAppStore` context with default params/telemetry + actions, wrapping `<AppProvider>` around the UI in `App.tsx`.
- Added `useSettingsDraft` hook that snapshots params, exposes draft/dirty/save/reset helpers for the Settings screen.
- Bootstrapped backend skeleton: `internal/config` (AllParams structs + JSON manager), and empty packages for `app`, `camera`, `tracking`, `mouse`, `overlay`, `stream` compiled via `go build ./...`. Added `internal/camera/manager.go` using GoCV to read device 0 frames with FPS measurement.
- Implemented `internal/tracking/tracker.go`: template extraction (SetPickPoint), MatchTemplate-based Update with score gating, adaptive template updates, and LOST state tracking.
- Mouse subsystem now has: `controller.go` interface, `robotgo.go` implementation using RobotGo (v0.110.3), and `mapping.go` for sensitivity/deadzone/gain/smoothing math.
- Added dwell clicking state machine (`internal/mouse/dwell.go`) that tracks cursor dwell radius/time and fires controller clicks when enabled + not LOST.
- Overlay + stream scaffolding added: `overlay/overlay.go` draws marker/score/LOST, while `stream/preview.go` encodes JPEG→base64 with throttling and `stream/broker.go` dispatches preview/telemetry events (with `stream/telemetry.go` struct definitions).
- Wired backend into Wails: `internal/app/service.go` orchestrates config, camera, tracker, overlay, stream broker, and bindings now expose Start/Stop/Params/Pick/Recenter/Toggle via `app.go` with events emitted as `preview:frame` + `telemetry:state`.
- Added working Settings tab state + a compact 2-column layout so active tabs stay legible within the 420px window.
- Camera preview placeholder now uses a fixed 4:3 aspect ratio box that's wider than it is tall.
- React frontend now subscribes to `preview:frame`/`telemetry:state`, updates the shared store, wires MainScreen controls (start/pause, recenter, dwell/right-click toggles, preview pick) and Settings draft save/cancel flows to Wails bindings.


## Next Up
- Flesh out MainScreen UI (preview, controls) + Settings tab contents per `docs/SCREENS.md` once backend data is available.
- Implement actual navigation/state handling for settings tabs + sticky actions.
- Continue Phase 1 tasks (types + state) before wiring real controls.

## Notes
- `frontend/src/style.css` only hosts Tailwind directives + base font/background layer (antialiased system stack).
- Run `cd frontend && npm run dev` for live reload or `npm run build` for production assets (verified once).
- All colors now come directly from Tailwind utility classes (e.g., `bg-zinc-900`, `bg-emerald-500`).
