# Testing

Every change needs a check a machine can repeat: a CI test, a replay metric or an `OCM_PERF`
measurement. Measure first, then change; no fix without evidence that it worked (ADR-0019,
ADR-0020).

## Gates

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked          # no webcam needed
scripts/check-core-deps.sh               # when touching core crate dependencies
cargo deny check                         # licences and bans (CI runs it)
```

CI (`.github/workflows/ci.yml`) runs all of them on macOS, Windows and Linux for every push
to any branch and for PRs into `main` from forks (a PR from a branch here is covered by its
push run, so the PR's jobs show as skipped), except website-only changes (website gates: `docs/WEBSITE.md`). A failing test is surfaced as a job annotation,
readable without auth:
`https://api.github.com/repos/grga1349/open-camera-mouse/check-runs/<job id>/annotations`.

On a laptop, run one heavy cargo command at a time, scoped (`-p <crate>`, a test-name
filter), not the whole set back to back.

## Where tests live

| What | Where |
|---|---|
| Tracker fixtures and metrics (synthetic, seeded) | `crates/tracking-core/tests/` (`fixtures.rs`, `appearance.rs`; `baseline.rs` prints tables, ignored) |
| Pointer maths, steady zone, metrics | unit tests in `crates/pointer-core/src/` |
| Controller state machine, rest / jump / edge tests at every gain | `crates/app-core/tests/controller.rs` |
| Settings store, camera decode, pointer geometry | `crates/platform` unit tests |
| UI driven by label (`egui_kittest`), engine thread with fake camera and pointer | `apps/desktop/src/app_tests.rs`, `engine_tests.rs` |
| Benchmarks (`divan`) | `cargo bench -p tracking-core`, `-p app-core --bench frame`, `-p platform --bench decode` |

Tests must be deterministic: injected clocks, counted rather than timed waits, no real
camera. Device-only tests are `#[ignore]`d:

```sh
cargo test -p platform --test camera_device  -- --ignored   # real webcam
cargo test -p platform --test pointer_device -- --ignored   # moves the real cursor
```

## Manual QA (on a device)

What headless tests can't show: real camera, real cursor, OS permissions, global hotkeys.
Run on a release candidate per OS. **`.app`** rows need the packaged app
(`cargo packager --release`, see README); a bare `cargo run` gets permissions through the
terminal.

| # | Step | Expected |
|---|---|---|
| 1 | Launch without pointer permission | Window opens Stopped; no crash |
| 2 | Start (button or F10 / F11) | Camera light on, colour preview, "Select a target"; Options → Camera shows 640×480 |
| 3 | Click a spot on the face | Marker ring; handover countdown; cursor not moved yet |
| 4 | Keep the mouse still 4 s (no permission) | "Permission needed"; cursor doesn't move; tracking continues |
| 5 | Open settings from the card, grant the permission | Settings open at the right pane; within ~3 s the camera takes the cursor, no restart |
| 6 | Head right / left / up / down | Cursor follows the same direction, smooth, still at rest |
| 7 | Move the physical mouse | Mouse takes over at once; 4 s still → camera takes over at the Main monitor's centre |
| 8 | F12 with another app focused | Recenter circle + 5-4-3-2-1; at 0 tracks the frame centre |
| 9 | Start/pause key with another app focused | Pauses (cursor to centre); again → resumes through the recenter |
| 10 | Laptop keyboard: do the F-keys need `Fn`? | Note which; the window's key chips match what works |
| 11 | Cover the camera with a hand | Lost (grey ring + X), mouse has control; waits for a click or Recenter (re-detection off) |
| 12 | Dwell on; hold still over a button in another app | One click after the dwell time; no repeat until moved away or refractory passed |
| 13 | Minimise the Tracking window while tracking | Cursor keeps following at full rate |
| 14 | Unplug the camera while tracking | "Camera disconnected", last frame frozen; Start works once plugged back |
| 15 | Switch camera in Options while running | Preview restarts on the chosen camera |
| 16 | Quit while running | Camera light off within ~2 s |
| 17 | Paper theme + Larger controls; relaunch | Both persist |
| 18 | Every running state on screen | Matches the design lab; nothing clipped |
| 19 | **`.app`** camera permission denied | "Allow camera access" card; Try again works after granting |
| 20 | **`.app`** pointer permission granted to the app itself | Same as row 5 |
| 21 | Reduce Motion on | Known gap: the Searching animation ignores it (open item) |
| 22 | **`.app`** update / rebuild, relaunch | No new camera prompt or permission card |

Last full run: macOS arm64, rows 1–20 and 22 pass (2026-11-15 dev build, 2026-11-19 `.app`).
Windows and Linux: never run on a device.

## Diagnostics

- `OCM_PERF=1`: the engine prints frame-path timing every few seconds (fps, `in`, `track`,
  `age` capture → cursor move, `ui`, `replaced`), p50 / p95 / max ms, and a `frames` line
  counting frames per tracker status (tracking, held at an edge, held after a fast move,
  lost, no target). Each loss prints a `track` line at once: the reason, `zncc` (against the
  selection), `anchor` (against the lag window, ADR-0018: below 0.7 = the lag check
  rejected, `zncc` below 0.3 = the floor did), the residual and the position in frame px; its
  end prints how many frames it lasted and whether re-detection or a re-selection ended
  it. See `apps/desktop/src/perf.rs`. "Show diagnostics" shows the same two scores live.
- `OCM_RECORD=<folder>`: records raw luma frames (`frame_NNNNNN.pgm`), `frames.csv`,
  `events.csv`, `desktop.csv` and `settings.toml` (≈ 9 MB/s). See
  `apps/desktop/src/recorder.rs`.

**Recordings are for diagnosis only.** They contain a face: keep them outside the repository
(e.g. `~/ocm-recordings/`, not `/tmp`), never commit frames, numbers or code that reads them.
Each finding becomes a small synthetic test that reproduces it. Useful short scenarios
(≈ 2 min total, select the nose tip, don't touch the mouse or Options while recording): still
30 s (shake at rest), slow turns and nods 45 s (losses, lag), the four screen corners 30 s
(edges, return error), a static textured object 20 s (tracker noise alone).
