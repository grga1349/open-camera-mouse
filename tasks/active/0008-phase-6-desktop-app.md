# 0008 — Phase 6: desktop app

Status: ACTIVE (2026-11-12)
Owner/branch: `rewrite/rust`
Depends on: 0006, 0007 (done)

## Context

Phase 6 per `docs/ROADMAP.md`: egui/eframe UI wired to camera adapters, tracker and pointer
mapping. Covers target selection, settings, calibration/recenter, lifecycle and errors,
cursor-permission paths, persistence, and handover and dwell per ADR-0006.

Inputs: BEHAVIOR_SPEC §7 (R-C*, R-P*, R-K*, R-A*), ADR-0006 (2018 defaults table), ADR-0007
(crate boundaries), ADR-0008 (mirror at ingest, 640×480@30), ADR-0011 (`LkParams::recommended()`,
`redetect_armed()`), `docs/design/design-lab.html` v3 + `docs/design/EGUI_FEASIBILITY.md`
(§"Phase 6 acceptance criteria", §"Inputs for Phase 6"), `docs/architecture/PLATFORM_RISKS.md`
(P1, P2, P5, P6, P7, P11, P13 are owned by this phase), `docs/architecture/DEPENDENCIES.md`
(candidates). Skills: `rust-architecture`, `platform-integration`, `egui-development`,
`rust-testing`, `code-review`.

Starting point (verified 2026-11-12): only `tracking-core` is real. `app-core` has the R-A1 enum,
two commands and a toy `Controller`. `pointer-core` has `map_absolute` only. `platform` has no OS
crates. `apps/desktop` is one window with a status label and a button.

## Objective (slices, in this order; each ends green on the quality gates)

- **D0 — `app-core` controller (pure, no I/O).** The full R-A1 state machine driven by commands
  and inputs with an injected clock (`now_us` passed in, no `Instant::now()` inside):
  `ToggleStart` (F11), `Recenter` (F12), `SelectTarget(Point2)` (preview click, display space),
  `Frame(CameraFrame)`, `Tick(now_us)`, `PointerObserved(Option<Point2>)`, camera/pointer
  errors. Adds the states the approved UI needs and R-A1 lacks: pointer permission needed and
  mouse handover (variant or sub-state, decided here and written into BEHAVIOR_SPEC R-A1).
  Recenter: 5 s deadline, then select the frame centre (R-A3). Resume from pause goes through
  recenter. Recenter ends a pause. Owns the tracker (`LkTracker` with `LkParams::recommended()`)
  and mirroring at ingest (ADR-0008). Settings struct with validation on every update and on
  load, schema version (R-A5), including `loss_recovery: bool` (default false → `redetect = None`). A param change that invalidates the tracker re-initialises it
  (R-A6). Output per step: state, events, optional pointer target, overlay data for the UI
  (marker position/status, countdown remaining, diagnostics).
- **D1 — `pointer-core` behaviour.** dt-aware EMA smoothing equivalent to α 0.95 per 70 ms
  (R-P2, ADR-0006), W/E/N/S screen limits with the E/S origin fix, offset drift as a rate,
  re-acquisition jump bound (R-P4, tested), handover detector (5 px move yields control, 4 s
  inactivity resumes, cursor to desktop centre on takeover, R-A2), dwell (off by default,
  1 s, 2 s refractory, radius 0.05 of screen, fires once, never while lost or not in control,
  R-K1/R-K2). All pure, all unit-tested. `app-core` composes them.
