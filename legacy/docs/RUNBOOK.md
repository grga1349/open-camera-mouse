# Runbook

## Prerequisites

Install platform-specific native dependencies before anything else.

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

## Setup

```bash
# Go tooling
go install github.com/wailsapp/wails/v2/cmd/wails@latest
go install github.com/segmentio/golines@latest

# Frontend dependencies
cd frontend && npm install && cd ..
```

## Development

```bash
make wails-dev        # start Wails dev server (hot-reload)
make frontend-dev     # start Vite dev server only
```

## Formatting

```bash
make format           # golines (Go, 120 char) + Prettier (frontend)
make format-go        # Go only
make format-frontend  # frontend only
```

## Testing & Linting

```bash
go test ./...         # run all tests
go test -race ./...   # run with race detector (acceptance criterion)
go vet ./...          # static analysis
```

## Production Build

```bash
make wails-build      # wails build -clean → outputs to build/bin/
```

## Config File Locations

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/open-camera-mouse/config.json` |
| Windows | `%APPDATA%/open-camera-mouse/config.json` |
| Linux | `~/.config/open-camera-mouse/config.json` |

## Release Checklist

1. Update `VERSION` file (e.g. `0.3.0`)
2. Commit: `git commit -am "0.3.0"`
3. Tag: `git tag v0.3.0 && git push --tags`
4. GitHub Actions builds both platforms and publishes the release automatically

See [CI.md](CI.md) for details on the release pipeline.

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `pkg-config: command not found` | Install `pkg-config` for your platform (see Prerequisites) |
| `opencv4.pc not found` | Ensure OpenCV is installed and pkg-config can find it: `pkg-config --modversion opencv4` |
| `wails: command not found` | Run `go install github.com/wailsapp/wails/v2/cmd/wails@latest` and ensure `$GOPATH/bin` is in `$PATH` |
| macOS: cursor doesn't move | Grant Accessibility permissions: System Settings → Privacy & Security → Accessibility |
| macOS: app won't open | Right-click → Open to bypass Gatekeeper (app is unsigned) |
| Windows: missing DLLs | Run from MSYS2 shell or ensure MSYS2 MINGW64 `bin/` is in `PATH` |
| Build fails on Linux | Ensure `libx11-dev` is installed alongside OpenCV deps |
