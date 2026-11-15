# Diagnostic recording protocol (Phase 6b, task 0009)

Short scripted camera recordings used **only to diagnose** tracking and cursor problems on
real footage. Nothing from a recording enters the repository: not the frames, not the
numbers, not code that reads them. Each problem found becomes a small synthetic test
(`crates/tracking-core/tests/`, `crates/pointer-core`, `crates/app-core/tests/`) that
reproduces the exact failure point and runs fast in CI. Recordings are deleted afterwards
(they are large).

## Where recordings live

`~/ocm-recordings/<scenario>-<YYYY-MM-DD>/`, e.g. `~/ocm-recordings/S1-still-2026-11-15/`.
Not in the repo and **not in /tmp** (the first test-drive recording was lost with /tmp).
Size: raw 640×480 luma ≈ 9 MB/s, so ≈ 0.55 GB per minute; the set ≈ 1.2 GB.

## Recording one scenario

```sh
scripts/record-scenario.sh S1        # = OCM_RECORD=~/ocm-recordings/S1-still-<date> cargo run -p open-camera-mouse
```

1. Sit as you normally use the computer, normal room light, camera where it usually is.
2. Press **F10** (Start), then click the **tip of the nose** in the preview (every scenario
   selects the same spot, so they are comparable).
3. Keep the mouse and your head still: after **4 s** the camera takes over (the cursor jumps
   to the screen centre). Wait 2 s more, do the scenario below, then **quit the app**
   (closing the window ends the recording; everything queued is flushed). Don't touch the
   mouse during the scenario: that hands control back to the mouse.
4. The terminal must not print `Recorder: … frame(s) dropped`; if it does, record again
   (dropped frames look like jumps to the tracker).

Settings: keep whatever you normally use; they are saved with the recording
(`settings.toml`). Don't touch Options while recording.

## Scenarios (≈ 2 min in total)

Kept to what slices 2c (jitter) and 2d (losses) actually measure. Add a scenario only when a
question needs it.

| ID | Name | Duration | What to do | Measures |
|---|---|---|---|---|
| S0 | `static` | 20 s | Optional. No head: a printed photo or textured object (book cover) in front of the camera, select a textured spot, touch nothing. | Tracker **measurement noise** alone. |
| S1 | `still` | 30 s | Look at the centre of the main screen and hold as still as is natural. | Cursor wander at rest (screen px). S1 − S0 = real head micro-movement. |
| S2 | `turns` | 45 s | Slowly turn left → right ×3, nod down → up ×3, tilt ear-to-shoulder ×2, comfortable range. | Losses, % tracking, lag. |
| S3 | `corners` | 30 s | Look at the screen centre 3 s, point the cursor at the 4 corners once (~1 s each), back to the centre and hold 5 s. | Losses at the extremes, return-to-start error. |

## Files in a recording

See `apps/desktop/src/recorder.rs`: `frame_NNNNNN.pgm`, `frames.csv`, `events.csv`,
`desktop.csv`, `settings.toml`.
