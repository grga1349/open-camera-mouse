# Open Camera Mouse

Control the mouse pointer with your head, using an ordinary webcam. Click a spot on your face
in the camera preview (the tip of the nose works well); the app tracks it and moves the
pointer as you move your head. Hold still over something to click it (dwell click, optional).
Everything runs on your computer; no camera image leaves it.

Written in Rust (egui), no OpenCV. Behaviour follows the classic Camera Mouse 2018, with
improvements. Free software under the MIT licence.

**Website and user manual: <https://grga1349.github.io/open-camera-mouse/>.** The
[manual](https://grga1349.github.io/open-camera-mouse/manual/) covers installation, permissions, everyday use, every setting, accessibility
options and troubleshooting.

## Install

Download the file for your system from the
[latest release](https://github.com/grga1349/open-camera-mouse/releases/latest) (or the
[website](https://grga1349.github.io/open-camera-mouse/#download)) and check it against
`SHA256SUMS` if you like. Step-by-step instructions: [Installation](https://grga1349.github.io/open-camera-mouse/manual/installation/).

- **macOS (Apple Silicon):** open the `.dmg` and drag the app to Applications. It isn't
  notarized yet: after the first blocked launch, **System Settings → Privacy & Security →
  Open Anyway**. Then allow Camera and Accessibility.
- **Windows (x64):** run the `-setup.exe` (per-user, no admin). It isn't signed yet:
  SmartScreen → **More info → Run anyway**.
- **Linux (x86_64, X11):** run the AppImage (`chmod +x`, needs FUSE 2) or unpack the
  `.tar.gz`. **Use an X11 session**: on Wayland the app can't move the pointer or catch
  global hotkeys.

## Use

Press **Start** and click a point on your face in the preview; after 4 s of a still mouse the
camera moves the pointer. **Recenter** (F12) counts down 5 s and takes the point in the
middle of the preview as the new target. **Options** has speed, smoothing, dwell click,
camera and screen settings. More in the manual: [Getting started](https://grga1349.github.io/open-camera-mouse/manual/getting-started/),
[Using the pointer](https://grga1349.github.io/open-camera-mouse/manual/using-the-pointer/), [Settings](https://grga1349.github.io/open-camera-mouse/manual/settings/).

| Key | macOS | Windows / Linux |
|---|---|---|
| Start / pause | F10 | F11 |
| Recenter | F12 | F12 |

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
[`docs/ROADMAP.md`](docs/ROADMAP.md), [`docs/WEBSITE.md`](docs/WEBSITE.md) (the website), decisions in
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
