# Behavior specification — Camera Mouse 2018, `legacy/`, and the Rust rewrite

Phase 1 deliverable (task 0003). Supersedes the Phase 0 first pass in `RE_EVIDENCE.md` where
they disagree (see §6). Written 2026-11-11.

This document has three jobs: (1) chart the *actual* end-to-end behavior and defaults of both
reference systems, traceably; (2) list every discrepancy between them; (3) state the expected
behavior and benchmark baselines for the Rust rewrite (§7–§8), informed by but not bound to
either system. Unanswered questions are collected in §9 — they are not papered over.

## 1. Sources and labels

**Camera Mouse 2018** — `Camera Mouse 2018.exe` v1.1.0.0, SHA-256 `8cc85088…a0e6d`, MVID
`51111f1a-beb1-4b59-82bc-fb935bb3ea9e`. Evidence is the static REA export
`../camera-mouse-reverse/research/rea-evidence.json` (SHA-256 `ef4366a8…9e7`), *not* the prose
research files. Every 2018 claim below was re-checked against the CIL instruction anchors in
that export during this task — the prose files (`01`–`08`, `TRACKING_SPEC.md`) were read in
full and used as a map, not as authority. Tokens/RVAs are build-local to the MVID above.

| Short name | Evidence ID | Covers |
|---|---|---|
| EV-FT | `ev_566abdd684865d1bd8b8fd6a528cef7f76457e565da0bd0710de4fd0001429d6` | `FeatureTracker` (type `0x02000029`) |
| EV-MF | `ev_d25da58337fd471e485bcd04afad4b4a14c9fa85d39f2e3fc104122cfce36b34` | `MainForm` |
| EV-CFG | `ev_33f697bc2310ae84930f972af09511d45d670416046eb060fa66c493c88bebbe` | `CameraMouseConfig` |
| EV-M | `ev_2a1d562e7598dfeaca31d5b2b46d5411aacaefbb42ad011d794a9e3c8f1b51ff` | cursor worker class `m` |
| EV-CALL | `ev_34508ac090ca0954ba553b62ade177e52b26914c42423af8c68fc2b97fef0159` | call edges + user strings |
| EV-CV | `ev_eda9ffa5597342dd9707f64cfa9a2bc12296cfb4131b53a21f1d5a110b68b679` | `cv210.dll` exports |

Citation form: `EV-FT 0x06000101 @225–346` = method token, IL offset range. Reproduce any
citation with `python3 -I docs/research/tools/dump_il.py <EV prefix> <token>` (§1.1).

**`legacy/`** — this repo's archived Go/Wails/GoCV v0.3.0 app, read directly. Citation form:
`legacy/internal/mouse/mouse.go:91`.

**Labels.** `VERIFIED` = direct static evidence (decoded IL constants/control flow, or legacy
source). `INFERRED` = follows from verified facts plus a stated assumption. `HYPOTHESIS` =
plausible, unconfirmed. `UNRESOLVED` = no evidence either way. `MEASURED` is reserved for
runtime measurements — **there are none yet for either system**. All 2018 evidence is static
CIL; nothing here was observed at runtime.

### 1.1 What "verified" did and did not cover

- All IL in the six records above that bears on tracking, mapping, cursor control, config
  defaults and selection was decoded with `docs/research/tools/dump_il.py`. Member-ref names
  (e.g. `System.Drawing.Size::get_Width`) were resolved from the same export.
- **Not covered by any evidence:** native `cv210.dll` internals (only the export address
  `0x100ff550`, ordinal 1131, EV-CV); the `MouseClicker` thread body (dwell clicking); the
  `CenterDetect` thread body (5-4-3-2-1 routine); `CameraMouse.VideoCapture.dll` /
  DirectShow negotiation; `MainForm.a(PointF)→PointF` (used by `ResetOffset` mode 1); the
  `SendInput` caller `n.d 0x06000209`.

## 2. Camera Mouse 2018 — end-to-end behavior

### 2.1 Runtime topology — VERIFIED

Named worker threads (string literals in EV-CALL/EV-MF, started via `MainForm.StartThread`
`0x06000187`, which looks names up in a table): `ImageCapture`, `CursorControl` (the `m` class,
ctor `0x060001f5`), `MouseClicker`, `CenterDetect`. Frames arrive on a bitmap the UI thread
locks; the tracker runs inside `MainForm.f 0x060001a6`; the cursor runs independently in
`CursorControl` on a ~70 ms sleep loop. **Tracking rate and cursor rate are decoupled.**

### 2.2 Defaults — VERIFIED (`CameraMouseConfig.DefaultConfig`, EV-CFG `0x06000062`)

