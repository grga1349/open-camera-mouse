# UI design plan (Phase 5 preparation — no HTML built yet)

## Intent

An elegant, professional, native-feeling accessibility utility, not a gimmicky dashboard.
Explore visual direction in a standalone interactive HTML file *before* committing to egui, so
iteration is cheap (browser reload) instead of expensive (Rust recompile + manual state
puppeting). This document plans that lab; it does not contain the final HTML or final tokens.

## Decided direction (user, 2026-11-12)

Monochrome UI chrome with exactly two accent hues: the neon-green target marker on the preview
and yellow primary buttons. The camera preview itself is shown in colour (user, 2026-11-12). Two
fixed-size windows: Tracking (preview, status, start/pause, recenter, options button) and Options.
No auto-recenter in the initial rewrite. Themes: Carbon (dark) and Paper (light), switchable in
Options. **Approved 2026-11-12:** `design-lab.html` v3: thin neon-green (`#00ff00`) circle-ring marker,
yellow primary buttons, everything else monochrome. egui mapping: `EGUI_FEASIBILITY.md`. History: task 0007.
Added 2026-11-12 (task 0008): Options → Camera → "Find the target again" switch (loss recovery,
off by default, ADR-0011 amendment). The Searching state only occurs with it on.

## Candidate directions (v1, superseded by the decision above)

- **A — Dark technical, understated.** Charcoal/slate, mint-teal accent, minimal chrome.
- **B — Light native utility.** Bright background, strong contrast, subtle borders, compact
  but generous hit targets.
- **C — High-contrast accessibility.** Larger text/hit areas, explicit status text, minimal
  reliance on color alone to convey state.

None of these are approved; the design lab is where the user picks and iterates, not this doc.

## Core UI elements to prototype

Camera preview with target reticle and selection; tracking/paused/lost/permission status;
pause/resume; sensitivity X/Y; smoothing mode and amount; pointer limits and calibration;
camera selector; dwell-click settings; accessible onboarding/help; diagnostics toggle.

## States to prototype

First run / no permission, no camera, preview running / unselected, target selected and
tracking, temporarily lost, recovering, manually paused, camera disconnected, loading, error.
Every state needs a non-color indicator (icon/text), not hue alone.

## Deliverable (Phase 5, future repo work)

`docs/design/design-lab.html` — self-contained (no backend, no build step), CSS custom
properties for tokens, a way to switch theme/component variants and simulated app states in
the browser, realistic component hierarchy rather than static marketing art.

## Handoff into egui (Phase 5→6)

For each approved visual affordance: confirm immediate-mode feasibility, font embedding,
preview-repaint performance, native window sizing and high-DPI behavior, and map tokens to
egui `Visuals`/`Style`. Keep the accessibility requirements (large hit targets, non-color
status, keyboard navigation) as acceptance criteria for the egui port, not nice-to-haves that
quietly drop out during translation.

## Gate

Phase 5 is done when there's an approved UI reference (this lab, iterated with the user) and
an explicit egui-feasibility note per affordance — not when the HTML merely exists.
