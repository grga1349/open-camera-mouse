# Overview — Camera Mouse MVP


## Goal
Build an MVP “Camera Mouse”-style desktop app (Wails v2 + React + GoCV) that Ivan can use daily, then publish as open source.


## Non-goals (MVP)
- ML face landmarks / gaze tracking
- Multi-window settings (Wails v2 is single-window)
- Global hotkeys outside app focus
- Profiles UI / plugin system
- Complex calibration wizard


## Tech stack
- Desktop: Wails v2
- UI: React
- Backend: Go
- CV: GoCV (OpenCV)
- OS mouse input: behind an interface (e.g., RobotGo)


## Window + navigation
- Single window
- Size: **420×820** (fixed), non-resizable
- Two screens:
- Main (operational panel)
- Settings (tabs + sticky Save/Cancel)


## MVP acceptance criteria
1. Live camera preview renders.
2. Clicking preview sets tracking point.
3. Marker follows point; cursor follows smoothly.
4. Recenter works instantly.
5. Dwell click works (time/radius).
6. LOST state prevents cursor jumps.
7. Settings Save/Cancel works (draft model).
8. Config persists and auto-loads on startup.
9. Stable run for 30 minutes.