| Setting | Default | Notes |
|---|---|---|
| `PixelDepth` / `PixelChannels` | 8 / 1 | tracking image format |
| `HorizG` / `VertG` | 6.0 / 6.0 | per-axis gain |
| `Damping` | 0.95 (`0.949999988` as f32) | weight of new mapped position |
| `RevHoriz` | false | gates `FeatureTracker.ImageFlipping` (§2.4) |
| `WLim` `ELim` `NLim` `SLim` | 0, 0, 0, 0 | fractions of desktop (§2.6) |
| `MouseClicks` | false | dwell clicking **off** by default |
| `MouseClickRadius` | 0.05 | units UNRESOLVED (likely fraction of screen — HYPOTHESIS) |
| `MouseClickDwell` | 1.0 | seconds (HYPOTHESIS on unit) |
| `MouseClickRefract` | 2.0 | refractory period, seconds (HYPOTHESIS on unit) |
| `MouseClickSound`, `EnableDoubleClicking`, `EnableRightClicking` | false | |
| `ClickRecoveryEnable` / `ClickRecoveryTime` | false / 1.0 | semantics UNRESOLVED |
| `StartOnMouseInactivity` / `…Delay` | true / 4.0 s | §2.7; unit VERIFIED (×10⁷ ticks) |
| `MoveStop` / `ClickStop` | true / false | §2.7 |
| `CtrlStart` `CtrlStop` `F9Start` `F9Stop` | true ×4 | `BeginKey`=`EndKey`=`"F9"` |
| `Toggle54321Key` | `"F5"` | |
| `AutoCenterScheme` | 0 | §2.5 |
| `Run_54321_OnLossOfTrackPoint` | **false** | §2.4 loss policy |
| `Display_54321_ButtonOnVideoDisplay` | false | |
| `Auto54321Enable` / `Auto54321Interval` | false / 3 (minutes) | §2.8 |
| `RecenterTrackingPointOnF5` | false | |
| `MultiMonitorSupport` | false | changes MoveStop threshold only (§2.7) |
| `CameraResolutionManip`, `ShowFPS`, `ShowBlueDot`, `ControlChangeSound`, `SaveFrameTimingData` | false | |
| `PauseEnabled` / `PauseDuration` | false / 20 | units UNRESOLVED |
| `Moniker`, `CameraSettings` | `""` | camera chosen by moniker string |

### 2.3 Feature selection — VERIFIED (EV-MF `0x06000163`, `0x06000191`, `0x060001a8`)

On a mouse click in the video panel (ignored if `BlockMouseClicks`):
1. `ValidTrackPoint := false` (which also arms the one-frame grace gate, §2.4).
2. Click `(e.X, e.Y)` is scaled per axis by `VideoSize / panelSize` — a plain stretch; no
   letterbox/cover offset is applied (`0x06000191`, ratios from `ComputeVideoRatios`).
3. `JustClickedFeatureToTrack := true`; offset reset by `AutoCenterScheme`: scheme 1 →
   `ResetOffset(p, mode 0)`; scheme 0 (default) → `ResetOffset(p, mode 1)`.
4. `TrackPoint := p` (f32), `ValidTrackPoint := true`. The setter (`0x06000100`) copies the
   point into the "previous" slot and **recreates all five tracking buffers**
   (`CreateTrackingImages 0x06000104`).

`ResetOffset 0x060001b0`: mode 0 sets `offset = image_center − p` so `p` maps to desktop
center. Mode 1 sets `offset = a(GetCursorPos) − p` using `MainForm.a(PointF)→PointF`, which
is not in the evidence — INFERRED to be the inverse mapping, i.e. "keep the cursor where it
is". The offset field `0x0400010c` is written only by the ctor, `ResetOffset`, `T` (§2.6).

A separate clamp helper `MainForm.c(PointF) 0x060001ab` limits a point to
`[3, VideoW−5] × [3, VideoH−5]`; its call sites are UNRESOLVED.

### 2.4 Frame pipeline — VERIFIED (EV-MF `0x060001a6`; EV-FT `0x06000101` and helpers)

`MainForm.f` runs **only while `ValidTrackPoint` is true**; otherwise tracking buffers are not
advanced at all. It locks the bitmap in its own pixel format and calls
`ProcessNewImage(Scan0, w, h)` → `Process`:

1. **Size check** (`0x06000105`): if the incoming size differs from tracking buffer A, set
   `CurrentImageSize`, recreate buffers, return true (no tracking this frame).
2. Convert the incoming image into buffer A with OpenCV code 6 (= `CV_BGR2GRAY` in OpenCV
   2.1 — INFERRED from the constant; implies a 3-channel BGR source, consistent with
   `ImageFlipping` swapping 3-byte pixels).
3. Swap the previous/current point slots.
4. `cvCalcOpticalFlowPyrLK(prev=B, curr=A, prevPyr=D, currPyr=C, prevPts, currPts, count=1,
   win=10×10, level=3, status, err=null, criteria=(type 3, 20 iter, ε 0.03), flags=i)` @220.
   Window/criteria from the ctor `0x060000f8` @93–137. **`flags` (field `0x040000f8`) is
   written only in the ctor, to `0|1` = `CV_LKFLOW_PYR_A_READY`** — the previous pyramid is
   always declared ready (correct in steady state because pyramids swap; see step 8).
