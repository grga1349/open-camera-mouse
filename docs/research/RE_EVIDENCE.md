# Reverse-engineering evidence synthesis — Camera Mouse 2018 vs. `legacy/` Go app

Source: `../camera-mouse-reverse/research/` (a separate prior investigation of the 2018
Windows binary, not produced in this repository) and this repository's own `legacy/` source,
read directly on 2026-11-11. This is a synthesis for the Rust rewrite's Phase 1 spec, not a
replacement for the original research files — go there for addresses/tokens/REA evidence IDs.

**Labels** (carried from the source research, which already distinguishes these):
`VERIFIED` = direct binary/managed-code evidence. `STRONGLY INFERRED` = behavior pattern
implied by verified call sites, not independently reconstructed. `HYPOTHESIS` = plausible but
unconfirmed. Do not upgrade a label without new evidence.

## Camera Mouse 2018 (Windows, .NET 4.0, bundled OpenCV 2.1) — reported pipeline

| Stage | Reported behavior | Label |
|---|---|---|
| Algorithm | One-point pyramidal Lucas-Kanade, `cvCalcOpticalFlowPyrLK` | VERIFIED (call site) |
| LK window | 10×10 | VERIFIED |
| Pyramid levels | 3 | VERIFIED |
| Termination | 20 iterations, epsilon 0.03 | VERIFIED |
| Frame format | 8-bit, 1 channel; OpenCV color-conversion code 6 | VERIFIED (defaults) |
| Outlier guard | Euclidean displacement > 35px → hold previous point | VERIFIED |
| Loss/boundary | 10px edge margin, config-gated hold-or-invalidate | VERIFIED (branch exists), exact policy unresolved |
| Template model | No fixed template; previous/current image + pyramid buffers swapped each frame | VERIFIED (swap call sites), absence of counter-evidence not proof |
| LK confidence | Binary status byte only; error-output pointer is null | VERIFIED |
| Cursor mapping | `desktop_center + (track_point + offset − image_center) * gain * desktop_size/image_size` | VERIFIED |
| Default gain | x=6, y=6 | VERIFIED |
| Cursor smoothing | `damping*mapped + (1-damping)*previous`, damping ≈ 0.95 | VERIFIED (formula + default) |
| Cursor output | Absolute position normalized to 0..65535 via `WindowsInput` | VERIFIED |
| Cursor loop | ~70ms sleep on observed paths | VERIFIED (observed), not a measured effective frame rate |
| LK native internals (interpolation, gradients, exact flags) | — | **UNRESOLVED** — never independently reconstructed |
| Reacquisition / dwell-click state machine | — | **UNRESOLVED** |

The underlying math is the standard local least-squares optical-flow model
`argmin_d Σ_{u∈W} (I_prev(u) − I_curr(u+d))²`, solved iteratively per pyramid level — this
describes the *algorithm family* the verified call site uses, not a line-by-line
reconstruction of the shipped `cv210.dll`.

## `legacy/` Go app — actual implementation (read directly, not reverse-engineered)

This is a **materially different algorithm**, not a port of the above:

| Stage | `legacy/` behavior | Source |
|---|---|---|
| Algorithm | Fixed-template normalized cross-correlation (`gocv.MatchTemplate`, `TmCcoeffNormed`) over a bounded search window, re-centered on the template every successful match | `legacy/internal/tracking/tracking.go` |
| Template | Fixed at pick time (`TemplateSizePx`, default 45px square), **never updated** after `Pick()` — opposite of the 2018 app's swapped-image-pair model | same |
| Search window | `TemplateSizePx * 2` margin around last known point | same |
| Match acceptance | NCC score ≥ 0.68, else reported lost (falls back to last template point) | same |
| Loss handling | Any frame below threshold → `Result{Lost: true}` at last known point; no countdown/hysteresis | same |
| Cursor gain | Single scalar `GainMultiplier`, default 8.0 (not separate x/y) | `legacy/internal/config/config.go` |
| Cursor smoothing | Exponential: `smooth += (target - smooth) * Smoothing`, default 0.30 — **lower is smoother here**, opposite convention from the 2018 app's damping (where higher damping ≈ less smoothing) | `legacy/internal/mouse/mouse.go` |
| Deadzone / max step | `DeadzonePx = 1.0`, `MaxSpeedPx = 35.0` per-frame delta clamp (coincidentally the same numeral as the 2018 app's 35px outlier guard, but a different mechanism — a per-frame pointer-delta clamp, not an image-space tracking-outlier rejection) | `legacy/internal/mouse/mouse.go` |
| Dwell click | Reference point + radius (`DwellRadiusPx = 30.0`) + configurable duration (default 500ms); resets on drift beyond radius | same |
| Cursor output | Relative `robotgo.Move` from current OS cursor position, not absolute 0..65535 | same |

## What this means for the rewrite

1. There is **no existing Rust-relevant optical-flow code to port** — `legacy/` never
   implemented PyrLK. The 2018 evidence is the only source for that algorithm family.
2. `legacy/`'s NCC template matcher is a legitimate, working, shipped alternative baseline —
   Phase 3 should treat it as a real comparator, not a placeholder, per
   `docs/archive/rust-rewrite-seed/05-architecture-and-testing.md`.
3. The "35 pixels" appearing in both systems for different reasons is a trap: don't assume
   shared lineage or copy the number across mechanisms without re-deriving it for whichever
   algorithm Phase 3 actually implements.
4. Cursor-smoothing conventions are inverted between the two systems (2018: damping↑ = less
   smoothing; `legacy/`: smoothing↑ = more smoothing). Pick one convention explicitly in the
   Rust `pointer-core` design and document it — don't silently inherit either name's polarity.
5. Default gain values are not comparable as-is (2018: per-axis gain ≈6 combined with a
   desktop/image-size ratio multiply; `legacy/`: single scalar 8.0 applied to a raw pixel
   delta already clamped to ±35px). Any claim that one "feels more sensitive" needs a measured
   comparison, not a numeric comparison of these constants.

## Unresolved questions carried into Phase 1

From `../camera-mouse-reverse/research/08-unanswered-questions.md`: offset evolution/limits
and optional image flipping in the 2018 app's cursor mapping; its full loss/countdown and
reacquisition flow; exact native LK flags, interpolation and first-frame pyramid
initialization; `PixelDepth`/`PixelChannels` defaults and exact color-conversion semantics;
DirectShow device negotiation; the dwell-click state machine and input-injection relationship;
native LK thresholds; actual frame rate and resource costs under runtime measurement. None of
these block Phase 0; they're real open items for whoever picks up Phase 1's behavior spec.
