# Open Camera Mouse

Control your computer cursor with head movements using just a webcam. No special hardware needed.

<img src="docs/screenshot.png" alt="Screenshot" width="288" />

## Download

Get the latest release for your platform:

- **[macOS](https://github.com/ivangrga/open-camera-mouse/releases/latest)** (Apple Silicon)
- **[Windows](https://github.com/ivangrga/open-camera-mouse/releases/latest)** (64-bit)

## Features

- **Head tracking** — Move your cursor by moving your head
- **Dwell clicking** — Hover to click, no physical input needed
- **Adjustable sensitivity** — Fine-tune cursor speed and smoothing
- **Global hotkeys** — F11 to start/stop, F12 to recenter
- **Works in background** — Hotkeys work even when app is minimized

## Quick Start

1. **Launch** the app and click **Start** (or press F11)
2. **Click** on your face in the preview to set the tracking point
3. **Move** your head to control the cursor
4. **Enable Dwell** to click by hovering (optional)

## Settings

| Tab | Options |
|-----|---------|
| **Tracking** | Template size, search margin, score threshold, marker shape |
| **Pointer** | Sensitivity, deadzone, max speed, advanced gain controls |
| **Clicking** | Dwell time, radius, click type (left/right) |
| **General** | Hotkey bindings, auto-start, dwell on startup |

## Tips

- **Good lighting** helps tracking accuracy
- **Marker turns red** when tracking is lost — try a larger template size
- **Press F12** to recenter both the tracker and cursor
- **Increase sensitivity** for less head movement, decrease for more precision

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Cursor jumps around | Increase template size, improve lighting |
| Tracking lost frequently | Lower score threshold, use higher contrast point |
| Cursor too fast/slow | Adjust sensitivity in Pointer settings |
| Hotkeys don't work | macOS: Grant accessibility permissions |

## Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| macOS | ✅ | Requires accessibility permissions |
| Windows | ✅ | Tested on Windows 10/11 |
| Linux | ⚠️ | X11 only, requires libx11-dev |

---

## Contributing

### Requirements

- Go 1.21+
- Node.js 18+
- OpenCV 4.x
- [Wails CLI](https://wails.io/)

### Setup

```bash
# Install Wails
go install github.com/wailsapp/wails/v2/cmd/wails@latest

# Install frontend dependencies
cd frontend && npm install && cd ..

# Run in development mode
wails dev

# Build for production
wails build
```

### Platform-specific Dependencies

**macOS:**
```bash
brew install opencv pkg-config
```

**Windows (MSYS2):**
```bash
pacman -S mingw-w64-x86_64-opencv mingw-w64-x86_64-pkg-config
```

**Linux:**
```bash
sudo apt-get install libopencv-dev libx11-dev pkg-config
```

### Project Structure

```
├── main.go              # Entry point
├── app.go               # Wails bindings
├── frontend/            # React UI
└── internal/
    ├── app/             # Core logic
    ├── camera/          # Webcam capture
    ├── tracking/        # Template matching
    ├── mouse/           # Cursor control
    ├── config/          # Settings persistence
    ├── stream/          # Preview streaming
    └── hotkeys/         # Global shortcuts
```

### Releasing

1. Update `VERSION` file
2. Create and push tag: `git tag v0.x.x && git push --tags`
3. GitHub Actions builds and publishes releases

## License

MIT License. See [LICENSE](LICENSE) for details.