5. **Status gate** @225–346: only if `ValidTrackPoint ∧ ¬grace ∧ status[0] = 0`, apply the
   loss policy (table below). `grace` (field `0x04000105`) is true after construction and
   after every `set_ValidTrackPoint(false)`; it is cleared @349 on every frame that reaches
   step 6. Net effect: **the LK status of the first tracked frame after any selection is
   ignored.**
6. **Outlier guard** (`0x0600010a`): if `√((x_new−x_prev)² + (y_new−y_prev)²) > 35` image px,
   `new := prev`. Runs whether or not the status gate fired.
7. `TrackPoint := new` (f32; `CvPoint2D32f`).
8. Swap image buffers A↔B and pyramid buffers C↔D. Return true.

**Loss policy** (EV-FT `0x06000101` @251–346; `edge` = `FeatureTracker.b 0x06000102`):

| `Run_54321_OnLossOfTrackPoint` | `edge` | Action |
|---|---|---|
| false (default) | true | hold previous point, keep tracking |
| false (default) | false | message, `ValidTrackPoint := false`, return false → `f` releases cursor control (`MainForm.e() 0x060001a9`, "Control: Mouse") |
| true | true | as row 2, plus `StartThread("CenterDetect")` from `f` @163–191 |
| true | false | **status ignored** — LK output used, only the 35 px guard applies |

**`edge` predicate** (`0x06000102`, VERIFIED decode; `A` = previous point, `B` = new point,
`W×H` = `CurrentImageSize`, margin 10):
`edge = (A.x ≤ W−10 ∧ B.x < A.x) ∨ (A.x ≥ 10 ∧ B.x > A.x) ∨ (A.y ≤ 10 ∧ B.y < A.y) ∨ (A.y ≥ H−10 ∧ B.y > A.y)`.
The y terms read as "near top/bottom and moving outward". The x terms have the margins
swapped relative to y, so they are true for almost any horizontal motion away from the
edges. INFERRED: an original bug. Practical consequence under defaults: a failed LK status
with any horizontal displacement *holds* the point, and only failures with no x motion
invalidate tracking. See Q-2018-1.

**First frame after selection.** The buffers are recreated at selection (§2.3 step 4), so the
first `Process` runs LK against a freshly allocated previous image/pyramid that has not been
filled with the selection frame, while flags still claim the previous pyramid is ready. Its
status is ignored (grace), and only the 35 px guard bounds the result. INFERRED: the first
post-selection estimate is unreliable. Whether `cvCreateImage` memory is zeroed is
UNRESOLVED (Q-2018-2).

**Image flipping.** `FeatureTracker.ImageFlipping 0x06000106` mirrors a 3-byte-per-pixel
buffer horizontally in place, gated by `RevHoriz` (default false). Its caller is
`MainForm.TrackingPaint` (EV-MF call edges). Whether it runs before or after the tracker sees
the same bitmap is UNRESOLVED. The loop bounds leave the centre column pair unswapped on
even widths (INFERRED, cosmetic).

### 2.5 Cursor control loop — VERIFIED (EV-M `0x060001f6`)

On start: previous cursor := `(deskW/2, deskH/2)` (without desktop origin). Loop until quit:

- **If `ControlCursor`**: restore the foreground window if the On-Screen Keyboard stole it
  (`m.a() 0x060001fb` / `m.b() 0x060001fa`, title literal `"On-Screen Keyboard"`);
  `p := T(TrackPoint, prevCursor)` (§2.6); `MoveMouseTo(p)` (§2.6); `prevCursor := p`;
  sleep 70 ms; if `MoveStop`: run the handover check (§2.7). The "move failed → release
  control" branch @185–215 is dead code: `m.a 0x060001f8` always returns false.
- **Else**, if `ValidTrackPoint`: read the OS cursor position. If `MainFormRequestsAutoTracking`,
  clear it and force-start control. Else, if `StartOnMouseInactivity ∧ ¬PauseTracking`, run
  the inactivity check (§2.7). Then sleep 70 ms.

Cursor updates are therefore ≤ ~14 Hz, plus processing time. This is a structural bound, not a
measured rate.

### 2.6 Mapping, damping, limits, output — VERIFIED

- **Map** (`MainForm.b 0x060001ad`, ratios `0x060001a8`):
  `m = desk_center + (p + offset − img_center) ⊙ (HorizG, VertG) ⊙ (deskW/imgW, deskH/imgH)`.
  `desk_center` includes the desktop rectangle origin. `img_center = CurrentImageSize/2`. The
  two `×(−1)` in the IL cancel, so the sign is positive on both axes. **Absolute mapping:**
  a given head position always maps to the same screen point, modulo the offset.
- **Damp** (`MainForm.T 0x060001af` @0–80): `c = d·m + (1−d)·c_prev`, `d = Damping`.
  Per tick, not time-based.
