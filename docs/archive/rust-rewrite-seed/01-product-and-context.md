# Product context and non-negotiable goals (seed)

## Product

Open Camera Mouse: a desktop accessibility/utility program allowing a user to select and track a visually distinctive point in a webcam image and translate that motion into pointer movement, with appropriate settings and optional dwell-click behavior. The existing Open Camera Mouse implementation is reference material; it should not be thrown away conceptually when rewriting in Rust.

## Motivation

The original third-party Camera Mouse 2018 reportedly provides strong real-world tracking, and a reverse-engineering investigation identified how its tracking pipeline works. Use those findings to build a lightweight, maintainable, cross-platform Rust application, minimizing large native dependencies—**especially no OpenCV dependency in the shipped Rust engine**. AI tools (Codex and Claude Code) should act as coordinated engineering assistants, not substitute for verification.

## Product principles

- Accurate, stable, low-latency head/feature tracking on ordinary webcams.
- Works when internet/AI services are unavailable: on-device CV, not cloud inference.
- Low overhead, predictable native packaging, simple setup and permissions.
- Accessibility-first UI; usable by people who may not easily operate a traditional mouse.
- Local-first and privacy-conscious; avoid recording or uploading camera frames by default.
- Maintainability and reproducible tests trump clever but unproven algorithms.
- Cross-platform target: macOS, Windows, Linux, with platform parity treated as a measured, staged goal—not an assumption.

## Scope: v1

Camera feed and preview; point selection; grayscale frame pipeline; robust point tracking; cursor mapping; sensitivity/smoothing settings; pause/resume; clear loss handling; configurable dwell click if feasible; user-visible permission guidance; deterministic fixtures and benchmarks; settings persistence.

## Non-goals at kickoff

No cloud accounts, SaaS, face recognition, generic AI inference, unsupported claims of medical/accessibility certification, or wholesale OpenCV adoption. Do not implement all exploratory features before establishing an accurate baseline.

## Success criteria (to refine with measurements)

- Deterministic tracking tests for known synthetic motion and recorded fixtures.
- Measured error, failure/loss/recovery, jitter and latency vs baseline.
- Clear separation between capture, tracking, pointer mapping and UI.
- Builds/tests on supported platforms in CI or documented manual validation.
- UI can be navigated with non-pointer alternatives where possible.
- Install/bundle free of OpenCV native dynamic-library chaos.

## Inputs

- Current repository: existing Open Camera Mouse implementation, including documentation and old agent files.
- Sibling research: `../camera-mouse-reverse/`, especially `research/TRACKING_SPEC.md` and individual evidence files.
- Historical Windows application: Camera Mouse 2018; reverse engineering was of the *installed application*, not merely Inno Setup installer.

## Evidence policy

External reverse-engineering findings are reported by a previous Codex run. Treat them as **candidate evidence** until checked in actual research files; avoid treating observations as performance proof. No extracted proprietary code is to be pasted wholesale into the new product.
