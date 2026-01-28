# Handover — Current Snapshot

## Completed
- **Backend pipeline**: `internal/app.Service` now orchestrates config loading, camera capture, template tracking, pointer mapping, dwell clicks, overlay rendering, preview encoding, telemetry emission, and RobotGo cursor control. Runtime params can be hot-reloaded and pushed to the UI via `params:update`.
- **Camera + tracking**: GoCV capture on device 0 feeds the template-matching tracker with adaptive template updates, mirrored marker rendering, and LOST gating so cursor motion pauses when no target is found.
- **Pointer & clicking**: Pointer sensitivity maps to gain/smoothing with optional advanced overrides, deltas are inverted horizontally to match the mirrored preview, and dwell clicking fires through the controller while auto-clearing the temporary right-click toggle.
- **Config + persistence**: `config.Manager` reads/writes JSON in the user config dir; defaults align with current tuning (template 30px, sensitivity 65, etc.). Save/Update calls from the UI immediately apply and persist parameters.
- **Wails bindings + events**: Start/Stop/Recenter/SetPickPoint/ToggleTracking/Get+Update+Save params are exposed, with preview/telemetry emissions wired to Wails events in `app.go`.
- **React app**: Tailwind-only styling, shared `useAppStore` context, preview + telemetry subscriptions, settings draft hook, reusable Button, and two screens sized for 420×820. MainScreen implements preview click-to-pick, start/pause, recenter with countdown pause, dwell/right-click toggles, and settings navigation.
- **Settings UI**: Settings tabs edit all Tracking/Pointer/Clicking options with sliders/toggles, showing hotkey documentation and sticky Save/Cancel actions that call `SaveParams` then return to Main.

## Next Up
- Run extended (30+ min) stability soak tests while verifying CPU usage + FPS; capture issues.
- Decide whether Start/Stop should differentiate from tracker enabled state or if ToggleTracking needs UI exposure.
- Expand preview UX (e.g., overlay telemetry text) once real frames confirm sizing, and consider instructions for pick flow.
- Phase 10 verification items: document/automate config-path testing across OSes, confirm dwell + right-click behavior across edge cases, and tighten defaults after real-world usage.
- Phase 11 release prep: README rewrite (build/run instructions, requirements), demo assets, and contributing guidelines.
