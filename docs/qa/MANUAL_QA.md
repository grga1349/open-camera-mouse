# Manual QA — macOS (Phase 6, task 0008 D7)

What the headless tests can't show: a real camera, the real cursor, OS permissions, global
hotkeys from another app. Run on the dev Mac, then record the date and the result per row
(`pass`, `fail: …`, `n/a: …`). Windows and Linux on-device runs are Phase 7.

The same flows run headless in CI (`apps/desktop/src/runtime_tests.rs`, fake camera + fake
pointer): this checklist is about the OS, not the logic.

## Setup

```
export PATH="$HOME/.cargo/bin:$PATH"
cargo run -p open-camera-mouse                 # real settings
OCM_DEV_SETTINGS=/tmp/ocm.toml cargo run -p open-camera-mouse   # throwaway settings (debug)
OCM_RECORD=/tmp/ocm-rec cargo run -p open-camera-mouse          # also record frames (below)
```

A bare `cargo run` binary gets its camera and Accessibility permissions through the terminal
app that launched it (macOS attributes them to the "responsible" process). Rows marked
**`.app`** need a bundled app to be honest; the bundle is Phase 7 packaging unless a row
fails only because of the terminal attribution.

## Checklist

| # | Step | Expected | Date | Result |
|---|---|---|---|---|
| 1 | Launch with Accessibility **not** granted to the terminal | Window opens Stopped; no crash | 2026-11-15 | pass |
| 2 | Start (button or F10) | Camera light on, colour preview, "Select a target" (NoTarget), capture 640×480 in Options → Camera | 2026-11-15 | pass |
| 3 | Click a spot on the face | Marker ring; pointer line shows the 4 s handover countdown; cursor not moved yet | 2026-11-15 | pass |
| 4 | Keep the mouse still 4 s (no Accessibility) | Pointer line "permission needed"; cursor doesn't move; tracking continues | 2026-11-15 | pass |
| 5 | Open settings from the card, grant Accessibility | System Settings opens at Accessibility; app is listed; within ~3 s of granting, the camera takes the cursor (no restart) | 2026-11-15 | pass |
| 6 | Head right / left / up / down | Cursor follows in the same direction, smooth, no jitter at rest | 2026-11-15 | pass |
| 7 | Move the physical mouse | Mouse takes over at once (Handover + countdown); after 4 s still, the camera takes over at the desktop centre | 2026-11-15 | pass |
| 8 | F12 (Recenter) with another app focused | Recenter circle + 5-4-3-2-1 badge; at 0 tracks the frame centre | 2026-11-15 | pass |
| 9 | F10 with another app focused | Pauses (cursor to desktop centre); F10 again resumes through the 5 s recenter | 2026-11-15 | pass |
| 10 | On a Mac laptop keyboard: F10/F12 need `Fn`? | Note which; the window's key chips should match what works | 2026-11-15 | pass |
| 11 | Cover the camera with a hand | Lost (grey ring + X), mouse has control; with "Find the target again" off it waits for a click or Recenter | 2026-11-15 | pass |
| 12 | Options → Clicking: dwell on; hold still on a button in another app | One click after the dwell time; no repeat until moved away or refractory passed | 2026-11-15 | pass |
| 13 | Minimise the Tracking window while tracking | Cursor keeps following (eframe runs `App::logic` while minimised, throttled to ~10 Hz: check it's usable) | 2026-11-15 | pass |
| 14 | Unplug an external camera while tracking (or close the lid camera if external only) | "Camera disconnected", last frame frozen; Start retries once plugged back | 2026-11-15 | pass |
| 15 | Options → Camera: switch camera while running | Preview restarts on the chosen camera | 2026-11-15 | pass |
| 16 | Quit while running | Camera light goes off within ~2 s (nokhwa #224) | 2026-11-15 | pass |
| 17 | Theme Paper, Larger controls; quit; relaunch (Text size removed 2026-11-16, TEST_DRIVE #10) | Both persist | 2026-11-15 | pass |
| 18 | Running states on screen (Tracking, Held, Lost, Searching, Recentering, Paused, Handover, Permission needed) | Match the design lab; nothing clipped | 2026-11-15 | pass |
| 19 | **`.app`**: camera permission denied (System Settings → Camera off) | "Allow camera access" card, no crash; Try again works after granting | 2026-11-15 | n/a: Phase 7 (.app bundle) |
| 20 | **`.app`**: Accessibility granted to the app itself (not the terminal) | Same as row 5 | 2026-11-15 | n/a: Phase 7 (.app bundle) |
| 21 | Reduce Motion on (System Settings → Accessibility → Display) | Searching animation — known gap: egui has no reduced-motion flag, record what it does | 2026-11-15 | deferred: Phase 8 (accessibility testing), user decision |

## Frame recordings (for Phase 4 threshold re-measurement)

`OCM_RECORD=<folder>` writes every raw camera frame as `frame_NNNNNN.pgm` (luma, native
orientation, not mirrored) plus `frames.csv` (timestamps) and `events.csv` (selections,
state and pointer changes, with times). About 9 MB per second. Recordings contain your face:
keep them out of the repository. Useful sessions: a still face; slow and fast head turns;
partial occlusion (hand); turning away and back (re-detection arming); dim light.

## Runs

(Newest first. One line per run: date, build commit, who, rows run, summary.)

- 2026-11-15, `7e9a2bb` (debug build, `OCM_DEV_SETTINGS`), user + Claude, rows 1–18: all pass (global F10/F12 work from another app; minimised ~10 Hz usable). Rows 19–20 n/a until the Phase 7 `.app`; row 21 and the `OCM_RECORD` threshold sessions deferred to Phase 8 (user). Row 10: whether F10/F12 needed `Fn` was not noted.
