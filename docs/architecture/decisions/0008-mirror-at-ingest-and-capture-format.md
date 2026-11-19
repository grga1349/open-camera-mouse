# ADR-0008: Mirror at ingest (default on); request 640×480 @ 30 fps

Status: Accepted
Date: 2026-11-11
Resolves: BEHAVIOR_SPEC Q-R5 (R-C1) and Q-R6.

## Context

R-C1 requires one canonical display space for selection, overlay, tracking and pointer
mapping. The constraint from ADR-0006 is that moving the head right moves the cursor right.
The 2018 evidence is unresolved: `RevHoriz` defaults to off, and nobody knows whether
DirectShow drivers pre-mirror (Q-2018-7). `legacy/` tracks raw frames, mirrors only the
preview and negates dx (BEHAVIOR_SPEC §3.3). That is the per-call `W−1−x` juggling R-C1
forbids. For capture, `legacy/` takes the driver default (L-12), and the 2018 negotiated
format is unknown (Q-2018-6).

## Options considered

**Mirroring.**
- *Mirror at ingest.* Flip the luma frame once, before the tracker and preview see it.
  Everything downstream lives in one space. The cost is one row-reverse pass per frame,
  about 0.3 MB at 640×480, with no allocation (`tracking_core::ingest`). LK and NCC are
  both exactly equivariant under a whole-pixel flip, so tracking quality doesn't change.
- *Track raw, mirror the output.* This saves the pass but makes selection, overlay and
  pointer mapping each convert coordinates. That is the legacy hazard.

**Capture format.**
- *Driver default.* Resolutions differ by device (often 720p or 1080p), so motion bounds
  and the cost per frame vary per user.
- *Request 640×480 @ 30 fps, nearest match.* This is a widely supported mode and the
  benchmark midpoint (§8). Per-frame cost and bounds become predictable.
- *Exact 320×240.* That is the 2018-era size, but modern webcams often don't offer it
  natively.

## Decision

1. **Mirror at ingest.** `tracking_core::ingest(frame, mirror)` produces canonical display
   space. Nothing downstream mirrors again. `pointer-core` applies no sign flip, which is
   pinned by `mirrored_ingest_turns_head_right_into_cursor_right`.
2. **Mirroring defaults to on.** For a webcam that doesn't pre-mirror, this is the setting
   that meets the head-right → cursor-right constraint. It deviates from 2018's
   `RevHoriz = false` for ADR-0006 reason **(b) unknown**: whether 2018's tracking space was
   effectively mirrored is unresolved. For drivers that do pre-mirror, the user setting
   turns it off. Phase 6 adds a "move your head right" check to setup.
3. **Request 640×480 @ 30 fps, nearest supported format** (`app_core::ports::CaptureRequest`
   default). Adapters report the negotiated `CaptureFormat`, and the app never assumes the
   request was honoured. If the delivered width is above 640, app-core reduces it by an
   integer box downsample to ≤ 640 before ingest. That is implemented with the Phase 3
   pyramid code and decided here so that Q-R1 (units of motion bounds) can assume a
   canonical width ≤ 640. ADR-0006 reason **(b)**: the 2018 size is unknown.

## Consequences

- The preview shows the same mirrored frame the tracker uses. A preview click maps
  straight to tracker coordinates (R-C2).
- Benchmarks keep 320/640/1280 inputs. 1280 exercises the downsample path.
- If Q-2018-7 resolves to "2018 tracked unmirrored, and drivers didn't pre-mirror", item 2
  still stands, because the head-right constraint wins.

## Amendment (2026-11-12): colour preview

User decision: the preview is shown in **colour**, not grayscale. The tracker still runs on
the 8-bit luma plane only. The camera adapter delivers a colour image (RGB/RGBA) for display
next to the luma, and ingest applies the same mirror and downsample to both, so the preview,
selection, overlay and tracking stay in one canonical display space (R-C1). Implemented in
task 0008 D3 (adapter) and D6 (texture).