- **Limits and offset evolution** @85–547: `E = (1−ELim)·deskW`, `Wl = deskX + WLim·deskW`,
  `N = deskY + NLim·deskH`, `S = (1−SLim)·deskH`. If `c.x > E`: clamp to E and decrement
  `offset.x` by 1. Else if `c.x < Wl`: clamp and increment it by 1. Same for y with S and N.
  So **while the cursor is pinned at a limit, the offset drifts 1 image px per tick back
  toward centre**. Holding the head past the edge slowly re-centres the mapping. This
  resolves research Q1 ("offset evolution"). E and S omit the desktop origin; W and N
  include it (INFERRED multi-monitor defect, Q-2018-3).
- **Output** (`m.b 0x060001f7`, `m.a 0x060001f8`): `(c / CurrentScreenSize) × 65535` →
  `WindowsInput IMouseSimulator.MoveMouseTo` (absolute). Primary-screen size only; no origin
  subtraction.

### 2.7 Control handover: physical mouse ↔ camera — VERIFIED

- **Inactivity start** (`m.a 0x060001fd`): if the OS cursor has not moved since the last tick
  (or start is forced): once `StartOnMouseInactivityDelay` (4 s) has elapsed, or immediately
  if forced, and the foreground window is not the On-Screen Keyboard, set
  `ControlCursor := true` ("Control: Camera Mouse"). Any cursor movement resets the timer.
  **Correction:** research `05` called this "dwell-related timing". It is not dwell clicking.
- **Move-stop** (`m.c 0x060001f9`): if the distance between the last commanded position and
  the actual OS cursor is > 5 px (> 100 px with `MultiMonitorSupport`), and the commanded
  point is not (0, 0): `ControlCursor := false` ("Control: Mouse"). Physically moving the
  mouse takes control back.
- **On control start** (`set_ControlCursor 0x0600015a`): optional sound. If scheme 0, or
  scheme 1 right after a click: `ResetOffset(TrackPoint, mode 0)` (current head position maps
  to desktop centre) and `prevCursor := desktop centre`. Otherwise mode 1 (keep the cursor
  where it is). Then `StartThread("MouseClicker")`. **Under defaults, the cursor jumps to
  screen centre when the camera takes control.** The control-off half of the setter is
  truncated in the evidence (UNRESOLVED).

### 2.8 Pause, 5-4-3-2-1, camera loss — VERIFIED structure, UNRESOLVED bodies

- **Pause toggle** (`0x06000181`): show `PauseWindow`, stop `MouseClicker`,
  `ValidTrackPoint := false`, `ControlCursor := false`, hide the main form, and move the OS
  cursor to desktop centre.
- **5-4-3-2-1** (`0x06000182`, likely the button handler): release control,
  `MainFormRequestsAutoTracking := true`, `StartThread("CenterDetect")`. The CursorControl
  loop then force-starts control once a valid point exists. `CenterDetect`'s body (how it
  picks a point) is UNRESOLVED. The bundled `haarcascade_frontalface_default.xml` makes face
  detection a HYPOTHESIS, nothing more.
- **Auto 5-4-3-2-1** (`f` @196–268): if `Auto54321Enable`, start `CenterDetect` every
  `Auto54321Interval × 60` s (default off).
- **Camera disconnect** (`MainForm.c() 0x060001ac`): polls camera names every 500 ms. If the
  configured moniker vanishes: stop `ImageCapture`, invalidate tracking, release control, and
  show `"… has closed because the video source has been disconnected."` (EV-CALL literal).

### 2.9 Dwell clicking — UNRESOLVED

Defaults are known (§2.2: off, radius 0.05, dwell 1 s, refractory 2 s) and the thread name
(`MouseClicker`) is known. The state machine itself is not in any evidence record.

## 3. `legacy/` — end-to-end behavior (VERIFIED from source)

### 3.1 Topology

Single run goroutine `legacy/internal/app/app.go:177-218` selects over commands and frames.
Tracking, pointer update and preview encoding all happen per camera frame
(`handleFrame`, `app.go:220-274`). The camera goroutine `legacy/internal/camera/camera.go:32-68`
uses device 0 at the driver's default resolution and frame rate, and a 1-slot channel. It
ends the stream after 30 consecutive read failures 10 ms apart (`camera.go:10,48-55`).
**Pointer update rate = camera frame rate.**

### 3.2 Defaults and ranges

| Param | Default | UI range | Load-time validation | Source |
|---|---|---|---|---|
| `templateSizePx` | 45 | {30, 45, 60} | ≤0 → default | `config.go:11,64`; `SettingsScreen.tsx:11` |
| `gainMultiplier` | 8.0 | 1–30 step 0.5 | ≤0 → default | `config.go:12,67`; `SettingsScreen.tsx:87-94` |
| `smoothing` | 0.30 | 0–0.85 step 0.05 | ≤0 or >1 → default | `config.go:13,70`; `SettingsScreen.tsx:96-103` |
| `dwellEnabled` | false | toggle | — | `config.go:30-37`; `ClickModeControls.tsx` |
| `dwellTimeMs` | 500 | 200–1500 step 50 | ≤0 → default | `config.go:14,73` |
| `rightClickEnabled` | false | toggle | — | |
| `autoStart` | false | checkbox | — | `app.go:57-64` |
| Hard-coded | NCC threshold 0.68, search margin 2×template, deadzone 1 px, max step 35 px, dwell radius 30 px, preview 66 ms / JPEG q80, preview panel 360×270 | | | `tracking.go:11-12`; `mouse.go:11-13`; `preview.go:12-13`; `previewLayout.ts` |

