# 0012 — Phase 7: multiplatform / release

Status: ACTIVE (started 2026-11-19)
Owner/branch: `rewrite/rust`
Depends on: 0011 (done)

## Context

Phase 7 per `docs/ROADMAP.md`: validate macOS (Apple Silicon, permissions), Windows (cursor
injection, capture), Linux (X11/Wayland limits); packaging, signing, licenses, CI, reproducible
builds, release docs. Build under test starts at `687a9fc` (CI 43797400148 green on 3 OS).

Inputs: `docs/architecture/PLATFORM_RISKS.md` (P1–P20, Phase-7 rows P1, P3, P8–P12, P18, P19),
ADR-0012 (adapters), `docs/qa/MANUAL_QA.md`, `docs/qa/TEST_DRIVE.md`, `deny.toml`,
`.github/workflows/ci.yml`. Skills: `platform-integration`, `rust-architecture`,
`code-review`.

Carried in: MANUAL_QA rows 19–20 (`.app` permissions); TEST_DRIVE #7 CI flake (macOS exit 101
run 43773727473, Windows exit 1 runs 43779616732 and 43797804357 — the last on a docs-only
commit); Windows/X11 pointer + primary-monitor detection on device (#14); external webcams
(P1); Wayland limits (P10–P12); from 6b/6c: minimised rate and dwell on device, `ui` 15/17 ms
with Options open.

## User decisions (2026-11-19)

- **macOS:** Apple Silicon (arm64) only. Ad-hoc signed `.app` in a `.dmg` for v1; Gatekeeper
  steps documented. Developer ID + notarization plumbed into the release workflow behind
  secrets, off until an Apple Developer account exists.
- **Windows:** unsigned for v1, SmartScreen steps documented; a signing hook left in the
  workflow. Ships **an installer** (format chosen in the packaging ADR, slice 2).
- **Linux:** minimal — `.tar.gz` + AppImage, X11. Wayland: pointer/hotkeys documented as
  unsupported (P10–P12).
- **Channel:** GitHub Releases on this repo.
- **App icon (all OS):** the user's yellow camera-outline with a concentric "C" around "M"
  (black line art on `#FFD100`-ish yellow), source `~/Downloads/Black Camera Outline with
  Concentric C and M.png`, 1254×1254 raster (AI-generated, slightly noisy). Slice 2 copies it
  into the repo and sizes it properly per OS: macOS `.icns` on Apple's rounded-rect grid
  (art inset, 16–1024 px), Windows `.ico` 16–256, Linux PNG 256/512 + `.desktop`, eframe window
  icon. Thin strokes vanish at 16–32 px: likely a redrawn SVG with heavier strokes for small
  sizes — show the user the small sizes before adopting.

## Slices (one per session, planned one at a time)

1. **CI flake #7 diagnosable.** Failing test names and the cargo error surface as GitHub
   annotations (readable via the unauthenticated API), `--no-fail-fast`; then fix the flaky
   test(s) deterministically once named.
2. **Packaging ADR.** Tool (e.g. `cargo-packager` vs hand scripts), Windows installer format
   (NSIS / WiX MSI / Inno), app id, version, icon, licences of the tooling, third-party notices
   generation (`cargo-about` or similar, via `deny.toml` policy).
3. **macOS `.app`.** `Info.plist` (`NSCameraUsageDescription`, bundle id), ad-hoc signing, dmg;
   MANUAL_QA rows 19–20 on the bundle; permissions survive a rebuild or the reset is documented.
4. **Release workflow.** Tag → build macOS arm64 dmg, Windows installer, Linux tar.gz +
   AppImage → draft GitHub Release with checksums; notarization / Windows signing steps
   present but skipped without secrets.
5. **Windows on device.** Pointer (`SendInput`, UIPI P8, mixed DPI P9), primary monitor (#14),
   camera (P3 FPS / hang), F11/F12 hotkeys, installer install/uninstall. Needs a Windows
   machine — ask the user.
6. **Linux X11 on device + Wayland limits.** Pointer (XTest), RandR primary (#14), hotkeys,
   camera (V4L2), AppImage run; Wayland behaviour recorded as known exceptions.
7. **External webcams (P1).** At least one external USB webcam per available OS; panics caught.
8. **Release docs + criteria.** `docs/release/` install / permissions / Gatekeeper /
   SmartScreen / Wayland notes, reproducible-build notes, per-platform release criteria with
   known exceptions explicit (the gate).

## Non-goals

- Phase 8 work: reduced motion (MANUAL_QA row 21), `OCM_RECORD` threshold re-measurement,
  ADR-0018 slow-creep limit, CPU/latency budgets.
- Intel macOS builds; Wayland pointer injection (libei/portal); paid signing in v1.
- Behaviour changes, unless a platform bug forces one (then a TEST_DRIVE row + test).

## Acceptance criteria (gate)

- Written per-platform release criteria (macOS arm64, Windows, Linux X11) with known
  exceptions explicit (Wayland, unsigned/ad-hoc, UIPI, …).
- Release workflow produces all three platforms' artifacts from a tag, with checksums.
- MANUAL_QA rows 19–20 pass on the `.app`; Windows and Linux on-device checks recorded (or
  explicitly deferred with a reason if no device is available).
- #7 fixed or root-caused; CI green on 3 OS + cargo-deny.
- Third-party licence notices shipped in every package.
- The user says ready for Phase 8.

## Verification

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (scoped locally, one heavy command at a time),
`scripts/check-core-deps.sh`, CI run on the final commit, release workflow dry run.

## Progress

- 2026-11-19: task written; user decisions recorded above. Slice 1 started: CI test step
  surfaces failures as annotations.
- 2026-11-19: slice 1 step verified on a green run (`2ae2a5c`, CI 44604964205, 3 OS green,
  summary step skipped). #7 not reproduced; fix waits for the next failure's annotation.
