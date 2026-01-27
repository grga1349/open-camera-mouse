camera-mouse/
  README.md
  go.mod
  wails.json

  docs/
    OVERVIEW.md
    TECH.md
    ARCHITECTURE.md
    ENGINE.md
    SCREENS.md
    TASKS.md

  build/                  # Wails build artifacts (auto)
  frontend/               # React app (Wails default)
    package.json
    src/
      app/
        App.tsx           # screen switch: main <-> settings
      screens/
        MainScreen.tsx
        SettingsScreen.tsx
        settings/
          TrackingTab.tsx
          PointerTab.tsx
          ClickingTab.tsx
          HotkeysTab.tsx
      components/
        PreviewBox.tsx
        Tabs.tsx
        StickyActions.tsx
        Toggle.tsx
        Slider.tsx
      state/
        useAppStore.ts
        useSettingsDraft.ts
      services/
        backend.ts         # Wails bindings + EventsOn wrappers
      types/
        params.ts
        telemetry.ts
      styles/
        tailwind.css

  cmd/
    app/
      main.go              # Wails bootstrap, window size, bind App

  internal/                # Go modules (your real app)
    app/
      app.go               # Wails-bound API: Start/Stop/Recenter/SaveParams...
      coordinator.go       # single main loop
      events.go            # event names + payload structs
    config/
      config.go            # defaults + load/save json
      paths.go             # app-data dir helpers
    camera/
      capture.go           # open/read camera, switch device
      devices.go           # list cameras (best effort)
    tracking/
      tracker.go           # interface + shared types
      templatematch.go     # template matching engine
      params.go            # TrackingParams + validation
      state.go             # TrackerState
    mouse/
      controller.go        # interface for OS mouse
      robotgo.go           # implementation (or alternative)
      mapping.go           # gain/smoothing/deadzone/clamp
      dwell.go             # dwell click state machine
      params.go            # PointerParams + ClickParams
    overlay/
      overlay.go           # draw marker/status/score onto preview frames
    stream/
      preview.go           # encode JPEG/WebP -> base64, throttle
      telemetry.go         # throttle + emit telemetry
    util/
      math.go              # clamp/lerp helpers
      time.go              # fps helper, throttle helper
      img.go               # rect clamp helpers
