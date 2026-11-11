# Platform risk matrix

Phase 2 deliverable (task 0004). Likelihood and impact are rated H/M/L. "Evidence" says how
we know: **VERIFIED** (tested here), **SOURCED** (vendor docs or an issue tracker, linked
in DEPENDENCIES.md), **INFERRED** (reasoning, untested). Owner phase = where it gets
retired.

| # | Area | OS | Risk | L | I | Evidence | Mitigation | Owner |
|---|---|---|---|---|---|---|---|---|
| P1 | Camera | macOS | `nokhwa` panics or aborts selecting a format/FPS on some webcams (C922 and others) | H | H | SOURCED (nokhwa #247, #180, #202) | Spike in Phase 6: nokhwa vs a thin `objc2-av-foundation` adapter. Catch panics at the adapter boundary and report `CameraError::Backend` | 6 |
| P2 | Camera | macOS | Camera permission: needs `NSCameraUsageDescription` in an app bundle. A bare `cargo run` binary inherits the terminal's permission | H | M | INFERRED | Map denial to `PermissionDenied` state (R-A1). Package as `.app` for QA | 6/7 |
| P3 | Camera | Windows | Reported FPS wrong (nokhwa #110, #144). Some capture devices hang on init (#101) | M | M | SOURCED | Measure FPS from timestamps, never trust the reported value. Open with a timeout | 6/7 |
| P4 | Camera | all | The device ignores the 640×480@30 request (ADR-0008) | M | L | INFERRED | Use the negotiated `CaptureFormat`, downsample to ≤ 640 wide | 3/6 |
| P5 | Camera | all | Device disconnect not signalled | M | M | INFERRED | Detect from frame timeout plus backend error, state `CameraDisconnected` | 6 |
| P6 | Pointer | macOS | `CGEventPost` needs Accessibility permission. Whether `CGWarpMouseCursorPosition` does is unknown. A warp also briefly suppresses local mouse events | H | H | SOURCED (Apple forums) / unknown | Phase 6 test on the dev Mac. Show `PermissionDenied` with a deep link to System Settings. Avoid suppression artefacts in handover detection | 6 |
| P7 | Pointer | macOS | Clicks (dwell) need event posting, which needs Accessibility. Sandboxed / App Store builds can't | H | M | SOURCED | Ship notarized, non-sandboxed. Dwell is off by default (ADR-0006) | 6/7 |
| P8 | Pointer | Windows | `SendInput` can't drive elevated (UIPI) windows or the secure desktop (UAC, lock screen) | H | M | INFERRED (well-known Windows behaviour) | Document. Detect the failed injection (`SendInput` returns 0) and surface it | 7 |
| P9 | Pointer | Windows | Mixed-DPI multi-monitor: absolute `SendInput` coordinates normalise to the virtual desk | M | M | INFERRED | Declare per-monitor DPI awareness. Tests with negative origins (pointer-core has one) | 7 |
| P10 | Pointer | Linux/Wayland | No general global absolute pointer injection. RemoteDesktop portal + libei needs a consent dialog per session, and support differs by compositor | H | H | SOURCED (libei docs, wayland-devel) | X11 via XTest is the supported path for v1. Wayland via portal is best-effort, behind a clear `Unsupported` message | 7 |
| P11 | Pointer | Linux/Wayland | Can't read the global cursor position → physical-mouse handover (ADR-0006) can't detect moves | H | M | INFERRED | `PointerSink::position()` returns `None`. Handover falls back to hotkey/pause only, and the UI says so | 6/7 |
| P12 | Hotkeys | Linux/Wayland | `global-hotkey` is X11 only | H | M | SOURCED (crate docs) | Portal GlobalShortcuts, otherwise in-window keys | 7 |
| P13 | Hotkeys | macOS | F11 is bound to "Show Desktop" by default, and F-keys may need Fn | M | M | INFERRED | Document. Detect registration failure and show it | 6 |
| P14 | UI | Linux | eframe build/run needs system libs (xkbcommon, wayland, GL). Headless CI has no display | M | L | VERIFIED in CI config (apt list) | `egui_kittest` without wgpu runs headless. No GPU tests in CI | 2 ✓ |
| P15 | UI | all | AccessKit/screen-reader behaviour differs per OS | M | M | INFERRED | Phase 8 accessibility testing | 8 |
| P16 | Build | Windows | Symlinks / POSIX scripts in a checkout | L | M | INFERRED | No symlinks (ADR-0002). The only script, `check-core-deps.sh`, runs on the Linux job | 2 ✓ |
| P17 | Build | all | Toolchain drift between dev and CI | L | M | VERIFIED | `rust-toolchain.toml` pins 1.99.1. `--locked` in CI | 2 ✓ |
| P18 | Release | macOS | Notarization, hardened runtime and camera entitlement | H | H | INFERRED | Phase 7 packaging task | 7 |
| P19 | Release | Windows | Unsigned binaries trigger SmartScreen | H | M | INFERRED | Code-signing decision in Phase 7 | 7 |
| P20 | Deps | all | A transitive dependency adds a non-permissive license or OpenCV | L | H | VERIFIED policy (`deny.toml`) | `cargo deny check` in CI | 2 ✓ |

## Cross-platform claims this phase actually checked

- The workspace builds and passes clippy/tests on macOS arm64 locally (task 0004 log).
- `cargo check --workspace --all-targets` succeeds for `x86_64-pc-windows-msvc` and
  `x86_64-unknown-linux-gnu` from macOS. That catches `cfg` mistakes, but it doesn't link
  or run anything.
- Native builds and test runs on Windows and Linux: only the CI run proves these.
