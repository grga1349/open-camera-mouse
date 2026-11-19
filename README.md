# Open Camera Mouse

Control the mouse pointer with your head, using an ordinary webcam. Click a spot on your face
in the camera preview (the tip of the nose works well); the app tracks it and moves the
pointer as you move your head. Hold still over something to click it (dwell click, optional).
Everything runs on your computer; no camera image leaves it.

Written in Rust (egui), no OpenCV. Behaviour follows the classic Camera Mouse 2018, with
improvements. Free software under the MIT licence.

## Install

Download the file for your system from the
[latest release](https://github.com/grga1349/open-camera-mouse/releases/latest) and check
it against `SHA256SUMS` if you like.

### macOS (Apple Silicon)

1. Open `open-camera-mouse-<version>-macos-arm64.dmg` and drag **Open Camera Mouse** to
   Applications.
2. The app isn't notarized yet, so macOS blocks the first launch. Open it once, then go to
   **System Settings → Privacy & Security** and click **Open Anyway**. (Or, in Terminal:
   `xattr -dr com.apple.quarantine "/Applications/Open Camera Mouse.app"`.)
3. Allow **Camera** when asked. Then grant **Accessibility** (System Settings → Privacy &
   Security → Accessibility → Open Camera Mouse on); the app shows a button that opens the
   right page. It notices the grant within a few seconds, no restart needed.

Updates keep both permissions. Intel Macs aren't supported.

### Windows (x64)

1. Run `open-camera-mouse-<version>-windows-x64-setup.exe` (installs for your user only, no
   admin rights needed).
2. The installer isn't signed yet, so SmartScreen may warn: click **More info → Run anyway**.
3. Allow camera access if Windows asks (Settings → Privacy & security → Camera → let desktop
   apps access your camera).

### Linux (x86_64, X11)

AppImage: `chmod +x open-camera-mouse-<version>-linux-x86_64.AppImage` and run it (needs FUSE
2, e.g. `libfuse2`). Or unpack the `.tar.gz` and run `open-camera-mouse`; a `.desktop` file
and icon are included. Your user needs access to the camera (usually the `video` group).

**Use an X11 session.** On Wayland the app can't move the pointer or catch global hotkeys
(see Known limitations).

## Use

1. Press **Start** (or the start key) and click a spot on your face in the preview.
2. Keep the mouse still for 4 s: the camera takes over the pointer. Moving the physical mouse
   takes it back at once; 4 s still hands it to the camera again.
3. **Recenter** (F12) puts a circle in the middle of the preview and counts 5-4-3-2-1; look
   straight at the screen, and the spot in the circle becomes the new target.
4. **Options** has speed (gain), smoothing, dwell click, camera, screen limits, main monitor,
   theme and larger controls. Changes are saved at once.

| Key | macOS | Windows / Linux |
|---|---|---|
| Start / pause | F10 | F11 |
| Recenter | F12 | F12 |

On a Mac laptop keyboard you may need `Fn`. The keys work from any app where the OS allows
it, and always inside the app window; the window shows which ones work globally.

## Known limitations

These are the release criteria's explicit exceptions for v0.4.0:

| Platform | Limitation |
|---|---|
| macOS | Ad-hoc signed, not notarized: Gatekeeper blocks the first launch (see Install). Apple Silicon only. |
| Windows | Unsigned: SmartScreen warning. The pointer can't control windows running as administrator, the UAC prompt or the lock screen (Windows UIPI). **Not yet tested on a real Windows device**: built and unit-tested in CI only. |
| Linux | X11 only; on Wayland the pointer and global hotkeys are unsupported (in-window keys still work). **Not yet tested on a real Linux device**: built and unit-tested in CI only. |
| All | External USB webcams are untested on device; cameras that only deliver MJPEG are unsupported. The "Searching" animation ignores the OS reduce-motion setting. |

Please report problems on [GitHub Issues](https://github.com/grga1349/open-camera-mouse/issues).

## Develop

Needs [rustup](https://rustup.rs); the toolchain is pinned in `rust-toolchain.toml`. On Linux
install the eframe/camera libraries first:
`libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libclang-dev`.

```sh
cargo run -p open-camera-mouse     # dev build (optimised deps, fast enough to use)
cargo test --workspace             # no webcam needed
```

When run from a terminal, macOS asks for camera and Accessibility permission for the
**terminal app**, not for Open Camera Mouse. Diagnostics: `OCM_PERF=1` prints frame timing;
`OCM_RECORD=<folder>` records raw frames for debugging (contains your face; never commit it).

Docs: [`AGENTS.md`](AGENTS.md) (how to work in this repo, also for AI agents),
[`docs/PRODUCT.md`](docs/PRODUCT.md), [`docs/BEHAVIOR.md`](docs/BEHAVIOR.md),
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), [`docs/TESTING.md`](docs/TESTING.md),
[`docs/ROADMAP.md`](docs/ROADMAP.md), decisions in
[`docs/architecture/decisions/`](docs/architecture/decisions/).

### Package

```sh
cargo install cargo-packager --version 0.11.8 --locked
cargo install cargo-about --version 0.9.3 --locked --features cli
cd apps/desktop && cargo packager --release
```

Builds the package for the current OS into `target/packager/` (macOS: `.app` + `.dmg`,
Windows: NSIS installer, Linux: AppImage), with `THIRD_PARTY_LICENSES.html` inside.

### Release

1. Set the version in the root `Cargo.toml` (`[workspace.package] version`) and commit.
2. Tag `vX.Y.Z` (must equal that version) on `main` and push the tag.
3. `.github/workflows/release.yml` builds all three platforms and creates a **draft**
   GitHub Release with `SHA256SUMS`. Check it, then publish.

Signing turns on by itself when the repository secrets exist (macOS Developer ID +
notarization, Windows certificate; listed at the top of `release.yml`).

## Licence

MIT, see [`LICENSE`](LICENSE). Third-party licences ship with every package.
