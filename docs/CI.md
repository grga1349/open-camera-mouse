# CI/CD

## Trigger

GitHub Actions fires on any push of a `v*` tag:

```bash
git tag v0.3.0 && git push --tags
```

## Jobs

```
build-macos ──┐
              ├──▶ release (ubuntu-latest)
build-windows─┘
```

| Job | Runner | Output |
|-----|--------|--------|
| `build-macos` | `macos-14` (arm64) | `open-camera-mouse_vX.X.X_macOS.zip` |
| `build-windows` | `windows-2022` + MSYS2 MINGW64 | `open-camera-mouse_vX.X.X_Windows.zip` |
| `release` | `ubuntu-latest` | GitHub Release with generated notes |

## Per-Platform Build Steps

Both platforms follow the same four-stage pipeline:

### macOS (`ci/macos/`)

1. **`build.sh`** — sets CGO env vars, calls `wails build -platform darwin/arm64`
2. **`bundle.sh`** — collects Homebrew dylibs (OpenCV, etc.), fixes rpaths with `install_name_tool`, ad-hoc codesigns the `.app`
3. **`package.sh`** — `ditto` zips the `.app` bundle
4. **`smoke-test.sh`** — launches the binary with `--smoke-test` flag, verifies it exits 0

### Windows (`ci/windows/`)

1. **`build.sh`** — MSYS2 environment, calls `wails build -platform windows/amd64`
2. **`bundle.sh`** — `objdump`-based DLL dependency walk, copies required Qt6 plugins, creates zip
3. **`smoke-test.sh`** — runs the exe with `--smoke-test`

## Tag Verification

`ci/verify-tag.sh` runs as the first step on both build jobs. It asserts that the pushed tag exactly matches the content of the `VERSION` file. The release will fail fast if they diverge.

## Release Process

1. Bump the `VERSION` file (e.g. `0.3.0`)
2. Commit: `git commit -am "0.3.0"`
3. Tag and push: `git tag v0.3.0 && git push --tags`
4. Actions builds both platforms, runs smoke tests, then publishes a GitHub Release with auto-generated release notes and the two zip artifacts attached