Params persist as JSON at `os.UserConfigDir()/open-camera-mouse/config.json`
(`config.go:43-49`). Validation runs **only on load**, not on `UpdateParams` (`app.go:120-132`).

### 3.3 Coordinates, selection, recenter

- Camera frames are never flipped. The preview is mirrored at encode time
  (`preview.go:46`). Clicks arrive in mirrored display space and are converted
  `rawX = W−1−x` (`app.go:225-231`). The overlay is converted back (`app.go:264`). The
  frontend maps the click through the same CSS `object-fit: cover` transform it draws with
  (`coverTransform.ts`, `usePickPoint.ts:21-27`).
- **Pick** (`tracking.go:53-80`): template = `size×size` grayscale patch, centre clamped so
  the patch fits in the frame. The template is fixed until the next pick.
- **Recenter** (`useRecenter.ts`, `app.go:141-158, 233-238, 282-285`): `BeginRecenter` pauses
  tracking, pointer and overlay, the UI shows a white box at frame centre with a 3 s
  countdown, then `ConfirmRecenter` picks the raw frame centre. F12 emits
  `recenter:hotkey`, which runs the same flow (`app.go:48-51`).

### 3.4 Tracking (`legacy/internal/tracking/tracking.go`)

Grayscale (`BGR2Gray`) → `MatchTemplate(TM_CCOEFF_NORMED)` over a window of ±2×template
around the last good centre (`:92-112`). If max < 0.68: result lost at the last good centre,
template centre not moved (`:83, 115-117`). Else: new integer centre; the search window
re-centres there (`:119-129`). Integer-pixel output only, and no outlier guard beyond the
window itself (max per-frame motion ≈ 2×template − template/2 ≈ 67 px at 45).

### 3.5 Pointer (`legacy/internal/mouse/mouse.go`)

**Relative mode**, driven by tracked-point *velocity*:
`dx = x_prev − x` (sign flipped because frames are unmirrored), `dy = y − y_prev` (`:73-74`).
Deadzone: |d| < 1 → 0 (`:78-83`). Clamp to ±35 px/frame (`:85-86`). `target = d × gain`.
`v += (target − v) × smoothing` (`:91-92`). Then `OS cursor += round(v)` (`:94-97`). When lost
or uninitialised: re-seed the previous point and zero `v`, so the cursor stops instantly
(`:62-70`). Physical-mouse use coexists naturally, because each step is relative to the
current OS position.

**Dwell** (`:100-129`): a reference point and timer on the OS cursor. Leaving the 30 px
radius resets both. After `dwellTimeMs` inside the radius it clicks and restarts the timer
**with the reference unchanged**, so it keeps clicking every `dwellTimeMs` while the cursor
stays put. Disabled or lost → reset.

### 3.6 Lifecycle, status, hotkeys

Start/Stop (`app.go:72-102`), F11 toggles them (`hotkeys.go`, fixed keys; see project memory:
keep fixed). `Status{Running, Lost}` is emitted only on `Lost` edges (`app.go:254-258`). The
frontend starts at `lost:false` (`useStatus.tsx:15`). AutoStart (`app.go:57-64`).
`--smoke-test` (`main.go:22-26, 54-61`).

### 3.7 Defects and hazards found in `legacy/` (do not carry forward)

| ID | Finding | Evidence |
|---|---|---|
| L-1 | Smoothing slider minimum 0 → `v` never moves → **cursor frozen** until restart (load-time validation then silently resets it to 0.30) | `SettingsScreen.tsx:98`, `mouse.go:91`, `config.go:70`, `app.go:120-132` |
| L-2 | "Smoothing %" polarity is inverted: higher value = *less* smoothing (it is the EMA α) | `mouse.go:91-92`, `SettingsScreen.tsx:97` |
| L-3 | Dwell repeat-clicks every `dwellTimeMs` while still. No refractory period, no re-arm on leaving the radius | `mouse.go:124-128` |
| L-4 | Camera stream ending (unplug, 30 failures) leaves `running=false` silently. The UI still shows "running" | `app.go:178-183, 211-213` (no `EmitRunning`) vs `:186-190` |
| L-5 | Start race: `EmitRunning(false)` from a fast `Stream` error can precede `service:running=true` | `app.go:85,186-190` vs root `app.go:67-73` |
| L-6 | UI shows "OK" while running with no target, and after Stop. `lastLost` starts true, so no edge is emitted until the first good frame | `useStatus.tsx:15`, `app.go:200,254` |
| L-7 | Data race: `a.params` read without the mutex in the run goroutine | `app.go:266` vs `:124-127` |
| L-8 | Template-size change while tracking: the old template is kept, but the search margin and overlay use the new size | `app.go:286-288`, `tracking.go:92`, `app.go:266` |
| L-9 | Gain and smoothing are per frame, so cursor speed scales with camera FPS | `mouse.go:88-92`, `camera.go` (no FPS control) |
| L-10 | Reacquisition after loss produces a jump of up to 35 × gain (280 px default, before EMA) | `mouse.go:62-70, 85-88`; `tracking.go:83` |
| L-11 | Recenter picks raw `W/2`, while the UI box is drawn at display `W/2`: 1 px mismatch on even widths (cosmetic) | `app.go:236`, `CameraPreview.tsx:85-86` |
| L-12 | Camera fixed to device 0. No resolution/FPS negotiation and no device choice | `app.go:64`, `camera.go:27` |

