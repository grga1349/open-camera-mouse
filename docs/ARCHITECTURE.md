# Architecture

## Workspace (ADR-0007)

| Crate | Role | May depend on |
|---|---|---|
| `crates/tracking-core` | Pure tracker: `GrayFrame`, pyramids, LK (+ anchored appearance gate, re-detection), NCC comparator. No I/O, no clock (R-T1) | `thiserror` only |
| `crates/pointer-core` | Pure pointer maths: absolute mapping, 1€ smoothing, steady zone, limits + offset drift, handover, dwell, metrics | `thiserror` only |
| `crates/app-core` | Controller state machine (R-A1), validated `Settings`, ports (`CameraSource`, `PointerSink`, clock) | the two cores, `thiserror`, `serde` |
| `crates/platform` | OS adapters: camera (`nokhwa`), pointer (CGEvent / SendInput / XTest), hotkeys, settings file | `app-core` + OS crates |
| `apps/desktop` | The egui/eframe app (binary `open-camera-mouse`): engine thread, UI, packaging | everything |

Dependencies point down only; `scripts/check-core-deps.sh` enforces the core crates'
allowlists (CI). A new crate or cross-layer dependency needs an ADR.

`website/` (the project website, ADR-0023) is a separate Bun/Astro package, not a Cargo
member; nothing in the workspace depends on it. See `docs/WEBSITE.md`.

## Runtime (ADR-0016)

```
camera worker ──newest-wins slot──▶ engine thread ──Arc<Snapshot>──▶ UI (egui)
 (ThreadedCamera)                   Controller + pointer              Shell: sends EngineCmd,
                                    + recorder + perf                 paints, saves settings
```

- **Camera worker** (`apps/desktop/src/engine.rs`): opens the device, pushes frames into a
  one-frame slot; older unread frames are replaced, never queued (`replaced` counter).
- **Engine thread**: wakes on a frame, a command or every 100 ms; drives
  `Controller::drive` on capture timestamps (clamped monotonic), applies `Effect`s (pointer
  moves, clicks), builds the preview image off the UI thread, publishes a `Snapshot`.
- **UI** (`app.rs`, `status.rs`, `preview.rs`, `options.rs`, `widgets.rs`, `theme.rs`,
  `icons.rs`): immediate mode over `dyn EngineHandle`; tests use `InlineEngine`.
- Frames are mirrored at ingest and capped at 640 px wide (ADR-0008); luma goes to the
  tracker, RGB to the preview only.
- Global hotkeys are registered in the eframe creator closure (main thread); `KeyRouter`
  de-duplicates global and in-window presses.

## Tracking and pointer

- Tracker: `LkParams::recommended()` = 2018 LK preset + eigenvalue floor 0.25 / ratio 0.05 +
  anchored appearance gate (≥ 0.7 against the window accepted 10 steps earlier, selection
  floor 0.3, ADR-0018). Re-detection is opt-in ("Find the target again", ADR-0010/-0011).
- Pointer: camera-space 1€ filter → steady zone of `STEADY_ZONE_CAM_PX` = 0.2 camera px,
  scaled through gain and the Main monitor (ADR-0015, ADR-0017) → absolute map → limits with
  offset drift capped at the head's overshoot → snap to the nearest display. Handover and
  dwell per R-A2 / R-K1.
- Defaults that change behaviour need the user's OK and an amendment to the owning ADR
  (ADR-0006, -0014, -0015, -0017).

## Platform adapters and known risks

Numbers are cited from code comments (`P12`, …). ✓ = retired.

