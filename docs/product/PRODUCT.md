# Product context and goals

## What this is

Open Camera Mouse is a desktop accessibility/utility application: a user selects a visually
distinctive point in their webcam image, the app tracks it frame-to-frame, and that motion
drives the OS pointer, with sensitivity/smoothing settings and an optional dwell-click. The
v0.3.0 Go/Wails implementation (now `legacy/`) already ships this end-to-end on macOS and
Windows. This is a from-scratch Rust rewrite of the same product, not a new product.

## Motivation for the rewrite

The original third-party **Camera Mouse 2018** (Windows, .NET 4.0 + bundled OpenCV 2.1) has a
reported reputation for strong real-world tracking; a sibling reverse-engineering project
(`../camera-mouse-reverse/`, archived synthesis in `docs/research/RE_EVIDENCE.md`) recovered
its tracking pipeline to use as comparison evidence. Separately, GoCV/OpenCV as a shipped
native dependency has been a source of packaging and build friction for the current Go app.
The rewrite's goal is a maintainable, cross-platform Rust app with **no OpenCV dependency**,
using the 2018 app's recovered behavior and the current Go app's own tracker as two different
reference points — not a mandate to copy either one.

## Product principles

- Accurate, stable, low-latency point tracking on ordinary webcams.
- Fully on-device; no cloud inference, no uploaded camera frames.
- Low overhead, predictable native packaging, simple setup and permissions.
- Accessibility-first UI, usable by people who may not easily operate a conventional mouse.
- Local-first and privacy-conscious by default.
- Maintainability and reproducible tests over clever, unproven algorithms.
- Cross-platform: macOS (primary dev target), Windows and Linux required for release — treated
  as a staged, measured goal per platform, not an assumption that one implementation covers all.

## Scope: v1

Camera feed and preview; point selection; grayscale frame pipeline; robust point tracking;
cursor mapping; sensitivity/smoothing settings; pause/resume; clear tracking-loss handling;
configurable dwell click; user-visible permission guidance; deterministic fixtures and
benchmarks; settings persistence. This matches the current `legacy/` app's feature set
(see `legacy/docs/ARCHITECTURE.md` and `legacy/docs/ALGORITHM.md` for exactly what it does
today) plus the two-phase guided recenter flow and global hotkeys (F11 start/pause, F12
recenter) described there.

## Non-goals at kickoff

Cloud accounts, SaaS, face recognition, generic AI inference, medical/accessibility
certification claims, wholesale OpenCV adoption, a standalone `tracking-cli`. Don't build
every exploratory tracking improvement before an accurate, measured baseline exists.

## Success criteria (to refine with measurements once fixtures exist)

- Deterministic tracking tests for known synthetic motion and recorded fixtures.
- Measured error, loss/recovery behavior, jitter and latency versus the `legacy/` baseline.
- Clear separation between capture, tracking, pointer mapping and UI (see
  `docs/architecture/decisions/`).
- CI-verified builds/tests on supported platforms, or an explicitly documented manual
  validation gap.
- UI navigable with non-pointer input alternatives where feasible.
- Install/bundle free of OpenCV/GoCV native-library packaging issues.

## Inputs

- `legacy/` — the current, working Go/Wails/GoCV implementation and its docs.
- `../camera-mouse-reverse/` — Camera Mouse 2018 reverse-engineering evidence, especially
  `research/TRACKING_SPEC.md`. Synthesized and re-labeled in `docs/research/RE_EVIDENCE.md`.
- `docs/archive/rust-rewrite-seed/` — the original planning conversation's seed documents.

## Evidence policy

RE findings from `../camera-mouse-reverse/` are candidate evidence from a separate prior
investigation; treat observations about the 2018 binary's *implementation* as distinct from
claims about its *tracking quality*. No decompiled/proprietary code is pasted into this
repository's shipped source, and the original binary is not redistributed.
