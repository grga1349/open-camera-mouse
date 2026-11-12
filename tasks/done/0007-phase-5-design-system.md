# 0007 — Phase 5: design system

Status: DONE (2026-11-12)
Owner/branch: `rewrite/rust`
Depends on: 0006 (done)

## Context

Phase 5 per `docs/ROADMAP.md`. Inputs: `docs/design/DESIGN_PLAN.md` (intent, candidate
directions A/B/C, elements, states, egui handoff), BEHAVIOR_SPEC §7 (R-A1 states, R-A2 handover,
R-A3 recenter, R-A4 F11/F12, R-K1 dwell, R-P1..R-P3 pointer), ADR-0006 (2018 defaults), ADR-0011
consequences (`Lost(Searching)`, `redetect_armed()`). Skill: `egui-development`.

The lab is a design reference, not a product. It has no backend and no build step. Settings
values in it are ADR-0006 defaults so that the layout is realistic, not a new spec.

## Objective

1. `docs/design/design-lab.html` — self-contained (no network, no build), CSS custom properties
   for every token, with:
   - the three directions from DESIGN_PLAN (A dark technical, B light native, C high contrast),
     switchable live;
   - a realistic app window: camera preview with target reticle and click-to-select, status,
     start/pause (F11), recenter (F12), settings (camera, pointer gain X/Y linked, smoothing,
     screen limits W/E/N/S, dwell click), help/onboarding, diagnostics toggle (no auto-recenter, see
     Decisions);
   - simulated states, switchable: first run / camera permission, pointer (accessibility)
     permission, loading, no camera, preview running / no target, tracking, held (motion
     outlier, near edge), searching (re-acquiring, ADR-0010), lost (re-select needed, with the
     "can't auto-recover this selection" hint when `redetect_armed() == false`), mouse handover
     (physical mouse has control, resume countdown), recentering countdown, paused, camera
     disconnected, error;
   - every state conveyed by icon + text, never hue alone;
   - keyboard: everything reachable by Tab, visible focus ring, F11/F12 simulated in the lab;
   - a text-scale control (100–150 %) and reduced-motion respect.
2. User iteration until a direction (or a mix) is approved. Approval is recorded in this file
   and in `DESIGN_PLAN.md`.
3. `docs/design/EGUI_FEASIBILITY.md` — per affordance in the approved direction: immediate-mode
   feasibility, font embedding, preview repaint cost, window sizing / high-DPI, and the token →
   egui `Visuals`/`Style` mapping. Accessibility requirements (large hit targets, non-colour
   status, keyboard navigation, AccessKit labels) are listed as Phase 6 acceptance criteria.

## Decisions (user, 2026-11-12, after lab v1)

1. Monochrome (grayscale) dark UI. The **only** colour is the target marker, which is a **green
   rectangle** (not a round reticle). Status is carried by icon shape, text and border style.
2. Options live in a **separate window**, not a side panel.
3. **No auto-recenter** in the initial rewrite (ADR-0006 row amended).
4. Both windows are **fixed size** (not resizable): Tracking 560 × 660, Options 400 × 660 (lab sizes,
   to confirm in egui at real DPI).
5. Offer five neutral schemes to choose from; **Graphite (grayscale) is the favourite**.
   v1 directions A/B/C are superseded; C's large targets survive as a "Larger controls" option.
6. (after v2) Themes narrowed to **Carbon (dark) and Paper (light)**, switchable in Options →
   General → Appearance. Status is a plain icon + text line under the preview (no container).
   Options button uses a cog. Marker is a plain shape with no outline: circle (filled) or
   rectangle, still on trial. Accent trial: green vs yellow (yellow also colours primary buttons;
   Carbon `#ffd60a`, Paper `#f5c518`, black text on both). Windows 560 × 620 and 400 × 620.
7. **Approved 2026-11-12 (user):** lab v3 with **yellow** accent (marker + primary buttons) and
   the marker as a **thin circle ring** (not filled). Carbon (default) + Paper themes.
8. **Marker colour revised 2026-11-12 (user, final):** a colour camera stand-in showed yellow is
   near-invisible on light skin. Measured candidates (ΔE vs four skin tones) and chose **neon
   green `#00ff00`**, the same in both themes. Primary buttons stay **yellow**.