## 4. Discrepancy matrix (2018 vs `legacy/`)

| Aspect | Camera Mouse 2018 | `legacy/` |
|---|---|---|
| Tracker | 1-point pyramidal LK, frame-to-frame (drifts, adapts) | fixed-template NCC (no drift, no adaptation) |
| Precision | f32 subpixel | integer px |
| Confidence | binary LK status; first post-pick frame unchecked | NCC score ≥ 0.68 |
| Motion bound | 35 px Euclidean guard (hold) | search window ±90 px at template 45 |
| Loss | config-gated 4-way policy with buggy edge predicate (§2.4); invalidates → mouse regains control | per-frame `Lost` flag, freeze at last point, auto-resumes when score recovers |
| Pointer model | **absolute** (head pose ↔ screen point) + offset | **relative** velocity from current OS cursor |
| Gain | per-axis 6 × desk/img ratio | scalar 8 × raw px delta |
| Smoothing | position EMA, α = 0.95, per 70 ms tick | velocity EMA, α = 0.30, per camera frame |
| Same polarity? | yes. Both are α on the new value (higher = more responsive). Only legacy's *label* is inverted (L-2) | |
| Edge handling | clamp to limits + offset drift 1 px/tick | none; OS clamps the cursor |
| Pointer rate | ≤ ~14 Hz (70 ms sleep) | camera FPS |
| Mirroring | `RevHoriz` (default off); tracking space mirrored only if on | always: preview mirrored, tracking raw, dx negated |
| Mouse/camera arbitration | explicit: move-stop > 5 px yields; 4 s inactivity resumes | none needed (relative), but dwell sees physical moves |
| Dwell | off; 1 s dwell, radius 0.05, 2 s refractory (logic UNRESOLVED) | off; 500 ms, 30 px, repeat-clicks (L-3) |
| Recenter | 5-4-3-2-1 `CenterDetect` (body UNRESOLVED); optional auto every 3 min | guided 3 s countdown → pick frame centre |
| Hotkeys | F9 start/stop, F5 5-4-3-2-1, Ctrl start/stop | F11 start/stop, F12 recenter |
| Camera loss | detected (500 ms poll), message, tracking invalidated | silent (L-4) |
| Pause | pause window, cursor to centre | "tracking enabled" toggle exists in the backend; no UI found |

## 5. Things that look shared but aren't

- **35** appears in both systems: an image-space Euclidean outlier guard in 2018, and a
  per-frame pointer-delta clamp per axis in legacy. These are different mechanisms in
  different spaces.
- **Smoothing polarity** is *not* inverted between the systems (see §6).
- **Gain** values are not comparable: 2018's effective screen-px-per-image-px at 320-wide on a
  1920-wide desktop is 6 × 6 = 36 (absolute). Legacy's 8 multiplies a velocity. These figures
  are derived, not measured.

## 6. Corrections to Phase 0 / prior research

| Claim | Where | Correction |
|---|---|---|
| "Cursor-smoothing conventions are inverted between the two systems" | `RE_EVIDENCE.md` §legacy table + point 4 | Wrong. Both are α-on-new-value. Legacy's *UI label* is what's inverted (L-2). Fixed in `RE_EVIDENCE.md`. |
| "VERIFIED — dwell-related timing: `m.a 0x060001fd`" | research `05` | It's mouse-inactivity auto-start (§2.7), not dwell. |
| "Loss/boundary … exact policy unresolved" | research `02`/`04`, `RE_EVIDENCE.md` | Resolved statically: §2.4 table + edge predicate. |
| "Offset … exact evolution needs further investigation" | research `05` | Resolved: ±1 image px per tick while clamped at a limit (§2.6). |
| "`PixelDepth`/`PixelChannels` defaults" listed as unanswered | research `08` item 4 | Resolved: 8 / 1 (already in `02`); conversion code 6 = `CV_BGR2GRAY` (INFERRED). |
| (absent) LK `flags` | — | `CV_LKFLOW_PYR_A_READY` constant; first-frame buffers fresh (§2.4). |

## 7. Expected behavior for the Rust rewrite