- **D2 — Dependencies ADR + persistence.** One ADR (0012) for the Phase 6 crates after
  re-querying versions/licences: camera backend (from D3's spike), cursor crates, `global-hotkey`,
  `serde` + `toml` + `directories`. The ADR also states which crates `app-core` may take
  (e.g. `serde` derive for settings) and keeps `scripts/check-core-deps.sh` in step. Settings
  file read/write lives outside the pure cores. A corrupt or newer-schema file falls back to
  defaults with a visible notice, never a crash.
- **D3 — Camera adapter (macOS first).** Spike `nokhwa` (`input-native` only) vs a thin
  `objc2-av-foundation` adapter against P1. Pick by measurement on the dev Mac (opens, 640×480
  negotiated, real FPS from timestamps, release on close). Panics caught at the adapter boundary
  → `CameraError::Backend`. Capture thread with a bounded channel (latest frame wins). Disconnect
  = backend error or frame timeout → `CameraDisconnected` (P5). Permission denial →
  `PermissionDenied` (P2). Windows and Linux: same crate if the spike picks `nokhwa`, compiled
  in CI; on-device validation is Phase 7.
- **D4 — Pointer adapter (macOS first).** Decide warp (`CGWarpMouseCursorPosition`) vs posted
  move events on the dev Mac, including whether each needs Accessibility (P6) and the handover
  side effects of warp suppression. Accessibility check + deep link to System Settings → pointer
  permission state. `position()` for handover. Virtual-desktop rect (R-P5, multi-monitor
  origin). Dwell click via posted events (P7). Windows (`SendInput`) and Linux X11 (XTest)
  implemented and compiled in CI, validated in Phase 7. Wayland returns `Unsupported` (P10/P11).
- **D5 — Global hotkeys.** F11/F12 through `global-hotkey` in `platform`, mapped to
  `ToggleStart`/`Recenter`. Registration failure is shown in the UI (P13: macOS F11 = Show
  Desktop). In-window F11/F12 always work as a fallback.
- **D6 — Desktop UI per the approved lab.** Tracking window (560×620, fixed) and Options window
  (400×620, fixed, `show_viewport_immediate`). Carbon/Paper visuals from the token table. Preview
  texture updated per frame. Click-to-select through the same rect as drawing (R-C2). Marker
  variants (ring, dashed held, searching, lost grey + X). Recenter circle + badge. Status line
  (icon + text) per state. Options tabs: Pointer, Clicking, Camera, General. Painted icons, no SVG
  crate. Text scale, larger controls. The desktop crate only renders `app-core` output and sends
  commands.
- **D7 — Runtime integration and QA.** Runtime loop (camera thread → controller → pointer sink,
  repaint on new frame). End-to-end integration test with a fake camera (scripted synthetic
  motion) and a fake pointer sink: select → track → cursor moves right for head-right → lose →
  pause → recenter → resume. A frame recorder (dev/diagnostics only) so the Phase 4 thresholds
  can be re-measured on real recordings (open risk). `docs/qa/MANUAL_QA.md`: macOS manual
  checklist (permissions, start, select, track, recenter, pause, handover, dwell, disconnect,
  theme persistence). A macOS `.app` bundle path for permission QA if a bare `cargo run` can't
  exercise P2 honestly.

## Decisions owed in this phase (record each where noted)

| Question | Default if the user doesn't decide | Where |
|---|---|---|
| Preview grayscale (luma) vs colour | Grayscale. Ports already carry luma, so this is the cheap path, and the green ring was chosen to work on it | this file + EGUI_FEASIBILITY |
| Font | egui default fonts for the first build. Revisit with an embedded OFL font later | this file |
| ~~Pointer permission + handover in R-A1~~ **Decided (D0):** orthogonal `PointerOwner` sub-state; plus top-level `Error` | — | BEHAVIOR_SPEC R-A1 |
| ~~Re-acquisition → control handover~~ **Decided (user, 2026-11-12):** re-detection is an option, Options → Camera → "Find the target again" (`loss_recovery`), **off by default**, now in the design lab (ADR-0011 amended). Off: `Lost` waits for re-selection or recenter (2018). On: `Lost(Searching)` → `Tracking` gives control back only within R-P4's jump bound | — | ADR-0011 amendment; D0 setting; BEHAVIOR_SPEC R-A1 note in D0 |
| Camera backend | From the D3 spike | ADR-0012 |
| Warp vs posted move | From the D4 test | ADR-0012 / PLATFORM_RISKS P6 |

## Non-goals

Packaging, signing and notarization (Phase 7). On-device Windows/Linux validation (Phase 7).
Wayland pointer injection beyond `Unsupported` (Phase 7). Auto-recenter (ADR-0006 amended).
Configurable hotkeys. Relative pointer mode. Any change to `tracking-core` behaviour beyond what
integration strictly needs (record the need first). No `tracking-cli`. No OpenCV.

## Files/scope

`crates/app-core`, `crates/pointer-core`, `crates/platform`, `apps/desktop`, workspace
`Cargo.toml`, `deny.toml` if needed, `scripts/check-core-deps.sh`, new ADR(s),
`docs/research/BEHAVIOR_SPEC.md` R-A1 note, `docs/architecture/PLATFORM_RISKS.md`,
`docs/architecture/DEPENDENCIES.md`, `docs/qa/MANUAL_QA.md`, this file, `tasks/BACKLOG.md`,
`agent/`.

## Acceptance criteria (ROADMAP Phase 6 gate + EGUI_FEASIBILITY)

1. End-to-end on the dev Mac: start → camera preview → click select → cursor follows head
   (head right → cursor right) → F12 recenter (5 s) → F11 pause/resume through recenter →
   physical mouse takes over and hands back after 4 s → dwell click when enabled → camera unplug
   shows `CameraDisconnected`. Recorded in `MANUAL_QA.md` with date and result per item.
2. Integration tests with fake camera + fake pointer cover the same flows headless, in CI on
   all three OS.
3. Every R-A1 state (plus the new sub-states) is reachable in a test and renders as icon + text.
   One kittest assertion per state on the status text.
4. Every interactive control is reachable by Tab, has a visible focus ring and an AccessKit
   label. One kittest test per control, driven by its label.
5. Both windows are fixed size, not resizable, and contents fit or scroll at zoom 1.0–1.3 and
   with larger controls.
6. Preview click → frame coordinate uses the drawing rect (R-C2), tested with letterboxing.
7. Countdowns (recenter, handover) come from `app-core` deadlines, not UI timers. Tested with
   the injected clock.
8. Settings (incl. theme) persist, are validated on load and update, and carry a schema
   version (R-A5). Corrupt-file and newer-schema tests.
9. Only hues: green marker, yellow primary buttons (one helper each).
10. Pointer: dt-aware smoothing FPS-independence test (R-P2), re-acquisition jump bound test
    (R-P4), dwell fires-once and no-fire-when-lost tests (R-K1/R-K2), negative-origin desktop
    test (R-P5).
11. Quality gates green locally and in CI on macOS, Windows, Linux + `cargo deny`.

## Verification

```
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/check-core-deps.sh
cargo run -p open-camera-mouse   # manual QA per docs/qa/MANUAL_QA.md
```
Plus the CI run ID for the closing commit.

## Progress

- 2026-11-12: task written from ROADMAP Phase 6 and EGUI_FEASIBILITY. Next: D0.
- 2026-11-12: user decision: re-detection = option, off by default. ADR-0011 amended (it was on
  in `recommended()`, which stays unchanged as the pinned candidate). General direction: improve
  on 2018, take the best of 2018 and legacy.
- 2026-11-12: design lab, DESIGN_PLAN, EGUI_FEASIBILITY, BEHAVIOR_SPEC R-P4, ADR-0006/-0010
  updated for the option. Lab: switch on the Camera tab (default OFF), synced with the
  Searching/Lost states, diagnostics show `redetect off`. `node --check` OK, 0 external URLs,
  headless Chrome DOM check (lost → OFF, searching → ON) and screenshot reviewed.
- 2026-11-12: lab fixes (user): "Keep both speeds the same" uses the label colour (was muted);
  switches align with the label line, not the help text. Screenshots of Pointer/Camera tabs reviewed.
- 2026-11-12: **D0 done.** `app-core`: `controller.rs` (R-A1 + `Error`, `PointerOwner` sub-state,
  `Input`/`Command`/`Event`/`Effect`, `Step`, `View` with marker/recenter/handover countdowns
  from deadlines, injected `now_us`), `settings.rs` (validated, normalised, `SCHEMA_VERSION = 1`,
  `loss_recovery` off → `redetect = None`). `pointer-core::handover` (5 px move-stop, 4 s
  inactivity timer, R-P4 jump bound 0.05 × shorter desktop side, provisional) pulled forward from
  D1 because D0's handover states need it. Mapping is `map_absolute` unsmoothed; smoothing,
  limits, drift and dwell stay D1. 26 controller tests (`crates/app-core/tests/controller.rs`,
  synthetic mirrored frames: head right → cursor right, handover, recenter 5 s, pause/resume via
  recenter, loss off/on, jump bound, permission, settings R-A6, every state reachable) + 4
  settings + 4 handover unit tests. Gates green locally.
- 2026-11-12: **Finding (re-detection arming, open risk):** the selection self-test (ADR-0010)
  only arms on fine texture at 640×480. Value noise with 4 px cells arms; 8 px cells fail at 2 of
  3 points; 16 px cells fail everywhere (best 1.0, "second" 0.97–0.99: the true peak's shoulder
  just beyond `hw0` = 10 px counts as a distinct lookalike, so the 0.1 margin never holds).
  Real faces are smooth, so "Find the target again" may rarely arm in practice and the UI would
  mostly show the "can't find again" hint. Not changed here (non-goal). Re-measure on D7
  recordings; a fix would be in `redetect.rs` (e.g. a distinctness radius tied to the ZNCC
  peak width rather than `hw0`) and needs its own measurement + ADR-0010 amendment.
- 2026-11-12: **Finding (design lab vs R-P2):** the lab's Smoothing slider shows default 0.95,
  "higher is steadier". 2018's `Damping = 0.95` is the weight of the *new* position (very light
  smoothing), so 0.95 labelled "steadier" repeats legacy defect L-2. `Settings::smoothing` is
  `1 − d`, default 0.05, range 0–0.95, higher = steadier. The lab label/default need fixing
  (either show 0.05, or rename the slider "Responsiveness" with default 0.95). Ask the user.