9. **Recenter flow (user, 2026-11-12):** recenter = white circle (same style as the pick circle) at the preview
   centre + **5 s** countdown 5-4-3-2-1 in a bottom badge;
   at 0 the point inside the circle becomes the target. Resume after Pause always goes through
   recenter. Recenter is enabled while paused and ends the pause. Recorded in BEHAVIOR_SPEC R-A3.
10. **Find the target again (user, 2026-11-12, during task 0008):** re-detection is a switch in
    Options → Camera, off by default (ADR-0011 amendment). Added to the lab after approval.

## Non-goals

No egui code (Phase 6). No new crates. No change to `app-core` states or settings semantics —
if the design needs a state that R-A1 lacks (e.g. pointer-permission, handover), record it as a
Phase 6 input, not an edit here. No final iconography assets beyond inline SVG.

## Files/scope

`docs/design/` (lab, plan, feasibility note), this task file, `tasks/BACKLOG.md`, `agent/`.

## Acceptance criteria (ROADMAP Phase 5 gate)

- The lab opens from disk in a current browser with no network and shows all three directions
  and every state listed above.
- Each state is distinguishable with colour removed (icon + text), checked in the lab's
  grayscale toggle.
- All interactive controls are keyboard reachable with a visible focus indicator.
- The user has explicitly approved a UI reference (recorded here with date).
- `EGUI_FEASIBILITY.md` exists and covers every affordance in the approved reference.

## Verification

- Open `docs/design/design-lab.html` (`open docs/design/design-lab.html` on macOS); step through
  every direction × state; Tab through all controls; toggle grayscale.
- `grep -c 'https\?://' docs/design/design-lab.html` shows no external resource loads (only
  inert text such as the SVG namespace).
- No Rust changes, so the cargo gates are unaffected; run `cargo fmt --check` only if a Rust
  file was touched.

## Progress

- 2026-11-12: task written. Lab v1 built: directions A/B/C, 17 simulated states, text scale,
  grayscale, reduced motion, window widths, token table, deep links (`#dir=B&state=searching`).
  JS parses (`node --check`), 0 external URLs, headless Chrome renders A/B/C. Awaiting user review.
- 2026-11-12: lab v2 per Decisions: two fixed windows, monochrome UI + green target rectangle,
  schemes Graphite / Carbon / Slate / Stone / Paper, "Larger controls" check, auto-recenter removed.
  `node --check` OK, 0 external URLs, headless Chrome screenshots of Graphite and Paper reviewed.
  Awaiting the user's final scheme pick, then EGUI_FEASIBILITY.md (needs eframe multi-viewport for
  the second window).
- 2026-11-12: lab v3 per Decision 6. `node --check` OK, 0 external URLs, screenshots of Carbon/green/
  circle, Paper/yellow/rect and Carbon/yellow reviewed. Open: marker shape (circle vs rectangle) and
  accent (green vs yellow).
- 2026-11-12: user approved v3 with yellow + thin ring (Decision 7); lab defaults set to it
  (`node --check` OK, screenshot reviewed). `docs/design/EGUI_FEASIBILITY.md` written against
  egui/eframe 0.37.0 registry sources (API names grep-verified). Gate met: approved reference + egui
  feasibility note. No Rust files touched, so cargo gates are unaffected.
- 2026-11-12: lab gained a colour camera stand-in (skin-tone selector), ring-shadow check and
  marker-colour picker. Yellow failed (ΔE 58, luminance ratio 1.10 on light skin). The user
  chose neon green `#00ff00` (ΔE ≥ 110 vs all tones). EGUI_FEASIBILITY, DESIGN_PLAN and the lab
  were updated to match. `node --check` OK, screenshots on light and dark skin reviewed.
- 2026-11-12: lab recenter reworked per Decision 9 (centre circle, pause → recenter, recenter
  breaks pause); the old white countdown box is removed. `node --check` OK; screenshots of the
  countdown, its completion (tracking at the circled point) and paused (Recenter enabled) reviewed.
- 2026-11-12: countdown moved off the camera image into a bottom badge (user: the digit covered the
  circle and lingered). DOM check: badge/title 3 → 2 → 1 at 0.5/1.5/2.5 s, then hidden + Tracking
  at 3.5 s.
- 2026-11-12: recenter circle now uses the same style as the pick circle (white dashed ring,
  shared draw helper). User request.
- 2026-11-12: recenter countdown set to 5 s (user; matches 2018's 5-4-3-2-1).
- 2026-11-12 (task 0008): Decision 10 added to the lab (Camera tab switch, synced with the
  Searching/Lost states). Verified there; see task 0008 Progress.