Normative (`MUST`/`SHOULD`) unless marked **DECIDE**, which means a product/architecture
decision owed by the named phase. **Product direction (ADR-0006): favor 2018 behavior where
practical.** Deviations must cite defect / unknown / platform / prior user decision. Items
resolved by ADR-0006 are marked *→ ADR-0006*. None of these are compatibility requirements with either
source system.

**Tracking (`tracking-core`)**
- R-T1 MUST be a pure function of (frames, params, selection events). No wall-clock time, no
  I/O. Same inputs give bit-identical outputs on one platform.
- R-T2 MUST output an f32 position plus an explicit status (`Tracking`,
  `Held{reason}`, `Lost{reason}`) plus diagnostics (algorithm-specific residual/score). No
  silent holds (2018 holds silently in two places).
- R-T3 MUST initialise the reference (previous frame/pyramid or template) from the selection
  frame itself. No grace frame against unfilled buffers (2018 §2.4).
- R-T4 Loss detection MUST be symmetric across axes and covered by tests at all four edges
  (the 2018 x/y asymmetry is a known defect).
- R-T5 Motion/outlier bounds SHOULD be expressed relative to frame size or re-derived per
  resolution. 35 px means very different things at 320×240 and 1280×720 (Q-R1).
- R-T6 Algorithm choice stays per ADR-0004: LK (2018-preset) vs NCC (legacy-preset), measured
  on the §8 corpus.

**Coordinates**
- R-C1 One canonical "display space" end to end (mirrored if mirroring is on). Selection,
  overlay, tracking and pointer mapping all use it, so there is no per-call `W−1−x` juggling
  (legacy §3.3). **DECIDE (Phase 2)**: mirror at ingest vs. track raw and mirror the output.
  Not settled by ADR-0006 (2018 evidence unresolved, Q-2018-7). Constraint: head-right moves
  the cursor right.
- R-C2 Preview click → frame coordinate MUST use the exact transform used to draw (legacy got
  this right; 2018 plain stretch is only correct without letterboxing).

**Pointer (`pointer-core`)**
- R-P1 *→ ADR-0006:* **absolute** mapping with offset, screen limits and offset drift (2018
  §2.6, with the E/S desktop-origin defect fixed). No relative mode unless requested. Original
  framing: absolute (2018) vs relative (legacy) mapping, or both
  as modes. Trade-off: absolute = repeatable head↔screen correspondence, needs limits/offset
  logic and recentering. Relative = coexists with the physical mouse and needs no
  calibration, but drifts.
- R-P2 Filtering MUST be time-based (dt-aware), so behaviour is FPS-independent (fixes L-9). The
  user-facing parameter's name MUST match its polarity (fixes L-2). Invalid values (e.g. α = 0)
  MUST be impossible to set, not only corrected on load (fixes L-1).
- R-P3 Gain SHOULD be per-axis (2018) with a linked default.
- R-P4 Reacquisition after loss MUST NOT cause a pointer jump larger than a configured bound
  (fixes L-10). Test it explicitly.
- R-P5 Multi-monitor: map into the full virtual desktop with correct origin handling (2018 and
  legacy both assume primary-at-origin in places). Validated in Phase 7 per OS.

**Clicking**
- R-K1 Dwell MUST fire once per dwell, then require either leaving the radius or a refractory
  period before re-arming (fixes L-3; 2018 has a 2 s `MouseClickRefract` default).
  *→ ADR-0006 (provisional):* off by default, 1 s dwell, 2 s refractory, radius 0.05 as a
  fraction of screen. Revisit when Q-2018-4 is resolved.
- R-K2 Dwell MUST NOT fire while tracking is lost or the pointer is not under camera control.

**Control arbitration and lifecycle (`app-core`)**
- R-A1 Explicit state machine with every state user-visible: `Stopped`, `Starting`,
  `NoCamera`, `PermissionDenied`, `NoTarget`, `Tracking`, `Lost`, `Recentering`, `Paused`,
  `CameraDisconnected`. Every transition is emitted. No silent stop (fixes L-4, L-5, L-6).
- R-A2 *→ ADR-0006: adopted* (5 px move-stop, 4 s inactivity resume, cursor to centre on
  takeover). Original question: adopt 2018-style handover (physical move > threshold yields
  control; inactivity N s resumes)? Strongly recommended if R-P1 picks absolute mode.
- R-A3 Guided recenter (legacy 3 s countdown → pick centre) is the baseline. 2018's
  `CenterDetect` is not reproducible without more evidence.
- R-A4 Hotkeys: keep legacy's fixed F11 start/stop + F12 recenter unless the user decides
  otherwise. **Open with the user**: ADR-0006 favors 2018 (F9/F5/Ctrl) but conflicts with the
  earlier fixed-F11/F12 decision. Per the existing project decision, do not make them configurable without asking.
- R-A5 Settings are validated at the API boundary on every update and on load, with schema
  versioning for the persisted file.
