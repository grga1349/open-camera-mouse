# Open Camera Mouse

Open Camera Mouse turns your webcam into a hands-free mouse controller. Track a marker on your face or body, move the
pointer with smooth gain control, and trigger clicks through dwell timing—no additional hardware required.

## Highlights
- **Webcam tracking**: Template-matching tracker keeps a marker (circle or square) locked onto the selected spot.
- **Pointer control**: Sensitivity slider maps directly to cursor gain with advanced per-axis overrides.
- **Hands-free clicking**: Optional dwell clicking fires left/right clicks when you hover for a configurable duration.
- **Quick recentering**: Pause tracking, recenter the marker + cursor, then resume after a short countdown or via F12.
- **Global hotkeys**: F11 toggles camera start/stop and F12 recenters, even when the window is in the background (macOS).
- **Simple settings**: Tidy tabs cover Tracking, Pointer, Clicking, Hotkeys, and General options with immediate previews.

## Requirements
- macOS, Windows, or Linux with a webcam supported by GoCV (macOS tested most thoroughly).
- Go 1.21+ and Node.js 18+ (or newer) for building from source.
- [Wails CLI](https://wails.io/) installed globally: `go install github.com/wailsapp/wails/v2/cmd/wails@latest`.

## Getting Started
1. **Install dependencies**
   ```bash
   npm install --prefix frontend
   wails deps
   ```
2. **Run in dev mode** (hot-reload frontend + backend):
   ```bash
   wails dev
   ```
3. **Build a desktop app bundle**:
   ```bash
   wails build
   ```
   Output binaries live under `build/bin`.

## Using the App
1. Launch the desktop app (either from `wails dev` or the built binary).
2. Hit **Start** (or press `F11`) to enable the camera and tracker preview.
3. Click the live preview to select the feature you want to track. The marker turns green when locked, red when lost.
4. Use the **Dwell** button to enable auto-clicking. Hover over the button for 0.5s to turn it on without clicking, or
   click the button to toggle manually. Configure dwell time/radius inside Settings → Clicking.
5. Need to reset the tracking point? Click **Recenter** (or press `F12`). The marker + cursor jump to screen center,
   the button counts down for a few seconds, then tracking resumes.
6. Open **Settings** for full control:
   - **Tracking**: Template size, search margin, score threshold, adaptive template updates, marker shape.
   - **Pointer**: Sensitivity (gain), deadzone, max speed, advanced Gain X/Y + smoothing overrides.
   - **Clicking**: Dwell enable, time, radius, click type, and temporary right-click toggle.
   - **General**: Start/Pause + Recenter hotkeys, auto-start camera, reset parameters to defaults.
7. Save changes to persist them between launches. Use the Reset button to restore factory tuning before saving.

## Tips & Troubleshooting
- Lighting matters: ensure your tracker target has good contrast against the background.
- If the marker flashes red (LOST), try a larger template size or reduce sensitivity to slow cursor motion.
- When Recenering, the marker stays visible and turns white while the countdown runs; the cursor recenters as well.
- Global hotkeys currently rely on the native OS APIs provided by the Wails hotkey module; some Linux window managers
  may require additional permissions.
- Logs are printed to the console when running via `wails dev`; check them for camera or hotkey errors.

## License
This project is licensed under the MIT License. See `LICENSE` for details.
