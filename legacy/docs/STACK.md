# Tech Stack

## Backend

| Component | Version | Purpose |
|-----------|---------|---------|
| Go | 1.24 (toolchain go1.24.12) | Application language |
| Wails | v2.11.0 | Desktop app framework (Go + WebView) |
| GoCV | v0.43.0 | OpenCV 4.x bindings — camera capture & template matching |
| robotgo | v1.0.0 | System mouse control |
| golang.design/x/hotkey | v0.4.1 | Global hotkeys (Carbon on macOS, X11 on Linux) |

## Frontend

| Component | Purpose |
|-----------|---------|
| React + TypeScript | UI framework |
| Vite | Build tool & dev server |
| Tailwind CSS | Utility-first styling |

## Build Tooling

| Tool | Install | Purpose |
|------|---------|---------|
| Wails CLI | `go install github.com/wailsapp/wails/v2/cmd/wails@latest` | Dev server + production builds |
| golines | `go install github.com/segmentio/golines@latest` | Go formatter (120 char line width) |
| Node.js | v20 | Frontend builds |

## Platform Matrix

| Platform | Architecture | Status | Notes |
|----------|-------------|--------|-------|
| macOS | arm64 (Apple Silicon) | ✅ | Requires Accessibility permissions for mouse control |
| Windows | amd64 | ✅ | Requires WebView2 Runtime (pre-installed on Win 11) |
| Linux | x86_64 | ⚠️ | X11 only — no Wayland support; requires `libx11-dev` |

## Platform-Specific Dependencies

**macOS:**
```bash
brew install opencv pkg-config
```

**Windows (MSYS2 MINGW64):**
```bash
pacman -S mingw-w64-x86_64-toolchain mingw-w64-x86_64-pkg-config \
          mingw-w64-x86_64-opencv mingw-w64-x86_64-qt6-base \
          mingw-w64-x86_64-qt6-5compat
```

**Linux:**
```bash
sudo apt-get install libopencv-dev libx11-dev pkg-config
```
