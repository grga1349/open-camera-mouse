# Proposed architecture and testing direction

> Hypothesis requiring research and ADRs—not an already-approved implementation.

## Rust boundaries

- **`tracking-core`**: pure algorithmic component. Input: pixel buffers/gray frames, dimensions/stride, timestamp and target state. Output: subpixel position, status/quality and diagnostics. No camera/OS/GUI/OpenCV.
- **`pointer-core`**: image-to-screen transform, sensitivity, calibration, smoothing, limits, dwell-click state machine. Pure and testable.
- **`platform`**: camera capture, permissions, pointer injection, platform detection and persistence adapters; OS-specific implementation modules.
- **`apps/desktop`**: egui/eframe composition, app state controller, preview and UI. Keep UI thin, scheduling/frame ownership explicit.

Start with 3 crates plus one desktop app. Split only on demonstrated dependency boundaries. Favor standard Rust libraries, typed errors and small interfaces over unnecessary trait forests.

## Pipeline

Camera → format conversion → grayscale frame → tracker update → target state and validity → pointer mapping/smoothing → platform cursor command; UI observes status and displays preview/settings without blocking tracking. Define ownership and bounded queues, drop stale frames rather than accumulate latency where appropriate. Avoid sending raw webcam frames to remote AI services.

## Tracking experiment sequence

1. Read/reconstruct original reported PyrLK parameters and existing implementation's template matcher.
2. Create reference fixtures/test runner and metrics before optimizing.
3. Implement own baseline Gaussian/image pyramid, gradients/interpolation and iterative LK estimate in Rust.
4. Compare against known mathematical fixtures, recorded scenes and legacy baseline.
5. Test forward-backward check, outlier threshold and confidence/residual gates separately.
6. Investigate lost-point recovery and adaptation without silently drifting target semantics.
7. Explore SIMD and buffer reuse only after profiling; document CPU budgets.

## Performance traps

Resolution-dependent CPU cost, allocations per frame, high-frequency UI paint, camera frame stall, pointer injection frequency, pointer-worker sleeps, unreliable camera timestamps, input permissions, multimonitor coordinate transforms, Wayland limitations, retina/high-DPI scaling, differing camera backends.

## Technology research questions

- egui/eframe supported versions and rendering backend choices.
- Native webcam library options (e.g., `nokhwa`) vs direct AVFoundation/Media Foundation/V4L2; verify maintenance, licenses and binaries.
- Cursor movement alternatives per platform and permissions.
- Asset storage/fixture privacy/licensing.
- CI strategy for headless algorithm tests and optional GUI/integration tests.
- Distribution and codesigning/notarization constraints.

## Testing layers

**Math unit tests:** known gradients, pyramids, interpolation and 2D displacement. **Property tests:** bounds/non-NaN and invariants. **Recorded fixtures:** frame pairs/sequences with known/annotated motion. **Integration:** fake capture and pointer adapters. **Platform smoke:** permissions, webcam and cursor. **Benchmark:** median/tail latency, memory/allocations, error, drift, lost/reacquisition. **Accessibility:** keyboard-only usage, contrast, large controls and feedback.

## Architecture decision records

Document important changes (camera backend, graphics layer, tracker family, queue strategy, coords, target loss policy) with context, options, tradeoffs, evidence and consequences; do not create an ADR for trivia.