| # | OS | Risk → handling |
|---|---|---|
| P1 | macOS | `nokhwa` can panic on some webcams' formats → every call under `catch_unwind` → `CameraError::Backend`. External webcams untested on device |
| P2 | macOS | Camera permission needs `NSCameraUsageDescription` (in the `.app`); `cargo run` inherits the terminal's permission → `PermissionDenied` state ✓ |
| P3 | Windows | Reported FPS can be wrong; some devices hang on open → FPS measured from timestamps, 10 s open timeout |
| P4 | all | Device ignores 640×480@30 → use the negotiated format, downsample ✓ |
| P5 | all | Disconnect not signalled → 3 s frame timeout → `CameraDisconnected` ✓ |
| P6 | macOS | Posted CGEvents need Accessibility; warping blocks the physical mouse (measured) → posted events, access = `AXIsProcessTrusted ‖ CGPreflightPostEventAccess`, polled every 3 s (≈10 ms per call) ✓ |
| P7 | macOS | Clicks need event posting → no sandbox / App Store build; dwell off by default ✓ |
| P8 | Windows | `SendInput` can't reach elevated windows or the secure desktop (UIPI) → documented |
| P9 | Windows | Mixed-DPI multi-monitor coordinates → per-monitor DPI awareness; untested on device |
| P10 | Wayland | No global pointer injection → `Unsupported`, X11 only for v1 |
| P11 | Wayland | Global cursor position unreadable → no handover detection; `position()` = `None` |
| P12 | Wayland | `global-hotkey` is X11-only → `Unsupported`, in-window keys only |
| P13 | macOS | F11 is Show Desktop, F-keys may need `Fn` → start/pause is F10 on macOS ✓ |
| P14 | Linux | eframe needs system libs; CI is headless → `egui_kittest` without GPU ✓ |
| P15 | all | Screen-reader (AccessKit) behaviour differs per OS → open |
| P16 | Windows | Symlinks / POSIX scripts → none required to build (ADR-0002) ✓ |
| P17 | all | Toolchain drift → `rust-toolchain.toml` + `--locked` ✓ |
| P18 | macOS | Ad-hoc signature: Gatekeeper warns; grants survive updates via an identifier-only requirement (ADR-0021). Notarization hook off until a Developer ID exists |
| P19 | Windows | Unsigned → SmartScreen warning; signing hook off until a certificate exists |
| P20 | all | A dependency brings a non-permissive licence or OpenCV → `cargo deny` in CI ✓ |

Adapter specifics: macOS pointer = `objc2-core-graphics` posted events (source suppression 0);
Windows = `windows-sys` `SetCursorPos` / `SendInput`, primary = monitor at (0, 0); Linux =
`x11rb` XTest + RandR primary. Windows and Linux paths are compiled and unit-tested in CI but
**not yet run on a device**.

## Dependencies

Policy in `deny.toml` (licence allowlist incl. OFL-1.1 / Ubuntu-font-1.0 for egui's fonts,
OpenCV banned, crates.io only), checked by CI. Main crates: `eframe`/`egui` 0.37 (UI),
`egui_kittest` (UI tests), `nokhwa` 0.10 `input-native` (camera; its `input-opencv` must stay
off), `objc2-core-graphics`, `windows-sys`, `x11rb` (pointer), `global-hotkey` 0.10, `serde` +
`toml` (settings), `thiserror`, `divan` (benches). Rejected: `directories` (MPL-2.0
dependency; config path hand-written), `enigo`, anything OpenCV.

## Packaging (ADR-0021)

`cargo-packager` 0.11.8 from `apps/desktop` (`[package.metadata.packager]`): macOS arm64 `.app`
+ `.dmg` (hardened runtime, camera entitlement, `scripts/macos-adhoc-sign.sh`), Windows NSIS
per-user installer, Linux AppImage (+ tar.gz by the workflow). App id
`io.github.grga1349.OpenCameraMouse`. Third-party notices by `cargo-about` 0.9.3 (`about.toml`,
`apps/desktop/packaging/about.hbs`) shipped as `THIRD_PARTY_LICENSES.html` in every package.
Icons are generated by `scripts/render-icon.py` (Pillow, dev only) into `assets/icon/`.
`.github/workflows/release.yml` builds all three from a `v*` tag into a draft GitHub Release
with `SHA256SUMS`; signing steps run only when their secrets exist.