- R-A6 Shared state is never read across threads without synchronisation (fixes L-7). Param
  changes that invalidate tracker state (template size) MUST re-initialise it or be refused
  while tracking (fixes L-8).

## 8. Benchmark and fixture baselines

**No numeric baseline exists today for either system.** Every number Phase 3 produces is the
first measurement. Baselines are therefore defined as *procedures* here. Numbers marked
PROPOSED are targets to validate, not facts.

**Comparators** (same corpus, same harness):
- B-LK18: pure-Rust pyramidal LK with the 2018 preset (10×10 window, 3 levels, 20 iter / ε 0.03,
  35 px guard), correctly initialised (R-T3).
- B-NCC: pure-Rust fixed-template NCC with the legacy preset (template 45, margin 2×, threshold 0.68).
- Optional, for parity only: `legacy/` NCC via GoCV on recorded fixtures. That requires
  restoring the legacy build (`legacy/README.md`) and is outside the default toolchain.

**Fixture corpus** (`tracking-validation` skill; seeded, ground truth derived independently of
the tracker): integer and subpixel translation; translation crossing 35 px/frame; brightness
step and ramp; contrast change; partial and full occlusion with a known gap; in-plane
rotation ±15°; scale ±10%; motion blur; target at each of the 4 edges, moving out and in;
textureless patch; repeated-texture distractor; dropped frames / stall; sensor noise. At
three resolutions: 320×240 (2018-era), 640×480, 1280×720. Plus ≥ 3 recorded real head-motion
sequences with hand-labelled points (Phase 3/4).

**Metrics per comparator per fixture:** endpoint error (mean/median/p95/max px, and as % of
frame width); % frames tracked; false-`Tracking` frames during true occlusion; time to
detect loss; time to recover; drift over a 60 s static sequence; pointer jitter while
stationary (after `pointer-core`, std-dev in screen px); max pointer jump on reacquisition
(R-P4).

**Performance** (`benchmarking` skill; divan or Criterion, record device metadata, report
min/median/p95): grayscale conversion, pyramid build, per-point LK or NCC search, end-to-end
`tracking-core` step, and allocations per steady-state frame, at each resolution.
PROPOSED targets, to confirm or revise with Phase 3 data: zero steady-state heap
allocations per frame; `tracking-core` step p95 < 2 ms at 640×480 on the primary dev Mac;
camera-frame-to-cursor-move latency p95 < 1 frame interval + 10 ms (measured in Phase 6/8).

**2018 runtime comparison** (optional, Windows only): a controlled run of the original
against replayed fixtures was suggested in research `07`. It needs a Windows host and a
virtual camera, and it is not on the critical path. Record as Q-2018-6.

## 9. Open questions

**About Camera Mouse 2018** (owner: whoever runs further REA; none block Phase 2)
- Q-2018-1 Is the x-axis margin swap in `edge` (§2.4) a real behaviour or a misread of
  NaN/unordered compare semantics? The decode uses standard ECMA-335 branch semantics. A
  runtime check would settle it.
- Q-2018-2 First post-selection LK runs on freshly created buffers: zeroed or garbage? What does
  OpenCV 2.1 PyrLK output on a failed point (last iterate vs. unchanged)?
- Q-2018-3 Multi-monitor: E/S limits omit the desktop origin; output divides by primary
  screen size. Real-world effect UNRESOLVED.
- Q-2018-4 `MouseClicker` (dwell) state machine, units of `MouseClickRadius`, interplay with
  `ClickStop`, `ClickRecovery*`, double/right-click options. Needs the thread class's IL.
- Q-2018-5 `CenterDetect` routine: countdown UI, how the point is chosen (face cascade?).
- Q-2018-6 Effective tracking FPS, capture resolution negotiated via DirectShow, and CPU
  cost. Runtime only.
- Q-2018-7 Does `ImageFlipping` (in `TrackingPaint`) run before the tracker reads the
  bitmap, i.e. is tracking space mirrored when `RevHoriz` is on?
- Q-2018-8 `MainForm.a(PointF)→PointF`: confirm it is the inverse mapping (ResetOffset mode 1).

**For the rewrite** (owner: named phase)
- Q-R1 Units for motion bounds/outlier guard: px at a canonical resolution, or a fraction of
  frame size? (Phase 3)
- ~~Q-R2~~ Resolved: absolute (ADR-0006).
- ~~Q-R3~~ Resolved: 2018 handover (ADR-0006).
- Q-R4 Dwell: provisional 2018 defaults (ADR-0006). Final rule waits on Q-2018-4.
- Q-R5 Mirror at ingest vs. output (R-C1). (Phase 2)
- Q-R6 Camera resolution/FPS policy: request a fixed size (e.g. 640×480@30), or take the
  device default? This interacts with Q-R1 and the benchmark resolutions. (Phase 2)
- Q-R8 Hotkeys: 2018 (F9 start/stop, F5 recenter, Ctrl) or the earlier fixed F11/F12? (user)
- Q-R7 Is a 2018 runtime comparison (Q-2018-6) worth a Windows setup? (user decision)
